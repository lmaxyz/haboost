# Haboost

Haboost - это ограниченный неофициальный клиент ресурса [Habr](https://habr.com).

Приложение написано в качестве демонстрации возможности разработки под ОС Аврора (и, возможно, Sailfish OS) с использованием языка Rust и библиотеки [egui](https://github.com/emilk/egui).

## Скриншоты с устройства R570E

<div align="center">
    <img src="screenshots/1.png" alt="screenshot 1" width=200>
    <img src="screenshots/2.png" alt="screenshot 2" width=200>
    <img src="screenshots/3.png" alt="screenshot 3" width=200>
    <img src="screenshots/4.png" alt="screenshot 4" width=200>
</div>

## Roadmap

- [x] Поиск по хабам
- [ ] Чтение статей
  - [x] Заголовки
  - [x] Обычный текст
  - [ ] Изображения
    - [x] Отображение
    - [ ] Возможность масштабирования
  - [ ] Ссылки
    - [x] Отображение
    - [ ] Возможность перехода по ссылке
  - [x] Курсив
  - [x] Жирный шрифт
  - [ ] Списки
  - [ ] Цитаты
  - [x] Код
  - [ ] Спойлеры
  - [ ] Другие тэги, про которые я забыл
- [ ] Локальное хранение статей
- [ ] Копирование выделенного текста
- [ ] Поиск по статьям
- [ ] Настройки
  - [ ] Размер шрифта
  - [ ] Директория сохранения статей
  - [ ] Коэффициент масштабирования

## Особенности сборки

Для сборки под устройство с ОС Аврора необходимо в файле [Cargo.toml](Cargo.toml) прописать патчи на библиотеки `winit` и `glutin`, без них приложение не сможет запуститься.

```toml
[patch.crates-io]
winit = { git = "https://github.com/lmaxyz/winit", branch = "aurora" }
glutin = { git = "https://github.com/lmaxyz/glutin", branch = "aurora_device_fix" }
```

## License

Copyright 2025 Leiman Maksim

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
