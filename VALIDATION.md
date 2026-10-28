# Validation — Leditor 1.5.0

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
