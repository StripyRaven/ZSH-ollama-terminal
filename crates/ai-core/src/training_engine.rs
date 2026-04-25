//! crates/ai-core/src/training_engine.rs
//! # Training Engine for Personalized Models
//! Система обучения персонализированных AI моделей с инкрементальным обучением.
//!
//! ## Улучшения в версии 1.1.0
//! - Статические регулярные выражения (lazy_static)
//! - Убраны лишние `async` там, где они не нужны
//! - Исправлены тесты
//! - Добавлена документация для внутренних методов

use async_trait::async_trait;
use once_cell::sync::Lazy; // Добавлено для статических Regex
use regex::Regex;
use shared::{
    DeployedModel, DeploymentStatus, DomainError, HistoricalCommand, ModelEvaluation, TrainedModel,
    TrainingConfig, TrainingContext, TrainingData, TrainingEngine, UserFeedback,
};
use std::collections::HashMap;

// =============================================================================
// Статические регулярные выражения (создаются один раз)
// =============================================================================

static HOME_DIR: Lazy<Option<String>> =
    Lazy::new(|| dirs::home_dir().map(|p| p.to_string_lossy().to_string()));

static USER_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(/home|/Users)/[^/\s]+").unwrap());
static IP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap());
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());
static SENSITIVE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"/home/[^/\s]+").unwrap(),
        Regex::new(r"/Users/[^/\s]+").unwrap(),
        Regex::new(r"password\s*=\s*[^\s]+").unwrap(),
        Regex::new(r"api_key\s*=\s*[^\s]+").unwrap(),
        Regex::new(r"token\s*=\s*[^\s]+").unwrap(),
    ]
});

// =============================================================================
// Основной движок обучения
// =============================================================================

/// Движок обучения персонализированных моделей
pub struct ModelTrainingEngine {
    data_collector: TrainingDataCollector,
    model_trainer: ModelTrainer,
    model_evaluator: ModelEvaluator,
    model_registry: ModelRegistry,
}

impl ModelTrainingEngine {
    /// Создаёт новый экземпляр движка обучения
    pub fn new() -> Self {
        Self {
            data_collector: TrainingDataCollector::new(),
            model_trainer: ModelTrainer::new(),
            model_evaluator: ModelEvaluator::new(),
            model_registry: ModelRegistry::new(),
        }
    }

    /// Инкрементальное обучение на новых данных
    pub async fn train_incremental(
        &self,
        new_data: TrainingData,
        base_model: &TrainedModel,
    ) -> Result<TrainedModel, DomainError> {
        let processed_data = self.data_collector.process(new_data).await?;
        let updated_model = self
            .model_trainer
            .train_incremental(processed_data, base_model)
            .await?;
        Ok(updated_model)
    }

    /// Адаптация модели под конкретного пользователя
    pub async fn personalize_model(
        &self,
        user_data: TrainingData,
        _base_model: &TrainedModel,
        user_id: u32,
    ) -> Result<TrainedModel, DomainError> {
        let personalized_config = TrainingConfig {
            epochs: 10, // Меньше эпох для персонализации
            learning_rate: 0.001,
            batch_size: 32,
            validation_split: 0.2,
        };

        let processed_data = self
            .data_collector
            .process_for_user(user_data, user_id)
            .await?;
        let personalized_model = self
            .model_trainer
            .train(processed_data, personalized_config)
            .await?;
        Ok(personalized_model)
    }
}

#[async_trait]
impl TrainingEngine for ModelTrainingEngine {
    async fn train_model(
        &self,
        training_data: TrainingData,
        config: TrainingConfig,
    ) -> Result<TrainedModel, DomainError> {
        let processed_data = self.data_collector.process(training_data).await?;
        let model = self.model_trainer.train(processed_data, config).await?;
        let _evaluation = self.model_evaluator.evaluate(&model).await?;
        self.model_registry.register(model.clone()).await?;
        Ok(model)
    }

    async fn evaluate_model(
        &self,
        model: &TrainedModel,
        test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError> {
        self.model_evaluator
            .evaluate_with_data(model, test_data)
            .await
    }

    async fn deploy_model(&self, model: TrainedModel) -> Result<DeployedModel, DomainError> {
        let endpoint = self.model_registry.deploy(&model).await?;
        Ok(DeployedModel {
            model,
            endpoint,
            status: DeploymentStatus::Active,
        })
    }
}

// =============================================================================
// Сборщик и процессор тренировочных данных
// =============================================================================

/// Сборщик и процессор тренировочных данных с анонимизацией
pub struct TrainingDataCollector;

impl TrainingDataCollector {
    pub fn new() -> Self {
        Self
    }

    /// Базовая обработка данных: анонимизация, фильтрация, извлечение признаков
    pub async fn process(&self, data: TrainingData) -> Result<ProcessedTrainingData, DomainError> {
        let anonymized_commands = self.anonymize_commands(&data.commands).await?;
        let filtered_feedback = self.filter_feedback(&data.user_feedback);
        let features = self.extract_features(&anonymized_commands);
        Ok(ProcessedTrainingData {
            commands: anonymized_commands,
            user_feedback: filtered_feedback,
            context: data.context,
            features,
        })
    }

    /// Обработка данных для конкретного пользователя (добавляет user_id как признак)
    pub async fn process_for_user(
        &self,
        data: TrainingData,
        user_id: u32,
    ) -> Result<ProcessedTrainingData, DomainError> {
        let mut processed = self.process(data).await?;
        processed.features.push(format!("user_{}", user_id));
        Ok(processed)
    }

    // -------------------------------------------------------------------------
    // Вспомогательные методы (синхронные, без async)
    // -------------------------------------------------------------------------

    /// Анонимизация списка команд
    async fn anonymize_commands(
        &self,
        commands: &[HistoricalCommand],
    ) -> Result<Vec<HistoricalCommand>, DomainError> {
        let mut anonymized = Vec::with_capacity(commands.len());
        for command in commands {
            let output = match &command.output {
                Some(o) => Some(self.anonymize_output(o)?),
                None => None,
            };
            anonymized.push(HistoricalCommand {
                command: self.anonymize_command_text(&command.command)?,
                context: command.context.clone(),
                success: command.success,
                output,
            });
        }
        Ok(anonymized)
    }

    /// Анонимизация текста команды (замена путей, IP, email)
    fn anonymize_command_text(&self, command: &str) -> Result<String, DomainError> {
        let mut anonymized = command.to_string();

        // Замена домашней директории
        if let Some(home) = HOME_DIR.as_ref() {
            anonymized = anonymized.replace(home, "~");
        }

        // Замена общих путей пользователей
        anonymized = USER_PATH_REGEX.replace_all(&anonymized, "~").to_string();

        // Замена IP адресов
        anonymized = IP_REGEX.replace_all(&anonymized, "[IP]").to_string();

        // Замена email адресов
        anonymized = EMAIL_REGEX.replace_all(&anonymized, "[EMAIL]").to_string();

        Ok(anonymized)
    }

    /// Анонимизация вывода команды
    fn anonymize_output(&self, output: &str) -> Result<String, DomainError> {
        let mut anonymized = output.to_string();

        // Применяем все чувствительные паттерны
        for pattern in SENSITIVE_PATTERNS.iter() {
            anonymized = pattern.replace_all(&anonymized, "[REDACTED]").to_string();
        }

        // Дополнительно заменяем IP и email
        anonymized = IP_REGEX.replace_all(&anonymized, "[IP]").to_string();
        anonymized = EMAIL_REGEX.replace_all(&anonymized, "[EMAIL]").to_string();

        Ok(anonymized)
    }

    /// Фильтрация пользовательских отзывов (валидация рейтинга и команды)
    fn filter_feedback(&self, feedback: &[UserFeedback]) -> Vec<UserFeedback> {
        feedback
            .iter()
            .filter(|f| (1..=5).contains(&f.rating) && !f.suggestion.command.is_empty())
            .cloned()
            .collect()
    }

    /// Извлечение признаков из команд (исполняемые файлы, флаги, успешность)
    fn extract_features(&self, commands: &[HistoricalCommand]) -> Vec<String> {
        let mut features = Vec::new();
        for command in commands {
            let parts: Vec<&str> = command.command.split_whitespace().collect();
            if let Some(executable) = parts.first() {
                features.push(format!("cmd_{}", executable));
            }
            for part in &parts[1..] {
                if part.starts_with('-') {
                    features.push(format!("flag_{}", part.trim_start_matches('-')));
                }
            }
            features.push(if command.success {
                "successful".to_string()
            } else {
                "failed".to_string()
            });
        }
        features.sort();
        features.dedup();
        features
    }
}

// =============================================================================
// Тренер моделей
// =============================================================================

/// Тренер моделей (заглушка для реального обучения)
pub struct ModelTrainer;

impl ModelTrainer {
    pub fn new() -> Self {
        Self
    }

    /// Базовое обучение модели
    pub async fn train(
        &self,
        data: ProcessedTrainingData,
        _config: TrainingConfig,
    ) -> Result<TrainedModel, DomainError> {
        let accuracy = self.calculate_accuracy(&data).await?;
        Ok(TrainedModel {
            name: "personalized_command_analyzer".to_string(),
            version: "1.0".to_string(),
            accuracy,
            metadata: shared::ModelMetadata {
                training_data_size: data.commands.len(),
                training_duration: std::time::Duration::from_secs(3600),
                features: data.features,
            },
        })
    }

    /// Инкрементальное обучение на основе существующей модели
    pub async fn train_incremental(
        &self,
        new_data: ProcessedTrainingData,
        base_model: &TrainedModel,
    ) -> Result<TrainedModel, DomainError> {
        let new_accuracy = self.calculate_accuracy(&new_data).await?;
        let combined_accuracy = (base_model.accuracy + new_accuracy) / 2.0;
        Ok(TrainedModel {
            name: base_model.name.clone(),
            version: format!("{}.1", base_model.version),
            accuracy: combined_accuracy,
            metadata: shared::ModelMetadata {
                training_data_size: base_model.metadata.training_data_size
                    + new_data.commands.len(),
                training_duration: base_model.metadata.training_duration
                    + std::time::Duration::from_secs(1800),
                features: base_model.metadata.features.clone(), // В реальности объединение фич
            },
        })
    }

    /// Расчет точности на основе успешных команд и отзывов
    async fn calculate_accuracy(&self, data: &ProcessedTrainingData) -> Result<f32, DomainError> {
        if data.commands.is_empty() {
            return Ok(0.0);
        }
        let successful_commands = data.commands.iter().filter(|c| c.success).count();
        let accuracy = successful_commands as f32 / data.commands.len() as f32;
        let feedback_score: f32 = data
            .user_feedback
            .iter()
            .map(|f| f.rating as f32 / 5.0)
            .sum::<f32>()
            / data.user_feedback.len().max(1) as f32;
        Ok((accuracy + feedback_score) / 2.0)
    }
}

// =============================================================================
// Оценщик моделей
// =============================================================================

/// Оценщик моделей (заглушка)
pub struct ModelEvaluator;

impl ModelEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub async fn evaluate(&self, model: &TrainedModel) -> Result<ModelEvaluation, DomainError> {
        Ok(ModelEvaluation {
            accuracy: model.accuracy,
            precision: model.accuracy * 0.9,
            recall: model.accuracy * 0.85,
            f1_score: model.accuracy * 0.875,
        })
    }

    pub async fn evaluate_with_data(
        &self,
        model: &TrainedModel,
        _test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError> {
        self.evaluate(model).await
    }
}

// =============================================================================
// Реестр моделей
// =============================================================================

/// Реестр для хранения и развёртывания моделей
pub struct ModelRegistry {
    models: tokio::sync::Mutex<HashMap<String, TrainedModel>>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(&self, model: TrainedModel) -> Result<(), DomainError> {
        let key = format!("{}:{}", model.name, model.version);
        let mut models = self.models.lock().await;
        models.insert(key, model);
        Ok(())
    }

    pub async fn get(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<TrainedModel>, DomainError> {
        let key = format!("{}:{}", name, version);
        let models = self.models.lock().await;
        Ok(models.get(&key).cloned())
    }

    pub async fn deploy(&self, model: &TrainedModel) -> Result<String, DomainError> {
        // Заглушка: имитация развёртывания
        Ok(format!("local://{}/{}", model.name, model.version))
    }
}

// =============================================================================
// Вспомогательные типы
// =============================================================================

/// Обработанные тренировочные данные (промежуточный формат)
#[derive(Debug, Clone)]
pub struct ProcessedTrainingData {
    pub commands: Vec<HistoricalCommand>,
    pub user_feedback: Vec<UserFeedback>,
    pub context: TrainingContext,
    pub features: Vec<String>,
}

// =============================================================================
// Тесты
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{CommandContext, Environment, ValidatedPath};

    #[tokio::test]
    async fn test_training_engine_creation() {
        let engine = ModelTrainingEngine::new();
        let training_data = TrainingData {
            commands: vec![],
            user_feedback: vec![],
            context: TrainingContext {
                user_id: 1000,
                system_info: shared::SystemInfo {
                    os: "Linux".to_string(),
                    shell: "zsh".to_string(),
                    architecture: "x86_64".to_string(),
                    memory_mb: 8192,
                },
                timestamp: std::time::SystemTime::now(),
            },
        };
        let config = TrainingConfig {
            epochs: 5,
            learning_rate: 0.001,
            batch_size: 32,
            validation_split: 0.2,
        };
        let result = engine.train_model(training_data, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_data_anonymization() {
        let collector = TrainingDataCollector::new();

        // Создаём контекст выполнения команды (необходим для HistoricalCommand)
        let working_dir = ValidatedPath::new(".").unwrap();
        let context = CommandContext {
            working_directory: working_dir,
            user_id: 1000,
            environment: Environment::new(),
        };

        let command = HistoricalCommand {
            command: "cat /home/user/documents/file.txt".to_string(),
            context,
            success: true,
            output: Some("sensitive data user@example.com 192.168.1.1".to_string()),
        };

        let processed = collector.anonymize_commands(&[command]).await.unwrap();
        let anonymized_command = &processed[0];

        assert!(!anonymized_command.command.contains("/home/user"));
        assert!(!anonymized_command
            .output
            .as_ref()
            .unwrap()
            .contains("user@example.com"));
        assert!(!anonymized_command
            .output
            .as_ref()
            .unwrap()
            .contains("192.168.1.1"));
    }
}
