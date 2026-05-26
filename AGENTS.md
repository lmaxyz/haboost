# AGENTS.md — Haboost

Этот документ описывает архитектуру, сборку и соглашения проекта **Haboost** для AI-агентов, работающих с кодовой базой.

---

## Обзор проекта

**Haboost** — неофициальный клиент для сайта [Habr](https://habr.com), написанный на языке Rust с использованием графической библиотеки [egui](https://github.com/emilk/egui). Приложение демонстрирует возможность разработки под ОС Аврора (и потенциально Sailfish OS) на Rust.

Приложение поддерживает:
- просмотр списка хабов с поиском и пагинацией;
- просмотр статей (заголовки, текст, изображения, ссылки, курсив, жирный шрифт, цитаты, код, списки);
- масштабирование изображений;
- переход по внешним ссылкам;
- просмотр дерева комментариев;
- поиск по статьям;
- **офлайн-сохранение статей с изображениями**;
- **просмотр и удаление сохранённых статей**;
- настройки (коэффициент масштабирования, размер шрифта, тема оформления).

---

## Технологический стек

| Компонент | Библиотека / Инструмент |
|-----------|------------------------|
| Язык | Rust (edition 2024) |
| UI-фреймворк | egui 0.34.2 |
| Desktop runtime | eframe 0.34.2 (feature `desktop`, по умолчанию) |
| Aurora runtime | aurora_egui + aurora_services (feature `aurora`) |
| HTTP-клиент | reqwest 0.12 с rustls-tls |
| Async runtime | tokio (multi-thread + time + io) |
| HTML-парсинг | scraper 0.25 |
| Сериализация | serde, serde_json, toml |
| Изображения | image (jpeg, png, bmp, gif, webp), egui_extras (image, svg, gif, webp, file) |
| Логирование | log + env_logger |
| Layout | egui_flex 0.6 |
| Кросс-компиляция | cross |
| Пакетирование | cargo-generate-rpm |

---

## Архитектура приложения

### Двухплатформенная сборка

Проект собирается в два режима, управляемых Cargo features:

- **`desktop`** (default) — запуск на обычном Linux через `eframe`.
- **`aurora`** — запуск на ОС Аврора через `aurora_egui`.

Точка входа (`main.rs`) использует условную компиляцию `#[cfg(feature = "aurora")]` / `#[cfg(not(feature = "aurora"))]` для выбора runtime и реализации трейта `App`.

### Структура модулей

```
src/
├── main.rs           # точка входа, инициализация логгера, выбор runtime
├── app.rs            # MyApp + HabreState (глобальное состояние приложения)
├── state.rs          # (устаревший/дублирующий файл, см. app.rs)
├── view_stack.rs     # ViewStack — собственная навигация по экранам со свайпом назад
├── widgets.rs        # Общие виджеты (Pager, context_menu_button)
├── storage.rs        # Хранилище сохранённых статей и изображений (ArticleStorage)
├── habr_client/      # HTTP-клиент и модели данных Habr API
│   ├── mod.rs        # HabrClient (reqwest-обёртка)
│   ├── article.rs    # Модели статей + тесты десериализации
│   ├── comment.rs    # Модели комментариев + тесты десериализации
│   ├── hub.rs        # Модели хабов + тесты десериализации
│   └── html_parse.rs # Парсинг HTML-тела статьи в ArticleContent
└── views/            # Экраны приложения
    ├── mod.rs
    ├── hubs_list.rs          # Список хабов
    ├── articles_list.rs      # Список статей (с фильтрами/поиском + кнопки сохранения)
    ├── article_details.rs    # Детальная страница статьи
    ├── comments.rs           # Дерево комментариев
    ├── saved_articles_list.rs# Список сохранённых статей (офлайн)
    └── settings.rs           # Настройки (масштаб, шрифт, тема)
```

### Управление состоянием

- Глобальное состояние — `HabreState` (в `app.rs`), обёрнуто в `Rc<RefCell<HabreState>>`.
- `HabreState` содержит:
  - `selected_hub`, `selected_article` — выбранные пользователем сущности;
  - `settings` — настройки (тоже `Rc<RefCell<Settings>>`);
  - `tokio_rt` — собственный multi-thread runtime Tokio для выполнения HTTP-запросов.
- Данные, разделяемые между UI и async-задачами, оборачиваются в `Arc<RwLock<T>>` или `Arc<AtomicBool/AtomicU8>`.
- Навигация реализована через `ViewStack`: стек экранов (`Vec<Rc<RefCell<dyn UiView>>>`) с жестом свайпа вправо для возврата назад.

### Сетевое взаимодействие

`HabrClient` (в `habr_client/mod.rs`) — тонкая обёртка над `reqwest::Client`. Запросы выполняются к неофициальному API Habr (`https://habr.com/kek/v2/...`). Все запросы анонимные (guest), без аутентификации.

---

## Сборка и запуск

### Локальная разработка (Desktop)

```bash
# Запуск в режиме отладки
cargo run

# Сборка релиза (desktop)
cargo build --release

# Запуск с логами
RUST_LOG=debug cargo run
```

### Тесты

```bash
# Запуск всех unit-тестов
cargo test
```

Тесты находятся внутри исходных файлов (`habr_client/article.rs`, `comment.rs`, `hub.rs`) и проверяют корректность десериализации JSON-ответов от API Habr.

### Сборка для ОС Аврора

Для кросс-компиляции используется [`cross`](https://github.com/cross-rs/cross). В `Cross.toml` прописана установка `libdbus-1-dev` для целевых платформ.

**Важно:** перед сборкой для Авроры в `Cargo.toml` необходимо раскомментировать патч `winit` (строка 72). Скрипты сборки делают это автоматически.

```bash
# Установка cross
cargo install cross

# Рекомендуемый способ — через скрипты (требуется PSDK_DIR):
./aarch64_build.sh   # AArch64 (современные устройства)
./arm_build.sh       # ARMv7 (старые устройства)
./x86_64_rpm_build.sh # x86_64 (эмулятор/виртуалка)
```

Что делают скрипты:
1. Раскомментируют патч `winit` в `Cargo.toml`.
2. Собирают релиз через `cross` с флагом `--no-default-features --features aurora` (для ARM-таргетов).
3. Создают RPM-пакет через `cargo generate-rpm`.
4. Подписывают RPM сертификатами Platform SDK (`rpmsign-external`).
5. Автоматически комментируют патч `winit` обратно (через `trap cleanup EXIT`).

**Требования для скриптов:**
- Установленный Rust + cargo
- Установленный `cross`
- Platform SDK ОС Аврора
- Переменная окружения `PSDK_DIR`, указывающая на Platform SDK
- Сертификаты подписи по пути `$PSDK_DIR/../../certs/lmaxyz_key.pem` и `lmaxyz_cert.pem`

### Ручная кросс-компиляция

```bash
# ARM 32-bit
cross build --release --no-default-features --features aurora --target armv7-unknown-linux-gnueabihf

# ARM 64-bit
cross build --release --no-default-features --features aurora --target aarch64-unknown-linux-gnu

# x86_64
cross build --release --target x86_64-unknown-linux-gnu
```

---

## Патчи зависимостей

В `Cargo.toml` секция `[patch.crates-io]` содержит форки, необходимые для работы на ОС Аврора:

```toml
[patch.crates-io]
# winit = { git = "https://github.com/lmaxyz/winit", branch = "rm_maliit" }  # раскомментировать для сборки под Аврору
glutin = { git = "https://github.com/lmaxyz/glutin", branch = "aurora_device_fix" }
egui-winit = { git = "https://github.com/lmaxyz/egui/", branch = "more_flexible_egui-winit" }
```

- **`winit`** — закомментирован по умолчанию, чтобы локальная разработка работала без патчей. Раскомментируется скриптами сборки автоматически.
- **`glutin`** и **`egui-winit`** — активны всегда; содержат фиксы для ОС Аврора.

**Не изменяйте логику автоматического комментирования/раскомментирования `winit` в скриптах без веской причины.**

---

## Стиль кода и соглашения

- Язык комментариев и UI-строк: **русский**.
- Именование: `snake_case` для функций/переменных, `PascalCase` для типов/структур.
- UI-константы (размеры шрифтов, отступы) задаются жёстко в коде видов, без внешних тем.
- Для layout используется `egui_flex::Flex` вместо встроенных layout-ов egui.
- Все экраны реализуют трейт `UiView` из `view_stack.rs`.
- HTTP-запросы выполняются только внутри `tokio_rt.handle().spawn(...)` через `HabreState::async_handle()`.
- Состояние загрузки отображается через `Spinner::new().size(100.)`.

---

## Настройки и персистентность

Настройки сохраняются в файл:

```
~/.local/share/com.lmaxyz/Haboost/settings.toml
```

Сохранённые статьи хранятся в:

```
~/.local/share/com.lmaxyz/Haboost/saved_articles/{article_id}/
    ├── article.json   # метаданные + контент статьи
    └── images/        # скачанные изображения
```

Формат `settings.toml` — TOML, структура `SettingsData`:
- `font_size` — базовый размер шрифта (12.0..=36.0);
- `scale_factor` — масштаб интерфейса (1.0..=3.0);
- `dark_theme` — булево значение тёмной темы.

Значения по умолчанию отличаются по архитектуре: `scale_factor = 2.0` для не-x86_64 (мобильные устройства), `1.0` для десктопа.

---

## Деплой и пакетирование

- RPM-пакеты сохраняются в директорию `RPMS/`.
- Имя пакета: `com.lmaxyz.Haboost`.
- RPM содержит бинарник, `.desktop`-файл и иконки в нескольких разрешениях (`rpm/icons/86x86`, `108x108`, `128x128`, `172x172`).
- Пакет обязательно подписывается перед установкой на устройство с ОС Аврора.
- Старый скрипт `deploy.sh` устарёл и относится к другому проекту (`EguiAuroraExample`) — не используйте его для Haboost.

---

## Безопасность

- Приложение использует **только публичные guest-эндпоинты** Habr API. Никакой аутентификации, токенов или персональных данных не передаётся и не хранится.
- В коде нет секретов или приватных ключей. Сертификаты для подписи RPM находятся **вне репозитория** (в `PSDK_DIR/../../certs/`).
- Все HTTP-запросы идут по HTTPS.

---

## Известные ограничения и TODO

- Не реализованы: спойлеры, таблицы, некоторые HTML-теги.
- Избранное (bookmarks) не реализовано.
- `state.rs` дублирует часть кода из `app.rs` и, вероятно, устарел.
- `deploy.sh` содержит ссылки на другой проект и неактуален.
