//! crates/shared/src/serialization.rs
//! # Comprehensive Serialization System
//! Полная система сериализации с валидацией и версионированием.
//! ver 1.0.1

use crate::SystemInfo;
use crate::{states, Command};
use serde::{Deserialize, Serialize};

/// Версионированные структуры для обратной совместимости
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCommand {
    pub version: u32,
    pub command_type: CommandType,
    pub data: CommandData,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    Simple,
    Pipeline,
    Conditional,
    Background,
    Redirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandData {
    pub raw: String,
    pub parts: Vec<CommandPart>,
    pub context: SerializedContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPart {
    pub text: String,
    pub part_type: PartType,
    pub is_variable: bool,
    pub requires_expansion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartType {
    Executable,
    Argument,
    Option,
    Flag,
    Redirection,
    Pipe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedContext {
    pub working_directory: String,
    pub environment: Vec<EnvironmentVariable>,
    pub user_info: UserInfo,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub user_id: u32,
    pub group_id: u32,
    pub home_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub created_at: String,
    pub source: CommandSource,
    pub security_level: states::SecurityLevel,
    pub validation_checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandSource {
    UserInput,
    History,
    Suggestion,
    Automated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub check_type: CheckType,
    pub passed: bool,
    pub details: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckType {
    PathSafety,
    CommandInjection,
    ResourceUsage,
    PermissionCheck,
    NetworkAccess,
}

// Реализации сериализации для основных типов
impl SerializedCommand {
    pub fn from_command<S>(command: &Command<S>) -> Result<Self, crate::error::DomainError> {
        Ok(Self {
            version: 1,
            command_type: Self::classify_command(command.raw()),
            data: CommandData {
                raw: command.raw().to_string(),
                parts: command
                    .parts()
                    .iter()
                    .map(|part| CommandPart {
                        text: part.clone(),
                        part_type: Self::classify_part(part),
                        is_variable: part.starts_with('$'),
                        requires_expansion: part.contains('$'),
                    })
                    .collect(),
                context: SerializedContext {
                    working_directory: command.context().working_directory.to_string(),
                    environment: std::env::vars()
                        .map(|(k, v)| EnvironmentVariable {
                            is_sensitive: Self::is_sensitive(&k),
                            key: k,
                            value: v,
                        })
                        .collect(),
                    user_info: UserInfo {
                        username: whoami::username(),
                        user_id: unsafe { libc::getuid() },
                        group_id: unsafe { libc::getgid() },
                        home_directory: std::env::var("HOME").unwrap_or_default(),
                    },
                    system_info: SystemInfo {
                        os: whoami::platform().to_string(),
                        shell: std::env::var("SHELL").unwrap_or_default(),
                        architecture: whoami::arch().to_string(),
                        memory_mb: sys_info::mem_info().map(|m| m.total).unwrap_or(0) / 1024,
                    },
                },
            },
            metadata: CommandMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                source: CommandSource::UserInput,
                security_level: states::SecurityLevel::User,
                validation_checks: Vec::new(),
            },
        })
    }

    fn classify_command(raw: &str) -> CommandType {
        if raw.contains('|') {
            CommandType::Pipeline
        } else if raw.contains('&') {
            CommandType::Background
        } else if raw.contains('>') || raw.contains('<') {
            CommandType::Redirection
        } else if raw.contains("&&") || raw.contains("||") {
            CommandType::Conditional
        } else {
            CommandType::Simple
        }
    }

    fn classify_part(part: &str) -> PartType {
        if part.starts_with('-') {
            if part.len() == 2 {
                PartType::Flag
            } else {
                PartType::Option
            }
        } else if part == "|" {
            PartType::Pipe
        } else if part == ">" || part == "<" || part == ">>" {
            PartType::Redirection
        } else if part.contains('/') || !part.contains(char::is_whitespace) {
            PartType::Executable
        } else {
            PartType::Argument
        }
    }

    fn is_sensitive(key: &str) -> bool {
        let sensitive_keys = ["PASSWORD", "SECRET", "KEY", "TOKEN", "AUTH"];
        sensitive_keys
            .iter()
            .any(|s| key.to_uppercase().contains(s))
    }
}

// Тесты сериализации
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

    #[test]
    fn test_command_serialization_roundtrip() {
        let command = Command::new("ls -la".to_string()).unwrap();
        let serialized = SerializedCommand::from_command(&command).unwrap();

        let json = serde_json::to_string(&serialized).unwrap();
        let deserialized: SerializedCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(serialized.data.raw, deserialized.data.raw);
        assert_eq!(serialized.version, deserialized.version);
    }

    #[test]
    fn test_sensitive_data_detection() {
        assert!(SerializedCommand::is_sensitive("PASSWORD"));
        assert!(SerializedCommand::is_sensitive("SECRET_KEY"));
        assert!(!SerializedCommand::is_sensitive("PATH"));
    }
}
