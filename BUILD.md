# Build and Run — Leditor 1.1.0

Use a current stable Rust toolchain and GTK **4.10 or later**. Keep Cargo.lock and use `--locked`. The source archive does not include installers or Windows DLLs.

## Linux

On Ubuntu 24.04 or later:

```bash
sudo apt install build-essential pkg-config libgtk-4-dev
cargo build --locked --release
cargo run --locked --release
```

On Arch Linux:

```bash
sudo pacman -S --needed base-devel rust gtk4 pkgconf
cargo build --locked --release
./target/release/leditor
```

Run commands from the extracted project directory. Older distributions with GTK below 4.10 need newer GTK development packages. The app uses native GTK dialogs and works offline after dependencies have been downloaded.

## Windows x64

Install MSYS2, open its **UCRT64** terminal, update its packages, and install:

```bash
pacman -Syu
pacman -S --needed mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-pkgconf
cargo build --locked --release
./target/release/leditor.exe
```

Reopen UCRT64 if the MSYS2 update requests it. Build and run within UCRT64 so that GTK tools and DLLs are on PATH. A portable distribution must bundle GTK's runtime DLLs, schemas, icons, and relevant licenses; copying the EXE alone is insufficient. This release has not been compiled or executed on Windows in the current validation environment.

## Validation commands

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked -- --test-threads=1
```

The Linux GTK integration test requires a functioning graphical session. For a Linux headless host where Xvfb and sockets are permitted:

```bash
xvfb-run -a cargo test --locked -- --test-threads=1
```

Tests not requiring a display:

```bash
cargo test --locked --test document --test file_ops -- --test-threads=1
cargo test --locked --bin leditor session::tests -- --test-threads=1
```

The GTK integration test is Linux-only. Successful Windows filesystem tests alone would not validate the Windows interface.
