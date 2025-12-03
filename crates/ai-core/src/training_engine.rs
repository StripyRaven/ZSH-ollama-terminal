//! crates/ai-core/src/training_engine.rs
//! # Training Engine for Personalized Models
//! Система обучения персонализированных AI моделей с инкрементальным обучением.

use async_trait::async_trait;
use shared::{
    DeployedModel, DeploymentStatus, DomainError, HistoricalCommand, ModelEvaluation, TrainedModel,
    TrainingConfig, TrainingContext, TrainingData, TrainingEngine, UserFeedback,
};
use std::collections::HashMap;

/// Движок обучения персонализированных моделей
pub struct ModelTrainingEngine {
    data_collector: TrainingDataCollector,
    model_trainer: ModelTrainer,
    model_evaluator: ModelEvaluator,
    model_registry: ModelRegistry,
}

impl ModelTrainingEngine {
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
        // Предобработка данных
        let processed_data = self.data_collector.process(training_data).await?;

        // Обучение модели
        let model = self.model_trainer.train(processed_data, config).await?;

        // Оценка модели
        let _evaluation = self.model_evaluator.evaluate(&model).await?;

        // Регистрация модели
        self.model_registry.register(model.clone()).await?;

        Ok(model)
    }

    async fn evaluate_model(
        &self,
        model: &TrainedModel,
        _test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError> {
        self.model_evaluator
            .evaluate_with_data(model, _test_data)
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

/// Сборщик и процессор тренировочных данных
pub struct TrainingDataCollector;

impl TrainingDataCollector {
    pub fn new() -> Self {
        Self
    }

    /// Базовая обработка данных
    pub async fn process(&self, data: TrainingData) -> Result<ProcessedTrainingData, DomainError> {
        let anonymized_commands = self.anonymize_commands(&data.commands).await?;
        let filtered_feedback = self.filter_feedback(&data.user_feedback).await?;
        let features = self.extract_features(&anonymized_commands).await?; // ДО перемещения

        Ok(ProcessedTrainingData {
            commands: anonymized_commands, // теперь перемещаем
            user_feedback: filtered_feedback,
            context: data.context,
            features, // уже вычислено
        })
    }

    /// Обработка данных для конкретного пользователя
    pub async fn process_for_user(
        &self,
        data: TrainingData,
        user_id: u32,
    ) -> Result<ProcessedTrainingData, DomainError> {
        let mut processed = self.process(data).await?;

        // Добавляем пользовательские фичи
        processed.features.push(format!("user_{}", user_id));

        Ok(processed)
    }

    /// Анонимизация команд
    async fn anonymize_commands(
        &self,
        commands: &[HistoricalCommand],
    ) -> Result<Vec<HistoricalCommand>, DomainError> {
        let mut anonymized = Vec::new();

        for command in commands {
            // Обрабатываем output асинхронно
            let output = match &command.output {
                Some(o) => Some(self.anonymize_output(o).await?),
                None => None,
            };

            let anonymized_command = HistoricalCommand {
                command: self.anonymize_command_text(&command.command).await?,
                context: command.context.clone(),
                success: command.success,
                output, // используем обработанный output
            };
            anonymized.push(anonymized_command);
        }

        Ok(anonymized)
    }

    /// Анонимизация текста команды
    /// Анонимизация текста команды
    async fn anonymize_command_text(&self, command: &str) -> Result<String, DomainError> {
        let mut anonymized = command.to_string();

        // Замена путей пользователя
        if let Some(home_dir) = dirs::home_dir() {
            let home_str = home_dir.to_string_lossy();
            anonymized = anonymized.replace(&*home_str, "~");
        }

        // Замена общих путей пользователей (если не были заменены выше)
        let user_paths_regex = regex::Regex::new(r"(/home|/Users)/[^/\s]+").unwrap();
        anonymized = user_paths_regex.replace_all(&anonymized, "~").to_string();

        // Замена IP адресов
        let ip_regex = regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
        anonymized = ip_regex.replace_all(&anonymized, "[IP]").to_string();

        // Замена email адресов
        let email_regex =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        anonymized = email_regex.replace_all(&anonymized, "[EMAIL]").to_string();

        Ok(anonymized)
    }

    /// Анонимизация вывода команды
    async fn anonymize_output(&self, output: &str) -> Result<String, DomainError> {
        // Базовая анонимизация вывода
        let mut anonymized = output.to_string();

        // Удаление чувствительной информации из вывода
        let patterns = [
            r"/home/[^/\s]+",
            r"/Users/[^/\s]+",
            r"password\s*=\s*[^\s]+",
            r"api_key\s*=\s*[^\s]+",
            r"token\s*=\s*[^\s]+",
        ];

        for pattern in &patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            anonymized = regex.replace_all(&anonymized, "[REDACTED]").to_string();
        }

        // Также заменяем email и IP, как в команде
        let ip_regex = regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
        anonymized = ip_regex.replace_all(&anonymized, "[IP]").to_string();

        let email_regex =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        anonymized = email_regex.replace_all(&anonymized, "[EMAIL]").to_string();

        Ok(anonymized)
    }

    /// Фильтрация feedback данных
    async fn filter_feedback(
        &self,
        feedback: &[UserFeedback],
    ) -> Result<Vec<UserFeedback>, DomainError> {
        // Фильтрация некорректных feedback
        let filtered: Vec<UserFeedback> = feedback
            .iter()
            .filter(|f| f.rating > 0 && f.rating <= 5) // Валидный рейтинг
            .filter(|f| !f.suggestion.command.is_empty()) // Непустая команда
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Извлечение features из команд
    async fn extract_features(
        &self,
        commands: &[HistoricalCommand],
    ) -> Result<Vec<String>, DomainError> {
        let mut features = Vec::new();

        for command in commands {
            let parts: Vec<&str> = command.command.split_whitespace().collect();
            if let Some(executable) = parts.first() {
                features.push(format!("cmd_{}", executable));
            }

            // Добавляем фичи для флагов
            for part in &parts[1..] {
                if part.starts_with('-') {
                    features.push(format!("flag_{}", part.trim_start_matches('-')));
                }
            }

            // Фича для успешности выполнения
            if command.success {
                features.push("successful".to_string());
            } else {
                features.push("failed".to_string());
            }
        }

        // Уникальные фичи
        features.sort();
        features.dedup();

        Ok(features)
    }
}

/// Тренер моделей
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
        // Заглушка для обучения модели
        // В реальной реализации здесь будет интеграция с Ollama или другой ML системой

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

    /// Инкрементальное обучение
    pub async fn train_incremental(
        &self,
        new_data: ProcessedTrainingData,
        base_model: &TrainedModel,
    ) -> Result<TrainedModel, DomainError> {
        // Инкрементальное обучение на основе существующей модели
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

    /// Расчет точности (заглушка)
    async fn calculate_accuracy(&self, data: &ProcessedTrainingData) -> Result<f32, DomainError> {
        if data.commands.is_empty() {
            return Ok(0.0);
        }

        let successful_commands = data.commands.iter().filter(|c| c.success).count();
        let accuracy = successful_commands as f32 / data.commands.len() as f32;

        // Корректировка на основе feedback
        let feedback_score: f32 = data
            .user_feedback
            .iter()
            .map(|f| f.rating as f32 / 5.0)
            .sum::<f32>()
            / data.user_feedback.len().max(1) as f32;

        Ok((accuracy + feedback_score) / 2.0)
    }
}

/// Оценщик моделей
pub struct ModelEvaluator;

impl ModelEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub async fn evaluate(&self, model: &TrainedModel) -> Result<ModelEvaluation, DomainError> {
        // Базовая оценка модели
        Ok(ModelEvaluation {
            accuracy: model.accuracy,
            precision: model.accuracy * 0.9,  // Заглушка
            recall: model.accuracy * 0.85,    // Заглушка
            f1_score: model.accuracy * 0.875, // Заглушка
        })
    }

    pub async fn evaluate_with_data(
        &self,
        model: &TrainedModel,
        _test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError> {
        // Оценка модели на тестовых данных
        // В реальной реализации здесь будет сложная логика оценки
        self.evaluate(model).await
    }
}

/// Реестр моделей
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
        // В реальной реализации здесь будет развертывание модели
        // Пока возвращаем заглушку
        Ok(format!("local://{}/{}", model.name, model.version))
    }
}

/// Обработанные тренировочные данные
#[derive(Debug, Clone)]
pub struct ProcessedTrainingData {
    pub commands: Vec<HistoricalCommand>,
    pub user_feedback: Vec<UserFeedback>,
    pub context: TrainingContext,
    pub features: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let command = HistoricalCommand {
            command: "cat /home/user/documents/file.txt".to_string(),
            context: shared::CommandContext {
                working_directory: shared::ValidatedPath::new(std::path::Path::new(".")).unwrap(),
                user_id: 1000,
                environment: vec![],
            },
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
