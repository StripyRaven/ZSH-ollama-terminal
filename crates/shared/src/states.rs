//! crates/shared/src/states.rs
//! # Typestate System with Compile-Time Guarantees
//! Система состояний с гарантиями времени компиляции для правильной последовательности операций.

use serde::{Deserialize, Serialize};

/// Иерархия состояний команды с строгой последовательностью
pub struct Unvalidated {
    pub security_checks_pending: bool,
}

pub struct Validated {
    pub security_level: SecurityLevel,
    pub validation_timestamp: std::time::SystemTime,
}

pub struct Analyzed {
    pub analysis_id: uuid::Uuid,
    pub confidence_score: f32,
    pub model_version: String,
}

pub struct SafeToExecute {
    pub safety_guarantees: SafetyGuarantees,
    pub execution_context: ExecutionContext,
}

/// Гарантии безопасности для выполнения команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGuarantees {
    pub no_destructive_operations: bool,
    pub no_network_exfiltration: bool,
    pub no_privacy_violations: bool,
    pub sandboxed_environment: bool,
}

/// Контекст выполнения для безопасного исполнения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub working_directory: String,
    pub user_permissions: UserPermissions,
    pub environment_constraints: EnvironmentConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub can_write_to_home: bool,
    pub can_access_network: bool,
    pub can_execute_system_commands: bool,
    pub can_modify_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConstraints {
    pub max_memory_mb: u32,
    pub timeout_seconds: u32,
    pub network_access: bool,
    pub disk_quota_mb: u32,
}

/// Уровни безопасности с exhaustive matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    Untrusted,
    User,
    Trusted,
    System,
}

// Компилятор гарантирует обработку всех вариантов
impl SecurityLevel {
    pub fn can_execute_destructive(&self) -> bool {
        match self {
            SecurityLevel::Untrusted => false,
            SecurityLevel::User => false,
            SecurityLevel::Trusted => true,
            SecurityLevel::System => true,
        }
    }

    pub fn can_access_network(&self) -> bool {
        match self {
            SecurityLevel::Untrusted => false,
            SecurityLevel::User => true,
            SecurityLevel::Trusted => true,
            SecurityLevel::System => true,
        }
    }
}
