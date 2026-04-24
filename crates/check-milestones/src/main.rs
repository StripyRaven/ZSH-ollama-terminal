//! # Check Milestones CLI
//!
//! Командный интерфейс для системы Quality Gates и отслеживания вех проекта zsh-ollama-terminal.
//!
//! ## Основные возможности
//!
//! - Проверка качества отдельных вех через Quality Gates
//! - Комплексная проверка всех вех последовательно
//! - Генерация отчетов о прогрессе в различных форматах
//! - Просмотр информации о критериях вех
//! - Интеграция с CI/CD системами
//!
//! ## Архитектура CLI
//!
//! CLI построен на основе библиотеки `clap` и предоставляет:
//! - Иерархическую структуру команд
//! - Поддержку различных форматов вывода (текст, JSON, Markdown)
//! - Подробный и краткий режимы вывода
//! - Коды возврата для интеграции с скриптами
//!
//! ## Exit codes (УЛУЧШЕНО: добавлена секция с кодами выхода)
//!
//! - `0` – все проверки успешно пройдены
//! - `1` – одна или несколько проверок качества не пройдены, либо произошла внутренняя ошибка
//!
//! ## Примеры использования
//!
//! ```bash
//! # Проверить веху 1
//! check-milestones check-milestone-1
//!
//! # Проверить все вехи
//! check-milestones check-all
//!
//! # Сгенерировать отчет в Markdown
//! check-milestones progress-report --output markdown
//!
//! # Показать информацию о вехе 2
//! check-milestones info 2
//! ```

use check_milestones::{Milestone, MilestoneGates, ProgressReport, ProgressTracker, QualityResult};
use clap::{Parser, Subcommand};
use colored::*;
// УЛУЧШЕНО: добавлен anyhow для улучшенной обработки ошибок
use anyhow::{Context, Result};

/// Основные аргументы командной строки
///
/// Определяет структуру CLI с подкомандами и общими флагами,
/// которые применяются ко всем командам.
#[derive(Parser)]
#[command(
    name = "check-milestones",
    version = "0.1.0",
    author = "zsh-ollama-terminal Team",
    about = "Quality Gates and milestone tracking for zsh-ollama-terminal",
    long_about = "Automated quality checks and progress tracking for each development milestone. \
                 Provides comprehensive reporting and CI/CD integration capabilities."
)]
struct Cli {
    /// Основная команда для выполнения
    #[command(subcommand)]
    command: Commands,

    /// Включить подробный вывод с дополнительной информацией
    ///
    /// В подробном режиме выводятся дополнительные детали выполнения,
    /// включая промежуточные результаты и отладочную информацию.
    #[arg(short, long)]
    verbose: bool,

    /// Формат вывода результатов
    ///
    /// Определяет в каком формате будут представлены результаты работы:
    /// - `text` - Человеко-читаемый текст с цветами (по умолчанию)
    /// - `json` - Структурированный JSON для машинной обработки
    /// - `markdown` - Markdown формат для документации
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,
}

/// Подкоманды CLI
///
/// Определяет все доступные команды системы Check Milestones.
/// Каждая команда соответствует определенному действию с вехами.
#[derive(Subcommand)]
enum Commands {
    /// Проверить Веху 1: Foundation Complete
    ///
    /// Запускает все критерии качества для вехи 1, включая:
    /// - Компиляцию проекта
    /// - Тестирование
    /// - Документацию
    /// - Форматирование кода
    /// - Проверки clippy
    CheckMilestone1,

    /// Проверить Веху 2: Infrastructure Ready
    ///
    /// Запускает все критерии качества для вехи 2, включая:
    /// - Валидатор безопасности
    /// - Клиент Ollama
    /// - Файловые операции
    /// - Платформенные абстракции
    /// - Бенчмарки производительности
    CheckMilestone2,

    /// Проверить Веху 3: AI Core Functional
    ///
    /// Запускает все критерии качества для вехи 3, включая:
    /// - Анализ команд
    /// - Обнаружение галлюцинаций
    /// - Движок обучения
    /// - Целевые показатели производительности
    /// - Кеш-систему
    CheckMilestone3,

    /// Проверить Веху 4: Web Interface Live
    ///
    /// Запускает все критерии качества для вехи 4, включая:
    /// - Шаблоны Tera
    /// - HTMX взаимодействия
    /// - HTTP ответы
    /// - Переиспользуемые компоненты
    /// - Веб-сервер
    CheckMilestone4,

    /// Проверить Веху 5: Integration Complete
    ///
    /// Запускает все критерии качества для вехи 5, включая:
    /// - Демон
    /// - CLI команды
    /// - Интеграцию с shell
    /// - IPC коммуникацию
    /// - Мониторинг здоровья
    CheckMilestone5,

    /// Проверить Веху 6: Production Ready
    ///
    /// Запускает все критерии качества для вехи 6, включая:
    /// - Unit и интеграционные тесты
    /// - Бенчмарки производительности
    /// - Security аудиты
    /// - Документацию
    /// - Кросс-платформенное тестирование
    CheckMilestone6,

    /// Проверить все вехи последовательно
    ///
    /// Последовательно проверяет все 6 вех проекта и выводит
    /// сводный отчет. Полезно для комплексной проверки готовности
    /// проекта или интеграции в CI/CD пайплайны.
    CheckAll,

    /// Сгенерировать отчет о прогрессе
    ///
    /// Создает комплексный отчет о текущем состоянии всех вех
    /// без выполнения проверок качества. Использует последние
    /// известные результаты проверок.
    ProgressReport,

    /// Показать информацию о вехах
    ///
    /// Отображает детальную информацию о критериях вех:
    /// - Команды проверки
    /// - Описания критериев
    /// - Обязательность критериев
    Info {
        /// Конкретная веха для показа информации (1-6)
        ///
        /// Если не указана, показывается информация обо всех вехах.
        milestone: Option<u8>,
    },
}

/// Поддерживаемые форматы вывода
///
/// Определяет в каких форматах система может выводить результаты
/// своей работы для различных сценариев использования.
#[derive(Clone)]
enum OutputFormat {
    /// Текстовый формат с цветами и форматированием
    ///
    /// Оптимален для интерактивного использования в терминале.
    /// Включает эмодзи, цвета и человеко-читаемое форматирование.
    Text,

    /// JSON формат для машинной обработки
    ///
    /// Используется для интеграции с другими системами,
    /// скриптами или инструментами анализа.
    Json,

    /// Markdown формат для документации
    ///
    /// Подходит для включения в документацию, отчеты
    /// или системы вики.
    Markdown,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => Err("Format must be one of: text, json, markdown".to_string()),
        }
    }
}

// =============================================================================
// Основная точка входа (УЛУЧШЕНО: возвращает Result и использует anyhow)
// =============================================================================

/// Основная точка входа приложения
///
/// Инициализирует CLI, парсит аргументы и делегирует выполнение
/// соответствующей функции-обработчику команды.
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("{} Starting check-milestones...", "🔍".green());
    }

    // Выполняем команду и обрабатываем результат
    let result = match &cli.command {
        Commands::CheckMilestone1 => check_milestone(1, &cli),
        Commands::CheckMilestone2 => check_milestone(2, &cli),
        Commands::CheckMilestone3 => check_milestone(3, &cli),
        Commands::CheckMilestone4 => check_milestone(4, &cli),
        Commands::CheckMilestone5 => check_milestone(5, &cli),
        Commands::CheckMilestone6 => check_milestone(6, &cli),
        Commands::CheckAll => check_all_milestones(&cli),
        Commands::ProgressReport => generate_progress_report(&cli),
        Commands::Info { milestone } => show_info(milestone.as_ref(), &cli),
    };

    // Если произошла ошибка (не связанная с провалом проверок), выводим её и выходим с кодом 1
    if let Err(e) = result {
        eprintln!("{} Error: {}", "❌".red(), e);
        std::process::exit(1);
    }

    Ok(())
}

// =============================================================================
// Вспомогательные функции форматирования (вынесены для устранения дублирования)
// =============================================================================

/// Выводит результат проверки качества в заданном формате.
/// Возвращает true, если проверка пройдена, иначе false.
fn print_quality_result(result: &QualityResult, format: &OutputFormat) -> Result<bool> {
    let passed = result.passed;
    match format {
        OutputFormat::Text => {
            println!("{}", result.to_colored_string());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(result)
                .context("Failed to serialize quality result to JSON")?;
            println!("{}", json);
        }
        OutputFormat::Markdown => {
            let mut output = format!("# Quality Check: {}\n\n", result.gate_name);
            output.push_str(&format!(
                "**Status:** {}\n",
                if passed { "PASSED ✅" } else { "FAILED ❌" }
            ));
            output.push_str(&format!("**Duration:** {:?}\n", result.total_duration));
            output.push_str(&format!(
                "**Criteria Passed:** {}/{}\n\n",
                result.summary.passed_criteria, result.summary.total_criteria
            ));
            output.push_str("## Detailed Results\n\n");
            for detail in &result.details {
                let status = if detail.passed { "✅" } else { "❌" };
                output.push_str(&format!("### {} {}\n", status, detail.name));
                output.push_str(&format!(
                    "- **Status:** {}\n",
                    if detail.passed { "PASS" } else { "FAIL" }
                ));
                output.push_str(&format!("- **Duration:** {:?}\n", detail.duration));
                if !detail.passed {
                    if let Some(error) = &detail.error {
                        output.push_str(&format!("- **Error:** {}\n", error));
                    }
                }
                output.push_str("\n");
            }
            println!("{}", output);
        }
    }
    Ok(passed)
}

/// Выводит детальный отчёт о прогрессе в заданном формате.
fn print_progress_report(report: &ProgressReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!("{} Project Progress Report", "📊".green());
            println!("{}", "=".repeat(40));
            println!("Generated: {}", report.generated_at);
            println!("Overall Progress: {:.1}%", report.overall_progress);
            println!();
            println!("{}", report.progress_bar());
            println!();
            for milestone in Milestone::all() {
                if let Some(progress) = report.milestones.get(&milestone) {
                    let status_emoji = progress.status.emoji();
                    println!(
                        "{} {}: {}",
                        status_emoji,
                        milestone.display_name(),
                        progress.status.as_str()
                    );
                }
            }
        }
        OutputFormat::Json => {
            let json = report
                .to_json()
                .context("Failed to serialize progress report to JSON")?;
            println!("{}", json);
        }
        OutputFormat::Markdown => {
            println!("{}", report.to_markdown());
        }
    }
    Ok(())
}

/// Выводит информацию о вехе в заданном формате.
fn print_milestone_info(
    milestone_number: u8,
    gate: &QualityGate,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!(
                "{} Milestone {}: {}",
                "📋".blue(),
                milestone_number,
                gate.name
            );
            println!("{}", "=".repeat(50));
            println!("Criteria:");
            for criterion in &gate.criteria {
                let required = if criterion.required {
                    " (required)"
                } else {
                    " (optional)"
                };
                println!("  • {}{}", criterion.name, required);
                println!("    Command: {}", criterion.command);
                println!("    Description: {}", criterion.description);
                println!();
            }
        }
        OutputFormat::Json => {
            let info = serde_json::json!({
                "milestone": milestone_number,
                "name": gate.name,
                "criteria": gate.criteria.iter().map(|c| {
                    serde_json::json!({
                        "name": c.name,
                        "command": c.command,
                        "description": c.description,
                        "required": c.required,
                    })
                }).collect::<Vec<_>>()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&info)
                    .context("Failed to serialize milestone info to JSON")?
            );
        }
        OutputFormat::Markdown => {
            println!("# Milestone {}: {}\n", milestone_number, gate.name);
            println!("## Criteria\n");
            for criterion in &gate.criteria {
                let required = if criterion.required {
                    "**Required**"
                } else {
                    "Optional"
                };
                println!("### {}\n", criterion.name);
                println!("- **Status:** {}", required);
                println!("- **Command:** `{}`", criterion.command);
                println!("- **Description:** {}\n", criterion.description);
            }
        }
    }
    Ok(())
}

// =============================================================================
// Обработчики команд (оригинальные комментарии сохранены, изменена сигнатура на Result)
// =============================================================================

/// Проверяет конкретную веху
///
/// Создает Quality Gate для указанной вехи, выполняет все критерии
/// и выводит результаты в запрошенном формате.
///
/// # Аргументы
///
/// * `milestone_number` - Номер вехи для проверки (1-6)
/// * `cli` - Конфигурация CLI с флагами и настройками
///
/// # Возвращает
///
/// `Result<()>` - Успешное выполнение или ошибка
///
/// # Выходные коды
///
/// - `0` - Веха успешно пройдена
/// - `1` - Веха не пройдена (какие-то критерии failed) или внутренняя ошибка
fn check_milestone(milestone_number: u8, cli: &Cli) -> Result<()> {
    if cli.verbose {
        println!("{} Checking Milestone {}...", "🔍".blue(), milestone_number);
    }

    let gate = match milestone_number {
        1 => MilestoneGates::milestone_1(),
        2 => MilestoneGates::milestone_2(),
        3 => MilestoneGates::milestone_3(),
        4 => MilestoneGates::milestone_4(),
        5 => MilestoneGates::milestone_5(),
        6 => MilestoneGates::milestone_6(),
        _ => anyhow::bail!(
            "Invalid milestone number: {}. Must be between 1-6",
            milestone_number
        ),
    };

    let result = gate.check();
    let passed = print_quality_result(&result, &cli.output)?;

    if !passed {
        std::process::exit(1);
    }
    Ok(())
}

/// Проверяет все вехи последовательно
///
/// Выполняет проверку всех 6 вех проекта в порядке их следования,
/// обновляет трекер прогресса и выводит комплексный отчет.
///
/// # Аргументы
///
/// * `cli` - Конфигурация CLI с флагами и настройками
///
/// # Возвращает
///
/// `Result<()>` - Успешное выполнение или ошибка
///
/// # Выходные коды
///
/// - `0` - Все вехи успешно пройдены
/// - `1` - Одна или несколько вех не пройдены, либо внутренняя ошибка
fn check_all_milestones(cli: &Cli) -> Result<()> {
    println!("{} Checking all milestones...", "🔍".blue());

    let mut all_passed = true;
    let mut results = Vec::new();
    let mut tracker = ProgressTracker::new();

    // Последовательно проверяем все вехи
    for (milestone_number, gate) in MilestoneGates::all_milestones() {
        if cli.verbose {
            println!(
                "\n{} Checking Milestone {}: {}...",
                "🔍".blue(),
                milestone_number,
                gate.name
            );
        }

        let result = gate.check();
        results.push((milestone_number, result.clone()));

        // Обновляем трекер прогресса
        let milestone_enum = match milestone_number {
            1 => Milestone::Foundation,
            2 => Milestone::Infrastructure,
            3 => Milestone::AICore,
            4 => Milestone::WebInterface,
            5 => Milestone::Integration,
            6 => Milestone::Production,
            _ => continue,
        };
        tracker.update_milestone(milestone_enum, result.clone());

        if !result.passed {
            all_passed = false;
            if cli.verbose {
                println!("{} Milestone {} failed", "❌".red(), milestone_number);
            }
        }
    }

    // Генерируем финальный отчет
    let report = tracker.generate_report();
    print_progress_report(&report, &cli.output)?;

    if !all_passed {
        std::process::exit(1);
    }
    Ok(())
}

/// Генерирует отчет о прогрессе
///
/// Создает отчет о текущем состоянии всех вех без выполнения
/// проверок качества. Используется для мониторинга прогресса
/// и создания документации.
///
/// # Аргументы
///
/// * `cli` - Конфигурация CLI с флагами и настройками
///
/// # Возвращает
///
/// `Result<()>` - Успешное выполнение или ошибка
fn generate_progress_report(cli: &Cli) -> Result<()> {
    if cli.verbose {
        println!("{} Generating progress report...", "📊".blue());
    }
    let tracker = ProgressTracker::new();
    let report = tracker.generate_report();
    print_progress_report(&report, &cli.output)
}

/// Показывает информацию о вехах
///
/// Отображает детальную информацию о критериях вех, включая
/// команды проверки, описания и обязательность критериев.
///
/// # Аргументы
///
/// * `milestone` - Опциональный номер вехи для показа информации
/// * `cli` - Конфигурация CLI с флагами и настройками
///
/// # Возвращает
///
/// `Result<()>` - Успешное выполнение или ошибка
fn show_info(milestone: Option<&u8>, cli: &Cli) -> Result<()> {
    match milestone {
        Some(&number) if (1..=6).contains(&number) => {
            let gate = match number {
                1 => MilestoneGates::milestone_1(),
                2 => MilestoneGates::milestone_2(),
                3 => MilestoneGates::milestone_3(),
                4 => MilestoneGates::milestone_4(),
                5 => MilestoneGates::milestone_5(),
                6 => MilestoneGates::milestone_6(),
                _ => unreachable!(),
            };
            print_milestone_info(number, &gate, &cli.output)?;
        }
        Some(&number) => {
            anyhow::bail!("Invalid milestone number: {}. Must be between 1-6", number);
        }
        None => match cli.output {
            OutputFormat::Text => {
                println!("{} All Milestones", "📋".blue());
                println!("{}", "=".repeat(30));
                for milestone in Milestone::all() {
                    println!("{}: {}", milestone.display_name(), milestone.description());
                }
            }
            OutputFormat::Json => {
                let all_info = Milestone::all()
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "name": m.display_name(),
                            "description": m.description(),
                        })
                    })
                    .collect::<Vec<_>>();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&all_info)
                        .context("Failed to serialize milestone list to JSON")?
                );
            }
            OutputFormat::Markdown => {
                println!("# All Milestones\n");
                for milestone in Milestone::all() {
                    println!("## {}\n", milestone.display_name());
                    println!("{}\n", milestone.description());
                }
            }
        },
    }
    Ok(())
}
