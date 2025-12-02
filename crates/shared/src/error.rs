//! crates/shared/src/error.rs
//! # Comprehensive Error System with Exhaustive Matching
//! Полная система ошибок с гарантированной обработкой всех вариантов.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Иерархия ошибок домена с exhaustive matching гарантиями
///
/// Это основная ошибка для всей системы. Каждая ошибка делигируется
/// соответствующим подтипам для детализированной информации.
///
/// # Пример
/// ```
/// use shared::error::{DomainError, ValidationError};
/// let error: DomainError = ValidationError {
///     reason: "Invalid command".to_string(),
///     command: "rm -rf /".to_string(),
///     field: None,
///     constraints: vec!["no destructive commands".to_string()],
/// }.into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    /// Ошибка валидации входных данных
    Validation(ValidationError),
    /// Ошибка безопасности (нарушение политик)
    Security(SecurityError),
    /// Ошибка AI-анализа команды
    Analysis(AnalysisError),
    /// Ошибка ввода-вывода
    Io(IoError),
    /// Ошибка конфигурации системы
    Configuration(ConfigurationError),
    /// Ошибка обучения модели
    Training(TrainingError),
    /// Сетевая ошибка (HTTP, подключение и т.д.)
    Network(NetworkError),
}

/// Ошибка валидации входных данных или команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Человекочитаемое описание ошибки
    pub reason: String,
    /// Команда, вызвавшая ошибку
    pub command: String,
    /// Поле с ошибкой (если применимо)
    pub field: Option<String>,
    /// Список нарушенных ограничений
    pub constraints: Vec<String>,
}

/// Ошибка безопасности, возникает при нарушении политик безопасности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityError {
    /// Тип нарушения безопасности
    pub violation: SecurityViolation,
    /// Серьезность нарушения
    pub severity: SecuritySeverity,
    /// Контекст, в котором произошло нарушение
    pub context: SecurityContext,
}

/// Типы нарушений безопасности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolation {
    /// Попытка обхода пути (../ и т.д.)
    PathTraversalAttempt,
    /// Попытка инъекции команды
    CommandInjectionAttempt,
    /// Попытка повышения привилегий
    PermissionEscalation,
    /// Попытка извлечения данных
    DataExfiltration,
    /// Исчерпание ресурсов (DoS атака)
    ResourceExhaustion,
}

/// Уровни серьезности нарушения безопасности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Низкий риск
    Low,
    /// Средний риск
    Medium,
    /// Высокий риск
    High,
    /// Критический риск (немедленное реагирование)
    Critical,
}

/// Контекст безопасности для аудита и логгирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Идентификатор пользователя
    pub user_id: u32,
    /// Рабочая директория в момент нарушения
    pub working_directory: String,
    /// Операция, которую пытались выполнить
    pub attempted_operation: String,
}

/// Ошибка анализа AI модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisError {
    /// Имя модели, которая вызвала ошибку
    pub model: String,
    /// Тип ошибки анализа
    pub error_type: AnalysisErrorType,
    /// Детали ошибки для отладки
    pub details: String,
    /// Предложение по исправлению (если доступно)
    pub suggestion: Option<String>,
}

/// Типы ошибок AI анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisErrorType {
    /// Модель недоступна
    ModelUnavailable,
    /// Некорректный ответ от модели
    InvalidResponse,
    /// Таймаут запроса к модели
    Timeout,
    /// Обнаружена галлюцинация (некорректная информация от AI)
    HallucinationDetected,
    /// Слишком низкая уверенность модели
    ConfidenceTooLow,
}

/// Ошибка ввода-вывода с файловой системой
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoError {
    /// Путь к файлу или директории
    pub path: String,
    /// Операция, вызвавшая ошибку
    pub operation: IoOperation,
    /// Исходная ошибка (если доступна)
    pub source: Option<String>,
}

/// Типы операций ввода-вывода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoOperation {
    /// Чтение файла
    Read,
    /// Запись в файл
    Write,
    /// Удаление файла
    Delete,
    /// Список файлов в директории
    List,
    /// Получение метаданных
    Metadata,
}

/// Ошибка конфигурации системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationError {
    /// Ключ конфигурации
    pub key: String,
    /// Ожидаемый тип значения
    pub expected_type: String,
    /// Фактическое значение
    pub actual_value: String,
}

/// Ошибка обучения модели машинного обучения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingError {
    /// Имя модели
    pub model_name: String,
    /// Размер данных обучения
    pub training_data_size: usize,
    /// Тип ошибки обучения
    pub error: TrainingErrorType,
}

/// Типы ошибок обучения модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingErrorType {
    /// Недостаточно данных для обучения
    InsufficientData,
    /// Модель не сошлась
    ModelConvergenceFailed,
    /// Слишком низкая точность на валидации
    ValidationAccuracyTooLow,
    /// Недостаточно памяти
    MemoryExhausted,
}

/// Сетевая ошибка (HTTP, WebSocket и т.д.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkError {
    /// Конечная точка (URL)
    pub endpoint: String,
    /// Операция, вызвавшая ошибку
    pub operation: NetworkOperation,
    /// Код статуса HTTP (если применимо)
    pub status_code: Option<u16>,
}

/// Типы сетевых операций
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    /// Отправка запроса
    Request,
    /// Получение ответа
    Response,
    /// Установка соединения
    Connection,
    /// Таймаут
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

/// Автоматическое преобразование std::io::Error в DomainError::Io
impl From<std::io::Error> for DomainError {
    fn from(error: std::io::Error) -> Self {
        DomainError::Io(IoError {
            path: "unknown".to_string(),
            operation: IoOperation::Read,
            source: Some(error.to_string()),
        })
    }
}

// Zero-cost преобразования между ошибками

/// Преобразование ValidationError в DomainError без аллокаций
impl From<ValidationError> for DomainError {
    fn from(error: ValidationError) -> Self {
        DomainError::Validation(error)
    }
}

/// Преобразование SecurityError в DomainError без аллокаций
impl From<SecurityError> for DomainError {
    fn from(error: SecurityError) -> Self {
        DomainError::Security(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // SecurityLevel используется только в тестах для проверки exhaustive matching
    // Определение SecurityLevel находится в корне крейта (lib.rs) или security.rs
    use crate::SecurityLevel;

    /// Этот тест гарантирует, что все варианты DomainError обрабатываются
    ///
    /// Exhaustive matching гарантирует, что при добавлении нового типа ошибки
    /// компилятор заставит обновить все match-выражения.
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

    /// Тест exhaustive matching для SecurityLevel
    ///
    /// Гарантирует, что при добавлении нового уровня безопасности
    /// все match-выражения будут обновлены.
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
