//! # Quality Gates Core Module
//!
//! Основной модуль системы Quality Gates. Предоставляет функциональность для определения
//! и выполнения критериев качества, которые должны быть выполнены для каждой вехи проекта.
//!
//! ## Архитектура модуля
//!
//! Модуль построен вокруг трех основных структур:
//! - [`QualityGate`] - коллекция критериев для проверки качества вехи
//! - [`QualityResult`] - результат выполнения всех критериев quality gate
//! - [`QualityCriterion`] - индивидуальный критерий проверки с командой и метаданными
//!
//! ## Жизненный цикл Quality Gate
//!
//! 1. **Создание** - Определение критериев через builder pattern
//! 2. **Выполнение** - Последовательный запуск всех критериев
//! 3. **Анализ** - Сбор и агрегация результатов
//! 4. **Отчет** - Генерация детализированного отчета
//!
//! ## Пример использования
//!
//! ```rust
//! use check_milestones::quality_gates::{QualityGate, QualityCriterion};
//!
//! let gate = QualityGate::new("My Quality Gate")
//!     .add_criterion("compilation", "cargo check", "Project compiles")
//!     .add_criterion("testing", "cargo test", "Tests pass")
//!     .set_strict_mode(true);
//!
//! let result = gate.check();
//! if result.passed {
//!     println!("Quality gate passed!");
//! } else {
//!     eprintln!("Quality gate failed: {}", result.message);
//! }
//! ```

use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use crate::Error;

/// Результат выполнения одного критерия качества
///
/// Содержит детальную информацию о выполнении отдельного критерия,
/// включая статус, вывод команды, время выполнения и возможные ошибки.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    /// Уникальное имя критерия для идентификации
    pub name: String,
    /// Флаг успешного выполнения критерия
    pub passed: bool,
    /// Стандартный вывод выполнения команды
    pub output: String,
    /// Время, затраченное на выполнение критерия
    pub duration: Duration,
    /// Сообщение об ошибке, если критерий не пройден
    pub error: Option<String>,
}

/// Общий результат проверки Quality Gate
///
/// Агрегирует результаты всех критериев в gate и предоставляет
/// сводную информацию о прохождении проверки качества.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResult {
    /// Название quality gate для идентификации
    pub gate_name: String,
    /// Общий статус прохождения gate (true если все обязательные критерии пройдены)
    pub passed: bool,
    /// Сводное сообщение о результате проверки
    pub message: String,
    /// Детальные результаты по каждому отдельному критерию
    pub details: Vec<CriterionResult>,
    /// Общее время выполнения всех критериев в gate
    pub total_duration: Duration,
    /// Статистическая сводка по результатам
    pub summary: QualitySummary,
}

/// Статистическая сводка результатов Quality Gate
///
/// Предоставляет количественные метрики для быстрого анализа
/// результатов выполнения quality gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    /// Общее количество критериев в gate
    pub total_criteria: usize,
    /// Количество успешно пройденных критериев
    pub passed_criteria: usize,
    /// Количество непройденных критериев
    pub failed_criteria: usize,
    /// Количество пропущенных критериев (в нестрогом режиме)
    pub skipped_criteria: usize,
}

/// Индивидуальный критерий проверки в рамках Quality Gate
///
/// Представляет одну проверку, которая должна быть выполнена
/// для подтверждения качества конкретного аспекта вехи.
#[derive(Debug, Clone)]
pub struct QualityCriterion {
    /// Уникальное имя критерия для идентификации в отчетах
    pub name: String,
    /// Shell команда для выполнения проверки
    pub command: String,
    /// Человеко-читаемое описание того, что проверяет критерий
    pub description: String,
    /// Определяет, является ли критерий обязательным для прохождения gate
    pub required: bool,
    /// Максимальное время выполнения команды (None - без ограничений)
    pub timeout: Option<Duration>,
}

/// Коллекция критериев, определяющих стандарты качества для вехи
///
/// Quality Gate представляет собой набор связанных критериев, которые
/// должны быть выполнены для подтверждения готовности вехи.
/// Поддерживает два режима работы: строгий (остановка при первой ошибке)
/// и нестрогий (выполнение всех критериев).
#[derive(Debug, Clone)]
pub struct QualityGate {
    /// Человеко-читаемое имя quality gate
    pub name: String,
    /// Список всех критериев, входящих в gate
    pub criteria: Vec<QualityCriterion>,
    /// Режим строгой проверки - остановка при первой неудаче обязательного критерия
    pub strict_mode: bool,
    /// Переменные окружения для выполнения команд критериев
    pub environment: HashMap<String, String>,
}

// =============================================================================
// Реализации для CriterionResult
// =============================================================================

impl CriterionResult {
    /// Создает новый успешный результат критерия
    ///
    /// # Аргументы
    ///
    /// * `name` - Имя критерия
    /// * `output` - Вывод команды выполнения
    /// * `duration` - Время выполнения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use check_milestones::quality_gates::CriterionResult;
    ///
    /// let result = CriterionResult::success(
    ///     "compilation".to_string(),
    ///     "Compilation successful".to_string(),
    ///     Duration::from_secs(5)
    /// );
    /// assert!(result.passed);
    /// ```
    pub fn success(name: String, output: String, duration: Duration) -> Self {
        Self {
            name,
            passed: true,
            output,
            duration,
            error: None,
        }
    }

    /// Создает новый неуспешный результат критерия
    ///
    /// # Аргументы
    ///
    /// * `name` - Имя критерия
    /// * `output` - Вывод команды выполнения
    /// * `duration` - Время выполнения
    /// * `error` - Сообщение об ошибке
    ///
    /// # Пример
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use check_milestones::quality_gates::CriterionResult;
    ///
    /// let result = CriterionResult::failure(
    ///     "compilation".to_string(),
    ///     "".to_string(),
    ///     Duration::from_secs(5),
    ///     "Compilation failed".to_string()
    /// );
    /// assert!(!result.passed);
    /// ```
    pub fn failure(name: String, output: String, duration: Duration, error: String) -> Self {
        Self {
            name,
            passed: false,
            output,
            duration,
            error: Some(error),
        }
    }
}

// =============================================================================
// Реализации для QualityResult
// =============================================================================

impl QualityResult {
    /// Создает новый успешный результат Quality Gate
    ///
    /// # Аргументы
    ///
    /// * `gate_name` - Название quality gate
    /// * `details` - Детальные результаты критериев
    /// * `total_duration` - Общее время выполнения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use check_milestones::quality_gates::{QualityResult, CriterionResult};
    ///
    /// let details = vec![
    ///     CriterionResult::success("test".to_string(), "output".to_string(), Duration::from_secs(1))
    /// ];
    /// let result = QualityResult::success("Test Gate".to_string(), details, Duration::from_secs(1));
    /// assert!(result.passed);
    /// ```
    pub fn success(
        gate_name: String,
        details: Vec<CriterionResult>,
        total_duration: Duration,
    ) -> Self {
        let passed_count = details.iter().filter(|d| d.passed).count();
        let failed_count = details.len() - passed_count;

        Self {
            gate_name,
            passed: failed_count == 0,
            message: "All quality criteria passed successfully".to_string(),
            details: details.clone(),
            total_duration,
            summary: QualitySummary {
                total_criteria: details.len(),
                passed_criteria: passed_count,
                failed_criteria: failed_count,
                skipped_criteria: 0,
            },
        }
    }

    /// Создает новый неуспешный результат Quality Gate
    ///
    /// # Аргументы
    ///
    /// * `gate_name` - Название quality gate
    /// * `message` - Сообщение об ошибке
    /// * `details` - Детальные результаты критериев
    /// * `total_duration` - Общее время выполнения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use check_milestones::quality_gates::{QualityResult, CriterionResult};
    ///
    /// let details = vec![
    ///     CriterionResult::failure(
    ///         "test".to_string(),
    ///         "".to_string(),
    ///         Duration::from_secs(1),
    ///         "Failed".to_string()
    ///     )
    /// ];
    /// let result = QualityResult::failed(
    ///     "Test Gate".to_string(),
    ///     "Gate failed".to_string(),
    ///     details,
    ///     Duration::from_secs(1)
    /// );
    /// assert!(!result.passed);
    /// ```
    pub fn failed(
        gate_name: String,
        message: String,
        details: Vec<CriterionResult>,
        total_duration: Duration,
    ) -> Self {
        let passed_count = details.iter().filter(|d| d.passed).count();
        let failed_count = details.len() - passed_count;

        Self {
            gate_name,
            passed: false,
            message,
            details: details.clone(),
            total_duration,
            summary: QualitySummary {
                total_criteria: details.len(),
                passed_criteria: passed_count,
                failed_criteria: failed_count,
                skipped_criteria: 0,
            },
        }
    }

    /// Форматирует результат как цветную строку для вывода в терминал
    ///
    /// Создает человеко-читаемый вывод с использованием цветов и эмодзи
    /// для визуального отличия успешных и неуспешных проверок.
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::{QualityResult, CriterionResult};
    /// use std::time::Duration;
    ///
    /// let result = QualityResult::success(
    ///     "Test Gate".to_string(),
    ///     vec![],
    ///     Duration::from_secs(1)
    /// );
    /// let colored_output = result.to_colored_string();
    /// println!("{}", colored_output);
    /// ```
    pub fn to_colored_string(&self) -> String {
        let status = if self.passed {
            "PASSED".green().bold()
        } else {
            "FAILED".red().bold()
        };

        let mut output = format!("Quality Gate: {} - {}\n", self.gate_name.bold(), status);

        output.push_str(&format!("Message: {}\n", self.message));
        output.push_str(&format!("Duration: {:?}\n", self.total_duration));
        output.push_str(&format!(
            "Summary: {}/{} criteria passed\n\n",
            self.summary.passed_criteria, self.summary.total_criteria
        ));

        output.push_str("Detailed results:\n");
        for detail in &self.details {
            let status_icon = if detail.passed { "✅" } else { "❌" };
            let status_text = if detail.passed {
                "PASS".green()
            } else {
                "FAIL".red()
            };

            output.push_str(&format!(
                "  {} {}: {} ({:?})\n",
                status_icon, detail.name, status_text, detail.duration
            ));

            if !detail.passed {
                if let Some(error) = &detail.error {
                    output.push_str(&format!("    Error: {}\n", error.red()));
                }
                if !detail.output.is_empty() {
                    let first_line = detail.output.lines().next().unwrap_or("");
                    output.push_str(&format!("    Output: {}\n", first_line.yellow()));
                }
            }
        }

        output
    }
}

// =============================================================================
// Реализации для QualityGate
// =============================================================================

impl QualityGate {
    /// Создает новый Quality Gate с заданным именем
    ///
    /// # Аргументы
    ///
    /// * `name` - Человеко-читаемое имя quality gate
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("My Quality Gate");
    /// assert_eq!(gate.name, "My Quality Gate");
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            criteria: Vec::new(),
            strict_mode: true,
            environment: HashMap::new(),
        }
    }

    /// Добавляет обязательный критерий в Quality Gate
    ///
    /// Обязательные критерии должны быть пройдены для успешного
    /// прохождения quality gate. В строгом режиме выполнение
    /// прекращается при первой неудаче обязательного критерия.
    ///
    /// # Аргументы
    ///
    /// * `name` - Уникальное имя критерия
    /// * `command` - Shell команда для выполнения
    /// * `description` - Описание критерия
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .add_criterion("compilation", "cargo check", "Project compiles");
    /// ```
    pub fn add_criterion(mut self, name: &str, command: &str, description: &str) -> Self {
        self.criteria.push(QualityCriterion {
            name: name.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            required: true,
            timeout: Some(Duration::from_secs(300)), // 5 минут по умолчанию
        });
        self
    }

    /// Добавляет опциональный критерий в Quality Gate
    ///
    /// Опциональные критерии не блокируют прохождение quality gate
    /// при неудаче, но их результаты включаются в отчет.
    ///
    /// # Аргументы
    ///
    /// * `name` - Уникальное имя критерия
    /// * `command` - Shell команда для выполнения
    /// * `description` - Описание критерия
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .add_optional_criterion("benchmarks", "cargo bench", "Performance benchmarks");
    /// ```
    pub fn add_optional_criterion(mut self, name: &str, command: &str, description: &str) -> Self {
        self.criteria.push(QualityCriterion {
            name: name.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            required: false,
            timeout: Some(Duration::from_secs(300)),
        });
        self
    }

    /// Добавляет критерий с кастомным таймаутом
    ///
    /// Позволяет задать специфическое время ожидания для критериев,
    /// которые могут выполняться дольше стандартного таймаута.
    ///
    /// # Аргументы
    ///
    /// * `name` - Уникальное имя критерия
    /// * `command` - Shell команда для выполнения
    /// * `description` - Описание критерия
    /// * `timeout` - Максимальное время выполнения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .add_criterion_with_timeout(
    ///         "long_running",
    ///         "cargo test --release",
    ///         "Long running tests",
    ///         Duration::from_secs(600) // 10 минут
    ///     );
    /// ```
    pub fn add_criterion_with_timeout(
        mut self,
        name: &str,
        command: &str,
        description: &str,
        timeout: Duration,
    ) -> Self {
        self.criteria.push(QualityCriterion {
            name: name.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            required: true,
            timeout: Some(timeout),
        });
        self
    }

    /// Устанавливает режим строгой проверки
    ///
    /// В строгом режиме выполнение quality gate прекращается
    /// при первой неудаче обязательного критерия.
    ///
    /// # Аргументы
    ///
    /// * `strict` - true для включения строгого режима
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .set_strict_mode(true);
    /// ```
    pub fn set_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Добавляет переменную окружения для выполнения команд
    ///
    /// Переменные окружения устанавливаются для всех команд,
    /// выполняемых в рамках этого quality gate.
    ///
    /// # Аргументы
    ///
    /// * `key` - Ключ переменной окружения
    /// * `value` - Значение переменной окружения
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .with_env("RUST_BACKTRACE", "1");
    /// ```
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    /// Выполняет все критерии в этом Quality Gate и возвращает результаты
    ///
    /// Основной метод Quality Gate, который:
    /// 1. Последовательно выполняет все критерии
    /// 2. Собирает результаты выполнения
    /// 3. Агрегирует статистику
    /// 4. Формирует итоговый отчет
    ///
    /// В строгом режиме выполнение прекращается при первой неудаче
    /// обязательного критерия.
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::quality_gates::QualityGate;
    ///
    /// let gate = QualityGate::new("Test Gate")
    ///     .add_criterion("test", "echo 'test'", "Test criterion");
    ///
    /// let result = gate.check();
    /// println!("Gate passed: {}", result.passed);
    /// ```
    pub fn check(&self) -> QualityResult {
        let start_time = Instant::now();
        let mut details = Vec::new();
        let mut all_required_passed = true;

        for criterion in &self.criteria {
            let criterion_start = Instant::now();
            let criterion_result = self.execute_criterion(criterion);
            let criterion_duration = criterion_start.elapsed();

            // Обновляем длительность в результате
            let mut result_with_duration = criterion_result;
            result_with_duration.duration = criterion_duration;

            details.push(result_with_duration.clone());

            // Проверяем, нужно ли остановиться досрочно в строгом режиме
            if !result_with_duration.passed && criterion.required && self.strict_mode {
                all_required_passed = false;
                break;
            }
        }

        let total_duration = start_time.elapsed();

        if all_required_passed {
            QualityResult::success(self.name.clone(), details, total_duration)
        } else {
            QualityResult::failed(
                self.name.clone(),
                "One or more required criteria failed".to_string(),
                details,
                total_duration,
            )
        }
    }

    /// Выполняет один критерий и возвращает его результат
    ///
    /// Внутренний метод, который выполняет shell команду критерия
    /// и преобразует результат в структурированный формат.
    ///
    /// # Аргументы
    ///
    /// * `criterion` - Критерий для выполнения
    ///
    /// # Возвращает
    ///
    /// `CriterionResult` с результатом выполнения
    fn execute_criterion(&self, criterion: &QualityCriterion) -> CriterionResult {
        let output = self.execute_command(&criterion.command, criterion.timeout);

        match output {
            Ok(output) => {
                if output.status.success() {
                    CriterionResult::success(
                        criterion.name.clone(),
                        String::from_utf8_lossy(&output.stdout).to_string(),
                        Duration::from_secs(0), // Будет установлено вызывающей стороной
                    )
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
                    CriterionResult::failure(
                        criterion.name.clone(),
                        String::from_utf8_lossy(&output.stdout).to_string(),
                        Duration::from_secs(0),
                        error_msg,
                    )
                }
            }
            Err(e) => CriterionResult::failure(
                criterion.name.clone(),
                String::new(),
                Duration::from_secs(0),
                e.to_string(),
            ),
        }
    }

    /// Выполняет shell команду с опциональным таймаутом
    ///
    /// Внутренний метод для выполнения shell команд с поддержкой
    /// кросс-платформенности (Windows/Linux/macOS).
    ///
    /// # Аргументы
    ///
    /// * `command` - Команда для выполнения
    /// * `_timeout` - Таймаут выполнения (в текущей реализации не используется)
    ///
    /// # Возвращает
    ///
    /// `Result<Output, Error>` с результатом выполнения команды
    fn execute_command(&self, command: &str, _timeout: Option<Duration>) -> Result<Output, Error> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(command);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);
            cmd
        };

        // Устанавливаем переменные окружения
        for (key, value) in &self.environment {
            cmd.env(key, value);
        }

        // Выполняем команду
        let output = cmd.output().map_err(|e| Error::CommandExecution {
            command: command.to_string(),
            error: e.to_string(),
        })?;

        Ok(output)
    }
}

// =============================================================================
// Модуль тестирования
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Тестирует создание Quality Gate и добавление критериев
    #[test]
    fn test_quality_gate_creation() {
        let gate = QualityGate::new("Test Gate")
            .add_criterion("test1", "echo 'test'", "Test criterion")
            .add_optional_criterion("test2", "echo 'optional'", "Optional criterion");

        assert_eq!(gate.name, "Test Gate");
        assert_eq!(gate.criteria.len(), 2);
        assert!(gate.criteria[0].required);
        assert!(!gate.criteria[1].required);
    }

    /// Тестирует создание результатов критериев
    #[test]
    fn test_criterion_result_creation() {
        let success = CriterionResult::success(
            "test".to_string(),
            "output".to_string(),
            Duration::from_secs(1),
        );
        assert!(success.passed);
        assert_eq!(success.output, "output");
        assert!(success.error.is_none());

        let failure = CriterionResult::failure(
            "test".to_string(),
            "output".to_string(),
            Duration::from_secs(1),
            "error".to_string(),
        );
        assert!(!failure.passed);
        assert_eq!(failure.error, Some("error".to_string()));
    }

    /// Тестирует создание результатов Quality Gate
    #[test]
    fn test_quality_result_creation() {
        let details = vec![
            CriterionResult::success(
                "test1".to_string(),
                "output1".to_string(),
                Duration::from_secs(1),
            ),
            CriterionResult::failure(
                "test2".to_string(),
                "output2".to_string(),
                Duration::from_secs(1),
                "error".to_string(),
            ),
        ];

        let success_result = QualityResult::success(
            "Test Gate".to_string(),
            details.clone(),
            Duration::from_secs(2),
        );

        let failed_result = QualityResult::failed(
            "Test Gate".to_string(),
            "Failed".to_string(),
            details,
            Duration::from_secs(2),
        );

        assert!(success_result.passed);
        assert!(!failed_result.passed);
        assert_eq!(success_result.summary.total_criteria, 2);
        assert_eq!(success_result.summary.passed_criteria, 1);
        assert_eq!(success_result.summary.failed_criteria, 1);
    }

    /// Тестирует форматирование цветного вывода
    #[test]
    fn test_colored_output() {
        let details = vec![CriterionResult::success(
            "test1".to_string(),
            "output1".to_string(),
            Duration::from_secs(1),
        )];

        let result =
            QualityResult::success("Test Gate".to_string(), details, Duration::from_secs(1));

        let colored_output = result.to_colored_string();
        assert!(colored_output.contains("Test Gate"));
        assert!(colored_output.contains("PASSED"));
    }

    /// Тестирует builder методы Quality Gate
    #[test]
    fn test_quality_gate_builder_methods() {
        let gate = QualityGate::new("Test Gate")
            .add_criterion("req", "echo 'req'", "Required")
            .add_optional_criterion("opt", "echo 'opt'", "Optional")
            .set_strict_mode(false)
            .with_env("TEST", "value");

        assert_eq!(gate.criteria.len(), 2);
        assert!(!gate.strict_mode);
        assert_eq!(gate.environment.get("TEST"), Some(&"value".to_string()));
    }
}
