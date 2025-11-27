//! crates/shared/src/lib.rs
//! # Shared Types and Traits with Compile-Time Guarantees
//! Общие типы, трейты и системы ошибок для всей архитектуры.

pub mod error;
pub mod serialization;
pub mod states;
pub mod traits;

// Re-export для удобства использования
pub use error::DomainError;
pub use serialization::SerializedCommand;
pub use states::*;
pub use traits::*;

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command<S = states::Unvalidated> {
    pub raw: String,
    pub parts: Vec<String>,
    pub context: CommandContext,
    pub state: PhantomData<S>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    pub working_directory: ValidatedPath<'static>,
    pub user_id: u32,
    pub environment: Vec<(String, String)>,
}

impl<S> Command<S> {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    pub fn context(&self) -> &CommandContext {
        &self.context
    }
}

impl Command<states::Unvalidated> {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        let parts = raw.split_whitespace().map(|s| s.to_string()).collect();

        // Базовые проверки при создании
        if raw.len() > 4096 {
            return Err(DomainError::Validation(error::ValidationError {
                reason: "Command too long".to_string(),
                command: raw,
                field: Some("raw".to_string()),
                constraints: vec!["max_length: 4096".to_string()],
            }));
        }

        let current_dir = std::env::current_dir().map_err(|e| {
            DomainError::Io(error::IoError {
                path: ".".to_string(),
                operation: error::IoOperation::Read,
                source: Some(e.to_string()),
            })
        })?;

        let context = CommandContext {
            working_directory: ValidatedPath::new(current_dir)?,
            user_id: unsafe { libc::getuid() },
            environment: std::env::vars().collect(),
        };

        Ok(Self {
            raw,
            parts,
            context,
            state: PhantomData,
        })
    }

    /// Только невалидированные команды можно валидировать
    pub fn validate(
        self,
        validator: &dyn SecurityValidator,
    ) -> Result<Command<states::Validated>, DomainError> {
        validator.validate_command(self)
    }
}

impl Command<states::Validated> {
    /// Только валидированные команды можно анализировать
    pub async fn analyze(
        self,
        analyzer: &dyn CommandAnalyzer,
    ) -> Result<Command<states::Analyzed>, DomainError> {
        analyzer.analyze_command(self).await
    }
}

impl Command<states::Analyzed> {
    /// Только проанализированные команды можно маркировать как безопасные
    pub fn mark_safe(self) -> Command<states::SafeToExecute> {
        Command {
            raw: self.raw,
            parts: self.parts,
            context: self.context,
            state: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPath<'a> {
    inner: std::borrow::Cow<'a, std::path::Path>,
    metadata: PathMetadata,
}

impl<'a> ValidatedPath<'a> {
    pub fn new(
        path: impl Into<std::borrow::Cow<'a, std::path::Path>>,
    ) -> Result<Self, DomainError> {
        let inner = path.into();

        // Runtime проверки безопасности
        let path_str = inner.to_string_lossy();
        if path_str.contains("..") {
            return Err(DomainError::Security(error::SecurityError {
                violation: error::SecurityViolation::PathTraversalAttempt,
                severity: error::SecuritySeverity::High,
                context: error::SecurityContext {
                    user_id: unsafe { libc::getuid() },
                    working_directory: std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    attempted_operation: "path_validation".to_string(),
                },
            }));
        }

        let metadata = PathMetadata::new(&inner);
        Ok(Self { inner, metadata })
    }

    /// Zero-copy доступ к пути
    pub fn as_path(&self) -> &std::path::Path {
        &self.inner
    }

    pub fn to_string(&self) -> String {
        self.inner.to_string_lossy().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMetadata {
    pub is_absolute: bool,
    pub depth: usize,
    pub normalized: String,
}

impl PathMetadata {
    fn new(path: &std::path::Path) -> Self {
        Self {
            is_absolute: path.is_absolute(),
            depth: path.components().count(),
            normalized: path.to_string_lossy().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let command = Command::new("ls -la".to_string()).unwrap();
        assert_eq!(command.raw(), "ls -la");
        assert_eq!(command.parts(), &["ls", "-la"]);
    }

    #[test]
    fn test_command_too_long() {
        let long_command = "a".repeat(5000);
        let result = Command::new(long_command);
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[test]
    fn test_validated_path_creation() {
        let path = ValidatedPath::new(".").unwrap();
        assert!(path.to_string().contains('.'));
    }

    #[test]
    fn test_validated_path_traversal_protection() {
        let result = ValidatedPath::new("../sensitive");
        assert!(matches!(result, Err(DomainError::Security(_))));
    }
}
