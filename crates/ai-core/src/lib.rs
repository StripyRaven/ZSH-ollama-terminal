//! crates/ai-core/src/lib.rs
//! # AI Core Business Logic
//! Бизнес-логика AI анализа команд с кэшированием, dependency injection и детекцией галлюцинаций.

pub mod cache;
pub mod hallucination_detector;
pub mod integration;
pub mod training_engine;

use async_trait::async_trait;

use lru::LruCache;
use ollama_client::OllamaClient;
use shared::TrainedModel;
use shared::{
    states::{Analyzed, Validated},
    Command, CommandAnalysis, CommandAnalyzer, CommandSuggestion, DomainError, ModelInfo,
    SecurityValidator,
};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// AI анализатор команд с кэшированием и resilience patterns
pub struct AiAnalyzer {
    security: Arc<dyn SecurityValidator>,
    ollama: Arc<OllamaClient>,
    cache: tokio::sync::Mutex<LruCache<String, Arc<Command<Analyzed>>>>,
    hallucination_detector: HallucinationDetector,
    performance_monitor: PerformanceMonitor,
}

impl AiAnalyzer {
    pub fn new(
        security: Arc<dyn SecurityValidator>,
        ollama: Arc<OllamaClient>,
        cache_size: usize,
    ) -> Self {
        Self {
            security,
            ollama,
            cache: tokio::sync::Mutex::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
            hallucination_detector: HallucinationDetector::new(),
            performance_monitor: PerformanceMonitor::new(),
        }
    }

    /// Анализ команды с кэшированием и fallback механизмами
    async fn analyze_command_with_fallback_arc(
        &self,
        command: Command<Validated>,
    ) -> Result<Arc<Command<Analyzed>>, DomainError> {
        let cache_key = command.raw().to_string();

        // Проверка кэша
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                tracing::info!("Cache hit for command: {}", cache_key);
                return Ok(Arc::clone(cached));
            }
        }

        // Анализ
        let analysis = match self.analyze_with_ollama(&command).await {
            Ok(analysis) => analysis,
            Err(_) => self.analyze_with_heuristics(&command).await?,
        };

        let hallucination_score = self.hallucination_detector.detect(&analysis).await?;

        if self
            .hallucination_detector
            .should_reject(hallucination_score)
        {
            return Err(DomainError::Analysis(shared::error::AnalysisError {
                model: "ollama".to_string(),
                error_type: shared::error::AnalysisErrorType::HallucinationDetected,
                details: format!(
                    "Hallucination detected with score: {:.2}",
                    hallucination_score
                ),
                suggestion: Some("Using heuristic analyzer instead".to_string()),
            }));
        }

        // Создаем Command<Analyzed> и оборачиваем в Arc (только один раз!)
        let analyzed = Arc::new(command.into_analyzed(analysis, hallucination_score)?);

        // Сохраняем в кэш
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key, Arc::clone(&analyzed));
        }

        Ok(analyzed)
    }

    async fn analyze_with_ollama(
        &self,
        command: &Command<Validated>,
    ) -> Result<CommandAnalysis, DomainError> {
        let prompt = self.build_analysis_prompt(command);

        // Метрики производительности
        let start = std::time::Instant::now();
        let response = self.ollama.generate_completion(prompt).await?;
        let duration = start.elapsed();

        self.performance_monitor.record_analysis_time(duration);

        self.parse_ollama_response(response)
    }

    async fn analyze_with_heuristics(
        &self,
        command: &Command<Validated>,
    ) -> Result<CommandAnalysis, DomainError> {
        // Эвристический анализ на основе известных паттернов команд
        let parts = command.parts();
        if parts.is_empty() {
            return Ok(CommandAnalysis::empty());
        }

        let executable = &parts[0];
        let analysis = match executable.as_str() {
            "ls" => self.analyze_ls_command(parts).await,
            "cd" => self.analyze_cd_command(parts).await,
            "git" => self.analyze_git_command(parts).await,
            "docker" => self.analyze_docker_command(parts).await,
            "cargo" => self.analyze_cargo_command(parts).await,
            _ => self.analyze_generic_command(parts).await,
        }?;

        Ok(analysis)
    }

    fn build_analysis_prompt(&self, command: &Command<Validated>) -> String {
        format!(
            "Analyze the following terminal command and provide structured JSON response:\n\n\
            Command: {}\n\
            Working Directory: {}\n\
            User ID: {}\n\n\
            Please analyze:\n\
            1. What does this command do?\n\
            2. Are there any security risks?\n\
            3. Suggest improvements or alternatives\n\
            4. Provide confidence score (0.0-1.0)\n\n\
            Respond with JSON format:\n\
            {{\n\
              \"explanation\": \"string\",\n\
              \"risks\": [\"string\"],\n\
              \"suggestions\": [\"string\"],\n\
              \"confidence\": 0.95,\n\
              \"alternatives\": [\"string\"]\n\
            }}",
            command.raw(),
            command.context().working_directory.to_string(),
            command.context().user_id
        )
    }

    fn parse_ollama_response(&self, response: String) -> Result<CommandAnalysis, DomainError> {
        serde_json::from_str(&response).map_err(|e| {
            DomainError::Analysis(shared::error::AnalysisError {
                model: "ollama".to_string(),
                error_type: shared::error::AnalysisErrorType::InvalidResponse,
                details: format!("Failed to parse Ollama response: {}", e),
                suggestion: Some("Check the Ollama response format".to_string()),
            })
        })
    }

    // Эвристические анализаторы для конкретных команд
    async fn analyze_ls_command(&self, parts: &[String]) -> Result<CommandAnalysis, DomainError> {
        let mut suggestions = vec![
            "Use -la for detailed view with hidden files".to_string(),
            "Use -lh for human-readable file sizes".to_string(),
        ];

        if !parts.contains(&"-a".to_string()) && !parts.contains(&"-la".to_string()) {
            suggestions.push("Consider using -a to show hidden files".to_string());
        }

        Ok(CommandAnalysis {
            explanation: "Lists directory contents".to_string(),
            risks: vec![],
            suggestions,
            confidence: 0.9,
            alternatives: vec![
                "ls -la".to_string(),
                "ls -lh".to_string(),
                "exa".to_string(),
            ],
        })
    }

    async fn analyze_cd_command(&self, parts: &[String]) -> Result<CommandAnalysis, DomainError> {
        let explanation = if parts.len() > 1 {
            format!("Changes directory to {}", parts[1])
        } else {
            "Changes to home directory".to_string()
        };

        Ok(CommandAnalysis {
            explanation,
            risks: vec![],
            suggestions: vec!["Use pushd/popd for directory stack".to_string()],
            confidence: 0.95,
            alternatives: vec![],
        })
    }

    async fn analyze_git_command(&self, parts: &[String]) -> Result<CommandAnalysis, DomainError> {
        let subcommand = if parts.len() > 1 { &parts[1] } else { "status" };

        let explanation = match subcommand {
            "status" => "Shows the working tree status".to_string(),
            "commit" => "Records changes to the repository".to_string(),
            "push" => "Updates remote refs along with associated objects".to_string(),
            "pull" => "Fetches from and integrates with another repository".to_string(),
            _ => format!("Git command: {}", subcommand),
        };

        Ok(CommandAnalysis {
            explanation,
            risks: vec![],
            suggestions: vec!["Consider using git add before commit".to_string()],
            confidence: 0.85,
            alternatives: vec![],
        })
    }

    async fn analyze_docker_command(
        &self,
        parts: &[String],
    ) -> Result<CommandAnalysis, DomainError> {
        let explanation = if parts.len() > 1 {
            format!("Docker command: {}", parts[1])
        } else {
            "Docker management command".to_string()
        };

        let risks = if parts.contains(&"rm".to_string()) && parts.contains(&"-f".to_string()) {
            vec!["Force removal may cause data loss".to_string()]
        } else {
            vec![]
        };

        Ok(CommandAnalysis {
            explanation,
            risks,
            suggestions: vec!["Consider using docker compose for multi-container apps".to_string()],
            confidence: 0.8,
            alternatives: vec!["podman".to_string()],
        })
    }

    async fn analyze_cargo_command(
        &self,
        parts: &[String],
    ) -> Result<CommandAnalysis, DomainError> {
        let subcommand = if parts.len() > 1 { &parts[1] } else { "build" };

        let explanation = match subcommand {
            "build" => "Compiles the current package".to_string(),
            "run" => "Runs the main binary".to_string(),
            "test" => "Runs the tests".to_string(),
            "clippy" => "Runs the Clippy linter".to_string(),
            "fmt" => "Formats the code".to_string(),
            _ => format!("Cargo command: {}", subcommand),
        };

        let suggestions = match subcommand {
            "build" => vec!["Use --release for production builds".to_string()],
            "test" => vec!["Use --nocapture to see println output".to_string()],
            _ => vec![],
        };

        Ok(CommandAnalysis {
            explanation,
            risks: vec![],
            suggestions,
            confidence: 0.9,
            alternatives: vec![],
        })
    }

    async fn analyze_generic_command(
        &self,
        parts: &[String],
    ) -> Result<CommandAnalysis, DomainError> {
        Ok(CommandAnalysis {
            explanation: format!("Executes program: {}", parts[0]),
            risks: vec!["Unknown command, verify safety before running".to_string()],
            suggestions: vec!["Check command documentation with man pages".to_string()],
            confidence: 0.5,
            alternatives: vec![],
        })
    }
}

#[async_trait]
impl CommandAnalyzer for AiAnalyzer {
    async fn analyze_command(
        &self,
        command: Command<Validated>,
    ) -> Result<Command<Analyzed>, DomainError> {
        // Убираем кэш для простоты
        let analysis = match self.analyze_with_ollama(&command).await {
            Ok(analysis) => analysis,
            Err(ollama_error) => {
                tracing::warn!(
                    "Ollama analysis failed, using heuristic fallback: {}",
                    ollama_error
                );
                self.analyze_with_heuristics(&command).await?
            }
        };

        let hallucination_score = self.hallucination_detector.detect(&analysis).await?;

        if self
            .hallucination_detector
            .should_reject(hallucination_score)
        {
            return Err(DomainError::Analysis(shared::error::AnalysisError {
                model: "ollama".to_string(),
                error_type: shared::error::AnalysisErrorType::HallucinationDetected,
                details: format!(
                    "Hallucination detected with score: {:.2}",
                    hallucination_score
                ),
                suggestion: Some("Using heuristic analyzer instead".to_string()),
            }));
        }

        command.into_analyzed(analysis, hallucination_score)
    }

    async fn get_suggestions(
        &self,
        analysis: &Command<Analyzed>,
    ) -> Result<Vec<CommandSuggestion>, DomainError> {
        let analysis_data = analysis.analysis_data().ok_or_else(|| {
            DomainError::Analysis(shared::error::AnalysisError {
                model: "internal".to_string(),
                error_type: shared::error::AnalysisErrorType::InvalidResponse,
                details: "No analysis data available".to_string(),
                suggestion: None,
            })
        })?;

        let suggestions = analysis_data
            .suggestions
            .iter()
            .map(|suggestion| CommandSuggestion {
                command: suggestion.clone(),
                explanation: format!("Suggestion: {}", suggestion),
                confidence: analysis_data.confidence,
                safety_level: shared::states::SecurityLevel::User,
            })
            .collect();

        Ok(suggestions)
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "ZSH AI Terminal Analyzer".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![
                "command_analysis".to_string(),
                "security_validation".to_string(),
                "suggestion_generation".to_string(),
                "hallucination_detection".to_string(),
            ],
            max_tokens: 4096,
        }
    }
}

/// Детектор галлюцинаций AI
pub struct HallucinationDetector {
    threshold: f32,
}

impl HallucinationDetector {
    pub fn new() -> Self {
        Self { threshold: 0.7 }
    }

    pub async fn detect(&self, analysis: &CommandAnalysis) -> Result<f32, DomainError> {
        // Простая эвристика: низкая уверенность + много рисков = возможные галлюцинации
        let mut score: f32 = 0.0; // Добавляем аннотацию типа

        // Штраф за низкую уверенность
        if analysis.confidence < 0.3 {
            score += 0.4;
        } else if analysis.confidence < 0.6 {
            score += 0.2;
        }

        // Штраф за отсутствие объяснения
        if analysis.explanation.is_empty() || analysis.explanation.len() < 10 {
            score += 0.3;
        }

        // Штраф за слишком общие предложения
        let generic_patterns = ["use", "consider", "try"];
        let mut generic_count = 0;
        for suggestion in &analysis.suggestions {
            if generic_patterns
                .iter()
                .any(|pattern| suggestion.to_lowercase().contains(pattern))
            {
                generic_count += 1;
            }
        }
        if generic_count > 2 {
            score += 0.2;
        }

        Ok(score.min(1.0))
    }

    pub fn should_reject(&self, score: f32) -> bool {
        score > self.threshold
    }
}

pub struct PerformanceMonitor {
    analysis_times: std::sync::Arc<std::sync::Mutex<Vec<std::time::Duration>>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            analysis_times: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn record_analysis_time(&self, duration: std::time::Duration) {
        let times = self.analysis_times.clone();
        tokio::task::spawn_blocking(move || {
            let mut times = times.lock().unwrap();
            times.push(duration);
            if times.len() > 100 {
                times.remove(0);
            }
        });
    }

    pub async fn average_analysis_time(&self) -> std::time::Duration {
        let times = self.analysis_times.lock().unwrap();
        if times.is_empty() {
            return std::time::Duration::from_millis(0);
        }
        let total: std::time::Duration = times.iter().sum();
        total / times.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{states::Unvalidated, Command};

    struct MockSecurityValidator;
    struct MockOllamaClient;

    #[async_trait]
    impl SecurityValidator for MockSecurityValidator {
        async fn validate_command(
            &self,
            command: Command<Unvalidated>,
        ) -> Result<Command<Validated>, DomainError> {
            Ok(Command::new(command.raw().to_string()).unwrap())
        }
    }

    impl MockOllamaClient {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ollama_client::OllamaClient for MockOllamaClient {
        async fn generate_completion(&self, prompt: String) -> Result<String, DomainError> {
            Ok(r#"{
                "explanation": "Lists directory contents with detailed view",
                "risks": ["None"],
                "suggestions": ["Use -lh for human-readable sizes"],
                "confidence": 0.95,
                "alternatives": ["ls -lh", "exa"]
            }"#
            .to_string())
        }
    }

    #[tokio::test]
    async fn test_ai_analyzer_with_mocks() {
        let security = Arc::new(MockSecurityValidator);
        let ollama = Arc::new(MockOllamaClient::new());
        let analyzer = AiAnalyzer::new(security, ollama, 100);

        let command = Command::new("ls -la".to_string()).unwrap();
        let validated = command.validate().await.unwrap();
        let result = analyzer.analyze_command(validated).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hallucination_detection() {
        let detector = HallucinationDetector::new();

        let good_analysis = CommandAnalysis {
            explanation: "This command lists directory contents with detailed information including hidden files".to_string(),
            risks: vec![],
            suggestions: vec!["Use -lh for human readable sizes".to_string()],
            confidence: 0.9,
            alternatives: vec![],
        };

        let bad_analysis = CommandAnalysis {
            explanation: "".to_string(),
            risks: vec![],
            suggestions: vec![],
            confidence: 0.1,
            alternatives: vec![],
        };

        let good_score = detector.detect(&good_analysis).await.unwrap();
        let bad_score = detector.detect(&bad_analysis).await.unwrap();

        assert!(good_score < bad_score);
        assert!(!detector.should_reject(good_score));
    }
}
