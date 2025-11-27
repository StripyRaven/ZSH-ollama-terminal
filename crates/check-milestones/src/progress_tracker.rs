// zsh-ollama-terminal/crates/check-milestones/src/progress_tracker.rs
//! # Progress Tracker Module
//!
//! Отслеживание прогресса по всем вехам и генерация комплексных отчетов.
//!
//! Этот модуль предоставляет систему для мониторинга выполнения вех проекта,
//! хранения истории прогресса и генерации детализированных отчетов в различных форматах.
//!
//! ## Основные компоненты
//!
//! - [`ProgressTracker`] - Основной трекер для мониторинга прогресса
//! - [`ProgressReport`] - Комплексный отчет о состоянии всех вех
//! - [`Milestone`] - Перечисление всех вех проекта
//! - [`MilestoneStatus`] - Статусы выполнения вех
//! - [`MilestoneProgress`] - Детальная информация о прогрессе вехи
//!
//! ## Жизненный цикл отслеживания
//!
//! 1. **Инициализация** - Создание трекера с начальным состоянием всех вех
//! 2. **Обновление** - Регулярное обновление статусов вех по результатам проверок
//! 3. **Отчетность** - Генерация отчетов в текстовом, JSON или Markdown формате
//! 4. **История** - Сохранение снимков прогресса для анализа трендов
//!
//! ## Пример использования
//!
//! ```rust
//! use check_milestones::{ProgressTracker, MilestoneGates, Milestone};
//!
//! let mut tracker = ProgressTracker::new();
//! let milestone_1 = MilestoneGates::milestone_1();
//! let result = milestone_1.check();
//!
//! // Обновляем статус вехи на основе результатов проверки
//! tracker.update_milestone(Milestone::Foundation, result);
//!
//! // Генерируем отчет
//! let report = tracker.generate_report();
//! println!("{}", report.to_markdown());
//!
//! // Сохраняем снимок для истории
//! tracker.snapshot();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::quality_gates::QualityResult;

/// Представляет различные вехи в проекте zsh-ollama-terminal
///
/// Каждая веха представляет собой значительный этап разработки,
/// который должен быть завершен перед переходом к следующему.
/// Вехи идут в строгом порядке и зависят друг от друга.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Milestone {
    /// Foundation Complete - Базовая инфраструктура и основные типы
    ///
    /// Включает реализацию core types, traits, error system,
    /// serialization/deserialization и базовую документацию.
    Foundation,

    /// Infrastructure Ready - Безопасность, клиент и платформенные абстракции
    ///
    /// Включает security validator, Ollama client, safe file operations,
    /// platform abstractions и performance benchmarks.
    Infrastructure,

    /// AI Core Functional - Анализ команд и AI возможности
    ///
    /// Включает command analysis pipeline, hallucination detection,
    /// training engine, performance targets и cache system.
    AICore,

    /// Web Interface Live - Веб-сервер и пользовательский интерфейс
    ///
    /// Включает Tera templates, HTMX interactions, typed HTTP responses,
    /// reusable components и web server performance.
    WebInterface,

    /// Integration Complete - Системная интеграция и CLI
    ///
    /// Включает daemon operation, CLI commands, shell integration,
    /// IPC communication и health monitoring.
    Integration,

    /// Production Ready - Готовность к продакшену
    ///
    /// Включает comprehensive testing, performance benchmarks,
    /// security audits, documentation и cross-platform testing.
    Production,
}

/// Статус выполнения вехи
///
/// Определяет текущее состояние вехи в процессе разработки.
/// Статусы используются для отслеживания прогресса и принятия решений
/// о переходе к следующим этапам разработки.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneStatus {
    /// Веха еще не начата - планирование или ожидание зависимостей
    NotStarted,

    /// Веха в процессе выполнения - активная разработка
    InProgress,

    /// Веха завершена с результатами проверки качества
    Completed(QualityResult),

    /// Веха заблокирована с указанием причины блокировки
    Blocked(String),

    /// Веха пропущена с указанием причины пропуска
    Skipped(String),
}

/// Детальная информация о прогрессе конкретной вехи
///
/// Содержит всю необходимую информацию для отслеживания состояния вехи,
/// включая требования, заметки, даты и результаты проверок.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    /// Текущий статус выполнения вехи
    pub status: MilestoneStatus,

    /// Временная метка последней проверки статуса вехи
    pub last_checked: DateTime<Utc>,

    /// Список требований, которые должны быть выполнены для завершения вехи
    pub requirements: Vec<String>,

    /// Дополнительные заметки или комментарии по вехе
    pub notes: Option<String>,

    /// Целевая дата завершения вехи (опционально)
    pub target_date: Option<DateTime<Utc>>,
}

/// Комплексный отчет о прогрессе всех вех проекта
///
/// Агрегирует информацию о всех вехах и предоставляет сводные метрики
/// для быстрой оценки общего состояния проекта.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    /// Коллекция всех вех с их текущим прогрессом
    pub milestones: HashMap<Milestone, MilestoneProgress>,

    /// Временная метка генерации отчета
    pub generated_at: DateTime<Utc>,

    /// Общий прогресс проекта в процентах (0-100)
    /// Рассчитывается на основе количества завершенных вех
    pub overall_progress: f32,

    /// Сводная статистика по статусам вех
    pub summary: ProgressSummary,
}

/// Сводная статистика для отчетов о прогрессе
///
/// Предоставляет количественные метрики о распределении вех
/// по различным статусам выполнения.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSummary {
    /// Общее количество вех в проекте
    pub total_milestones: usize,

    /// Количество завершенных вех
    pub completed_milestones: usize,

    /// Количество вех в процессе выполнения
    pub in_progress_milestones: usize,

    /// Количество вех, которые еще не начаты
    pub not_started_milestones: usize,

    /// Количество заблокированных вех
    pub blocked_milestones: usize,
}

/// Основной трекер для мониторинга прогресса по всем вехам
///
/// Центральный компонент системы отслеживания прогресса, который:
/// - Хранит текущее состояние всех вех
/// - Предоставляет методы для обновления статусов
/// - Генерирует комплексные отчеты
/// - Сохраняет историю изменений
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    /// Текущее состояние всех вех проекта
    milestones: HashMap<Milestone, MilestoneProgress>,

    /// История снимков прогресса для анализа трендов
    history: Vec<ProgressReport>,
}

// =============================================================================
// Реализации для Milestone
// =============================================================================

impl Milestone {
    /// Возвращает человеко-читаемое отображаемое имя вехи
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::Milestone;
    ///
    /// let name = Milestone::Foundation.display_name();
    /// assert_eq!(name, "Foundation Complete");
    /// ```
    pub fn display_name(&self) -> &'static str {
        match self {
            Milestone::Foundation => "Foundation Complete",
            Milestone::Infrastructure => "Infrastructure Ready",
            Milestone::AICore => "AI Core Functional",
            Milestone::WebInterface => "Web Interface Live",
            Milestone::Integration => "Integration Complete",
            Milestone::Production => "Production Ready",
        }
    }

    /// Возвращает подробное описание вехи
    ///
    /// Описание содержит ключевые компоненты, которые включаются в веху.
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::Milestone;
    ///
    /// let description = Milestone::Foundation.description();
    /// assert!(!description.is_empty());
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            Milestone::Foundation => "Core types and traits implemented and tested with basic infrastructure",
            Milestone::Infrastructure => "Security validator, Ollama client, safe file operations, and platform abstractions",
            Milestone::AICore => "Command analysis pipeline, hallucination detection, training engine, and performance targets",
            Milestone::WebInterface => "Tera templates, HTMX interactions, typed HTTP responses, and web server",
            Milestone::Integration => "Daemon operation, CLI commands, shell integration, and IPC communication",
            Milestone::Production => "Comprehensive testing, performance benchmarks, security audits, and documentation",
        }
    }

    /// Возвращает все вехи проекта в порядке выполнения
    ///
    /// Порядок вех фиксирован и отражает последовательность разработки.
    ///
    /// # Возвращает
    ///
    /// Вектор всех вех в порядке от первой к последней
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::Milestone;
    ///
    /// let all_milestones = Milestone::all();
    /// assert_eq!(all_milestones.len(), 6);
    /// assert!(matches!(all_milestones[0], Milestone::Foundation));
    /// ```
    pub fn all() -> Vec<Milestone> {
        vec![
            Milestone::Foundation,
            Milestone::Infrastructure,
            Milestone::AICore,
            Milestone::WebInterface,
            Milestone::Integration,
            Milestone::Production,
        ]
    }
}

// =============================================================================
// Реализации для MilestoneStatus
// =============================================================================

impl MilestoneStatus {
    /// Проверяет, завершена ли веха
    ///
    /// # Возвращает
    ///
    /// `true` если статус вехи `Completed`, иначе `false`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneStatus;
    ///
    /// let status = MilestoneStatus::NotStarted;
    /// assert!(!status.is_completed());
    /// ```
    pub fn is_completed(&self) -> bool {
        matches!(self, MilestoneStatus::Completed(_))
    }

    /// Проверяет, заблокирована ли веха
    ///
    /// # Возвращает
    ///
    /// `true` если статус вехи `Blocked`, иначе `false`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneStatus;
    ///
    /// let status = MilestoneStatus::Blocked("Waiting for dependencies".to_string());
    /// assert!(status.is_blocked());
    /// ```
    pub fn is_blocked(&self) -> bool {
        matches!(self, MilestoneStatus::Blocked(_))
    }

    /// Проверяет, находится ли веха в процессе выполнения
    ///
    /// # Возвращает
    ///
    /// `true` если статус вехи `InProgress`, иначе `false`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneStatus;
    ///
    /// let status = MilestoneStatus::InProgress;
    /// assert!(status.is_in_progress());
    /// ```
    pub fn is_in_progress(&self) -> bool {
        matches!(self, MilestoneStatus::InProgress)
    }

    /// Возвращает текстовое представление статуса
    ///
    /// # Возвращает
    ///
    /// Строковое представление статуса для отображения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneStatus;
    ///
    /// let status = MilestoneStatus::InProgress;
    /// assert_eq!(status.as_str(), "In Progress");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            MilestoneStatus::NotStarted => "Not Started",
            MilestoneStatus::InProgress => "In Progress",
            MilestoneStatus::Completed(_) => "Completed",
            MilestoneStatus::Blocked(_) => "Blocked",
            MilestoneStatus::Skipped(_) => "Skipped",
        }
    }

    /// Возвращает emoji представление статуса
    ///
    /// Используется для визуального выделения статусов в отчетах.
    ///
    /// # Возвращает
    ///
    /// Emoji символ, соответствующий статусу
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneStatus;
    ///
    /// let status = MilestoneStatus::Completed(
    ///     check_milestones::QualityResult::success(
    ///         "Test".to_string(),
    ///         vec![],
    ///         std::time::Duration::from_secs(1)
    ///     )
    /// );
    /// assert_eq!(status.emoji(), "✅");
    /// ```
    pub fn emoji(&self) -> &'static str {
        match self {
            MilestoneStatus::NotStarted => "⏳",
            MilestoneStatus::InProgress => "🔄",
            MilestoneStatus::Completed(_) => "✅",
            MilestoneStatus::Blocked(_) => "🚫",
            MilestoneStatus::Skipped(_) => "⏭️",
        }
    }
}

// =============================================================================
// Реализации для ProgressTracker
// =============================================================================

impl ProgressTracker {
    /// Создает новый трекер прогресса со всеми вехами в начальном состоянии
    ///
    /// Инициализирует все вехи проекта со статусом `NotStarted`
    /// и базовыми требованиями для каждой вехи.
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let tracker = ProgressTracker::new();
    /// let report = tracker.generate_report();
    /// assert_eq!(report.overall_progress, 0.0);
    /// ```
    pub fn new() -> Self {
        let mut milestones = HashMap::new();

        // Инициализируем все вехи с прогрессом по умолчанию
        for milestone in Milestone::all() {
            milestones.insert(
                milestone.clone(),
                MilestoneProgress {
                    status: MilestoneStatus::NotStarted,
                    last_checked: Utc::now(),
                    requirements: Self::get_requirements(&milestone),
                    notes: None,
                    target_date: None,
                },
            );
        }

        Self {
            milestones,
            history: Vec::new(),
        }
    }

    /// Обновляет статус вехи на основе результатов проверки качества
    ///
    /// Если проверка качества пройдена успешно, веха отмечается как завершенная.
    /// В противном случае статус меняется на "In Progress".
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для обновления
    /// * `result` - Результат проверки качества
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::{ProgressTracker, Milestone, MilestoneGates};
    ///
    /// let mut tracker = ProgressTracker::new();
    /// let gate = MilestoneGates::milestone_1();
    /// let result = gate.check();
    ///
    /// tracker.update_milestone(Milestone::Foundation, result);
    /// ```
    pub fn update_milestone(&mut self, milestone: Milestone, result: QualityResult) {
        if let Some(progress) = self.milestones.get_mut(&milestone) {
            progress.last_checked = Utc::now();
            progress.status = if result.passed {
                MilestoneStatus::Completed(result)
            } else {
                MilestoneStatus::InProgress
            };
        }
    }

    /// Устанавливает веху как заблокированную с указанием причины
    ///
    /// Заблокированные вехи не учитываются в общем прогрессе до снятия блокировки.
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для блокировки
    /// * `reason` - Причина блокировки
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::{ProgressTracker, Milestone};
    ///
    /// let mut tracker = ProgressTracker::new();
    /// tracker.block_milestone(
    ///     Milestone::Foundation,
    ///     "Waiting for external dependencies".to_string()
    /// );
    /// ```
    pub fn block_milestone(&mut self, milestone: Milestone, reason: String) {
        if let Some(progress) = self.milestones.get_mut(&milestone) {
            progress.last_checked = Utc::now();
            progress.status = MilestoneStatus::Blocked(reason);
        }
    }

    /// Устанавливает веху как находящуюся в процессе выполнения
    ///
    /// Используется когда начинается активная работа над вехой.
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для отметки как "In Progress"
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::{ProgressTracker, Milestone};
    ///
    /// let mut tracker = ProgressTracker::new();
    /// tracker.start_milestone(Milestone::Foundation);
    /// ```
    pub fn start_milestone(&mut self, milestone: Milestone) {
        if let Some(progress) = self.milestones.get_mut(&milestone) {
            progress.last_checked = Utc::now();
            progress.status = MilestoneStatus::InProgress;
        }
    }

    /// Добавляет заметку к вехе
    ///
    /// Заметки могут содержать дополнительную информацию о состоянии вехи,
    /// проблемах, зависимостях или других relevant details.
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для добавления заметки
    /// * `note` - Текст заметки
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::{ProgressTracker, Milestone};
    ///
    /// let mut tracker = ProgressTracker::new();
    /// tracker.add_note(
    ///     Milestone::Foundation,
    ///     "Additional work needed on error handling".to_string()
    /// );
    /// ```
    pub fn add_note(&mut self, milestone: Milestone, note: String) {
        if let Some(progress) = self.milestones.get_mut(&milestone) {
            progress.notes = Some(note);
        }
    }

    /// Получает текущий прогресс для конкретной вехи
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для получения прогресса
    ///
    /// # Возвращает
    ///
    /// `Some(&MilestoneProgress)` если веха существует, иначе `None`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::{ProgressTracker, Milestone};
    ///
    /// let tracker = ProgressTracker::new();
    /// let progress = tracker.get_progress(&Milestone::Foundation);
    /// assert!(progress.is_some());
    /// ```
    pub fn get_progress(&self, milestone: &Milestone) -> Option<&MilestoneProgress> {
        self.milestones.get(milestone)
    }

    /// Генерирует комплексный отчет о прогрессе всех вех
    ///
    /// Собирает информацию о всех вехах, вычисляет общий прогресс
    /// и формирует структурированный отчет.
    ///
    /// # Возвращает
    ///
    /// `ProgressReport` с текущим состоянием всех вех
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let tracker = ProgressTracker::new();
    /// let report = tracker.generate_report();
    /// println!("Overall progress: {:.1}%", report.overall_progress);
    /// ```
    pub fn generate_report(&self) -> ProgressReport {
        let mut report = ProgressReport {
            milestones: self.milestones.clone(),
            generated_at: Utc::now(),
            overall_progress: 0.0,
            summary: ProgressSummary {
                total_milestones: 0,
                completed_milestones: 0,
                in_progress_milestones: 0,
                not_started_milestones: 0,
                blocked_milestones: 0,
            },
        };

        report.calculate_summary();
        report
    }

    /// Сохраняет текущее состояние в историю прогресса
    ///
    /// Позволяет отслеживать изменения прогресса во времени
    /// и анализировать тренды разработки.
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let mut tracker = ProgressTracker::new();
    /// tracker.snapshot(); // Сохраняем начальное состояние
    /// ```
    pub fn snapshot(&mut self) {
        let report = self.generate_report();
        self.history.push(report);
    }

    /// Получает историю прогресса
    ///
    /// # Возвращает
    ///
    /// Срез со всеми сохраненными отчетами прогресса
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let mut tracker = ProgressTracker::new();
    /// tracker.snapshot();
    /// let history = tracker.history();
    /// assert_eq!(history.len(), 1);
    /// ```
    pub fn history(&self) -> &[ProgressReport] {
        &self.history
    }

    /// Получает требования для конкретной вехи
    ///
    /// Внутренний метод для инициализации требований каждой вехи.
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для получения требований
    ///
    /// # Возвращает
    ///
    /// Вектор строк с требованиями вехи
    fn get_requirements(milestone: &Milestone) -> Vec<String> {
        match milestone {
            Milestone::Foundation => vec![
                "Core types and traits implemented and tested".to_string(),
                "Basic error system with proper error conversions".to_string(),
                "Serialization/deserialization tests passing for all types".to_string(),
                "Documentation generated without warnings".to_string(),
                "CI/CD pipeline running successfully".to_string(),
            ],
            Milestone::Infrastructure => vec![
                "Security validator detecting all known attack patterns".to_string(),
                "Ollama client with circuit breaker and retry logic".to_string(),
                "Safe file operations with atomic writes".to_string(),
                "Platform abstractions working on all target platforms".to_string(),
                "Performance benchmarks meeting targets".to_string(),
            ],
            Milestone::AICore => vec![
                "Command analysis pipeline working end-to-end".to_string(),
                "Hallucination detection operational with >90% accuracy".to_string(),
                "Training engine implemented and tested".to_string(),
                "Performance targets met for all analysis types".to_string(),
                "Cache system reducing latency by >60%".to_string(),
            ],
            Milestone::WebInterface => vec![
                "Tera templates rendering correctly without errors".to_string(),
                "HTMX interactions working without JavaScript".to_string(),
                "Typed HTTP responses with guaranteed security headers".to_string(),
                "All components reusable and properly documented".to_string(),
                "Web server handling >100 RPS".to_string(),
            ],
            Milestone::Integration => vec![
                "Daemon running with all services integrated".to_string(),
                "CLI commands functional with proper error handling".to_string(),
                "Shell integration working for ZSH, Bash, Fish".to_string(),
                "IPC communication reliable and performant".to_string(),
                "Health monitoring operational".to_string(),
            ],
            Milestone::Production => vec![
                "All unit and integration tests passing".to_string(),
                "Performance benchmarks meeting all targets".to_string(),
                "Security audits clean with no critical vulnerabilities".to_string(),
                "Documentation complete and up-to-date".to_string(),
                "Cross-platform testing successful".to_string(),
            ],
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Реализации для ProgressReport
// =============================================================================

impl ProgressReport {
    /// Вычисляет сводную статистику для отчета
    ///
    /// Подсчитывает количество вех в каждом статусе и вычисляет
    /// общий прогресс проекта как процент завершенных вех.
    pub fn calculate_summary(&mut self) {
        let mut completed = 0;
        let mut in_progress = 0;
        let mut not_started = 0;
        let mut blocked = 0;

        for progress in self.milestones.values() {
            match progress.status {
                MilestoneStatus::Completed(_) => completed += 1,
                MilestoneStatus::InProgress => in_progress += 1,
                MilestoneStatus::NotStarted => not_started += 1,
                MilestoneStatus::Blocked(_) => blocked += 1,
                MilestoneStatus::Skipped(_) => {} // Пропущенные не учитываются в прогрессе
            }
        }

        let total = completed + in_progress + not_started + blocked;
        self.summary = ProgressSummary {
            total_milestones: total,
            completed_milestones: completed,
            in_progress_milestones: in_progress,
            not_started_milestones: not_started,
            blocked_milestones: blocked,
        };

        self.overall_progress = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };
    }

    /// Конвертирует отчет в формат Markdown
    ///
    /// Создает хорошо структурированный Markdown документ с:
    /// - Заголовком и метаинформацией
    /// - Визуальным прогресс-баром
    /// - Сводной таблицей статистики
    /// - Детальными секциями для каждой вехи
    ///
    /// # Возвращает
    ///
    /// Строку с отчетом в Markdown формате
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let tracker = ProgressTracker::new();
    /// let report = tracker.generate_report();
    /// let markdown = report.to_markdown();
    /// std::fs::write("progress_report.md", markdown).unwrap();
    /// ```
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Заголовок
        md.push_str(&format!("# Project Progress Report\n\n"));
        md.push_str(&format!("**Generated:** {}\n", self.generated_at));
        md.push_str(&format!(
            "**Overall Progress:** {:.1}%\n\n",
            self.overall_progress
        ));

        // Прогресс-бар
        md.push_str(&self.progress_bar());
        md.push_str("\n\n");

        // Сводная таблица
        md.push_str("## Summary\n\n");
        md.push_str(&self.summary_table());
        md.push_str("\n");

        // Детали по вехам
        md.push_str("## Milestone Details\n\n");
        for milestone in Milestone::all() {
            if let Some(progress) = self.milestones.get(&milestone) {
                md.push_str(&self.milestone_section(&milestone, progress));
                md.push_str("\n");
            }
        }

        md
    }

    /// Генерирует визуальный прогресс-бар
    ///
    /// Создает текстовый прогресс-бар вида `[██████░░░░] 60.0%`
    /// для визуального представления общего прогресса.
    ///
    /// # Возвращает
    ///
    /// Строку с прогресс-баром
    pub fn progress_bar(&self) -> String {
        const WIDTH: usize = 20;
        let filled = (self.overall_progress / 100.0 * WIDTH as f32).round() as usize;
        let empty = WIDTH - filled;

        format!(
            "[{}{}] {:.1}%",
            "█".repeat(filled),
            "░".repeat(empty),
            self.overall_progress
        )
    }

    /// Генерирует сводную таблицу в Markdown формате
    ///
    /// Создает таблицу с количеством вех в каждом статусе.
    ///
    /// # Возвращает
    ///
    /// Строку с Markdown таблицей
    fn summary_table(&self) -> String {
        format!(
            "| Status | Count |\n|--------|-------|\n\
             | ✅ Completed | {} |\n\
             | 🔄 In Progress | {} |\n\
             | ⏳ Not Started | {} |\n\
             | 🚫 Blocked | {} |\n\
             | **Total** | **{}** |",
            self.summary.completed_milestones,
            self.summary.in_progress_milestones,
            self.summary.not_started_milestones,
            self.summary.blocked_milestones,
            self.summary.total_milestones,
        )
    }

    /// Генерирует секцию для конкретной вехи
    ///
    /// Создает детализированную секцию Markdown для одной вехи,
    /// включая статус, описание, требования и результаты проверок.
    ///
    /// # Аргументы
    ///
    /// * `milestone` - Веха для генерации секции
    /// * `progress` - Прогресс вехи
    ///
    /// # Возвращает
    ///
    /// Строку с Markdown секцией вехи
    fn milestone_section(&self, milestone: &Milestone, progress: &MilestoneProgress) -> String {
        let status_emoji = progress.status.emoji();
        let status_text = progress.status.as_str();

        let mut section = format!(
            "### {} {}: {}\n\n",
            status_emoji,
            milestone.display_name(),
            status_text
        );

        section.push_str(&format!("**Description:** {}\n\n", milestone.description()));
        section.push_str(&format!("**Last Checked:** {}\n\n", progress.last_checked));

        if let Some(notes) = &progress.notes {
            section.push_str(&format!("**Notes:** {}\n\n", notes));
        }

        // Чеклист требований
        section.push_str("**Requirements:**\n\n");
        for requirement in &progress.requirements {
            let checkbox = if progress.status.is_completed() {
                "- [x]"
            } else {
                "- [ ]"
            };
            section.push_str(&format!("{} {}\n", checkbox, requirement));
        }

        // Результаты качества если завершено
        if let MilestoneStatus::Completed(quality_result) = &progress.status {
            section.push_str("\n**Quality Results:**\n\n");
            section.push_str(&format!(
                "- **Status:** {}\n",
                if quality_result.passed {
                    "PASSED ✅"
                } else {
                    "FAILED ❌"
                }
            ));
            section.push_str(&format!(
                "- **Duration:** {:?}\n",
                quality_result.total_duration
            ));
            section.push_str(&format!(
                "- **Criteria Passed:** {}/{}\n",
                quality_result.summary.passed_criteria, quality_result.summary.total_criteria
            ));
        }

        section
    }

    /// Конвертирует отчет в JSON формат
    ///
    /// Создает JSON представление отчета для машинной обработки
    /// или интеграции с другими системами.
    ///
    /// # Возвращает
    ///
    /// `Result<String, serde_json::Error>` с JSON строкой или ошибкой сериализации
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::ProgressTracker;
    ///
    /// let tracker = ProgressTracker::new();
    /// let report = tracker.generate_report();
    /// let json = report.to_json().unwrap();
    /// println!("{}", json);
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// =============================================================================
// Модуль тестирования
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Тестирует создание трекера прогресса
    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new();
        assert_eq!(tracker.milestones.len(), 6);

        let foundation_progress = tracker.get_progress(&Milestone::Foundation);
        assert!(foundation_progress.is_some());
        assert!(matches!(
            foundation_progress.unwrap().status,
            MilestoneStatus::NotStarted
        ));
    }

    /// Тестирует отображаемые имена вех
    #[test]
    fn test_milestone_display_names() {
        assert_eq!(Milestone::Foundation.display_name(), "Foundation Complete");
        assert_eq!(
            Milestone::Infrastructure.display_name(),
            "Infrastructure Ready"
        );
        assert_eq!(Milestone::AICore.display_name(), "AI Core Functional");
        assert_eq!(Milestone::WebInterface.display_name(), "Web Interface Live");
        assert_eq!(
            Milestone::Integration.display_name(),
            "Integration Complete"
        );
        assert_eq!(Milestone::Production.display_name(), "Production Ready");
    }

    /// Тестирует описания вех
    #[test]
    fn test_milestone_descriptions() {
        for milestone in Milestone::all() {
            let description = milestone.description();
            assert!(
                !description.is_empty(),
                "Description for {:?} is empty",
                milestone
            );
        }
    }

    /// Тестирует статусы вех
    #[test]
    fn test_milestone_status() {
        let not_started = MilestoneStatus::NotStarted;
        assert!(!not_started.is_completed());
        assert!(!not_started.is_blocked());
        assert!(!not_started.is_in_progress());
        assert_eq!(not_started.as_str(), "Not Started");
        assert_eq!(not_started.emoji(), "⏳");

        let in_progress = MilestoneStatus::InProgress;
        assert!(!in_progress.is_completed());
        assert!(!in_progress.is_blocked());
        assert!(in_progress.is_in_progress());
        assert_eq!(in_progress.as_str(), "In Progress");
        assert_eq!(in_progress.emoji(), "🔄");

        let blocked = MilestoneStatus::Blocked("Test".to_string());
        assert!(!blocked.is_completed());
        assert!(blocked.is_blocked());
        assert!(!blocked.is_in_progress());
        assert_eq!(blocked.as_str(), "Blocked");
        assert_eq!(blocked.emoji(), "🚫");
    }

    /// Тестирует генерацию отчета
    #[test]
    fn test_progress_report_generation() {
        let tracker = ProgressTracker::new();
        let report = tracker.generate_report();

        assert_eq!(report.milestones.len(), 6);
        assert!(report.overall_progress >= 0.0);
        assert!(report.overall_progress <= 100.0);

        let markdown = report.to_markdown();
        assert!(markdown.contains("Project Progress Report"));
        assert!(markdown.contains("Foundation Complete"));
        assert!(markdown.contains("Summary"));
    }

    /// Тестирует обновление статуса вехи
    #[test]
    #[allow(unused_imports)] // в тесте ругалось на QualitySummary
    fn test_update_milestone() {
        use crate::quality_gates::{CriterionResult, QualityResult, QualitySummary};
        use std::time::Duration;

        let mut tracker = ProgressTracker::new();

        let quality_result = QualityResult::success(
            "Test".to_string(),
            vec![CriterionResult::success(
                "test".to_string(),
                "output".to_string(),
                Duration::from_secs(1),
            )],
            Duration::from_secs(1),
        );

        tracker.update_milestone(Milestone::Foundation, quality_result);

        let progress = tracker.get_progress(&Milestone::Foundation).unwrap();
        assert!(matches!(progress.status, MilestoneStatus::Completed(_)));
    }

    /// Тестирует блокировку вехи
    #[test]
    fn test_block_milestone() {
        let mut tracker = ProgressTracker::new();

        tracker.block_milestone(
            Milestone::Foundation,
            "Waiting for dependencies".to_string(),
        );

        let progress = tracker.get_progress(&Milestone::Foundation).unwrap();
        assert!(matches!(progress.status, MilestoneStatus::Blocked(_)));

        if let MilestoneStatus::Blocked(reason) = &progress.status {
            assert_eq!(reason, "Waiting for dependencies");
        }
    }

    /// Тестирует добавление заметок
    #[test]
    fn test_add_note() {
        let mut tracker = ProgressTracker::new();

        tracker.add_note(Milestone::Foundation, "Additional work needed".to_string());

        let progress = tracker.get_progress(&Milestone::Foundation).unwrap();
        assert_eq!(progress.notes, Some("Additional work needed".to_string()));
    }

    /// Тестирует создание снимков истории
    #[test]
    fn test_snapshot() {
        let mut tracker = ProgressTracker::new();

        assert_eq!(tracker.history().len(), 0);

        tracker.snapshot();
        assert_eq!(tracker.history().len(), 1);

        tracker.snapshot();
        assert_eq!(tracker.history().len(), 2);
    }

    /// Тестирует прогресс-бар
    #[test]
    fn test_progress_bar() {
        let mut report = ProgressReport {
            milestones: HashMap::new(),
            generated_at: Utc::now(),
            overall_progress: 75.0,
            summary: ProgressSummary {
                total_milestones: 4,
                completed_milestones: 3,
                in_progress_milestones: 1,
                not_started_milestones: 0,
                blocked_milestones: 0,
            },
        };

        let progress_bar = report.progress_bar();
        assert!(progress_bar.contains("75.0%"));

        report.overall_progress = 0.0;
        let zero_progress = report.progress_bar();
        assert!(zero_progress.contains("0.0%"));

        report.overall_progress = 100.0;
        let full_progress = report.progress_bar();
        assert!(full_progress.contains("100.0%"));
    }
}
