# Validation — Leditor 1.4.1

Date: 2026-10-27 (America/Sao_Paulo). Base: published GTK 1.4.0, Git commit `1696cdbdfcc43df03178c30f751c8705b6546649`. No push, tag rewrite, Qt code or alpha relabeling.

## Results

| Check | Result |
|---|---|
| `cargo build --locked` (Linux debug) | Passed |
| `cargo check --locked` | Passed |
| `cargo fmt --check` | Passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| Library tests | 1 passed |
| Document integration tests | 12 passed |
| Filesystem integration tests | 9 passed |
| Non-GUI binary tests | 11 passed |
| GTK lifecycle/files/dialogs regression suite | 1 passed |
| GTK search/minimap/focus and new corrections suite | 1 passed |
| Total tests | **35 passed**, no failures in the final runs |
| Windows icon resource | Compiled with MinGW windres; verified x86-64 COFF `.rsrc` section |
| Windows application compilation/execution | Not performed in this environment |
| Rendered guide review | Dark/light screenshots inspected; dashed, offset guides continue across blank lines |
| Version/dependency metadata and Git whitespace | Verified |

Environment: Linux x86-64, Rust/Cargo 1.89, GTK 4.14.5, Xvfb virtual X11 display, Cairo renderer. The GTK suites run in separate processes. Tests use temporary documents and settings. Toolchains and machine-specific build paths are excluded from the ZIP.

## New coverage

- Always on top action is absent and no corresponding preference is persisted.
- Ctrl+F toggles the bar. Insert Above is disabled and directly guarded while the bar is visible, then enabled again after closing.
- A Shift+Enter event delivered to the window search controller selects the previous match without changing document text.
- System selection uses exactly the same light stylesheet as explicit Light and the same base dark stylesheet as explicit Dark.
- GTK theme change notifications switch the resolved preference after opening the application. A regression test caught and led to correction of monitor initialization placement.
- Guide depth calculation covers blank lines, mixed tabs/spaces and different widths; screenshots cover actual rendering in both palettes.
- The About image has a decoded paintable and the embedded application icon is discoverable by GtkIconTheme.
- Linux installation writes the provided PNG and a matching desktop icon name, preserving existing update/conflict tests.

Existing suites also cover file safety, unsaved prompts, tabs, undo/redo, recovery, explorer operations, search replacements, minimap navigation, focus, zoom, Unicode and preference persistence.

## Limitations and encountered issues

The temporary environment reset during development; these results come from fresh runs after reconstructing the working tree. The first build attempt after reset lacked the recovered icon asset and failed resource compilation; it passed after restoring the supplied asset. Earlier environment setup also required repairing local development-library linker symlinks. None of those toolchain paths are project requirements.

Xvfb printed expected EGL/DRI3 and keyboard-map warnings. Rendering and tests completed with the software renderer. A physical desktop, native COSMIC/Wayland portal notifications, GTK 4.20+ appearance-property behavior and Windows runtime behavior were not exercised. Validate those on your own desktop; the Windows resource check does not establish a working Windows application build.

No release/LTO build or large-file stress test was run in this validation. The ZIP is complete source, not an executable distribution. The existing Windows build instructions include GTK runtime requirements; no automatic Windows installation or Start menu registration is introduced. Linux launchers may cache an old icon until refreshed or the session restarts.

Reproduction commands and dependencies are in BUILD.md. This report does not claim bug-free operation.
