//! crates/shared/src/error.rs
//! # Comprehensive Error System with Exhaustive Matching
//! Полная система ошибок с гарантированной обработкой всех вариантов.
//! ver 1.2.0
//! NOTE: оптимизирован с использованием thiserror, макроса impl_from, объединены IoError и FileSystemError

use serde::{Deserialize, Serialize};
// use std::fmt;
use std::io;
use thiserror::Error;

// ==== Макрос для автоматической реализации From<SpecificError> ==========
/// Генерирует impl From<SpecificError> for DomainError.
/// Используется для всех вариантов, кроме тех, что требуют специальной логики.
#[macro_export]
macro_rules! impl_from {
    ($error_type:ty, $variant:ident) => {
        impl From<$error_type> for DomainError {
            fn from(err: $error_type) -> Self {
                DomainError::$variant(err)
            }
        }
    };
}

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
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum DomainError {
    /// Ошибка валидации входных данных
    #[error("Validation error: {0}")]
    Validation(ValidationError),
    /// Ошибка безопасности (нарушение политик)
    #[error("Security error: {0}")]
    Security(SecurityError),
    /// Ошибка AI-анализа команды
    #[error("Analysis error: {0}")]
    Analysis(AnalysisError),
    /// Ошибка файловой системы – замена ранее существовавшей IoError
    #[error("Filesystem error: {0}")]
    FileSystem(FileSystemError),
    /// Ошибка конфигурации системы
    #[error("Configuration error: {0}")]
    Configuration(ConfigurationError),
    /// Ошибка обучения модели
    #[error("Trainig error: {0}")]
    Training(TrainingError),
    /// Сетевая ошибка (HTTP, подключение и т.д.)
    #[error("Network error: {0}")]
    Network(NetworkError),
    /// Ошибки специфичные для Ollama
    #[error("Ollame fs error")]
    OllamaFs(OllamaFsError),
}

// ========== Детальные ошибки ==========

/// Ошибка валидации входных данных или команды
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{reason} - command: {command}")]
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
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("Security violation: {violation:?} ({severity:?}) in {context}")]
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
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("user {user_id} in {working_directory} attempted {attempted_operation}")]
pub struct SecurityContext {
    /// Идентификатор пользователя
    pub user_id: u32,
    /// Рабочая директория в момент нарушения
    pub working_directory: String,
    /// Операция, которую пытались выполнить
    pub attempted_operation: String,
}

/// Ошибка анализа AI модели
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("model {model}: {error_type:?} - {details}")]
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
    // добавлен для неизвестных операций
    Other(String),
}

/// Ошибка конфигурации системы
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("config {key}: expected {expected_type}, got {actual_value}")]
pub struct ConfigurationError {
    /// Ключ конфигурации
    pub key: String,
    /// Ожидаемый тип значения
    pub expected_type: String,
    /// Фактическое значение
    pub actual_value: String,
}

/// Ошибка обучения модели машинного обучения
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("training {model_name} (size={training_data_size}): {error:?}")]
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
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("network {operation:?} on {endpoint}")]
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

/// Объединённая ошибка файловой системы (ранее IoError + FileSystemError)
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{error_type:?} on {path} during {operation:?}: {context}")]
pub struct FileSystemError {
    pub error_type: FileSystemErrorType,
    pub path: String,
    pub operation: IoOperation, // перенесено из IoError
    pub context: String,
    /// Дополнительные детали ошибки (например, исходный текст IO-ошибки)
    pub detailed_message: Option<String>,
}

/// Типы ошибок файловой системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemErrorType {
    /// Файл не найден
    FileNotFound,
    /// Нет прав доступа
    PermissionDenied,
    /// Файл уже существует
    FileExists,
    /// Диск переполнен
    DiskFull,
    /// Недостаточно места
    InsufficientSpace,
    /// Некорректный путь
    InvalidPath,
    /// Слишком большой файл
    FileTooLarge,
    /// Ошибка рекурсивного обхода
    TreeTraversalError,
    /// Ошибка чтения дерева файлов
    TreeReadError,
    /// Универсальный вариант
    Other,
}

/// Ошибки специфичные для Ollama файловой системы
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("OllamaFS {error_type:?} at {ollama_path}: {context}")]
pub struct OllamaFsError {
    /// Тип ошибки Ollama
    pub error_type: OllamaFsErrorType,
    /// Имя модели (если применимо)
    pub model_name: Option<String>,
    /// Путь в хранилище Ollama
    pub ollama_path: String,
    /// Контекст операции
    pub context: String,
}

/// Типы ошибок Ollama файловой системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OllamaFsErrorType {
    /// Модель не найдена локально
    ModelNotFound,
    /// Некорректная структура хранилища Ollama
    InvalidOllamaStructure,
    /// Поврежденный файл модели
    CorruptedModelFile,
    /// Некорректный Modelfile
    InvalidModelfile,
    /// Конфликт версий модели
    ModelVersionConflict,
    /// Недостаточно места для модели
    InsufficientSpaceForModel,
}

// Преобразования из стандартных ошибок

// ========== Макросом генерируем From для всех вариантов, кроме FileSystem (требует контекста) ==========
impl_from!(ValidationError, Validation);
impl_from!(SecurityError, Security);
impl_from!(AnalysisError, Analysis);
impl_from!(ConfigurationError, Configuration);
impl_from!(TrainingError, Training);
impl_from!(NetworkError, Network);
impl_from!(OllamaFsError, OllamaFs);

// Для FileSystemError оставляем явный From (но он будет просто обёрткой)
impl From<FileSystemError> for DomainError {
    fn from(err: FileSystemError) -> Self {
        DomainError::FileSystem(err)
    }
}

// ========== Контекстное преобразование std::io::Error – теперь явное ==========
// Общий From удалён. Вместо этого предоставляем конструктор для FileSystemError:
impl FileSystemError {
    /// Создаёт ошибку файловой системы из std::io::Error с контекстом.
    pub fn from_io(
        io_err: io::Error,
        path: impl Into<String>,
        operation: IoOperation,
        context: impl Into<String>,
    ) -> Self {
        let error_type = match io_err.kind() {
            io::ErrorKind::NotFound => FileSystemErrorType::FileNotFound,
            io::ErrorKind::PermissionDenied => FileSystemErrorType::PermissionDenied,
            io::ErrorKind::AlreadyExists => FileSystemErrorType::FileExists,
            io::ErrorKind::StorageFull => FileSystemErrorType::DiskFull,
            io::ErrorKind::InvalidInput => FileSystemErrorType::InvalidPath,
            _ => FileSystemErrorType::Other,
        };
        FileSystemError {
            error_type,
            path: path.into(),
            operation,
            context: context.into(),
            detailed_message: Some(io_err.to_string()),
        }
    }
}
// ======= Обратная совместимость для кода, использующего старый IoError =======
// Удаляем структуру IoError, но для предотвращения поломок оставляем type alias (deprecated)
#[deprecated(note = "Use FileSystemError instead")]
pub type IoError = FileSystemError;

// ========== Тесты ==========
#[cfg(test)]
mod tests {
    use super::*;
    // SecurityLevel больше не нужен для тестов ошибок – удалён.
    // Если необходимо проверить exhaustive matching для SecurityLevel,
    // это следует делать в модуле, где определён SecurityLevel.
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
            DomainError::FileSystem(FileSystemError {
                error_type: FileSystemErrorType::FileNotFound,
                path: "/test".to_string(),
                operation: IoOperation::Read,
                context: "testing".to_string(),
                detailed_message: None,
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
            DomainError::OllamaFs(OllamaFsError {
                error_type: OllamaFsErrorType::ModelNotFound,
                model_name: Some("test".to_string()),
                ollama_path: "/ollama/models".to_string(),
                context: "test".to_string(),
            }),
        ];

        for error in errors {
            match error {
                DomainError::Validation(_) => assert!(true),
                DomainError::Security(_) => assert!(true),
                DomainError::Analysis(_) => assert!(true),
                DomainError::FileSystem(_) => assert!(true),
                DomainError::Configuration(_) => assert!(true),
                DomainError::Training(_) => assert!(true),
                DomainError::Network(_) => assert!(true),
                DomainError::OllamaFs(_) => assert!(true),
            }
        }
    }

    // Демонстрация использования контекстного преобразования IO-ошибок
    #[test]
    fn test_io_conversion_with_context() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let fs_err =
            FileSystemError::from_io(io_err, "/etc/passwd", IoOperation::Read, "loading config");
        let domain_err: DomainError = fs_err.into();
        match domain_err {
            DomainError::FileSystem(fs) => {
                assert!(matches!(fs.error_type, FileSystemErrorType::FileNotFound));
                assert_eq!(fs.path, "/etc/passwd");
            }
            _ => panic!("wrong variant"),
        }
    }
}
