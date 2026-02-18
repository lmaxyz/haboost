# Сборка для ОС Аврора

## Предпочтительный способ: скрипты сборки

Для сборки под ОС Аврора рекомендуется использовать bash-скрипты, которые автоматически выполняют все необходимые шаги:

```bash
# Убедитесь, что установлена переменная окружения PSDK_DIR
# (путь к Platform SDK ОС Аврора)

# Сборка для AArch64 (64-бит ARM) — современные устройства
./aarch64_build.sh

# Сборка для ARMv7 (32-бит ARM) — старые устройства
./arm_build.sh

# Сборка для x86_64 с подписью RPM
./x86_64_rpm_build.sh
```

**Преимущества использования скриптов:**
- Автоматически раскомментируют патч `winit` в Cargo.toml перед сборкой
- Собирают релизную версию через `cross`
- Создают RPM-пакет в директории `RPMS/`
- Подписывают RPM сертификатами Platform SDK
- Автоматически закомментируют патч обратно после сборки (даже при ошибке), чтобы локальная разработка работала без патчей

## Ручная сборка

Если вы хотите выполнить сборку вручную:

### Необходимые патчи в Cargo.toml

Для поддержки ОС Аврора необходимо использовать форки `winit` и `glutin` с кастомными ветками:

```toml
[patch.crates-io]
winit = { git = "https://github.com/lmaxyz/winit", branch = "aurora" }
glutin = { git = "https://github.com/lmaxyz/glutin", branch = "aurora_device_fix" }
```

**Примечание:** Патч `winit` закомментирован в текущем Cargo.toml для того, чтобы приложение можно было запускать и тестировать локально. Раскомментируйте его перед сборкой для ОС Аврора.

### Команды сборки

```bash
# Установка инструмента cross для кросс-компиляции
cargo install cross

# Сборка для ARM (armv7)
cross build --release --target armv7-unknown-linux-gnueabihf

# Сборка для AARCH64
cross build --release --target aarch64-unknown-linux-gnu

# Сборка для x86_64
cross build --release --target x86_64-unknown-linux-gnu

# Создание RPM-пакета
cargo generate-rpm -a <arch> --target <target> -o RPMS/
```

### Подпись RPM

Для установки на устройство с ОС Аврора пакет должен быть подписан:

```bash
# Через Platform SDK
$PSDK_DIR/sdk-chroot rpmsign-external sign \
  -k $PSDK_DIR/../../certs/your_key.pem \
  -c $PSDK_DIR/../../certs/your_cert.pem \
  your-package.rpm
```

## Требования

- Установленный Rust с cargo
- Установленный `cross` для кросс-компиляции
- Platform SDK ОС Аврора (для подписи RPM)
- Переменная окружения `PSDK_DIR`, указывающая на путь к Platform SDK
