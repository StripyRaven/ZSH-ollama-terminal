# Базовые проверки для скорости

```bash
cd zsh-ollama-terminal/crates/check-milestones

# 1. Проверка компиляции
cargo check

# 2. Запуск тестов
cargo test

# 3. Проверка бинарного крейта
cargo run -- --help

# 4. Проверка формата кода
cargo fmt -- --check

# 5. Проверка clippy
cargo clippy -- -D warnings

# 6. Генерация документации
cargo doc --no-deps --open

# Проверка CLI
cargo run -- check-milestone-1 --help

# Проверка Just
just list
just check-milestone-1
```
