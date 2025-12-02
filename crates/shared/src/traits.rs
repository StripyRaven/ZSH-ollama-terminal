//! crates/shared/src/traits.rs
//! # API Contracts for Clean Architecture
//! Порты (интерфейсы) с инверсией зависимостей для чистой архитектуры.
//!
//! Этот модуль определяет контракты (трейты) для всей системы.
//! Конкретные реализации находятся в других крейтах, что позволяет
//! легко подменять компоненты для тестирования или разных окружений.

use crate::error::DomainError;
use crate::CommandContext;
use crate::{
    states::{Analyzed, SafeToExecute, Unvalidated, Validated},
    Command,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Валидатор безопасности команд - порт для инфраструктуры
///
/// Отвечает за проверку безопасности входящих команд:
/// - Обнаружение инъекций
/// - Проверка на traversal атаки
/// - Валидация синтаксиса
///
/// # Реализации
/// - `security` крейт: реализация с правилами безопасности
/// - Тестовые моки: для unit-тестов
#[async_trait]
pub trait SecurityValidator: Send + Sync {
    /// Валидирует команду и переводит её в состояние `Validated`
    ///
    /// # Ошибки
    /// Возвращает `DomainError::Security` при нарушении правил безопасности
    /// Возвращает `DomainError::Validation` при синтаксических ошибках
    async fn validate_command(
        &self,
        command: Command<Unvalidated>,
    ) -> Result<Command<Validated>, DomainError>;

    /// Возвращает уровень безопасности валидатора
    ///
    /// Используется для аудита и логирования - какой валидатор обработал команду.
    fn get_security_level(&self) -> crate::states::SecurityLevel;

    /// Проверяет, может ли этот валидатор обработать данную команду
    ///
    /// Некоторые валидаторы специализируются на определённых типах команд
    /// (например, только файловые операции или только сетевые команды).
    fn can_handle_command(&self, command: &Command<Unvalidated>) -> bool;
}

/// AI анализатор команд - порт для AI инфраструктуры
///
/// Отвечает за анализ команд с помощью AI моделей:
/// - Объяснение команд
/// - Оценка рисков
/// - Предложение альтернатив
///
/// # Реализации
/// - `ai-core` крейт: интеграция с Ollama
/// - `training-engine` крейт: кастомные модели
#[async_trait]
pub trait CommandAnalyzer: Send + Sync {
    /// Анализирует команду с помощью AI модели
    ///
    /// Переводит команду из состояния `Validated` в `Analyzed`,
    /// добавляя данные анализа.
    ///
    /// # Асинхронность
    /// Анализ может занимать значительное время из-за запросов к AI API.
    async fn analyze_command(
        &self,
        command: Command<Validated>,
    ) -> Result<Command<Analyzed>, DomainError>;

    /// Получает предложения по улучшению или альтернативы для проанализированной команды
    ///
    /// Вызывается после анализа для предоставления пользователю рекомендаций.
    async fn get_suggestions(
        &self,
        analysis: &Command<Analyzed>,
    ) -> Result<Vec<CommandSuggestion>, DomainError>;

    /// Возвращает информацию о используемой AI модели
    ///
    /// Используется для логирования, отладки и отображения пользователю.
    fn get_model_info(&self) -> ModelInfo;
}

/// Система обучения моделей - порт для обучения
///
/// Отвечает за обучение и оценку AI моделей на исторических данных.
///
/// # Реализации
/// - `training-engine` крейт: обучение локальных моделей
/// - Внешние сервисы: облачное обучение
#[async_trait]
pub trait TrainingEngine: Send + Sync {
    /// Обучает новую модель на предоставленных данных
    ///
    /// # Параметры
    /// - `training_data`: исторические команды и фидбэк пользователей
    /// - `config`: гиперпараметры обучения
    async fn train_model(
        &self,
        training_data: TrainingData,
        config: TrainingConfig,
    ) -> Result<TrainedModel, DomainError>;

    /// Оценивает производительность модели на тестовых данных
    ///
    /// Используется для проверки качества модели перед деплоем.
    async fn evaluate_model(
        &self,
        model: &TrainedModel,
        test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError>;

    /// Деплоит модель для использования в продакшене
    ///
    /// После деплоя модель становится доступной для анализа команд.
    async fn deploy_model(&self, model: TrainedModel) -> Result<DeployedModel, DomainError>;
}

/// Безопасные файловые операции - порт для файловой системы
///
/// Обеспечивает безопасный доступ к файловой системе с валидацией путей.
/// Все пути автоматически проверяются на traversal атаки.
///
/// # Реализации
/// - `file-ops` крейт: безопасные файловые операции
/// - Тестовые моки: для изолированного тестирования
#[async_trait]
pub trait FileOperations: Send + Sync {
    /// Читает содержимое файла по безопасному пути
    ///
    /// # Безопасность
    /// Путь автоматически проверяется на traversal атаки.
    async fn read_file(&self, path: crate::ValidatedPath<'_>) -> Result<FileContent, DomainError>;

    /// Записывает содержимое в файл по безопасному пути
    ///
    /// # Безопасность
    /// Проверяет разрешения на запись и квоты диска.
    async fn write_file(
        &self,
        path: crate::ValidatedPath<'_>,
        content: FileContent,
    ) -> Result<(), DomainError>;

    /// Перечисляет содержимое директории по безопасному пути
    ///
    /// # Безопасность
    /// Ограничивает глубину рекурсии и размер вывода.
    async fn list_directory(
        &self,
        path: crate::ValidatedPath<'_>,
    ) -> Result<Vec<FileInfo>, DomainError>;

    /// Создаёт безопасный путь с проверками
    ///
    /// Используется для конвертации пользовательского ввода в безопасный путь.
    fn create_validated_path(&self, path: &str) -> Result<crate::ValidatedPath<'_>, DomainError>;
}

/// Кросс-платформенные адаптеры - порт для ОС
///
/// Абстрагирует операции, зависящие от операционной системы:
/// - Получение информации о системе
/// - Безопасное выполнение команд
/// - Определение типа shell
///
/// # Реализации
/// - `platform` крейт: реализации для Linux, macOS, Windows
/// - `terminal-integration` крейт: интеграция с терминалом
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Возвращает информацию о системе
    ///
    /// Используется для контекстно-зависимого анализа и логирования.
    async fn get_system_info(&self) -> Result<SystemInfo, DomainError>;

    /// Безопасно выполняет команду в изолированном окружении
    ///
    /// # Безопасность
    /// Команда выполняется с ограниченными разрешениями в песочнице.
    /// Используется только для команд в состоянии `SafeToExecute`.
    async fn execute_command(
        &self,
        command: Command<SafeToExecute>,
    ) -> Result<CommandOutput, DomainError>;

    /// Определяет тип shell, используемый пользователем
    ///
    /// Важно для правильного форматирования команд и подсказок.
    fn get_shell_type(&self) -> ShellType;

    /// Возвращает безопасный путь к домашней директории пользователя
    ///
    /// Используется для операций с файлами пользователя.
    fn get_user_home(&self) -> Result<crate::ValidatedPath<'_>, DomainError>;
}

/// Типы данных для API контрактов

/// Предложение команды от AI анализатора
///
/// Содержит альтернативную команду с объяснением и оценкой безопасности.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    /// Текст предлагаемой команды
    pub command: String,
    /// Объяснение, почему эта команда лучше или безопаснее
    pub explanation: String,
    /// Уверенность AI в предложении (0.0 - 1.0)
    pub confidence: f32,
    /// Уровень безопасности предложенной команды
    pub safety_level: crate::states::SecurityLevel,
}

/// Информация об AI модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Имя модели (например, "llama2", "gpt-4")
    pub name: String,
    /// Версия модели
    pub version: String,
    /// Возможности модели (токенизация, языки, домены)
    pub capabilities: Vec<String>,
    /// Максимальное количество токенов в контексте
    pub max_tokens: u32,
}

/// Данные для обучения AI модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    /// Исторические команды пользователей
    pub commands: Vec<HistoricalCommand>,
    /// Фидбэк пользователей на предложения AI
    pub user_feedback: Vec<UserFeedback>,
    /// Контекст обучения (пользователь, система, время)
    pub context: TrainingContext,
}

/// Историческая команда для обучения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCommand {
    /// Текст команды
    pub command: String,
    /// Контекст выполнения
    pub context: CommandContext,
    /// Успешно ли выполнилась команда
    pub success: bool,
    /// Вывод команды (если доступен)
    pub output: Option<String>,
}

/// Фидбэк пользователя на предложение AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// Предложение, на которое дан фидбэк
    pub suggestion: CommandSuggestion,
    /// Принял ли пользователь предложение
    pub accepted: bool,
    /// Рейтинг предложения (1-5)
    pub rating: u8,
}

/// Контекст обучения модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingContext {
    /// ID пользователя, чьи данные используются
    pub user_id: u32,
    /// Информация о системе в момент обучения
    pub system_info: SystemInfo,
    /// Время сбора данных
    pub timestamp: std::time::SystemTime,
}

/// Информация о системе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Операционная система
    pub os: String,
    /// Используемый shell
    pub shell: String,
    /// Архитектура процессора
    pub architecture: String,
    /// Объем памяти в MB
    pub memory_mb: u64,
}

/// Конфигурация обучения модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Количество эпох обучения
    pub epochs: u32,
    /// Скорость обучения
    pub learning_rate: f32,
    /// Размер батча
    pub batch_size: u32,
    /// Доля данных для валидации (0.0 - 1.0)
    pub validation_split: f32,
}

/// Обученная AI модель
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainedModel {
    /// Имя модели
    pub name: String,
    /// Версия модели
    pub version: String,
    /// Точность модели на валидационных данных
    pub accuracy: f32,
    /// Метаданные обучения
    pub metadata: ModelMetadata,
}

/// Метаданные обучения модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Размер данных обучения
    pub training_data_size: usize,
    /// Длительность обучения
    pub training_duration: std::time::Duration,
    /// Используемые фичи (признаки)
    pub features: Vec<String>,
}

/// Оценка качества модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvaluation {
    /// Общая точность
    pub accuracy: f32,
    /// Точность (precision)
    pub precision: f32,
    /// Полнота (recall)
    pub recall: f32,
    /// F1-мера
    pub f1_score: f32,
}

/// Развернутая модель для использования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedModel {
    /// Модель
    pub model: TrainedModel,
    /// Конечная точка API
    pub endpoint: String,
    /// Статус развертывания
    pub status: DeploymentStatus,
}

/// Статус развертывания модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Активна и принимает запросы
    Active,
    /// В режиме ожидания (горячий резерв)
    Standby,
    /// Обновляется
    Updating,
    /// Сломана, требует вмешательства
    Failed,
}

/// Содержимое файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    /// Бинарные данные
    pub data: Vec<u8>,
    /// Кодировка текстовых файлов
    pub encoding: String,
    /// Размер в байтах
    pub size: u64,
}

/// Информация о файле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Имя файла
    pub name: String,
    /// Размер в байтах
    pub size: u64,
    /// Является ли директорией
    pub is_directory: bool,
    /// Права доступа в строковом формате
    pub permissions: String,
}

/// Результат выполнения команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Стандартный вывод
    pub stdout: String,
    /// Стандартный вывод ошибок
    pub stderr: String,
    /// Код завершения
    pub exit_code: i32,
    /// Время выполнения
    pub duration: std::time::Duration,
}

/// Тип shell пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellType {
    /// Z shell
    Zsh,
    /// Bourne Again shell
    Bash,
    /// Friendly interactive shell
    Fish,
    /// PowerShell
    PowerShell,
    /// Неизвестный shell
    Unknown,
}
