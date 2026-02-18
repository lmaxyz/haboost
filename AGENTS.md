# AGENTS.md - Руководство по разработке Haboost

Haboost — неофициальный клиент Habr, написанный на Rust с использованием egui для ОС Аврора.

## Команды сборки и разработки

```bash
# Стандартные команды Rust
cargo build                    # Отладочная сборка
cargo build --release          # Релизная сборка (оптимизированная, stripped)
cargo run                      # Запуск приложения
cargo test                     # Запуск всех тестов
cargo test <test_name>         # Запуск одного теста
cargo clippy                   # Линтинг
cargo fmt                      # Форматирование кода

# Кросс-компиляция для ОС Аврора (требуется `cross`)
cargo install cross
cross build --target armv7-unknown-linux-gnueabihf
cross build --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-unknown-linux-gnu

# Создание RPM-пакета
cargo generate-rpm -a x86_64 --target x86_64-unknown-linux-gnu

# Запуск с логированием
RUST_LOG=debug cargo run
```

### Скрипты сборки для ОС Аврора (рекомендуется)

Скрипты для Platform SDK ОС Аврора. Требуется переменная окружения `PSDK_DIR` с путём к Aurora SDK.

```bash
# Сборка для AArch64 (64-бит ARM) — современные устройства Аврора
./aarch64_build.sh

# Сборка для ARMv7 (32-бит ARM) — старые устройства
./arm_build.sh

# Сборка для x86_64 с подписью RPM
./x86_64_rpm_build.sh
```

Каждый скрипт:
1. Собирает релизную версию с помощью `cross`
2. Создаёт RPM-пакет в директории `RPMS/`
3. Подписывает RPM сертификатами Platform SDK Аврора

**Примечание:** Скрипты автоматически раскомментируют патч `winit` в Cargo.toml перед сборкой и закомментируют обратно после завершения (даже при ошибке), чтобы локальная разработка работала без патчей.

## Структура проекта

```
src/
├── main.rs              # Точка входа приложения
├── view_stack.rs        # Управление навигацией
├── widgets.rs           # Пользовательские UI-виджеты
├── aurora_services/     # Специфичное для ОС Аврора (открытие URI)
├── habr_client/         # HTTP-клиент и API
│   ├── article.rs       # Структуры данных статей
│   ├── hub.rs           # Структуры данных хабов
│   └── html_parse.rs    # Парсинг HTML-контента
└── views/               # UI-представления
    ├── articles_list.rs
    ├── article_details.rs
    ├── hubs_list.rs
    └── settings.rs
```

## Руководство по стилю кода

### Версия Rust и тулчейн
- **Edition:** 2024 (последняя)
- Используйте стандартные настройки `cargo fmt` и `cargo clippy`
- Нет кастомных rustfmt.toml или clippy.toml

### Порядок импортов
```rust
// 1. Стандартная библиотека (группируются)
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

// 2. Внешние крейты (по алфавиту)
use chrono::{DateTime, Local};
use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};

// 3. Локальные модули (через crate::)
use crate::HabreState;
use crate::habr_client::article::ArticleData;
use crate::view_stack::{UiView, ViewStack};
```

### Соглашения об именовании
- **Типы/Структуры/Перечисления:** `CamelCase` (например, `ArticleData`, `ViewStack`)
- **Функции/Переменные:** `snake_case` (например, `get_articles`, `current_page`)
- **Константы:** `SCREAMING_SNAKE_CASE`
- **Псевдонимы типов:** `PascalCase` (например, `type PagesCount = usize;`)
- **Приватные поля:** префикс `pub(crate)` или опускается для приватных

### Обработка ошибок
- Используйте `.unwrap()` и `.expect("[префикс] Сообщение")` для неустранимых ошибок
- Используйте `Result<T, Error>` для методов API/клиента, которые могут завершиться ошибкой
- Распространённый паттерн: распространение ошибок оператором `?` в асинхронных функциях

### Атрибуты Serde
```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleResponse {
    #[serde(alias = "titleHtml")]      // API возвращает camelCase
    pub title: String,
    #[serde(rename(deserialize = "pagesCount"))]
    pub(crate) pages_count: usize,     // Rust использует snake_case
}
```

### Платформенно-зависимый код
```rust
#[cfg(not(target_arch = "x86_64"))]
mod aurora_services;  // Компилируется только на ARM
```

### Паттерны UI/egui
- Используйте `Rc<RefCell<T>>` для разделяемого изменяемого состояния между представлениями
- Представления реализуют трейт `UiView` с методом `ui()`
- Состояние управляется через структуру `HabreState` с хендлом Tokio runtime
- Используйте `egui_flex` для адаптивных layout'ов

### Перечисления и ветки match
```rust
pub enum ArticlesListSorting {
    Newest,
    Best,
}

impl ArticlesListSorting {
    pub fn to_string(&self) -> String {
        match self {
            ArticlesListSorting::Best => "date".to_string(),
            ArticlesListSorting::Newest => "rating".to_string(),
        }
    }
}
```

## Важные замечания

- **Патчи для ОС Аврора:** Проект требует форки `winit` и `glutin` (определены в Cargo.toml в секции patches) — не удаляйте их
- **Кросс-компиляция:** Используйте инструмент `cross` для ARM-сборок; Cross.toml настраивает зависимости libdbus
- **RPM-пакетирование:** ID приложения — `com.lmaxyz.Haboost` (нотация обратного DNS)
- **Лицензия:** Apache 2.0 (Copyright 2025 Leiman Maksim)

## Тестирование

В проекте минимальное покрытие тестами. При добавлении тестов:
- Размещайте unit-тесты в том же файле, что и тестируемый код
- Используйте модули `#[cfg(test)]` для кода, нужного только в тестах
- Интеграционные тесты помещайте в директорию `tests/` (если создаётся)
