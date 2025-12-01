//! crates/shared/src/error.rs
//! # Comprehensive Error System with Exhaustive Matching

//! crates/shared/src/error.rs
//! # Comprehensive Error System with Exhaustive Matching
//! Полная система ошибок с гарантированной обработкой всех вариантов.

#[allow(unused_imports)] // TODO chch
use crate::SecurityLevel;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Иерархия ошибок домена с exhaustive matching гарантиями
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    Validation(ValidationError),
    Security(SecurityError),
    Analysis(AnalysisError),
    Io(IoError),
    Configuration(ConfigurationError),
    Training(TrainingError),
    Network(NetworkError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub reason: String,
    pub command: String,
    pub field: Option<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityError {
    pub violation: SecurityViolation,
    pub severity: SecuritySeverity,
    pub context: SecurityContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolation {
    PathTraversalAttempt,
    CommandInjectionAttempt,
    PermissionEscalation,
    DataExfiltration,
    ResourceExhaustion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: u32,
    pub working_directory: String,
    pub attempted_operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisError {
    pub model: String,
    pub error_type: AnalysisErrorType,
    pub details: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisErrorType {
    ModelUnavailable,
    InvalidResponse,
    Timeout,
    HallucinationDetected,
    ConfidenceTooLow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoError {
    pub path: String,
    pub operation: IoOperation,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoOperation {
    Read,
    Write,
    Delete,
    List,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationError {
    pub key: String,
    pub expected_type: String,
    pub actual_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingError {
    pub model_name: String,
    pub training_data_size: usize,
    pub error: TrainingErrorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingErrorType {
    InsufficientData,
    ModelConvergenceFailed,
    ValidationAccuracyTooLow,
    MemoryExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkError {
    pub endpoint: String,
    pub operation: NetworkOperation,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    Request,
    Response,
    Connection,
    Timeout,
}

// Реализация Display для всех ошибок
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Validation(e) => {
                write!(f, "Validation error: {} - {}", e.reason, e.command)
            }
            DomainError::Security(e) => {
                write!(f, "Security error: {:?} - {:?}", e.violation, e.severity)
            }
            DomainError::Analysis(e) => write!(f, "Analysis error: {} - {}", e.model, e.details),
            DomainError::Io(e) => write!(f, "IO error: {:?} on {}", e.operation, e.path),
            DomainError::Configuration(e) => write!(
                f,
                "Configuration error: {} expected {} got {}",
                e.key, e.expected_type, e.actual_value
            ),
            DomainError::Training(e) => {
                write!(f, "Training error: {} - {:?}", e.model_name, e.error)
            }
            DomainError::Network(e) => {
                write!(f, "Network error: {} - {:?}", e.endpoint, e.operation)
            }
        }
    }
}

impl std::error::Error for DomainError {}

// Преобразования из стандартных ошибок
impl From<std::io::Error> for DomainError {
    fn from(error: std::io::Error) -> Self {
        DomainError::Io(IoError {
            path: "unknown".to_string(),
            operation: IoOperation::Read, // или определить по коду ошибки
            source: Some(error.to_string()),
        })
    }
}

// Zero-cost преобразования между ошибками
impl From<ValidationError> for DomainError {
    fn from(error: ValidationError) -> Self {
        DomainError::Validation(error)
    }
}

impl From<SecurityError> for DomainError {
    fn from(error: SecurityError) -> Self {
        DomainError::Security(error)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Этот тест гарантирует, что все варианты DomainError обрабатываются
    #[test]
    fn test_exhaustive_error_matching() {
        let errors = vec![
            DomainError::Validation(ValidationError {
                reason: "test".to_string(),
                command: "test".to_string(),
                field: None,
                constraints: vec![],
            }),
            DomainError::Security(SecurityError {
                violation: SecurityViolation::PathTraversalAttempt,
                severity: SecuritySeverity::High,
                context: SecurityContext {
                    user_id: 1000,
                    working_directory: "/test".to_string(),
                    attempted_operation: "test".to_string(),
                },
            }),
            DomainError::Analysis(AnalysisError {
                model: "test".to_string(),
                error_type: AnalysisErrorType::ModelUnavailable,
                details: "test".to_string(),
                suggestion: None,
            }),
            DomainError::Io(IoError {
                path: "test".to_string(),
                operation: IoOperation::Read,
                source: None,
            }),
            DomainError::Configuration(ConfigurationError {
                key: "test".to_string(),
                expected_type: "string".to_string(),
                actual_value: "123".to_string(),
            }),
            DomainError::Training(TrainingError {
                model_name: "test".to_string(),
                training_data_size: 100,
                error: TrainingErrorType::InsufficientData,
            }),
            DomainError::Network(NetworkError {
                endpoint: "test".to_string(),
                operation: NetworkOperation::Request,
                status_code: None,
            }),
        ];

        // Компилятор гарантирует, что мы обработали все варианты
        for error in errors {
            match error {
                DomainError::Validation(_) => assert!(true),
                DomainError::Security(_) => assert!(true),
                DomainError::Analysis(_) => assert!(true),
                DomainError::Io(_) => assert!(true),
                DomainError::Configuration(_) => assert!(true),
                DomainError::Training(_) => assert!(true),
                DomainError::Network(_) => assert!(true),
            }
        }
    }

    #[test]
    fn test_security_level_exhaustive_matching() {
        let levels = vec![
            SecurityLevel::Untrusted,
            SecurityLevel::User,
            SecurityLevel::Trusted,
            SecurityLevel::System,
        ];

        for level in levels {
            match level {
                SecurityLevel::Untrusted => assert!(!level.can_execute_destructive()),
                SecurityLevel::User => assert!(!level.can_execute_destructive()),
                SecurityLevel::Trusted => assert!(level.can_execute_destructive()),
                SecurityLevel::System => assert!(level.can_execute_destructive()),
            }
        }
    }
}
