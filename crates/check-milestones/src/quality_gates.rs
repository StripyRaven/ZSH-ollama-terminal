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
//!
//! ## Версия модуля
//! ver 1.1.0

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Результат выполнения одного критерия качества
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    /// Уникальное имя критерия
    pub name: String,
    /// Флаг успешного выполнения
    pub passed: bool,
    /// Вывод команды (stdout + stderr)
    pub output: String,
    /// Время выполнения критерия
    pub duration: Duration,
    /// Сообщение об ошибке (если есть)
    pub error: Option<String>,
}

/// Общий результат проверки Quality Gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResult {
    /// Название quality gate
    pub gate_name: String,
    /// Общий статус прохождения
    pub passed: bool,
    /// Сводное сообщение
    pub message: String,
    /// Детальные результаты по каждому критерию
    pub details: Vec<CriterionResult>,
    /// Общее время выполнения всех критериев
    pub total_duration: Duration,
    /// Статистическая сводка
    pub summary: QualitySummary,
}

/// Статистическая сводка результатов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    /// Общее количество критериев
    pub total_criteria: usize,
    /// Количество успешно пройденных
    pub passed_criteria: usize,
    /// Количество непройденных
    pub failed_criteria: usize,
    /// Количество пропущенных
    pub skipped_criteria: usize,
}

/// Индивидуальный критерий проверки
#[derive(Debug, Clone)]
pub struct QualityCriterion {
    /// Имя критерия
    pub name: String,
    /// Shell команда для выполнения
    pub command: String,
    /// Описание критерия
    pub description: String,
    /// Является ли обязательным
    pub required: bool,
    /// Максимальное время выполнения
    pub timeout: Option<Duration>,
}

/// Коллекция критериев качества
#[derive(Debug, Clone)]
pub struct QualityGate {
    /// Название gate
    pub name: String,
    /// Список критериев
    pub criteria: Vec<QualityCriterion>,
    /// Режим строгой проверки (остановка при первой ошибке)
    pub strict_mode: bool,
    /// Переменные окружения для команд
    pub environment: HashMap<String, String>,
}

// =============================================================================
// Реализации для CriterionResult
// =============================================================================

impl CriterionResult {
    /// Создаёт успешный результат критерия
    pub fn success(name: String, output: String, duration: Duration) -> Self {
        Self {
            name,
            passed: true,
            output,
            duration,
            error: None,
        }
    }

    /// Создаёт неуспешный результат критерия
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
    /// Создаёт успешный результат Quality Gate
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

    /// Создаёт неуспешный результат Quality Gate
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

    /// Форматирует результат как цветную строку для терминала
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
    /// Создаёт новый Quality Gate с именем
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            criteria: Vec::new(),
            strict_mode: true,
            environment: HashMap::new(),
        }
    }

    /// Добавляет обязательный критерий (таймаут 5 минут)
    pub fn add_criterion(mut self, name: &str, command: &str, description: &str) -> Self {
        self.criteria.push(QualityCriterion {
            name: name.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            required: true,
            timeout: Some(Duration::from_secs(300)),
        });
        self
    }

    /// Добавляет опциональный критерий (таймаут 5 минут)
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

    /// Добавляет критерий с пользовательским таймаутом
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
    pub fn set_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Добавляет переменную окружения для всех команд
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    /// Запускает все критерии и возвращает результат
    pub fn check(&self) -> QualityResult {
        let start_time = Instant::now();
        let mut details = Vec::new();
        let mut all_required_passed = true;

        for criterion in &self.criteria {
            let criterion_start = Instant::now();
            let criterion_result = match self.execute_criterion(criterion) {
                Ok(mut res) => {
                    res.duration = criterion_start.elapsed();
                    res
                }
                Err(e) => {
                    let error_msg = format!("Command execution error: {}", e);
                    CriterionResult::failure(
                        criterion.name.clone(),
                        String::new(),
                        criterion_start.elapsed(),
                        error_msg,
                    )
                }
            };

            details.push(criterion_result.clone());

            if !criterion_result.passed && criterion.required && self.strict_mode {
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

    /// Выполняет один критерий (внутренний метод)
    fn execute_criterion(&self, criterion: &QualityCriterion) -> Result<CriterionResult> {
        let output = self.execute_command(&criterion.command, criterion.timeout)?;

        if output.status.success() {
            Ok(CriterionResult::success(
                criterion.name.clone(),
                String::from_utf8_lossy(&output.stdout).to_string(),
                Duration::from_secs(0),
            ))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(CriterionResult::failure(
                criterion.name.clone(),
                String::from_utf8_lossy(&output.stdout).to_string(),
                Duration::from_secs(0),
                error_msg,
            ))
        }
    }

    /// Выполняет команду с поддержкой таймаута
    fn execute_command(&self, command: &str, timeout: Option<Duration>) -> Result<Output> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(command);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);
            cmd
        };

        for (key, value) in &self.environment {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn command: {}", command))?;

        match timeout {
            Some(duration) => {
                let start = Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => {
                            return child.wait_with_output().with_context(|| {
                                format!("Failed to wait for command: {}", command)
                            });
                        }
                        Ok(None) => {
                            if start.elapsed() >= duration {
                                let _ = child.kill();
                                let _ = child.wait_with_output();
                                anyhow::bail!(
                                    "Command timed out after {:?}: {}",
                                    duration,
                                    command
                                );
                            }
                            std::thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            anyhow::bail!("Failed to wait for command: {}", e);
                        }
                    }
                }
            }
            None => child
                .wait_with_output()
                .with_context(|| format!("Failed to execute command: {}", command)),
        }
    }
}

// =============================================================================
// Модуль тестирования
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_quality_result_creation() {
        let success_details = vec![
            CriterionResult::success(
                "test1".to_string(),
                "output1".to_string(),
                Duration::from_secs(1),
            ),
            CriterionResult::success(
                "test2".to_string(),
                "output2".to_string(),
                Duration::from_secs(1),
            ),
        ];
        let success_result = QualityResult::success(
            "Test Gate".to_string(),
            success_details,
            Duration::from_secs(2),
        );
        assert!(success_result.passed);
        assert_eq!(success_result.summary.passed_criteria, 2);

        let failure_details = vec![
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
        let failed_result = QualityResult::failed(
            "Test Gate".to_string(),
            "Some failed".to_string(),
            failure_details,
            Duration::from_secs(2),
        );
        assert!(!failed_result.passed);
        assert_eq!(failed_result.summary.failed_criteria, 1);
    }

    #[test]
    fn test_colored_output() {
        let details = vec![CriterionResult::success(
            "test1".to_string(),
            "output1".to_string(),
            Duration::from_secs(1),
        )];
        let result =
            QualityResult::success("Test Gate".to_string(), details, Duration::from_secs(1));
        let colored = result.to_colored_string();
        assert!(colored.contains("Test Gate") && colored.contains("PASSED"));
    }

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

    #[test]
    fn test_execute_real_command() {
        let gate = QualityGate::new("Test");
        let output = gate.execute_command("echo hello", None).unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
    }

    #[test]
    fn test_command_timeout() {
        let gate = QualityGate::new("Test");
        let result = gate.execute_command("sleep 2", Some(Duration::from_secs(1)));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }
}
