//! crates/shared/src/states.rs
//! # Typestate System with Compile-Time Guarantees
//! Система состояний с гарантиями времени компиляции для правильной последовательности операций.
//!
//! # Типобезопасный конечный автомат (Typestate Pattern)
//! Каждое состояние представлено отдельным типом, что гарантирует правильный порядок операций:
//! Unvalidated → Validated → Analyzed → SafeToExecute
//! Компилятор предотвращает вызов методов в неверном состоянии.

use serde::{Deserialize, Serialize};

/// Начальное состояние команды — невалидированное
///
/// Команды в этом состоянии:
/// - Созданы из пользовательского ввода
/// - Не прошли проверки безопасности
/// - Не могут быть выполнены
///
/// # Переходы
/// Единственный допустимый переход: `Unvalidated` → `Validated` через валидацию.
pub struct Unvalidated {
    /// Флаг, указывающий что проверки безопасности ещё не выполнены
    pub security_checks_pending: bool,
}

/// Состояние после успешной валидации безопасности
///
/// Команды в этом состоянии:
/// - Прошли базовые проверки безопасности
/// - Не содержат явных инъекций или traversal атак
/// - Готовы к анализу AI моделью
///
/// # Переходы
/// Допустимые переходы: `Validated` → `Analyzed` через анализ AI.
pub struct Validated {
    /// Уровень безопасности, присвоенный команде
    pub security_level: SecurityLevel,
    /// Время, когда команда была валидирована
    pub validation_timestamp: std::time::SystemTime,
}

/// Состояние после анализа AI моделью
///
/// Команды в этом состоянии:
/// - Проанализированы AI моделью на риски и альтернативы
/// - Имеют оценку уверенности и галлюцинаций
/// - Могут быть помечены как безопасные для выполнения
///
/// # Переходы
/// Допустимые переходы: `Analyzed` → `SafeToExecute` через проверку безопасности.
pub struct Analyzed {
    /// Уникальный идентификатор анализа (для отслеживания и логирования)
    pub analysis_id: uuid::Uuid,
    /// Уверенность AI модели в анализе (0.0 - 1.0)
    pub confidence_score: f32,
    /// Версия AI модели, выполнившей анализ
    pub model_version: String,
}

/// Финальное состояние — безопасно для выполнения
///
/// Команды в этом состоянии:
/// - Прошли все проверки безопасности
/// - Проанализированы AI моделью
/// - Могут быть безопасно выполнены в ограниченном окружении
///
/// # Переходы
/// Это конечное состояние — дальнейшие переходы не требуются.
pub struct SafeToExecute {
    /// Гарантии безопасности, предоставленные системой
    pub safety_guarantees: SafetyGuarantees,
    /// Ограниченный контекст выполнения команды
    pub execution_context: ExecutionContext,
}

/// Гарантии безопасности для выполнения команды
///
/// Эти гарантии проверяются системой перед помещением команды
/// в состояние `SafeToExecute`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGuarantees {
    /// Гарантия отсутствия деструктивных операций (rm -rf, dd, форматирование и т.д.)
    pub no_destructive_operations: bool,
    /// Гарантия отсутствия сетевой эксфильтрации данных
    pub no_network_exfiltration: bool,
    /// Гарантия отсутствия нарушений приватности (доступ к личным файлам и т.д.)
    pub no_privacy_violations: bool,
    /// Гарантия выполнения в изолированном окружении (песочнице)
    pub sandboxed_environment: bool,
}

/// Контекст выполнения для безопасного исполнения команды
///
/// Определяет ограничения и разрешения для выполнения команды.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Рабочая директория, в которой будет выполнена команда
    pub working_directory: String,
    /// Разрешения пользователя в этом контексте выполнения
    pub user_permissions: UserPermissions,
    /// Ограничения среды выполнения (память, время, сеть и т.д.)
    pub environment_constraints: EnvironmentConstraints,
}

/// Разрешения пользователя в контексте выполнения
///
/// Тонко-гранулярные разрешения для контроля доступа.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// Может ли команда писать в домашнюю директорию пользователя
    pub can_write_to_home: bool,
    /// Может ли команда обращаться к сети
    pub can_access_network: bool,
    /// Может ли команда выполнять системные команды (через system(), exec() и т.д.)
    pub can_execute_system_commands: bool,
    /// Может ли команда модифицировать файлы вне рабочей директории
    pub can_modify_files: bool,
}

/// Ограничения среды выполнения
///
/// Ограничивают ресурсы, доступные команде во время выполнения.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConstraints {
    /// Максимальный объем памяти в мегабайтах
    pub max_memory_mb: u32,
    /// Таймаут выполнения в секундах
    pub timeout_seconds: u32,
    /// Разрешён ли доступ к сети
    pub network_access: bool,
    /// Квота на дисковое пространство в мегабайтах
    pub disk_quota_mb: u32,
}

/// Уровни безопасности с exhaustive matching
///
/// Каждый уровень безопасности определяет, какие операции разрешены.
/// Компилятор гарантирует обработку всех вариантов при использовании match.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Непроверенный источник (например, из интернета)
    /// Максимальные ограничения, запрещены любые опасные операции.
    Untrusted,
    /// Обычный пользователь (локальный ввод)
    /// Разрешены базовые операции, запрещены деструктивные.
    User,
    /// Проверенный источник (внутренние скрипты, админ)
    /// Разрешены некоторые деструктивные операции под наблюдением.
    Trusted,
    /// Системный уровень (требуется для системного обслуживания)
    /// Минимальные ограничения, но требуется явное подтверждение.
    System,
}

// Компилятор гарантирует обработку всех вариантов
impl SecurityLevel {
    /// Проверяет, может ли команда с этим уровнем безопасности выполнять деструктивные операции
    ///
    /// Деструктивные операции: удаление файлов, форматирование, очистка данных и т.д.
    ///
    /// # Пример
    /// ```
    /// use shared::states::SecurityLevel;
    /// assert!(!SecurityLevel::User.can_execute_destructive());
    /// assert!(SecurityLevel::Trusted.can_execute_destructive());
    /// ```
    pub fn can_execute_destructive(&self) -> bool {
        match self {
            SecurityLevel::Untrusted => false,
            SecurityLevel::User => false,
            SecurityLevel::Trusted => true,
            SecurityLevel::System => true,
        }
    }

    /// Проверяет, может ли команда с этим уровнем безопасности обращаться к сети
    ///
    /// Сетевой доступ может использоваться для эксфильтрации данных или
    /// загрузки вредоносного контента.
    ///
    /// # Пример
    /// ```
    /// use shared::states::SecurityLevel;
    /// assert!(!SecurityLevel::Untrusted.can_access_network());
    /// assert!(SecurityLevel::User.can_access_network());
    /// ```
    pub fn can_access_network(&self) -> bool {
        match self {
            SecurityLevel::Untrusted => false,
            SecurityLevel::User => true,
            SecurityLevel::Trusted => true,
            SecurityLevel::System => true,
        }
    }
}
