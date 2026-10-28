# Build and Run — Leditor 1.4.1

Use a current stable Rust toolchain and GTK **4.10 or later**. Keep Cargo.lock and use `--locked`. The source archive does not include installers or Windows DLLs.

## Linux

On Ubuntu 24.04 or later:

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libglib2.0-dev-bin
cargo build --locked --release
cargo run --locked --release
```

On Arch Linux:

```bash
sudo pacman -S --needed base-devel rust gtk4 glib2-devel pkgconf
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
sudo install -Dm644 assets/leditor.png /usr/local/share/icons/hicolor/512x512/apps/io.github.leditor.Leditor.png
```

If `/usr/bin` is explicitly required, replace `/usr/local/bin/leditor` with `/usr/bin/leditor` in the first command; do not install twice into `/bin` and `/usr/bin`. Global installations skip per-user binary installation. Do not run Cargo or the GUI as root.

To remove per-user integration, delete only the installed symlink, the managed desktop entry and `leditor/bin/leditor` at the paths above. Keep `settings.ini` and `session.ini` unless you intentionally want to discard preferences and recovery data.

## Icon build tools and desktop integration

`build.rs` embeds the PNG with `glib-compile-resources`, supplied by GLib development packages (Ubuntu `libglib2.0-dev-bin`, Arch `glib2-devel`; MSYS2 UCRT64 `mingw-w64-ucrt-x86_64-glib2`). Set `GLIB_COMPILE_RESOURCES` if it is not on PATH. Windows additionally uses `windres` from MinGW; set `WINDRES` for a cross compiler such as `x86_64-w64-mingw32-windres`.

Linux startup installs the icon under `$XDG_DATA_HOME/icons/hicolor/512x512/apps/io.github.leditor.Leditor.png` alongside the existing managed desktop entry, without root access. Launchers may cache the old icon until a refresh or session restart.

The Windows executable contains the icon and About uses the embedded PNG. No automatic Windows installer or Start menu registration is added. A normal shortcut pointing to `leditor.exe` uses its embedded icon; place it in your Start menu if desired. A portable executable still requires GTK DLLs/runtime assets.

The dedicated GTK X11/Win32 bindings and x11rb are no longer required. windows-sys is retained only for reading the Windows application color-scheme preference. Leditor remains MIT; native GTK distribution obligations still apply.

## Validation commands

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --lib --test document --test file_ops
cargo test --locked --bin leditor -- --skip ui:: --test-threads=1
cargo test --locked --bin leditor ui::tests::tabs_history_layout_dialogs_and_file_safety -- --exact
cargo test --locked --bin leditor ui::search::tests::find_replace_focus_and_minimap -- --ignored --exact
```

Run GTK suites in separate processes with a graphical display: GTK must remain on one thread. Tests isolate settings/documents in temporary directories. See VALIDATION.md for checks actually performed.
