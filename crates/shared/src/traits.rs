//! crates/shared/src/traits.rs
//! # API Contracts for Clean Architecture
//! Порты (интерфейсы) с инверсией зависимостей для чистой архитектуры.

use crate::error::DomainError;
use crate::CommandContext;
use crate::{
    states::{Analyzed, SafeToExecute, Unvalidated, Validated},
    Command,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Валидатор безопасности команд - порт для инфраструктуры
#[async_trait]
pub trait SecurityValidator: Send + Sync {
    async fn validate_command(
        &self,
        command: Command<Unvalidated>,
    ) -> Result<Command<Validated>, DomainError>;

    fn get_security_level(&self) -> crate::states::SecurityLevel;

    fn can_handle_command(&self, command: &Command<Unvalidated>) -> bool;
}

/// AI анализатор команд - порт для AI инфраструктуры
#[async_trait]
pub trait CommandAnalyzer: Send + Sync {
    async fn analyze_command(
        &self,
        command: Command<Validated>,
    ) -> Result<Command<Analyzed>, DomainError>;

    async fn get_suggestions(
        &self,
        analysis: &Command<Analyzed>,
    ) -> Result<Vec<CommandSuggestion>, DomainError>;

    fn get_model_info(&self) -> ModelInfo;
}

/// Система обучения моделей - порт для обучения
#[async_trait]
pub trait TrainingEngine: Send + Sync {
    async fn train_model(
        &self,
        training_data: TrainingData,
        config: TrainingConfig,
    ) -> Result<TrainedModel, DomainError>;

    async fn evaluate_model(
        &self,
        model: &TrainedModel,
        test_data: &TrainingData,
    ) -> Result<ModelEvaluation, DomainError>;

    async fn deploy_model(&self, model: TrainedModel) -> Result<DeployedModel, DomainError>;
}

/// Безопасные файловые операции - порт для файловой системы
#[async_trait]
pub trait FileOperations: Send + Sync {
    async fn read_file(&self, path: crate::ValidatedPath<'_>) -> Result<FileContent, DomainError>;

    async fn write_file(
        &self,
        path: crate::ValidatedPath<'_>,
        content: FileContent,
    ) -> Result<(), DomainError>;

    async fn list_directory(
        &self,
        path: crate::ValidatedPath<'_>,
    ) -> Result<Vec<FileInfo>, DomainError>;

    fn create_validated_path(&self, path: &str) -> Result<crate::ValidatedPath<'_>, DomainError>;
}

/// Кросс-платформенные адаптеры - порт для ОС
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    async fn get_system_info(&self) -> Result<SystemInfo, DomainError>;

    async fn execute_command(
        &self,
        command: Command<SafeToExecute>,
    ) -> Result<CommandOutput, DomainError>;

    fn get_shell_type(&self) -> ShellType;

    fn get_user_home(&self) -> Result<crate::ValidatedPath<'_>, DomainError>;
}

/// Типы данных для API контрактов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub explanation: String,
    pub confidence: f32,
    pub safety_level: crate::states::SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub commands: Vec<HistoricalCommand>,
    pub user_feedback: Vec<UserFeedback>,
    pub context: TrainingContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCommand {
    pub command: String,
    pub context: CommandContext,
    pub success: bool,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub suggestion: CommandSuggestion,
    pub accepted: bool,
    pub rating: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingContext {
    pub user_id: u32,
    pub system_info: SystemInfo,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub shell: String,
    pub architecture: String,
    pub memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub epochs: u32,
    pub learning_rate: f32,
    pub batch_size: u32,
    pub validation_split: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainedModel {
    pub name: String,
    pub version: String,
    pub accuracy: f32,
    pub metadata: ModelMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub training_data_size: usize,
    pub training_duration: std::time::Duration,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvaluation {
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedModel {
    pub model: TrainedModel,
    pub endpoint: String,
    pub status: DeploymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Active,
    Standby,
    Updating,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub data: Vec<u8>,
    pub encoding: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellType {
    Zsh,
    Bash,
    Fish,
    PowerShell,
    Unknown,
}
