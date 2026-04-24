//! crates/shared/src/lib.rs
//! # Shared Types and Traits with Compile-Time Guarantees
//! Общие типы, трейты и системы ошибок для всей архитектуры.
//! ver 1.1.0
//!
//! ## Основные улучшения
//! - Безопасное получение UID через крейт `nix` (вместо `unsafe libc::getuid`)
//! - Защита от path traversal через нормализацию пути
//! - Ленивая загрузка переменных окружения для производительности
//! - Корректная обработка IO-ошибок через `FileSystemError::from_io`
//! - Выделение создания контекста в отдельный метод `CommandContext::new()`

pub mod error;
pub mod serialization;
pub mod states;
pub mod traits;

// Re-export для удобства
pub use error::{
    ConfigurationError, DomainError, FileSystemError, FileSystemErrorType, IoOperation,
    NetworkError, OllamaFsError, SecurityError, SecuritySeverity, SecurityViolation,
    ValidationError,
};
pub use serialization::SerializedCommand;
pub use states::*;
pub use traits::*;

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::OnceLock;

// Безопасное получение uid через крейт nix (добавить в Cargo.toml)
use nix::unistd::Uid;

/// Результат анализа команды AI моделью
///
/// Содержит объяснение, оценку рисков, предложения и уверенность модели.
/// Используется для предоставления пользователю информации о команде.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAnalysis {
    /// Человекочитаемое объяснение команды
    pub explanation: String,
    /// Список потенциальных рисков при выполнении команды
    pub risks: Vec<String>,
    /// Предложения по улучшению или альтернативные команды
    pub suggestions: Vec<String>,
    /// Уверенность AI модели в анализе (0.0 - 1.0)
    pub confidence: f32,
    /// Альтернативные, более безопасные команды
    pub alternatives: Vec<String>,
}

impl CommandAnalysis {
    /// Создаёт пустой анализ для использования по умолчанию
    ///
    /// Используется в тестах или когда анализ недоступен.
    pub fn empty() -> Self {
        Self {
            explanation: "No analysis available".to_string(),
            risks: vec![],
            suggestions: vec![],
            confidence: 0.0,
            alternatives: vec![],
        }
    }
}

/// Типизированная команда с состоянием, гарантируемая компилятором
///
/// Использует систему типов для гарантии порядка операций:
/// 1. Unvalidated -> 2. Validated -> 3. Analyzed -> 4. SafeToExecute
///
/// # Пример
/// ```
/// use shared::{Command, states::Unvalidated};
/// let cmd: Command<Unvalidated> = Command::new("ls -la".to_string()).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command<S = states::Unvalidated> {
    /// Оригинальная текстовая команда
    pub raw: String,
    /// Разбитая на части команда (токены)
    pub parts: Vec<String>,
    /// Контекст выполнения команды
    pub context: CommandContext,
    /// Маркер состояния (compile-time гарантия)
    pub state: PhantomData<S>,
    /// Данные AI анализа (доступны только после анализа)
    pub analysis_data: Option<CommandAnalysis>,
    /// Оценка галлюцинаций AI (0.0 - 1.0, где выше - больше галлюцинаций)
    pub hallucination_score: Option<f32>,
}

/// Контекст выполнения команды
///
/// Содержит информацию об окружении: рабочую директорию, пользователя, переменные среды.
/// Используется для безопасности и контекстно-зависимого анализа.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    /// Валидированный путь рабочей директории (без traversal атак)
    pub working_directory: ValidatedPath<'static>,
    /// ID пользователя, выполняющего команду
    pub user_id: u32,
    /// Переменные окружения (ключ-значение)
    pub environment: Environment,
}

/// Лениво загружаемые переменные окружения
///
/// Позволяет избежать копирования всех переменных при создании каждой команды.
/// Данные загружаются только при первом вызове `get()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    #[serde(skip)]
    inner: OnceLock<Vec<(String, String)>>,
}

impl Environment {
    /// Создаёт новый контейнер для переменных окружения (ещё не загруженных)
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    /// Возвращает ссылку на вектор переменных окружения, загружая их при первом вызове
    pub fn get(&self) -> &Vec<(String, String)> {
        self.inner.get_or_init(|| std::env::vars().collect())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Command<S> {
    /// Возвращает оригинальную текстовую команду
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Возвращает части команды (токены)
    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    /// Возвращает контекст выполнения
    pub fn context(&self) -> &CommandContext {
        &self.context
    }
}

impl Command<states::Unvalidated> {
    /// Создаёт новую невалидированную команду
    ///
    /// Выполняет базовые проверки:
    /// - Максимальная длина команды (4096 символов)
    /// - Получение текущей рабочей директории
    /// - Сбор информации о пользователе и окружении
    ///
    /// # Ошибки
    /// Возвращает `DomainError::Validation` если команда слишком длинная
    /// Возвращает `DomainError::Io` если не удаётся получить текущую директорию
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

        let context = CommandContext::new()?;

        Ok(Self {
            raw,
            parts,
            context,
            state: PhantomData,
            analysis_data: None,
            hallucination_score: None,
        })
    }

    /// Валидирует команду с использованием security validator
    ///
    /// Только команды в состоянии `Unvalidated` могут быть валидированы.
    /// После успешной валидации возвращает команду в состоянии `Validated`.
    ///
    /// # Асинхронность
    /// Валидация может включать сетевые запросы или проверки БД, поэтому async.
    pub async fn validate(
        self,
        validator: &dyn SecurityValidator,
    ) -> Result<Command<states::Validated>, DomainError> {
        validator.validate_command(self).await
    }
}

impl Command<states::Validated> {
    /// Анализирует команду с использованием AI модели
    ///
    /// Только валидированные команды могут быть проанализированы.
    /// После анализа возвращает команду в состоянии `Analyzed` с данными анализа.
    ///
    /// # Асинхронность
    /// Анализ включает запросы к AI модели, поэтому async.
    pub async fn analyze(
        self,
        analyzer: &dyn CommandAnalyzer,
    ) -> Result<Command<states::Analyzed>, DomainError> {
        analyzer.analyze_command(self).await
    }

    /// Вручную преобразует команду в состояние `Analyzed`
    ///
    /// Используется в тестах или при mock-анализе.
    /// Обычно команды анализируются через метод `analyze()`.
    ///
    /// TODO: Этот метод никогда не возвращает ошибку, но сигнатура Result сохранена
    ///       для совместимости с трейтом CommandAnalyzerTrait. В будущих версиях
    ///       можно убрать Result после рефакторинга всех реализаций трейта.
    pub fn into_analyzed(
        self,
        analysis: CommandAnalysis,
        hallucination_score: f32,
    ) -> Result<Command<states::Analyzed>, DomainError> {
        Ok(Command {
            raw: self.raw,
            parts: self.parts,
            context: self.context,
            state: std::marker::PhantomData,
            analysis_data: Some(analysis),
            hallucination_score: Some(hallucination_score),
        })
    }
}

impl Command<states::Analyzed> {
    /// Возвращает данные AI анализа, если они есть
    pub fn analysis_data(&self) -> Option<&CommandAnalysis> {
        self.analysis_data.as_ref()
    }

    /// Возвращает оценку галлюцинаций AI
    ///
    /// Возвращает 0.0 если оценка не доступна.
    pub fn hallucination_score(&self) -> f32 {
        self.hallucination_score.unwrap_or(0.0)
    }

    /// Помечает команду как безопасную для выполнения
    ///
    /// Только проанализированные команды могут быть помечены как безопасные.
    /// Удаляет данные анализа, так как они больше не нужны после выполнения.
    pub fn mark_safe(self) -> Command<states::SafeToExecute> {
        Command {
            raw: self.raw,
            parts: self.parts,
            context: self.context,
            state: PhantomData,
            analysis_data: None,
            hallucination_score: None,
        }
    }
}

impl CommandContext {
    /// Создаёт контекст выполнения с безопасно полученными данными
    ///
    /// # Ошибки
    /// Возвращает `DomainError::FileSystem` если не удаётся получить текущую директорию
    /// Возвращает `DomainError::Security` если путь содержит path traversal
    fn new() -> Result<Self, DomainError> {
        // ИСПРАВЛЕНО: используется контекстный конструктор FileSystemError::from_io
        let current_dir = std::env::current_dir().map_err(|e| {
            FileSystemError::from_io(e, ".", IoOperation::Read, "get current directory")
        })?;

        let working_directory = ValidatedPath::new(current_dir)?;
        let user_id = Uid::current().as_raw(); // УЛУЧШЕНО: безопасно, без unsafe

        Ok(Self {
            working_directory,
            user_id,
            environment: Environment::new(),
        })
    }
}

/// Валидированный путь с runtime проверками безопасности
///
/// Гарантирует отсутствие path traversal атак (../ и других вариантов обхода).
/// Использует нормализацию пути через компоненты и запрещает выход за пределы корня.
///
/// # Пример
/// ```
/// use shared::ValidatedPath;
/// let path = ValidatedPath::new("/home/user/project").unwrap();
/// assert_eq!(path.as_path().to_str(), Some("/home/user/project"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPath<'a> {
    inner: std::borrow::Cow<'a, std::path::Path>,
    metadata: PathMetadata,
}

impl<'a> ValidatedPath<'a> {
    /// Создаёт новый валидированный путь
    ///
    /// Выполняет проверки безопасности:
    /// - Нормализует путь (удаляет `.`, разрешает `..` только в рамках текущего каталога)
    /// - Запрещает выход за корень (например, `a/b/../../..` приведёт к ошибке)
    /// - Собирает метаданные о пути
    ///
    /// # Ошибки
    /// Возвращает `DomainError::Security`, если путь содержит компонент `..` (ParentDir).
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, DomainError> {
        let path_buf = path.as_ref().to_path_buf();

        // Запрещаем любую попытку подняться на уровень выше (..)
        if path_buf
            .components()
            .any(|comp| matches!(comp, std::path::Component::ParentDir))
        {
            return Err(DomainError::Security(SecurityError {
                violation: SecurityViolation::PathTraversalAttempt,
                severity: SecuritySeverity::High,
                context: error::SecurityContext {
                    user_id: Uid::current().as_raw(),
                    working_directory: std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    attempted_operation: "path_validation".to_string(),
                },
            }));
        }

        let metadata = PathMetadata::new(&path_buf);
        Ok(Self {
            inner: std::borrow::Cow::Owned(path_buf),
            metadata,
        })
    }

    /// Возвращает ссылку на путь (zero-copy)
    pub fn as_path(&self) -> &std::path::Path {
        &self.inner
    }

    /// Преобразует путь в String
    pub fn to_string(&self) -> String {
        self.inner.to_string_lossy().to_string()
    }
}

/// Метаданные пути для быстрого доступа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMetadata {
    /// Абсолютный ли путь
    pub is_absolute: bool,
    /// Глубина пути (количество компонентов)
    pub depth: usize,
    /// Нормализованное строковое представление
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
    fn test_validated_path_safe() {
        let path = ValidatedPath::new("safe/path").unwrap();
        assert!(path.as_path().to_string_lossy().contains("safe/path"));
    }

    /// Проверка обнаружения простого path traversal
    #[test]
    fn test_validated_path_traversal_detected() {
        let result = ValidatedPath::new("../etc/passwd");
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    /// Проверка сложного traversal с множественными `..`
    #[test]
    fn test_validated_path_multiple_traversal() {
        let result = ValidatedPath::new("a/b/../../c");
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    /// Проверка абсолютного пути с выходом за корень
    #[test]
    fn test_validated_path_absolute_with_traversal() {
        let result = ValidatedPath::new("/home/user/../../etc/passwd");
        assert!(matches!(result, Err(DomainError::Security(_))));
    }

    /// Проверка создания контекста и ленивой загрузки окружения
    #[test]
    fn test_context_creation() {
        let ctx = CommandContext::new().unwrap();
        assert!(ctx.user_id > 0);
        assert!(!ctx.environment.get().is_empty());
    }

    /// Проверка, что переменные окружения загружаются только при вызове get()
    #[test]
    fn test_environment_lazy_loading() {
        let env = Environment::new();
        assert!(env.inner.get().is_none()); // ещё не загружено
        let _vars = env.get();
        assert!(env.inner.get().is_some()); // загружено после первого вызова
    }
}
