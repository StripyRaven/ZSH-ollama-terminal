//! crates/ai-core/src/integration.rs
//! # Integration System for AI Core
//! Система интеграции всех бизнес-компонентов AI Core.

use async_trait::async_trait;
use ollama_client::OllamaClient;
use shared::{
    states::{Analyzed, Validated},
    Command, CommandAnalyzer, CommandSuggestion, DomainError, ModelInfo, SecurityValidator,
    TrainingEngine,
};
use std::sync::Arc;

use super::{
    AiAnalyzer, AnalysisCache, CacheManager, HallucinationDetector, ModelTrainingEngine,
    PerformanceMonitor,
};

/// Интегрированная система AI Core
pub struct IntegratedAICore {
    analyzer: Arc<AiAnalyzer>,
    training_engine: Arc<ModelTrainingEngine>,
    cache_manager: Arc<CacheManager>,
    performance_monitor: Arc<PerformanceMonitor>,
}

impl IntegratedAICore {
    pub fn new(
        security: Arc<dyn SecurityValidator>,
        ollama: Arc<OllamaClient>,
        cache_size: usize,
    ) -> Self {
        let analyzer = Arc::new(AiAnalyzer::new(security, ollama, cache_size));
        let training_engine = Arc::new(ModelTrainingEngine::new());
        let cache_manager = Arc::new(CacheManager::new(
            cache_size,
            std::time::Duration::from_secs(3600), // 1 hour TTL
        ));
        let performance_monitor = Arc::new(PerformanceMonitor::new());

        Self {
            analyzer,
            training_engine,
            cache_manager,
            performance_monitor,
        }
    }

    /// Полный цикл анализа команды
    pub async fn analyze_command_complete(
        &self,
        command: Command<Validated>,
    ) -> Result<AnalysisResult, DomainError> {
        let start_time = std::time::Instant::now();

        // Проверка кэша
        if let Some(cached) = self.cache_manager.get_analysis(command.raw()).await {
            return Ok(AnalysisResult {
                analysis: cached,
                source: AnalysisSource::Cache,
                processing_time: start_time.elapsed(),
                cache_hit: true,
            });
        }

        // AI анализ
        let analyzed_command = self.analyzer.analyze_command(command).await?;
        let analysis = analyzed_command
            .analysis_data()
            .ok_or_else(|| {
                DomainError::Analysis(shared::error::AnalysisError {
                    model: "internal".to_string(),
                    error_type: shared::error::AnalysisErrorType::InvalidResponse,
                    details: "No analysis data available".to_string(),
                    suggestion: None,
                })
            })?
            .clone();

        // Сохранение в кэш
        self.cache_manager
            .put_analysis(analyzed_command.raw().to_string(), analysis.clone())
            .await;

        Ok(AnalysisResult {
            analysis,
            source: AnalysisSource::AI,
            processing_time: start_time.elapsed(),
            cache_hit: false,
        })
    }

    /// Обучение с сбором данных из анализа
    pub async fn train_with_analysis_data(
        &self,
        training_data: shared::TrainingData,
    ) -> Result<shared::TrainedModel, DomainError> {
        self.training_engine
            .train_model(
                training_data,
                shared::TrainingConfig {
                    epochs: 10,
                    learning_rate: 0.001,
                    batch_size: 32,
                    validation_split: 0.2,
                },
            )
            .await
    }

    /// Получение системных метрик
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let analysis_metrics = self.performance_monitor.average_analysis_time().await;
        let cache_metrics = self.cache_manager.metrics().await;

        SystemMetrics {
            average_analysis_time: analysis_metrics,
            cache_hit_rate: cache_metrics.analysis.hit_rate,
            cache_size: cache_metrics.analysis.current_size,
            total_analysis_requests: cache_metrics.analysis.hits + cache_metrics.analysis.misses,
        }
    }

    /// Проверка здоровья системы
    pub async fn health_check(&self) -> HealthStatus {
        // Проверка доступности компонентов
        let cache_health = self.cache_manager.metrics().await.current_size < 1000; // Простая проверка
        let performance_health = self.performance_monitor.average_analysis_time().await
            < std::time::Duration::from_millis(1000);

        if cache_health && performance_health {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        }
    }
}

/// Результат анализа
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub analysis: super::CommandAnalysis,
    pub source: AnalysisSource,
    pub processing_time: std::time::Duration,
    pub cache_hit: bool,
}

/// Источник анализа
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisSource {
    Cache,
    AI,
    Heuristic,
}

/// Системные метрики
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub average_analysis_time: std::time::Duration,
    pub cache_hit_rate: f64,
    pub cache_size: usize,
    pub total_analysis_requests: u64,
}

/// Статус здоровья системы
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Билдер для IntegratedAICore
pub struct AICoreBuilder {
    security: Option<Arc<dyn SecurityValidator>>,
    ollama: Option<Arc<OllamaClient>>,
    cache_size: usize,
    enable_training: bool,
}

impl AICoreBuilder {
    pub fn new() -> Self {
        Self {
            security: None,
            ollama: None,
            cache_size: 1000,
            enable_training: true,
        }
    }

    pub fn with_security(mut self, security: Arc<dyn SecurityValidator>) -> Self {
        self.security = Some(security);
        self
    }

    pub fn with_ollama(mut self, ollama: Arc<OllamaClient>) -> Self {
        self.ollama = Some(ollama);
        self
    }

    pub fn with_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    pub fn enable_training(mut self, enable: bool) -> Self {
        self.enable_training = enable;
        self
    }

    pub fn build(self) -> Result<IntegratedAICore, BuilderError> {
        let security = self.security.ok_or(BuilderError::MissingSecurity)?;
        let ollama = self.ollama.ok_or(BuilderError::MissingOllama)?;

        Ok(IntegratedAICore::new(security, ollama, self.cache_size))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("Security validator is required")]
    MissingSecurity,
    #[error("Ollama client is required")]
    MissingOllama,
}

// Реализация трейтов для интеграции
#[async_trait]
impl CommandAnalyzer for IntegratedAICore {
    async fn analyze_command(
        &self,
        command: Command<Validated>,
    ) -> Result<Command<Analyzed>, DomainError> {
        let result = self.analyze_command_complete(command).await?;

        // Преобразование обратно в Command<Analyzed>
        // В реальной реализации здесь будет правильное преобразование
        Ok(Command::new(result.analysis.explanation).unwrap())
    }

    async fn get_suggestions(
        &self,
        analysis: &Command<Analyzed>,
    ) -> Result<Vec<CommandSuggestion>, DomainError> {
        self.analyzer.get_suggestions(analysis).await
    }

    fn get_model_info(&self) -> ModelInfo {
        self.analyzer.get_model_info()
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
                "explanation": "Test analysis",
                "risks": [],
                "suggestions": ["test suggestion"],
                "confidence": 0.9,
                "alternatives": []
            }"#
            .to_string())
        }
    }

    #[tokio::test]
    async fn test_integrated_core_creation() {
        let security = Arc::new(MockSecurityValidator);
        let ollama = Arc::new(MockOllamaClient::new());

        let core = AICoreBuilder::new()
            .with_security(security)
            .with_ollama(ollama)
            .with_cache_size(100)
            .build()
            .unwrap();

        assert!(core.health_check().await == HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_analysis_flow() {
        let security = Arc::new(MockSecurityValidator);
        let ollama = Arc::new(MockOllamaClient::new());

        let core = IntegratedAICore::new(security, ollama, 100);
        let command = Command::new("ls -la".to_string()).unwrap();
        let validated = command.validate().await.unwrap();

        let result = core.analyze_command_complete(validated).await.unwrap();

        assert_eq!(result.source, AnalysisSource::AI);
        assert!(!result.cache_hit);
        assert!(!result.analysis.explanation.is_empty());
    }

    #[tokio::test]
    async fn test_cache_behavior() {
        let security = Arc::new(MockSecurityValidator);
        let ollama = Arc::new(MockOllamaClient::new());

        let core = IntegratedAICore::new(security, ollama, 100);
        let command = Command::new("ls -la".to_string()).unwrap();
        let validated = command.validate().await.unwrap();

        // Первый вызов - кэш miss
        let result1 = core
            .analyze_command_complete(validated.clone())
            .await
            .unwrap();
        assert!(!result1.cache_hit);

        // Второй вызов - кэш hit
        let result2 = core.analyze_command_complete(validated).await.unwrap();
        assert!(result2.cache_hit);
    }
}
