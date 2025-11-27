// zsh-ollama-terminal/crates/check-milestones/src/milestone_gates.rs
//! # Milestone Gates Definitions
//!
//! Предопределенные Quality Gates для каждой вехи проекта zsh-ollama-terminal.
//!
//! Этот модуль содержит специфические критерии качества для каждой вехи разработки,
//! обеспечивая последовательный и стандартизированный подход к проверке готовности
//! каждого этапа проекта.
//!
//! ## Структура вех проекта
//!
//! Проект разделен на 6 последовательных вех:
//!
//! 1. **Foundation Complete** - Базовая инфраструктура и основные типы
//! 2. **Infrastructure Ready** - Безопасность, клиент и платформенные абстракции
//! 3. **AI Core Functional** - Анализ команд и AI возможности
//! 4. **Web Interface Live** - Веб-сервер и пользовательский интерфейс
//! 5. **Integration Complete** - Системная интеграция и CLI
//! 6. **Production Ready** - Готовность к продакшену и тестирование
//!
//! ## Принципы определения критериев
//!
//! Каждый критерий должен быть:
//! - **Измеримым** - иметь четкий pass/fail результат
//! - **Автоматизируемым** - выполняться через shell команды
//! - **Релевантным** - проверять важный аспект вехи
//! - **Быстрым** - выполняться за разумное время
//! - **Надежным** - давать стабильные результаты
//!
//! ## Пример использования
//!
//! ```rust
//! use check_milestones::MilestoneGates;
//!
//! // Проверка готовности вехи 1
//! let milestone_1 = MilestoneGates::milestone_1();
//! let result = milestone_1.check();
//!
//! if result.passed {
//!     println!("Веха 1 готова к переходу к следующему этапу!");
//! } else {
//!     eprintln!("Веха 1 требует доработки: {}", result.message);
//! }
//! ```

use super::quality_gates::QualityGate;
use std::time::Duration;

/// Предопределенные Quality Gates для каждой вехи проекта
///
/// Этот struct предоставляет статические методы для создания
/// предварительно настроенных Quality Gates для всех вех проекта.
/// Каждый метод возвращает готовый к использованию Quality Gate
/// с критериями, специфичными для соответствующей вехи.
pub struct MilestoneGates;

impl MilestoneGates {
    /// Создает Quality Gate для Вехи 1: Foundation Complete
    ///
    /// Проверяет базовую инфраструктуру проекта, включая компиляцию,
    /// тестирование, документацию и качество кода.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **compilation** - Компиляция всего workspace без ошибок
    /// - **testing** - Прохождение всех unit тестов
    /// - **documentation** - Генерация документации без предупреждений
    /// - **formatting** - Проверка форматирования кода
    /// - **clippy** - Отсутствие предупреждений clippy
    /// - **core_types** - Тестирование основных типов и трейтов
    /// - **error_system** - Работоспособность системы ошибок
    /// - **serialization** - Тесты сериализации/десериализации
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_1();
    /// let result = gate.check();
    /// ```
    pub fn milestone_1() -> QualityGate {
        QualityGate::new("Milestone 1: Foundation Complete")
            .add_criterion(
                "compilation",
                "cargo check --workspace",
                "Project compiles without errors across all crates",
            )
            .add_criterion(
                "testing",
                "cargo test --workspace",
                "All unit tests pass across the workspace",
            )
            .add_criterion(
                "documentation",
                "cargo doc --workspace --no-deps --document-private-items",
                "Documentation generates without warnings including private items",
            )
            .add_criterion(
                "formatting",
                "cargo fmt -- --check",
                "Code is properly formatted according to rustfmt standards",
            )
            .add_criterion(
                "clippy",
                "cargo clippy --workspace -- -D warnings",
                "No clippy warnings or errors detected",
            )
            .add_criterion(
                "core_types",
                "cargo test --lib --bins --tests",
                "Core types and traits are properly implemented and tested",
            )
            .add_criterion(
                "error_system",
                "cargo test --test error_*",
                "Error system with proper conversions is working",
            )
            .add_criterion(
                "serialization",
                "cargo test --test serialization* --test deserialization*",
                "Serialization and deserialization tests pass for all types",
            )
            .set_strict_mode(true)
    }

    /// Создает Quality Gate для Вехи 2: Infrastructure Ready
    ///
    /// Проверяет инфраструктурные компоненты проекта, включая безопасность,
    /// интеграцию с Ollama, файловые операции и кроссплатформенность.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **security_validator** - Работа валидатора безопасности
    /// - **ollama_client** - Интеграция клиента Ollama
    /// - **file_operations** - Безопасные файловые операции
    /// - **platform_abstractions** - Работоспособность платформенных абстракций
    /// - **performance_benchmarks** - Соответствие бенчмаркам производительности
    /// - **cross_platform_*** - Компиляция на всех целевых платформах
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_2();
    /// let result = gate.check();
    /// ```
    pub fn milestone_2() -> QualityGate {
        QualityGate::new("Milestone 2: Infrastructure Ready")
            .add_criterion(
                "security_validator",
                "cargo test security_validator -- --test-threads=1",
                "Security validator detects all known attack patterns",
            )
            .add_criterion(
                "ollama_client",
                "cargo test ollama_client --features=integration -- --test-threads=1",
                "Ollama client with circuit breaker and retry logic works",
            )
            .add_criterion(
                "file_operations",
                "cargo test file_operations -- --test-threads=1",
                "Safe file operations with atomic writes implemented",
            )
            .add_criterion(
                "platform_abstractions",
                "cargo test platform_abstractions -- --test-threads=1",
                "Platform abstractions work on all target platforms",
            )
            .add_criterion(
                "performance_benchmarks",
                "cargo bench infrastructure",
                "Performance benchmarks meet target requirements",
            )
            .add_criterion(
                "cross_platform_linux",
                "cargo check --target x86_64-unknown-linux-gnu",
                "Compiles on Linux target",
            )
            .add_criterion(
                "cross_platform_macos",
                "cargo check --target x86_64-apple-darwin",
                "Compiles on macOS target",
            )
            .add_criterion(
                "cross_platform_windows",
                "cargo check --target x86_64-pc-windows-msvc",
                "Compiles on Windows target",
            )
            .set_strict_mode(true)
            .with_env("RUST_BACKTRACE", "1")
    }

    /// Создает Quality Gate для Вехи 3: AI Core Functional
    ///
    /// Проверяет AI-функциональность проекта, включая анализ команд,
    /// обнаружение галлюцинаций, движок обучения и производительность.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **command_analysis** - Работоспособность пайплайна анализа команд
    /// - **hallucination_detection** - Обнаружение галлюцинаций с точностью >90%
    /// - **training_engine** - Реализация и тестирование движка обучения
    /// - **performance_targets** - Соответствие целевым показателям производительности
    /// - **cache_system** - Снижение задержки кеш-системой на >60%
    /// - **integration_tests** - Интеграционные тесты AI ядра
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_3();
    /// let result = gate.check();
    /// ```
    pub fn milestone_3() -> QualityGate {
        QualityGate::new("Milestone 3: AI Core Functional")
            .add_criterion(
                "command_analysis",
                "cargo test command_analysis -- --ignored --test-threads=1",
                "Command analysis pipeline works end-to-end",
            )
            .add_criterion(
                "hallucination_detection",
                "cargo test hallucination_detection -- --ignored --test-threads=1",
                "Hallucination detection operational with >90% accuracy",
            )
            .add_criterion(
                "training_engine",
                "cargo test training_engine -- --ignored --test-threads=1",
                "Training engine implemented and tested",
            )
            .add_criterion(
                "performance_targets",
                "cargo bench ai_core -- --verbose",
                "Performance targets met for all analysis types",
            )
            .add_criterion(
                "cache_system",
                "cargo test cache_system -- --ignored --test-threads=1",
                "Cache system reduces latency by >60%",
            )
            .add_criterion_with_timeout(
                "integration_tests",
                "cargo test ai_integration -- --test-threads=1",
                "AI core integration tests pass",
                Duration::from_secs(600), // 10-минутный таймаут для интеграционных тестов
            )
            .set_strict_mode(true)
    }

    /// Создает Quality Gate для Вехи 4: Web Interface Live
    ///
    /// Проверяет веб-интерфейс проекта, включая рендеринг шаблонов,
    /// HTMX взаимодействия, HTTP ответы и производительность сервера.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **tera_templates** - Корректный рендеринг шаблонов Tera
    /// - **htmx_interactions** - Работоспособность HTMX без JavaScript
    /// - **http_responses** - Типизированные HTTP ответы с security headers
    /// - **reusable_components** - Переиспользуемые и документированные компоненты
    /// - **web_server** - Обработка >100 RPS веб-сервером
    /// - **web_integration** - Интеграционные тесты веб-интерфейса
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_4();
    /// let result = gate.check();
    /// ```
    pub fn milestone_4() -> QualityGate {
        QualityGate::new("Milestone 4: Web Interface Live")
            .add_criterion(
                "tera_templates",
                "cargo test tera_templates -- --ignored --test-threads=1",
                "Tera templates render correctly without errors",
            )
            .add_criterion(
                "htmx_interactions",
                "cargo test htmx_interactions -- --ignored --test-threads=1",
                "HTMX interactions work without JavaScript",
            )
            .add_criterion(
                "http_responses",
                "cargo test http_responses -- --ignored --test-threads=1",
                "Typed HTTP responses with guaranteed security headers",
            )
            .add_criterion(
                "reusable_components",
                "cargo test components -- --ignored --test-threads=1",
                "All components are reusable and properly documented",
            )
            .add_criterion(
                "web_server",
                "cargo test web_server -- --ignored --test-threads=1",
                "Web server handling >100 RPS",
            )
            .add_criterion(
                "web_integration",
                "cargo test web_integration -- --ignored --test-threads=1",
                "Web interface integration tests pass",
            )
            .set_strict_mode(true)
    }

    /// Создает Quality Gate для Вехи 5: Integration Complete
    ///
    /// Проверяет интеграционные аспекты проекта, включая работу демона,
    /// CLI команды, интеграцию с shell и IPC коммуникацию.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **daemon_operation** - Демон со всеми интегрированными сервисами
    /// - **cli_commands** - Функциональные CLI команды с обработкой ошибок
    /// - **shell_integration** - Интеграция с ZSH, Bash, Fish
    /// - **ipc_communication** - Надежная и производительная IPC связь
    /// - **health_monitoring** - Работоспособность мониторинга здоровья
    /// - **end_to_end** - End-to-end интеграционные тесты
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_5();
    /// let result = gate.check();
    /// ```
    pub fn milestone_5() -> QualityGate {
        QualityGate::new("Milestone 5: Integration Complete")
            .add_criterion(
                "daemon_operation",
                "cargo test daemon -- --ignored --test-threads=1",
                "Daemon running with all services integrated",
            )
            .add_criterion(
                "cli_commands",
                "cargo test cli -- --ignored --test-threads=1",
                "CLI commands functional with proper error handling",
            )
            .add_criterion(
                "shell_integration",
                "cargo test shell_integration -- --ignored --test-threads=1",
                "Shell integration working for ZSH, Bash, Fish",
            )
            .add_criterion(
                "ipc_communication",
                "cargo test ipc -- --ignored --test-threads=1",
                "IPC communication reliable and performant",
            )
            .add_criterion(
                "health_monitoring",
                "cargo test health -- --ignored --test-threads=1",
                "Health monitoring operational",
            )
            .add_criterion(
                "end_to_end",
                "cargo test end_to_end -- --ignored --test-threads=1",
                "End-to-end integration tests pass",
            )
            .set_strict_mode(true)
    }

    /// Создает Quality Gate для Вехи 6: Production Ready
    ///
    /// Проверяет готовность проекта к продакшену, включая комплексное тестирование,
    /// бенчмарки производительности, security аудиты и документацию.
    ///
    /// ## Критерии включенные в gate:
    ///
    /// - **unit_tests** - Прохождение всех unit тестов с 100% coverage
    /// - **integration_tests** - Прохождение всех интеграционных тестов
    /// - **performance_benchmarks** - Соответствие всем целевым показателям
    /// - **security_audit** - Чистые security аудиты без критических уязвимостей
    /// - **documentation_complete** - Полная и актуальная документация
    /// - **cross_platform_testing** - Успешное кросс-платформенное тестирование
    /// - **production_readiness** - Проверки готовности к продакшену
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let gate = MilestoneGates::milestone_6();
    /// let result = gate.check();
    /// ```
    pub fn milestone_6() -> QualityGate {
        QualityGate::new("Milestone 6: Production Ready")
            .add_criterion(
                "unit_tests",
                "cargo test --workspace --lib --bins --tests",
                "All unit tests pass with 100% coverage"
            )
            .add_criterion(
                "integration_tests",
                "cargo test --workspace --test '*' -- --test-threads=1",
                "All integration tests pass"
            )
            .add_criterion(
                "performance_benchmarks",
                "cargo bench -- --verbose",
                "Performance benchmarks meeting all targets"
            )
            .add_criterion(
                "security_audit",
                "cargo audit --deny warnings",
                "Security audits clean with no critical vulnerabilities"
            )
            .add_criterion(
                "documentation_complete",
                "cargo doc --workspace --no-deps --document-private-items && cargo test --doc",
                "Documentation complete and up-to-date including doctests"
            )
            .add_criterion(
                "cross_platform_testing",
                "cargo test --workspace --target x86_64-unknown-linux-gnu && cargo test --workspace --target x86_64-apple-darwin",
                "Cross-platform testing successful on major platforms"
            )
            .add_criterion(
                "production_readiness",
                "cargo test production_ready -- --ignored --test-threads=1",
                "Production readiness checks pass"
            )
            .set_strict_mode(true)
    }

    /// Возвращает все Quality Gates в порядке вех
    ///
    /// Полезно для массовой проверки всех вех или создания
    /// комплексных отчетов о прогрессе всего проекта.
    ///
    /// # Возвращает
    ///
    /// Вектор кортежей `(номер_вехи, QualityGate)` для всех 6 вех
    ///
    /// # Пример
    ///
    /// ```rust
    /// use check_milestones::MilestoneGates;
    ///
    /// let all_gates = MilestoneGates::all_milestones();
    /// for (number, gate) in all_gates {
    ///     println!("Checking milestone {}: {}", number, gate.name);
    ///     let result = gate.check();
    ///     println!("Result: {}", if result.passed { "PASSED" } else { "FAILED" });
    /// }
    /// ```
    pub fn all_milestones() -> Vec<(u8, QualityGate)> {
        vec![
            (1, Self::milestone_1()),
            (2, Self::milestone_2()),
            (3, Self::milestone_3()),
            (4, Self::milestone_4()),
            (5, Self::milestone_5()),
            (6, Self::milestone_6()),
        ]
    }
}

// =============================================================================
// Модуль тестирования
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Тестирует создание Quality Gate для вехи 1
    #[test]
    fn test_milestone_1_creation() {
        let milestone_1 = MilestoneGates::milestone_1();
        assert_eq!(milestone_1.name, "Milestone 1: Foundation Complete");
        assert!(!milestone_1.criteria.is_empty());
        assert!(milestone_1.strict_mode);
    }

    /// Тестирует создание Quality Gate для вехи 2
    #[test]
    fn test_milestone_2_creation() {
        let milestone_2 = MilestoneGates::milestone_2();
        assert_eq!(milestone_2.name, "Milestone 2: Infrastructure Ready");
        assert!(!milestone_2.criteria.is_empty());
    }

    /// Тестирует создание Quality Gate для вехи 3
    #[test]
    fn test_milestone_3_creation() {
        let milestone_3 = MilestoneGates::milestone_3();
        assert_eq!(milestone_3.name, "Milestone 3: AI Core Functional");
        assert!(!milestone_3.criteria.is_empty());
    }

    /// Тестирует создание Quality Gate для вехи 4
    #[test]
    fn test_milestone_4_creation() {
        let milestone_4 = MilestoneGates::milestone_4();
        assert_eq!(milestone_4.name, "Milestone 4: Web Interface Live");
        assert!(!milestone_4.criteria.is_empty());
    }

    /// Тестирует создание Quality Gate для вехи 5
    #[test]
    fn test_milestone_5_creation() {
        let milestone_5 = MilestoneGates::milestone_5();
        assert_eq!(milestone_5.name, "Milestone 5: Integration Complete");
        assert!(!milestone_5.criteria.is_empty());
    }

    /// Тестирует создание Quality Gate для вехи 6
    #[test]
    fn test_milestone_6_creation() {
        let milestone_6 = MilestoneGates::milestone_6();
        assert_eq!(milestone_6.name, "Milestone 6: Production Ready");
        assert!(!milestone_6.criteria.is_empty());
    }

    /// Тестирует получение всех вех
    #[test]
    fn test_all_milestones() {
        let all_milestones = MilestoneGates::all_milestones();
        assert_eq!(all_milestones.len(), 6);

        // Проверяем, что вехи идут в правильном порядке
        for (i, (number, _)) in all_milestones.iter().enumerate() {
            assert_eq!(*number, (i + 1) as u8);
        }
    }

    /// Тестирует, что все критерии имеют описания
    #[test]
    fn test_all_criteria_have_descriptions() {
        let all_milestones = MilestoneGates::all_milestones();

        for (_, gate) in all_milestones {
            for criterion in &gate.criteria {
                assert!(
                    !criterion.description.is_empty(),
                    "Criterion '{}' in gate '{}' has empty description",
                    criterion.name,
                    gate.name
                );
                assert!(
                    !criterion.command.is_empty(),
                    "Criterion '{}' in gate '{}' has empty command",
                    criterion.name,
                    gate.name
                );
            }
        }
    }
}
