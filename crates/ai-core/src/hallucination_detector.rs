//! crates/ai-core/src/hallucination_detector.rs
//! # Hallucination Detection System
//! Многоуровневая система детекции галлюцинаций AI моделей.

use super::CommandAnalysis;
use shared::DomainError;
use std::collections::HashSet;

/// Детектор галлюцинаций с многоуровневой проверкой
pub struct HallucinationDetector {
    rules: Vec<DetectionRule>,
    threshold: f32,
    known_commands: HashSet<String>,
}

impl HallucinationDetector {
    pub fn new() -> Self {
        let mut known_commands = HashSet::new();
        known_commands.extend(
            vec![
                "ls", "cd", "pwd", "cat", "echo", "grep", "find", "git", "docker", "kubectl",
                "cargo", "rustc", "npm", "python", "pip", "apt", "yum", "dnf", "pacman", "brew",
            ]
            .into_iter()
            .map(String::from),
        );

        Self {
            rules: vec![
                DetectionRule::ConsistencyCheck,
                DetectionRule::FactualAccuracy,
                DetectionRule::ContextRelevance,
                DetectionRule::CommandValidity,
                DetectionRule::ConfidenceValidation,
            ],
            threshold: 0.7,
            known_commands,
        }
    }

    pub async fn detect(&self, analysis: &CommandAnalysis) -> Result<f32, DomainError> {
        let mut total_score = 0.0;
        let mut active_rules = 0;

        for rule in &self.rules {
            if let Some(score) = rule.apply(analysis, &self.known_commands).await? {
                total_score += score;
                active_rules += 1;
            }
        }

        let normalized_score = if active_rules > 0 {
            total_score / active_rules as f32
        } else {
            0.0
        };

        Ok(normalized_score.min(1.0))
    }

    pub fn should_reject(&self, score: f32) -> bool {
        score > self.threshold
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }
}

/// Правила детекции галлюцинаций
#[derive(Clone)]
pub enum DetectionRule {
    ConsistencyCheck,     // Проверка согласованности
    FactualAccuracy,      // Фактическая точность
    ContextRelevance,     // Релевантность контексту
    CommandValidity,      // Валидность команд
    ConfidenceValidation, // Валидация уверенности
}

impl DetectionRule {
    pub async fn apply(
        &self,
        analysis: &CommandAnalysis,
        known_commands: &HashSet<String>,
    ) -> Result<Option<f32>, DomainError> {
        match self {
            Self::ConsistencyCheck => self.check_consistency(analysis).await,
            Self::FactualAccuracy => self.check_factual_accuracy(analysis).await,
            Self::ContextRelevance => self.check_context_relevance(analysis).await,
            Self::CommandValidity => self.check_command_validity(analysis, known_commands).await,
            Self::ConfidenceValidation => self.validate_confidence(analysis).await,
        }
    }

    async fn check_consistency(
        &self,
        analysis: &CommandAnalysis,
    ) -> Result<Option<f32>, DomainError> {
        let mut score = 0.0;

        // Проверка внутренней согласованности
        if analysis.explanation.is_empty() && !analysis.suggestions.is_empty() {
            score += 0.3; // Предложения без объяснения
        }

        if analysis.confidence > 0.8 && analysis.risks.len() > 2 {
            score += 0.2; // Высокая уверенность при многих рисках
        }

        // Проверка противоречий между объяснением и рисками
        let explanation_lower = analysis.explanation.to_lowercase();
        if explanation_lower.contains("safe") && !analysis.risks.is_empty() {
            score += 0.2;
        }

        Ok(if score > 0.0 { Some(score) } else { None })
    }

    async fn check_factual_accuracy(
        &self,
        analysis: &CommandAnalysis,
    ) -> Result<Option<f32>, DomainError> {
        let mut score = 0.0;
        let explanation_lower = analysis.explanation.to_lowercase();

        // Проверка явно неверных утверждений
        let false_patterns = [
            "format the entire system",
            "delete all files",
            "reinstall operating system",
            "wipe disk",
        ];

        for pattern in &false_patterns {
            if explanation_lower.contains(pattern) {
                score += 0.5;
                break;
            }
        }

        // Проверка чрезмерно общих фраз
        let vague_patterns = [
            "this command does something",
            "it performs an action",
            "execute the program",
            "run the command",
        ];

        for pattern in &vague_patterns {
            if explanation_lower.contains(pattern) {
                score += 0.2;
                break;
            }
        }

        Ok(if score > 0.0 { Some(score) } else { None })
    }

    async fn check_context_relevance(
        &self,
        analysis: &CommandAnalysis,
    ) -> Result<Option<f32>, DomainError> {
        let mut score = 0.0;

        // Проверка релевантности контексту (базовые эвристики)
        if analysis.explanation.len() < 20 {
            score += 0.3; // Слишком короткое объяснение
        }

        if analysis.suggestions.iter().any(|s| s.len() < 5) {
            score += 0.2; // Слишком короткие предложения
        }

        // Проверка повторяющихся предложений
        let unique_suggestions: HashSet<_> = analysis.suggestions.iter().collect();
        if unique_suggestions.len() < analysis.suggestions.len() {
            score += 0.2; // Дублирующиеся предложения
        }

        Ok(if score > 0.0 { Some(score) } else { None })
    }

    async fn check_command_validity(
        &self,
        analysis: &CommandAnalysis,
        known_commands: &HashSet<String>,
    ) -> Result<Option<f32>, DomainError> {
        let mut score = 0.0;

        // Проверка валидности предлагаемых команд
        for alternative in &analysis.alternatives {
            let parts: Vec<&str> = alternative.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                if !known_commands.contains(&first_part.to_string()) {
                    score += 0.1; // Неизвестная команда
                }
            }
        }

        // Проверка опасных предложений
        let dangerous_patterns = [
            "rm -rf /",
            "dd if=/dev/zero",
            "mkfs",
            "fdisk",
            ":(){ :|:& };:",
        ];

        for suggestion in &analysis.suggestions {
            for pattern in &dangerous_patterns {
                if suggestion.contains(pattern) {
                    score += 0.5;
                    break;
                }
            }
        }

        Ok(if score > 0.0 { Some(score) } else { None })
    }

    async fn validate_confidence(
        &self,
        analysis: &CommandAnalysis,
    ) -> Result<Option<f32>, DomainError> {
        let mut score = 0.0;

        // Низкая уверенность при детализированном анализе
        if analysis.confidence < 0.3 && analysis.explanation.len() > 100 {
            score += 0.3;
        }

        // Высокая уверенность при пустом анализе
        if analysis.confidence > 0.8 && analysis.explanation.is_empty() {
            score += 0.4;
        }

        // Несоответствие между уверенностью и количеством рисков
        if analysis.confidence > 0.9 && analysis.risks.len() > 3 {
            score += 0.2;
        }

        Ok(if score > 0.0 { Some(score) } else { None })
    }
}

/// Расширенный детектор с машинным обучением (заглушка для будущей реализации)
pub struct MLEnhancedDetector {
    base_detector: HallucinationDetector,
    // В будущем: модель машинного обучения
}

impl MLEnhancedDetector {
    pub fn new() -> Self {
        Self {
            base_detector: HallucinationDetector::new(),
        }
    }

    pub async fn detect_with_ml(&self, analysis: &CommandAnalysis) -> Result<f32, DomainError> {
        // Базовая детекция
        let base_score = self.base_detector.detect(analysis).await?;

        // В будущем здесь будет интеграция с ML моделью
        // Пока возвращаем базовый счетчик
        Ok(base_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hallucination_detection_rules() {
        let detector = HallucinationDetector::new();

        // Тест с хорошим анализом
        let good_analysis = CommandAnalysis {
            explanation: "The 'ls -la' command lists all files in the current directory, including hidden files, with detailed information like permissions, ownership, size, and modification time.".to_string(),
            risks: vec![],
            suggestions: vec!["Use 'ls -lh' for human-readable file sizes".to_string()],
            confidence: 0.9,
            alternatives: vec!["ls -lh".to_string(), "exa -la".to_string()],
        };

        // Тест с плохим анализом (глубокие галлюцинации)
        let bad_analysis = CommandAnalysis {
            explanation: "This command will format your entire system and reinstall the operating system. It is completely safe to run.".to_string(),
            risks: vec![],
            suggestions: vec!["You should always run this command".to_string(), "It's perfectly safe".to_string()],
            confidence: 0.95,
            alternatives: vec!["rm -rf /".to_string()],
        };

        let good_score = detector.detect(&good_analysis).await.unwrap();
        let bad_score = detector.detect(&bad_analysis).await.unwrap();

        assert!(
            good_score < 0.5,
            "Good analysis should have low hallucination score"
        );
        assert!(
            bad_score > 0.5,
            "Bad analysis should have high hallucination score"
        );
        assert!(!detector.should_reject(good_score));
        assert!(detector.should_reject(bad_score));
    }

    #[tokio::test]
    async fn test_individual_rules() {
        let rule = DetectionRule::FactualAccuracy;

        // Тест с фактически неверным утверждением
        let false_analysis = CommandAnalysis {
            explanation: "This command will delete your entire operating system and reinstall everything from scratch.".to_string(),
            risks: vec![],
            suggestions: vec![],
            confidence: 0.8,
            alternatives: vec![],
        };

        let score = rule.apply(&false_analysis, &HashSet::new()).await.unwrap();
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }
}
