# Validation — Leditor 1.5.3

Prepared: 2026-11-05. Base: published main commit
`2407b418b8773905dd2ab6116fb3e33548fab3af` (1.5.2), fetched before editing.
No push or release publication was performed.

## Build and tests

Rust/GUI runs completed on 2026-11-04 in Linux x86_64 with Rust 1.99.1,
GTK 4.14.5, GtkSourceView 5.12.0, PDFium 7881, Xvfb, Openbox and Cairo.

- `cargo check --locked --all-targets`: passed during implementation.
- `cargo build --locked`: passed after the native style-loading correction.
- `cargo test --locked -- --test-threads=1`: 32 passed, 0 failed,
  5 graphical tests intentionally excluded from the default invocation.
- Each of those 5 graphical tests ran separately and passed: **37 Rust tests
  in total**, including the existing GUI lifecycle test in the default run.
- `python3 -m unittest discover -s tests -p 'test_*.py'`: **16 passed**;
  rerun successfully on 2026-11-05. Downloads/builds are mocked in this suite.
- Bash syntax checks and `git diff --check`: passed.
- No dependency versions changed. The About dialog continues to use Cargo's version.

## Regression coverage

- Native Tab stops, multiline Tab/Shift+Tab and keyboard Undo/Redo.
- Native minimap buffer sharing, actual pointer navigation, syntax colors,
  visibility, and fixed map width during editor zoom.
- Light/dark schemes, yellow current-line style, native line-number visibility,
  wrapped logical lines and zoom.
- Unicode case conversion (including Straße → STRASSE), preserved selection,
  first-letter modes, and one-step Undo/Redo.
- Circular tab navigation, reordered tab order, and Ctrl+Tab with focus in Find.
- Ctrl+Shift+W closes the explorer folder; all five new accelerators are registered
  and documented. Ctrl+Shift+O/N reuse existing open/create-folder actions; the
  native chooser interaction for these two shortcuts was reviewed but not driven
  end-to-end in the final automated run.
- Document lifecycle, unsaved prompts, atomic saving, autosave conflict protection,
  session recovery, filesystem safety, find/replace and Focus mode.
- Syntax detection, images, two-page PDF, media recovery and Save All.
- Linux resize/maximize/restore and Focus transitions.

## Issues found and resolved during validation

GtkSourceView 5.12 enumerated GResource style URIs but its parser could not open
them. The final implementation embeds the XML and loads it from a private
TempDir retained by the GUI thread. Scheme tests then passed. If preparation
fails, the editor reports the error to stderr and falls back to native schemes.

The new accelerator test initially checked Shift+Enter while Find was open,
although Leditor deliberately disables Insert Above in that state. The test now
closes Find before checking all registered accelerators; production behavior was
preserved. Initial environment setup also needed a corrected shared-library path.

## Limits

- No native Windows build/run, Windows installer execution or Wayland session
  was available for this release. Linux X11 tests do not prove those environments.
- The existing Windows native-frame + Cairo + DirectComposition mitigation is
  unchanged. Blank areas during interactive resizing remain a known issue (RESIZE.md).
- No release/LTO build or complete installation on a fresh machine was run.
- Xvfb emitted EGL/DRI3 warnings without accelerated graphics. Python emitted
  the existing tarfile future-default warning. Neither failed the final suites.
- A cached PowerShell runtime archive was incomplete, so the PowerShell helper
  suite was not rerun. Windows installer logic is unchanged.
- The temporary Rust/GTK environment was unavailable after the conversation
  resumed on 2026-11-05. The delivered Rust code matches the tested revision;
  only documentation/package preparation and Python/shell checks were finalized
  after that pause. An unexecuted extension of the keyboard test was removed.

## Repeating graphical checks

Use a graphical session (with a window manager), and run each ignored test in a
separate process to keep GTK on one test thread. Set LEDITOR_PDFIUM_LIBRARY to
the full path of PDFium when it is not installed on the runtime search path.
The native input test additionally requires X11 and xdotool on Linux.

```sh
cargo test --locked -- --test-threads=1
cargo test --locked source_engine_and_tab_navigation -- --ignored --test-threads=1
cargo test --locked native_keyboard_and_map_input -- --ignored --test-threads=1
cargo test --locked find_replace_focus_and_minimap -- --ignored --test-threads=1
cargo test --locked version_150_syntax_media_and_save_all -- --ignored --test-threads=1
cargo test --locked resize_maximize_and_focus_preserve_editor -- --ignored --test-threads=1
```
