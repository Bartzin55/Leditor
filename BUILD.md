# Build and Run — Leditor 1.5.1

## Automated installation / update

Supported targets: Linux x86_64 with glibc and Windows 10/11 x64. ARM, 32-bit,
macOS and musl/Alpine are rejected. Internet and several GB of free space are
required to download and compile the toolchain. Initial downloads can exceed
500 MB when no development tools are present. Close Leditor before updating.

For this **unpublished source ZIP**, extract it and run from its directory:

```sh
bash install.sh --source .
```

On Windows, use 64-bit PowerShell from the extracted directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Source .
```

ExecutionPolicy Bypass applies only to this invocation. Organizational policy
may still prohibit execution; the script does not circumvent it.

Both scripts ask for user or system installation. User installation never calls
sudo or requests UAC. Linux system installation uses `/usr/local/bin` and
`/usr/local/lib/leditor`, not distribution-owned `/usr/bin`; Windows uses Program
Files. Run the Linux installer as a normal user even for system installation.
The Linux user installation uses `$XDG_DATA_HOME/leditor-install` and
`~/.local/bin/leditor`; Windows uses `%LOCALAPPDATA%\Programs\Leditor`.

Once 1.5.1 is published, omit `--source .` / `-Source .` to clone main. A specific
branch/tag can be selected with `--ref REF` / `-Ref REF`. Do not assume a release
tag exists. Until publication, main is still 1.5.0 and lacks these scripts.
Re-running against the same Git revision does not rebuild; a changed revision
creates a new generation. Local source mode always rebuilds.

After tests pass, the installer reports preparation of the application files.
At the end (also after a failure), it reports removal of temporary build files
and tools; large folders can take several minutes. Windows lists each temporary
top-level item as it is removed. Linux also reports removal of the temporary
Python bootstrap when one was needed. Wait for the shell prompt to return.
The existing `.leditor-managed` marker is not rewritten during updates.

Linux reuses GTK >= 4.10, GtkSourceView 5 and build tools when available.
Otherwise micromamba 2.5.0 obtains a private GTK runtime from conda-forge; Rust
and missing build tools are installed temporarily. This avoids apt/pacman and
admin access in user mode. A working Python >= 3.9 is reused. Without Python,
the shell bootstrap needs curl, tar and bzip2 to provision it; those basic OS
utilities are prerequisites and are not bootstrapped from nothing.

Windows reuses a complete UCRT64 environment at `LEDITOR_MSYS2` or `C:\msys64`
without changing its packages. Otherwise a temporary MSYS2 environment provides
Rust, GCC, Python, GTK and GtkSourceView. Windows tar.exe and PowerShell are
bootstrap prerequisites. DLLs, GTK data and PDFium remain in the installed
generation after the temporary MSYS2 environment is removed.

The build uses `cargo build --locked --release`, followed by non-GUI document
and filesystem tests. PDFium 7881 downloads are SHA-256 checked. Exact Rust
licenses and native runtime notices are installed with the application.
Previously installed tools are not removed. You are asked whether to delete the
temporary source copy or retain it in a new `leditor-source-*` directory under
your home. Your original `--source` directory is never deleted.

Failed builds do not replace the active generation. Old successful generations
are retained for recovery and are removed by uninstall. Close the application
before uninstalling; Windows may refuse to remove files in use.

```sh
bash uninstall.sh --scope user
# or: bash uninstall.sh --scope system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1 -Scope user
# or: -Scope system
```

Uninstall requires typing REMOVE, then asks separately whether to discard the
current account's settings and recovered/unsaved documents. Other accounts,
original sources and shared system packages are never automatically removed.
The installed copy of uninstall.sh is in `leditor-install/current/`; on Windows,
uninstall.ps1 is in the Leditor installation root.

## Manual compilation and dependencies (fallback)

Use Rust 1.88+ and GTK **4.10 or later**. Keep Cargo.lock and use `--locked`. The source archive includes scripts, but not Windows DLLs or native runtime binaries.

## Linux

On Ubuntu 24.04 or later:

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libgtksourceview-5-dev libglib2.0-dev-bin
cargo build --locked --release
cargo run --locked --release
```

On Arch Linux:

```bash
sudo pacman -S --needed base-devel rust gtk4 gtksourceview5 glib2-devel pkgconf
cargo build --locked --release
./target/release/leditor
```

Run commands from the extracted project directory. Older distributions with GTK below 4.10 need newer GTK development packages. The app uses native GTK dialogs and works offline after dependencies have been downloaded.

## Windows x64

Install MSYS2, open its **UCRT64** terminal, update its packages, and install:

```bash
pacman -Syu
pacman -S --needed mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-pkgconf
cargo build --locked --release
./target/release/leditor.exe
```

Reopen UCRT64 if the MSYS2 update requests it. Build and run within UCRT64 so that GTK tools and DLLs are on PATH. A portable distribution must bundle GTK's runtime DLLs, schemas, icons, and relevant licenses; copying the EXE alone is insufficient. This release has not been compiled or executed on Windows in the current validation environment.

## PDFium runtime (Linux and Windows)

The Rust crate does not compile or download PDFium. The application dynamically loads it only when opening a PDF. Text editing and images work without it; PDF opening reports an error if it is absent. No PDFium binary is bundled in this source ZIP.

The tested ABI is **PDFium Chromium 7881**, selected explicitly by the `pdfium_7881` Cargo feature. [PDFium binaries](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7881) provides builds for both platforms. This is a third-party build distribution, not a Leditor dependency installer. Keep its LICENSE and licenses/ directory when redistributing its binaries. For future runtime updates, select a matching ABI supported by pdfium-render and retest it; do not silently exchange unrelated versions.

Linux x64 example (run from the project directory):

```sh
curl -fL https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-linux-x64.tgz -o pdfium-linux-x64.tgz
mkdir -p pdfium-runtime
tar -xzf pdfium-linux-x64.tgz -C pdfium-runtime
mkdir -p "${XDG_DATA_HOME:-$HOME/.local/share}/leditor/lib"
cp pdfium-runtime/lib/libpdfium.so "${XDG_DATA_HOME:-$HOME/.local/share}/leditor/lib/"
cargo run --locked --release
```

The per-user runtime location also works with the automatically installed launcher after the source folder is moved. The runtime and notices remain the user's responsibility; the app does not download or update libraries.

Windows x64, in MSYS2 UCRT64 after the Cargo build:

```sh
curl -fL https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz -o pdfium-win-x64.tgz
mkdir -p pdfium-runtime
tar -xzf pdfium-win-x64.tgz -C pdfium-runtime
cp pdfium-runtime/bin/pdfium.dll target/release/
./target/release/leditor.exe
```

Alternatively set `LEDITOR_PDFIUM_LIBRARY` to the absolute `.so`/`.dll` path. Without that override, lookup tries beside the executable, the per-user data location `leditor/lib`, then the OS library loader. It deliberately does not add the current document's folder to the library search path. Restart Leditor after installing/changing the runtime.

GtkSourceView's `language-specs` and `styles` data directories must accompany a portable GTK distribution (normally in `share/gtksourceview-5`); the native library alone is insufficient. PNG/JPEG need their GdkPixbuf loaders. Keep architecture and runtime ABI consistent across GTK, GtkSourceView, PDFium and the executable.

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

Run the new UI integration suite separately with a display and PDFium installed:

```sh
cargo test --locked --bin leditor ui::tests::version_150_syntax_media_and_save_all -- --ignored --exact
```
