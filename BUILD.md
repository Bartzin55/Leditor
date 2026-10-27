# Build and Run — Leditor 1.4.0

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



## Linux integration (1.3.0)

On startup, including `cargo run --locked` or `cargo run --locked --release`, Leditor installs/updates these **per-user** files, without sudo:

- `$XDG_DATA_HOME/leditor/bin/leditor` (defaults to `~/.local/share/leditor/bin/leditor`): independent executable copy, updated atomically only if different.
- `~/.local/bin/leditor`: symlink to that executable.
- `$XDG_DATA_HOME/applications/io.github.leditor.Leditor.desktop`: launcher entry.
- `$XDG_CONFIG_HOME/leditor/settings.ini` (defaults to `~/.config/leditor/settings.ini`): preferences. The pre-1.3 settings file is read when the new file does not exist, and is retained.
- `$XDG_DATA_HOME/leditor/session.ini`: session recovery, unchanged.

The launcher does not depend on keeping the source/build directory. GTK runtime libraries are still required. Installation failures are reported on stderr and do not prevent editing. Existing unrelated commands or unmanaged desktop entries are not replaced. No Windows installation runs.

If your shell does not include `~/.local/bin` in PATH, add the following to your shell's profile and reopen the terminal (or run it in the current terminal):

```sh
export PATH="$HOME/.local/bin:$PATH"
leditor
```

The application cannot change the PATH of its parent terminal. Desktops may refresh the application list only after logging out and back in.

### Optional global Linux installation

`/bin` and `/usr/bin` are administrator-managed locations (often the same directory through a symlink). The app never escalates privileges. For a conventional global installation use `/usr/local/bin`, after building:

```sh
sudo install -m755 target/release/leditor /usr/local/bin/leditor
sudo install -Dm644 assets/io.github.leditor.Leditor.desktop /usr/local/share/applications/io.github.leditor.Leditor.desktop
```

If `/usr/bin` is explicitly required, replace `/usr/local/bin/leditor` with `/usr/bin/leditor` in the first command; do not install twice into `/bin` and `/usr/bin`. Global installations skip per-user binary installation. Do not run Cargo or the GUI as root.

To remove per-user integration, delete only the installed symlink, the managed desktop entry and `leditor/bin/leditor` at the paths above. Keep `settings.ini` and `session.ini` unless you intentionally want to discard preferences and recovery data.

## 1.4.0 platform requirements and tests

Linux GTK must include its X11 backend (`pkg-config gtk4-x11`), as provided by the Ubuntu/Arch GTK packages above. Native Wayland remains supported for editing, search, minimap and Focus. Always on top is disabled on Wayland because standard GTK toplevels cannot request a universal above-other-windows state there. X11 requests are subject to window-manager policy.

Windows uses the GTK Win32 backend and the operating system's SetWindowPos API; no additional installer or service is introduced.

New platform bindings: gdk4-x11/gdk4-win32 0.10 (MIT), x11rb 0.13 (MIT OR Apache-2.0), windows-sys 0.61 (MIT OR Apache-2.0). These are platform-specific and Cargo selects the relevant ones. Cargo.lock pins exact versions. Leditor remains MIT; native GTK's existing distribution requirements still apply.

The existing GTK regression and new 1.4.0 graphical suite must run in separate processes because GTK initialization belongs to one thread. The new suite is ignored by default for that reason; invoke it explicitly:

```bash
cargo test --locked --lib --test document --test file_ops
cargo test --locked --bin leditor -- --skip ui:: --test-threads=1
cargo test --locked --bin leditor ui::tests::tabs_history_layout_dialogs_and_file_safety -- --exact
cargo test --locked --bin leditor ui::search::tests::find_replace_focus_and_minimap -- --ignored --exact
```

The last two commands require a graphical display. Tests isolate settings and files in temporary directories. The legacy UI regression is Linux-only; the new GTK suite is portable but has only been executed on Linux.
