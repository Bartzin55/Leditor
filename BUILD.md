# Install, Update and Build Leditor

Choose the first option unless you specifically want to install a downloaded
source tree or manage the build yourself.

1. **Recommended: download only the installer.** It downloads the published code
   from GitHub, prepares dependencies, builds and installs Leditor.
2. **Install a local source folder.** Use this for an extracted project ZIP or a
   checkout you want to test, including changes not yet published on GitHub.
3. **Advanced: compile and install manually.** Manage the toolchain, native
   libraries and installation yourself, without running the installers.

## 1. Recommended — download only the installer

You do **not** need to download the complete project or install Rust first.
Internet access and several GB of free disk space are required. Initial downloads
can exceed 500 MB, and compilation may take several minutes. Close Leditor before
installing an update.

Supported automatic-install targets: **Windows 10/11 x64** and **Linux x86_64
with glibc**. ARM, 32-bit, macOS and musl/Alpine are not supported by these scripts.

### Windows

1. Download [install.ps1](https://raw.githubusercontent.com/Bartzin55/Leditor/main/install.ps1)
   and save it as `install.ps1`, not `install.ps1.txt`.
2. Open **64-bit PowerShell** in the folder where you saved it.
3. Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

**Do not add `-Source .` when you downloaded only the script.** Without `-Source`,
Leditor's source code is downloaded from GitHub automatically.

Choose the current-user installation when prompted unless you need an installation
for all users. Open **Leditor** from the Start menu after installation succeeds.

PowerShell and the built-in Windows `tar.exe` must be available. The installer
prepares a temporary MSYS2/UCRT64 toolchain when a suitable existing installation
is unavailable. You do not need to configure MSYS2 yourself for this option.
`ExecutionPolicy Bypass` applies to this invocation only; organizational policy
may still prohibit running the script.

### Linux

1. Download [install.sh](https://raw.githubusercontent.com/Bartzin55/Leditor/main/install.sh)
   and save it as `install.sh`.
2. Open a terminal in that folder.
3. Run as your normal user:

```sh
bash install.sh
```

**Do not add `--source .` when you downloaded only the script.** The standalone
script downloads its Python installer helper into a temporary directory; that
helper downloads the project from GitHub. No manual download of `scripts/` is
needed. Use the updated `install.sh` accompanying this guide; older bootstraps
required that directory beside the script.

Choose the current-user installation when prompted. Open **Leditor** from your
application menu afterward, or run `~/.local/bin/leditor` for a user installation.

Bash and curl are required for the standalone download. Python 3.9+ is reused
when present; otherwise tar and bzip2 are also needed to prepare temporary Python.
Missing build tools and GTK libraries are prepared automatically. Do not run the
installer with `sudo`; it asks for elevation only if you choose system installation.

### Updating an automatic installation

Close Leditor, download the latest installer again, and run the same command
above. The default source is the published `main` branch. The installer skips
rebuilding an already installed Git revision.

Wait until the terminal reports success and finishes removing temporary files.
Cleanup also runs after an error and can take several minutes. A cleanup message
alone does **not** mean installation succeeded. Existing toolchains are not deleted.

## 2. Install a downloaded ZIP or local checkout

Download and extract the **complete project**, or clone it. Open a terminal in
its root folder: `Cargo.toml`, `Cargo.lock`, `src/`, `scripts/` and the installer
must be there. A folder containing only the installer is not a source tree.

### Windows

Check the folder first:

```powershell
Test-Path .\Cargo.toml
```

Continue only if it prints `True`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Source .
```

### Linux

Check the folder first:

```sh
ls Cargo.toml
```

Then run:

```sh
bash install.sh --source .
```

Here, `.` means the project root containing `Cargo.toml`. On Linux, the bootstrap
resolves relative paths from the directory containing `install.sh`; running it
from the project root avoids ambiguity. Absolute source paths are also accepted:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Source "C:\path\to\Leditor"
```

```sh
bash install.sh --source "/path/to/Leditor"
```

Local-source mode copies your project into temporary build space and always
rebuilds. Your original source folder is retained. Use this option to install an
unpublished ZIP; option 1 always fetches published code instead.

## 3. Advanced — compile and install manually

This option does not run `install.sh` or `install.ps1`. You manage dependencies,
updates and removal yourself. Keep `Cargo.lock` and use `--locked`.

Requirements: **Rust 1.88+**, Cargo, **GTK 4.10+**, **GtkSourceView 5**, a native C
compiler, pkg-config and GLib's `glib-compile-resources`. Windows also requires
MinGW's `windres`. The source archive does not include native runtime binaries.

### Obtain the source

Extract a complete source ZIP, or use Git:

```sh
git clone https://github.com/Bartzin55/Leditor.git
cd Leditor
```

Run the following build commands in this directory, where `Cargo.toml` is located.

### Linux: dependencies and compilation

On Ubuntu 24.04, install the native development packages:

```sh
sudo apt install build-essential git pkg-config libgtk-4-dev libgtksourceview-5-dev libglib2.0-dev-bin
```

Install Rust 1.88+ and Cargo separately using the instructions at
[rustup.rs](https://rustup.rs/). Older distribution Rust packages may not meet the
minimum version.

On Arch Linux:

```sh
sudo pacman -S --needed base-devel git rust gtk4 gtksourceview5 glib2-devel pkgconf
```

Check the toolchain and build:

```sh
rustc --version
cargo build --locked --release
./target/release/leditor
```

When run from a user-owned build folder, Leditor's existing startup integration
creates a per-user executable copy, command and menu entry without an installer:

- Executable: `${XDG_DATA_HOME:-$HOME/.local/share}/leditor/bin/leditor`.
- Command: `~/.local/bin/leditor`.
- Menu entry and icon: under `${XDG_DATA_HOME:-$HOME/.local/share}`.

GTK and GtkSourceView must remain installed. If necessary, add `~/.local/bin`
to your shell's PATH. Existing conflicting installations are not overwritten;
use the same installation method to update an existing installation.

For an administrator-managed installation, **instead of running the user build
first**, copy the compiled executable and desktop assets:

```sh
sudo install -Dm755 target/release/leditor /usr/local/bin/leditor
sudo install -Dm644 assets/io.github.leditor.Leditor.desktop /usr/local/share/applications/io.github.leditor.Leditor.desktop
sudo install -Dm644 assets/leditor.png /usr/local/share/icons/hicolor/512x512/apps/io.github.leditor.Leditor.png
```

Make sure `/usr/local/bin` is in PATH. Do not run Cargo or Leditor as root.
To update a manual installation, rebuild the desired source and repeat its
installation procedure with the old application closed.

### Windows: dependencies and compilation

Install [MSYS2](https://www.msys2.org/). Open its **UCRT64** terminal and update it:

```sh
pacman -Syu
```

If asked to close the terminal, reopen UCRT64 and run the update again before
installing build packages:

```sh
pacman -S --needed git mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-pkgconf
```

Obtain the source as described above, then compile and run from UCRT64:

```sh
rustc --version
cargo build --locked --release
./target/release/leditor.exe
```

For a manual installation that uses this permanent MSYS2 runtime:

1. Create a dedicated folder, such as `%LOCALAPPDATA%\Leditor-manual`.
2. Copy `target\release\leditor.exe` and the project's `LICENSE` and `LICENSES`
   into it. Add `pdfium.dll` as described below if you need PDF viewing.
3. Create `launch.cmd` in that folder with the following contents, adjusting
   `C:\msys64` if MSYS2 is installed elsewhere:

```bat
@echo off
set "PATH=C:\msys64\ucrt64\bin;%PATH%"
set "XDG_DATA_DIRS=C:\msys64\ucrt64\share"
set "GSETTINGS_SCHEMA_DIR=C:\msys64\ucrt64\share\glib-2.0\schemas"
start "" "%~dp0leditor.exe" %*
```

Run `launch.cmd`. To add a Start menu entry, create a shortcut to it and place the
shortcut in the folder opened by `shell:programs` in the Windows Run dialog.
Keep MSYS2 installed at the configured path. To update, close Leditor and replace
the executable with your new build.

**The executable alone is not portable.** A self-contained distribution requires
GTK/GtkSourceView DLLs, GLib schemas, icons, MIME data, GdkPixbuf loaders, and
GtkSourceView `language-specs`/`styles`, plus the applicable license notices.
The automatic installer handles packaging; this manual example uses the permanent
MSYS2 runtime instead.

### PDF support for manual builds

PDFium is loaded only when opening a PDF. Text editing and image viewing work
without it; opening a PDF reports an error if the runtime is missing. The automatic
installers already download it and verify its SHA-256 checksum.

For manual installation, download the matching **Chromium 7881** archive from
[PDFium binaries](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7881):

| Platform | Archive | Library inside archive |
| --- | --- | --- |
| Linux x64 | `pdfium-linux-x64.tgz` | `lib/libpdfium.so` |
| Windows x64 | `pdfium-win-x64.tgz` | `bin/pdfium.dll` |

Expected SHA-256 checksums used by the installers:

```text
pdfium-linux-x64.tgz  1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d
pdfium-win-x64.tgz    73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08
```

Verify with `sha256sum` on Linux or `Get-FileHash -Algorithm SHA256` in PowerShell
before extraction. Use the matching ABI rather than an arbitrary newer PDFium.
Place the library beside the installed executable, or set `LEDITOR_PDFIUM_LIBRARY`
to its absolute path. Linux per-user integration also searches
`${XDG_DATA_HOME:-$HOME/.local/share}/leditor/lib/libpdfium.so`. Retain PDFium's
license notices with it and restart Leditor after changing the runtime.

## Automatic installation locations and removal

| Scope | Linux | Windows |
| --- | --- | --- |
| Current user | `${XDG_DATA_HOME:-$HOME/.local/share}/leditor-install` | `%LOCALAPPDATA%\Programs\Leditor` |
| System | `/usr/local/lib/leditor` | `%ProgramFiles%\Leditor` |

Linux commands are placed in `~/.local/bin` or `/usr/local/bin`. Windows shortcuts
are created in the appropriate Start menu. Old installation generations are
retained for recovery. The installer offers to retain or remove its temporary
source copy; your original local source is not removed.

Close Leditor before uninstalling. For a current-user automatic installation:

```sh
bash "${XDG_DATA_HOME:-$HOME/.local/share}/leditor-install/current/uninstall.sh" --scope user
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "$env:LOCALAPPDATA\Programs\Leditor\uninstall.ps1" -Scope user
```

For system installations use the uninstaller in the system installation directory
with `--scope system` / `-Scope system`; Windows requires administrator privileges.
The uninstallers require confirmation and separately ask whether to discard your
settings and recovered documents. They do not manage manual installations.

## Troubleshooting

- **`could not find Cargo.toml`:** if you downloaded only the installer, use
  option 1 without `--source` / `-Source`. For option 2, point at the actual
  extracted project root, not its parent folder or a folder containing only the
  installer. Check this before retrying the downloads.
- **`scripts/install_linux.py` missing:** use the updated standalone `install.sh`
  supplied with this guide, or extract the complete project and use option 2.
- **Mirror timeouts or HTTP errors:** MSYS2 can try another mirror. If package
  installation completes successfully, earlier download warnings alone do not
  mean the installation failed. If the transaction stops, keep its final error.
- **Linux command not found:** use `~/.local/bin/leditor` or add `~/.local/bin` to PATH.
- **Windows resizing artifacts:** the existing native-frame/Cairo/DirectComposition
  mitigation is described in [RESIZE.md](RESIZE.md); it is not a complete fix.
- **Accents fail under Wayland:** close Leditor completely and try
  `GTK_IM_MODULE=simple leditor` as a diagnostic workaround for that execution.
  This bypasses external input methods; it is not forced globally by this guide.

## Developer checks and additional details

To select a specific published branch or tag with the automatic installers,
add `--ref NAME` on Linux or `-Ref NAME` on Windows. Confirm the ref exists first.
`LEDITOR_MSYS2` can select an existing complete UCRT64 installation on Windows.
`GLIB_COMPILE_RESOURCES` and `WINDRES` can override build-tool paths when needed.

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked --lib --test document --test file_ops
```

Graphical suites need a graphical session and must run separately with GTK on one
thread. See [VALIDATION.md](VALIDATION.md) for actual results, platform limitations
and the precise test commands. Documenting a build command does not mean that
platform was tested in the current environment.

Architecture notes: [DEVELOPMENT.md](DEVELOPMENT.md). Dependency licenses:
[THIRD_PARTY.md](THIRD_PARTY.md) and [LICENSES](LICENSES/README.md). Leditor remains MIT.
