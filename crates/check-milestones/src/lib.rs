//! # Check Milestones - Quality Gates System
//!
//! Автоматизированные проверки качества и отслеживание вех для проекта zsh-ollama-terminal.
//!
//! ## Основные компоненты
//!
//! - **Quality Gates** - система критериев качества для каждой вехи
//! - **Progress Tracker** - отслеживание выполнения вех
//! - **CLI Interface** - командный интерфейс для проверок
//! - **Reporting** - генерация отчетов в различных форматах
//!
//! ## Пример использования
//!
//! ```rust
//! use check_milestones::{MilestoneGates, ProgressTracker};
//!
//! let gate = MilestoneGates::milestone_1();
//! let result = gate.check();
//!
//! let mut tracker = ProgressTracker::new();
//! let report = tracker.generate_report();
//! ```
//!
//! ## Версия
//! ver 1.1.0

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod milestone_gates;
pub mod progress_tracker;
pub mod quality_gates;

// Re-export основных типов для удобного доступа
pub use milestone_gates::MilestoneGates;
pub use progress_tracker::{
    Milestone, MilestoneProgress, MilestoneStatus, ProgressReport, ProgressTracker,
};
pub use quality_gates::{CriterionResult, QualityGate, QualityResult};

/// Prelude модуль для удобного импорта часто используемых типов
pub mod prelude {
    pub use crate::{
        CriterionResult, Milestone, MilestoneGates, MilestoneStatus, ProgressReport,
        ProgressTracker, QualityGate, QualityResult,
    };
}
