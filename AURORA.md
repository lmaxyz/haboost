# Aurora OS Build Instructions

## Required Cargo.toml Patches

For Aurora OS support, you must patch `winit` and `glutin` crates with custom branches:

```toml
[patch.crates-io]
winit = { git = "https://github.com/lmaxyz/winit", branch = "aurora" }
glutin = { git = "https://github.com/lmaxyz/glutin", branch = "aurora_device_fix" }
```

**Note**: The `winit` patch is currently commented out in this project. Uncomment it to build for Aurora OS.

## Build Commands

```bash
# Install cross tool for ARM cross-compilation
cargo install cross

# Build for ARM (armv7)
cross build --target armv7-unknown-linux-gnueabihf

# Build for AARCH64
cross build --target aarch64-unknown-linux-gnu
```
