# Validation — Leditor 1.5.2, resize revision

Date: 2026-11-01. Base: published commit
`78af4daaa7954456231654fd7b464930bbb09a03`. `git pull --ff-only` found no
new changes. Version remains 1.5.2. This revision has not been pushed.

## Diagnosis and implementation

- The user confirmed that the initial Cairo-only workaround did not fix the
  Windows resize trails. That workaround is removed.
- Reviewed the supplied 38.4-second recording, application layout/drawing code,
  GTK documentation and GTK 4.24.1 Win32/decoration/Cairo source. See RESIZE.md
  for source links, observations, inference and the affected-machine checklist.
- Windows now defaults to native decoration (`GTK_CSD=0`) before GTK starts,
  changing the frame/interactive-resize path. Explicit environment overrides
  remain respected. Renderer selection is returned to GTK. Linux is unchanged.
- The enlarged PNG/ICO assets are unchanged from the published 1.5.2.

## Checks completed in this revision

Environment: Linux x86_64, Rust/Cargo 1.99.1, GTK 4.14.5, GtkSourceView 5.12.0,
Xvfb 1920x1080 with Openbox, Cairo for the headless test runs, PDFium 7881.
Test dependencies were provisioned in temporary directories; project manifests
and lockfile were not changed.

- `cargo check --locked`: passed.
- `cargo build --locked` (debug executable): passed.
- Windows-target Rust type checking (`cargo check --locked --target
  x86_64-pc-windows-gnu --all-targets`): passed, including the Windows startup
  branch and native-frame test assertions. This used cross pkg-config metadata
  from the Linux development packages; **it does not validate Windows DLL
  linkage, ABI compatibility or execution**. Native Windows building remains
  required. The only warning concerns existing media-test helpers unused by the
  Windows test target (`page_count`, `rendered_pages`); production check is clean.
- `cargo test --locked -- --test-threads=1`: 34 passed, 0 failed, 3 GUI tests
  intentionally ignored in that invocation; 0 doctests.
- Ran each ignored GUI test separately: all 3 passed. Total: **37 Rust tests**.
  - New resize test: eight size changes across light/dark themes, with search
    visible; maximize/restore; Focus fullscreen/restore; document text, caret,
    zoom and surrounding UI state preserved.
  - Existing find/replace, Focus and minimap integration test.
  - Existing syntax, image/PDF and Save all integration test.
- Python installer suite: **16 passed**. Downloads and compilation are mocked
  by that existing suite; it does not simulate Windows desktop interaction.
- `bash -n install.sh uninstall.sh` and `git diff --check`: passed.
- Rustfmt applied to changed/new Rust implementation/test files. No whole-project
  reformat or new dependency was introduced.
- Resource compilation succeeded for the existing seven-size Windows ICO
  (x86_64 COFF object). Icon artwork remains identical to the published ZIP.

The initial test-environment setup had package-cache permissions, development
symlink and runtime search-path problems. Those were resolved in temporary
paths before the successful build/test runs; no product changes were needed.
Headless runs emitted non-fatal EGL/DRI3 warnings because Xvfb has no accelerated
GPU. The Python suite emitted its existing tarfile future-default warning.

## Limits

- No native Windows machine/GPU/compositor is available here. Linux GUI tests
  validate layout/state, not Win32 interactive dragging or disappearance of the
  user's visual trails. Windows-native frame assertions are included for a
  local run, but are not executed by the Linux suite.
- This is a targeted compatibility correction supported by source inspection,
  not a proven diagnosis of a specific upstream GTK or graphics-driver defect.
- A Windows release executable and Windows end-to-end installer run were not
  produced/tested in this revision. No new release/LTO Linux build was run.
- Test the revised ZIP with `install.ps1 -Source .`; otherwise the installer
  can clone the previously published 1.5.2 instead. Close the old instance first.
- The native Windows title bar follows the system style; editor themes and
  the enlarged icon are preserved. Check resizing at the affected display scale,
  all edges/corners, maximize/restore, snapping and Focus as listed in RESIZE.md.

## Earlier validation (historical, not rerun unless stated above)

# Validation — Leditor 1.5.1

Date: 2026-10-29. Base: `a01cddeed5db882fb8ad95ca3a54d860c2b8b3da`
(1.5.0); `git pull --ff-only` confirmed it was current. No push performed.

## Checks performed for 1.5.1

### Windows extractor correction (2026-10-31)

- Base: published commit `1c8eb38aac4ae13f6f9cd306e2adecf5c7adfd8a`;
  pull found no additional changes. This correction has not been pushed.
- The reported `/usr/bin/tar: Cannot connect to C` corresponds to selecting
  MSYS GNU tar after prepending its directories to PATH. Both extraction calls
  now use the checked absolute `%SystemRoot%\System32\tar.exe` path.
- **6 PowerShell helper checks passed** on Linux with PowerShell 7.5.4.
  Added tests put an MSYS tar placeholder first in PATH and verify the native
  path is selected; a missing native tar fails without falling back to MSYS.
  These tests check selection, not execution of a Windows tar binary.
- **16 Python installer tests passed**; PowerShell AST parsing for both scripts
  and the helper tests, extraction-call inspection and `git diff --check` passed.
- The `portal` field and initializer are now Linux-only, matching the existing
  Linux-only DBus consumer. No compiler is available in this current environment;
  Rust compilation and Windows execution were not rerun. The reported warning
  did not cause installation failure; the tar exit code did.

### PDFium notice-directory correction (2026-10-30)

- Based on published 1.5.1 commit `4630c1c7f084b2f66f59377eaa0bee88cf00d481`;
  `git pull --ff-only` found no additional changes. This correction is not pushed.
- Reproduced the reported `FileExistsError` before changing the installer by
  including both bundled and archive PDFium notice directories in the existing
  installation-flow test. The earlier fixture omitted that directory, which
  explains why previous tests did not catch the failure.
- After the fix, **16 Python tests passed**. Checks cover archive notices
  replacing same-name bundled notices, preservation of other notices, nested
  files, unchanged original source, and publication of the completed generation.
- **4 PowerShell helper checks passed** with PowerShell 7.5.4 on Linux,
  including merging PDFium notices twice into an existing directory.
- Actual pinned PDFium Linux and Windows archives were checked against their
  SHA-256 values. Each platform's license-copy code was exercised twice against
  the bundled notice directory; **16 files per archive matched byte for byte**.
  The Windows copy ran under PowerShell on Linux, not native Windows.
- The Windows-archive test harness initially failed because Linux tar tried to
  restore unavailable owner IDs in this container. Rerunning that harness with
  `--no-same-owner` passed; Windows installer extraction was not changed.
- Bash syntax, Python module compilation and `git diff --check` passed. No Rust
  source, dependencies or version changed. No full compilation or native
  system-wide/Windows installation was rerun for this targeted script fix.

### Installer correction, same version (2026-10-29)

- Python installer suite: **16 passed**, including cleanup on success/failure,
  preservation of external tools, and preservation of the Linux marker during
  simulated update and failed build. Downloads and compilation remain mocked.
- `tests/test_installer_windows.ps1`: **3 checks passed** with PowerShell 7.5.4
  on Linux: initial marker creation, no rewrite of an existing marker opened
  for reading, and temporary-directory deletion with progress messages.
  Windows sharing semantics still require native Windows verification.
- PowerShell AST parsing passed for install, uninstall and the new test script;
  Bash syntax checks, Python compilation and `git diff --check` passed.
- The user reported passing Windows tests followed by a sharing violation on
  the redundant marker write. That write was removed; the process holding the
  file was not identified. No full Windows installation was rerun here.
- Application code/dependencies/version are unchanged by this correction;
  Rust compilation and GUI tests were not repeated for these script edits.

### Earlier 1.5.1 checks

- `cargo check --locked`: **passed** with Rust 1.99.1, GTK 4.24.1 and
  GtkSourceView 5.18.0 in a private conda-forge prefix. This exposed missing
  development packages (zlib, expat, libxml2-devel, glib), now explicitly listed.
- Python installer tests: **14 passed**. Real temporary filesystem operations
  cover uninstall confirmation, settings/recovery retention/deletion, unknown
  files, active-install rejection and malicious archive paths/links. Update and
  failed-build tests mock downloads/compiler execution while exercising actual
  publication/cleanup; the user branch asserts that sudo is never called.
  Windows dependency closure is tested with mocked objdump output.
- `bash -n install.sh uninstall.sh`: **passed**.
- PowerShell 7.5.4 AST parser: **both scripts passed**, on Linux.
- Python module compilation and `git diff --check`: **passed**.
- Rust license collection completed against every registry entry in Cargo.lock,
  validating archive SHA-256. Omitted workspace texts were collected from pinned
  repository commits. Native runtime notices and PDFium notices are included.
- Linux/Windows PDFium 7881 archives downloaded; SHA-256 values pinned in scripts.

## Not validated / limitations

- The new Rust test run (`cargo test --locked --lib --test document --test
  file_ops`) was interrupted by an environment reset during dependency
  compilation. **No passing Rust test count is claimed for 1.5.1.** The temporary
  toolchain/build directory was lost; source files and the completed check result
  survived. Results below belong to the prior release only.
- `cargo fmt --check` could not run: this conda Rust package did not include
  cargo-fmt. Only four Rust integration lines and version metadata changed.
- No full release/LTO build, GUI regression run, Windows build/execution or
  end-to-end installation on a clean user/system machine was completed.
- Windows UAC, Start menu, MSYS2 bootstrap/updates, image-loader relocation and
  runtime DLL deployment still require native Windows testing. Parser success
  is not proof of runtime correctness. Linux system elevation was not exercised.
- Native package resolution is online and may change. Private-runtime downloads
  are substantial; native Linux graphical drivers/session facilities remain OS
  prerequisites. Only glibc x86_64 Linux and x64 Windows are supported.
- Linux bootstrap requires Python >=3.9, or curl/tar/bzip2 to provision Python.
  Windows requires 64-bit PowerShell 5.1+ and tar.exe. Completely minimal OS
  images without these bootstrap facilities are not supported.
- Previous generations are retained for recovery; uninstall removes them. Only
  the invoking account's data is optionally removed. Retained source copies and
  shared dependencies are intentionally kept. Original source directories are
  never deleted by the installer.
- LICENSES supplies texts/notices; redistributing a future native binary bundle
  still requires its matching source/compliance review for LGPL components.

## Historical validation — 1.5.0 (not rerun for this release)

Date: 2026-10-28. Base: GTK 1.4.1, commit `1676a13d1fd9ef1c93c1462963f8888b8d835f27`. The remote was checked before work; no push was performed.

## Environment and successful checks

Linux x86_64 (Ubuntu 24.04), Rust/Cargo 1.99.1, GTK 4.14.5, GtkSourceView 5.12.0, pdfium-render 0.9.4 and PDFium Chromium 7881 x64. Native development packages were extracted into a local prefix. GTK suites ran in separate processes under Xvfb/X11 over local TCP, with Cairo rendering. Configuration and documents were isolated in temporary directories.

- `cargo build --locked`: passed, Linux debug executable.
- `cargo check --locked`: passed.
- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- Library tests: **1 passed**.
- Document tests: **12 passed**.
- Filesystem tests: **9 passed**.
- Non-GUI binary tests: **11 passed**.
- Existing GTK lifecycle/editing/file safety suite: **1 passed**.
- Existing GTK search/minimap/focus/1.4.1 regression suite: **1 passed**.
- New syntax/media/Save All GTK integration suite: **1 passed**.
- Total: **36 passed**, no failures in the final runs. GTK suites marked ignored are run explicitly in separate processes, not left untested.

## What the new integration suite exercised

- Five-menu order in EN/PT-BR/ES, Plain Text first and one menu entry for every runtime GtkSourceView language definition.
- Automatic Rust detection; actual keyword tags; manual language action and tab switching; Detect language; disabling and persisting the preference; Plain Text selection; both theme schemes; highlighting does not dirty text or add an undo step; ordinary edit/Undo remains correct.
- Save All for named and untitled tabs; cancellation leaves later tabs dirty; a failed write to a directory preserves an untitled document and its contents.
- Actual PNG/JPEG creation, decoding and read-only tabs; minimap hidden; only zoom in the footer; 200% geometry; editing and Save are rejected; original bytes remain identical.
- Two-page PDF load and rasterization, scrolling to the second page, zoom limits, malformed PDF/image errors, and session recovery of all three media tabs alongside unsaved text.
- Visual snapshots inspected for syntax highlighting in both light/dark palettes and rendered PDF content, without text gutters or other footer fields.

An initial PDF scrolling test found that a newly visible page was not invalidated after the viewport moved. The callback was corrected and the suite passed again. Early GUI starts failed because this execution environment blocks Unix sockets; running the X server and tests together through local TCP resolved that environment issue. Xvfb emits EGL/DRI3 acceleration warnings; the configured Cairo renderer continued successfully.

## Limits and untested platforms

- **Windows was not compiled or executed here.** MSYS2/UCRT64 build/runtime instructions are supplied, but native DLL packaging and Windows behavior need testing on the user's machine.
- Native Wayland/compositor behavior and real desktop appearance notifications were not revalidated in this release.
- No release/LTO build, sustained large-document stress test or exhaustive PDF compatibility suite was performed. The PDF tests use a simple two-page PDF; this is not a claim of universal PDF compatibility.
- PDFium must be installed separately. The ZIP contains source and required application assets, not third-party runtime binaries. Text/image use does not require PDFium.
- Basic PDF viewing does not support password entry, annotations, editing, text search or printing. Corrupt/password-protected files report a load error. Original media files are never rewritten by the viewers.
- PDF raster resolution is capped at 4096 pixels per axis, with up to 128 MiB of cached raster pixels per viewer. At high zoom the displayed page can exceed this raster resolution and appear softer. PDF file bytes, decoded images and page widgets consume additional memory; total app memory is not capped.
- Image decoding runs on the UI thread after asynchronous byte reading; very large PNG/JPEG files can briefly pause the interface. No image-size stress benchmark was performed.
- Installed GtkSourceView language/style data and GdkPixbuf PNG/JPEG loaders are required. Language labels come from that native catalog; low-level library/OS error details may remain in English.

See BUILD.md for the repeatable commands and runtime setup. No tests justify claiming the application is bug-free.

## Follow-up: user report, 2026-11-01

Windows 11 build 26200, GTK 4.24.1, AMD RX 6600, 180 Hz. Windows already
enables showing contents while dragging. All three cases exited successfully
but the user reported blank exposed client areas during resize in all of them.
The recording confirms a native frame growing beyond the old client drawing.

The default renderer already fell back to Cairo: GL and Vulkan reported that
they require DirectComposition. Case 2 rejected GDK_DISABLE=dcomp; it is not
a validated independent disable test. Case 3 accepted GDK_DEBUG=dcomp and
selected Cairo, with no reported initialization failure. These logs do not
independently prove which presentation path every frame used.

Frame-clock logs contain maximum inter-frame intervals of 3068.1, 2411.1 and
1411.1 ms respectively. Idle applications need not paint continuously, so
these maxima alone are NOT resize-stall measurements or a performance benchmark.
CSV traces observed native move/resize in 92/154, 196/289 and 221/278 samples.
They measure HWND client geometry, not GTK widget allocations. Exact root cause
remains unresolved: these records cannot separate frame scheduling, layout and
presentation conclusively. There is no evidence sufficient to blame the GPU
driver, GtkSourceView or missing D-Bus.

User-authorized mitigation: native frame + Cairo + DirectComposition on Windows.
Set GTK_CSD=0, GSK_RENDERER=cairo and GDK_DEBUG=dcomp before GTK initializes,
only when each variable is absent. No frame-debug logging is enabled. Explicit
overrides remain respected. Linux and version 1.5.2 are unchanged.
This supersedes the renderer-default policy described above. No guaranteed
fix or native Windows validation of this source revision is claimed.

Validation for the DirectComposition default change: git diff --check and ZIP
integrity/license inventory passed. A fresh cargo check was attempted but could
not start: the temporary Rust/native build environment from the previous turn
is no longer present. The earlier 37 Rust and 16 Python passes apply to the
preceding native-frame revision, not to a Windows execution of this change.
No native Windows build or live-resize test was performed here.
