//! crates/security/src/lib.rs
//! # Security Validator Implementation
//! Реализация SecurityValidator с изолированными правилами безопасности.

use regex::Regex;
use shared::error::{
    SecurityContext, SecurityError, SecuritySeverity, SecurityViolation, ValidationError,
};
use shared::{
    states, Command, DomainError, SecurityLevel, SecurityValidator as SecurityValidatorTrait,
};
use std::collections::HashSet;

pub struct SecurityValidator {
    rules: Vec<Box<dyn SecurityRule>>,
    allowed_commands: HashSet<String>,
    blocked_patterns: Vec<Regex>,
}

impl SecurityValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            allowed_commands: HashSet::new(),
            blocked_patterns: Vec::new(),
        };

        // Добавляем стандартные правила
        validator.add_rule(Box::new(PathTraversalRule));
        validator.add_rule(Box::new(CommandInjectionRule));
        validator.add_rule(Box::new(DestructiveOperationRule));
        validator.add_rule(Box::new(NetworkOperationRule));

        // Инициализируем набор разрешенных команд
        validator.allowed_commands.extend(
            [
                "ls", "cd", "pwd", "cat", "echo", "grep", "find", "mkdir", "rm", "cp", "mv",
                "chmod", "chown", "ps", "kill", "git", "docker", "kubectl", "cargo", "rustc",
                "python", "node",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        // Инициализируем блокируемые паттерны
        validator
            .blocked_patterns
            .push(Regex::new(r"(?:\.\./)+").unwrap()); // path traversal
        validator
            .blocked_patterns
            .push(Regex::new(r"[|&;`$]").unwrap()); // command injection

        validator
    }

    pub fn add_rule(&mut self, rule: Box<dyn SecurityRule>) {
        self.rules.push(rule);
    }
}

#[async_trait::async_trait]
impl SecurityValidatorTrait for SecurityValidator {
    async fn validate_command(
        &self,
        command: Command<states::Unvalidated>,
    ) -> Result<Command<states::Validated>, DomainError> {
        let raw = command.raw().to_string();

        // Проверка по блокируемым паттернам
        for pattern in &self.blocked_patterns {
            if pattern.is_match(&raw) {
                return Err(DomainError::Security(SecurityError {
                    violation: SecurityViolation::CommandInjectionAttempt,
                    severity: SecuritySeverity::High,
                    context: SecurityContext {
                        user_id: command.context().user_id,
                        working_directory: command.context().working_directory.to_string(),
                        attempted_operation: raw.clone(),
                    },
                }));
            }
        }

        // Проверка разрешенных команд
        if let Some(first_part) = command.parts().first() {
            if !self.allowed_commands.contains(first_part) {
                return Err(DomainError::Validation(ValidationError {
                    reason: format!("Command '{}' is not allowed", first_part),
                    command: raw,
                    field: Some("command".to_string()),
                    constraints: vec!["must be in allowed list".to_string()],
                }));
            }
        }

        // Применяем дополнительные правила
        for rule in &self.rules {
            if let Err(error) = rule.check(&command).await {
                return Err(error);
            }
        }

        // Преобразуем команду в Validated состояние
        Ok(Command {
            raw: command.raw().to_string(),
            parts: command.parts().to_vec(),
            context: command.context().clone(),
            state: std::marker::PhantomData,
            analysis_data: None,       // ДОБАВЛЯЕМ
            hallucination_score: None, // ДОБАВЛЯЕМ
        })
    }

    fn get_security_level(&self) -> SecurityLevel {
        // Исправляем тип
        SecurityLevel::User
    }

    fn can_handle_command(&self, _command: &Command<states::Unvalidated>) -> bool {
        // Можем обрабатывать все команды
        true
    }
}

// Трейт для правил безопасности
#[async_trait::async_trait]
pub trait SecurityRule: Send + Sync {
    async fn check(&self, command: &Command<states::Unvalidated>) -> Result<(), DomainError>;
}

// Правило для защиты от path traversal
struct PathTraversalRule;

#[async_trait::async_trait]
impl SecurityRule for PathTraversalRule {
    async fn check(&self, command: &Command<states::Unvalidated>) -> Result<(), DomainError> {
        for part in command.parts() {
            if part.contains("..") {
                return Err(DomainError::Security(SecurityError {
                    violation: SecurityViolation::PathTraversalAttempt,
                    severity: SecuritySeverity::High,
                    context: SecurityContext {
                        user_id: command.context().user_id,
                        working_directory: command.context().working_directory.to_string(),
                        attempted_operation: command.raw().to_string(),
                    },
                }));
            }
        }
        Ok(())
    }
}

// Правило для защиты от инъекций команд
struct CommandInjectionRule;

#[async_trait::async_trait]
impl SecurityRule for CommandInjectionRule {
    async fn check(&self, command: &Command<states::Unvalidated>) -> Result<(), DomainError> {
        let dangerous_chars = ['|', '&', ';', '`', '$', '>', '<'];
        for part in command.parts() {
            if dangerous_chars.iter().any(|c| part.contains(*c)) {
                return Err(DomainError::Security(SecurityError {
                    violation: SecurityViolation::CommandInjectionAttempt,
                    severity: SecuritySeverity::Critical,
                    context: SecurityContext {
                        user_id: command.context().user_id,
                        working_directory: command.context().working_directory.to_string(),
                        attempted_operation: command.raw().to_string(),
                    },
                }));
            }
        }
        Ok(())
    }
}

// Правило для блокировки деструктивных операций
struct DestructiveOperationRule;

#[async_trait::async_trait]
impl SecurityRule for DestructiveOperationRule {
    async fn check(&self, command: &Command<states::Unvalidated>) -> Result<(), DomainError> {
        let destructive_commands = ["rm", "dd", "mkfs", "fdisk", "shutdown", "reboot"];
        if let Some(first_part) = command.parts().first() {
            if destructive_commands.contains(&first_part.as_str()) {
                return Err(DomainError::Security(SecurityError {
                    violation: SecurityViolation::ResourceExhaustion,
                    severity: SecuritySeverity::Critical,
                    context: SecurityContext {
                        user_id: command.context().user_id,
                        working_directory: command.context().working_directory.to_string(),
                        attempted_operation: command.raw().to_string(),
                    },
                }));
            }
        }
        Ok(())
    }
}

// Правило для контроля сетевых операций
struct NetworkOperationRule;

#[async_trait::async_trait]
impl SecurityRule for NetworkOperationRule {
    async fn check(&self, command: &Command<states::Unvalidated>) -> Result<(), DomainError> {
        let network_commands = ["curl", "wget", "ssh", "scp", "nc", "telnet"];
        if let Some(first_part) = command.parts().first() {
            if network_commands.contains(&first_part.as_str()) {
                return Err(DomainError::Security(SecurityError {
                    violation: SecurityViolation::DataExfiltration,
                    severity: SecuritySeverity::Medium,
                    context: SecurityContext {
                        user_id: command.context().user_id,
                        working_directory: command.context().working_directory.to_string(),
                        attempted_operation: command.raw().to_string(),
                    },
                }));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_path_traversal_blocked() {
        let validator = SecurityValidator::new();
        let command = Command::new("cat ../../etc/passwd".to_string()).unwrap();

        let result = validator.validate_command(command).await;
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    #[tokio::test]
    async fn test_command_injection_blocked() {
        let validator = SecurityValidator::new();
        let command = Command::new("ls; rm -rf /".to_string()).unwrap();

        let result = validator.validate_command(command).await;
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    #[tokio::test]
    async fn test_destructive_operation_blocked() {
        let validator = SecurityValidator::new();
        let command = Command::new("rm -rf /".to_string()).unwrap();

        let result = validator.validate_command(command).await;
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    #[tokio::test]
    async fn test_allowed_command() {
        let validator = SecurityValidator::new();
        let command = Command::new("ls -la".to_string()).unwrap();

        let result = validator.validate_command(command).await;
        assert!(result.is_ok());
    }
}
