# Validation — Leditor 1.5.2

Date: 2026-10-31. Base: published commit
`7eeab9056ff8fc19c5e224af5e06e5ddb06ec55e`. Pull found no new changes.
No push performed for 1.5.2.

## Evidence and checks

- Inspected the supplied 38.4-second Windows recording via sampled frames.
  Visible trails include the title bar during resizing. The video does not
  prove whether the root cause is GTK, a GPU driver or desktop composition.
- Selected Cairo as a Windows-only compatibility workaround, before GTK
  initialization. Existing explicit GSK_RENDERER values are respected.
  GTK documents this renderer selection and its software fallback:
  https://docs.gtk.org/gtk4/running.html
- Python installer suite: **16 passed**, with downloads/compilation mocked as
  documented in the earlier sections. Bash syntax and git diff --check passed.
- PNG exported at 512x512 RGBA. Visible artwork (alpha >=128) spans 452 pixels
  vertically (88.3%), versus about 60% in the prior canvas.
- ICO verified at 16, 24, 32, 48, 64, 128 and 256 pixels, using the same artwork.
- Version manifests, About release date, locale comments and packaging reviewed.
  Dependency versions and license inventory are unchanged.

## Limits / required local testing

- Rust, Cargo, pkg-config and GTK development packages are unavailable in this
  environment. No new compilation, Rust test or GUI run is claimed for 1.5.2.
- Native Windows resize retesting is required; this is a rendering workaround,
  not a confirmed diagnosis or a verified elimination of the reported bug.
- Cairo can increase CPU use when rendering large windows or PDF pages. Linux
  keeps its existing renderer selection.
- Verify PNG/ICO appearance in the launcher, taskbar and About on both systems.
  Shell icon caches may require restarting the app or repinning its shortcut.

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
