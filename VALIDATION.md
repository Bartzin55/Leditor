# Validation — Leditor 1.6.0-alpha

Completed on 2026-11-05. Base: published main commit
`b7232eae5dc5d9cd54c46bf54e45894c80287fea` (1.5.3-alpha).
No push, tag rewrite or release publication was performed.

## Build and tests

Environment: Linux x86_64, Rust/Cargo 1.99.1, GTK 4.14.5,
GtkSourceView 5.12.0, GLib 2.80.0, PDFium 7881, Xvfb/Openbox and Cairo.

- `cargo check --locked --all-targets`: passed.
- `cargo build --locked`: passed (development profile, debug information disabled).
- `cargo test --locked -- --test-threads=1`: 33 passed, 0 failed;
  6 graphical tests intentionally ignored by the default invocation.
- All 6 ignored graphical tests ran in separate processes and passed:
  **39 Rust tests total**.
- `python3 -m unittest discover -s tests -p 'test_*.py'`: **16 passed**.
  Installer network/build operations are mocked in this suite.
- `cargo fmt --all -- --check`, Bash syntax and `git diff --check`: passed.
- English, Brazilian Portuguese and Spanish catalogs: 129 matching keys,
  no duplicate entries.
- Dependency license inventory matches all 134 registry entries and checksums
  in Cargo.lock. No dependency version or license changed.
- Light/dark Go to Line and whitespace-rendering screenshots inspected.

## New behavior covered

- Native matching of parentheses, brackets and braces in plain text, toggle
  state, and independence from syntax-color visibility.
- Native whitespace drawer configuration and propagation to new tabs.
- Native Home/End and Shift+Home through actual X11 key events.
- All five auto-close pairs, wrapping a Unicode selection, skipping a closer,
  one-step Undo/Redo, disabled mode, escaped quotes and word apostrophes.
  Preedit/overwrite guards and literal programmatic insertion are checked.
- Whole selected lines, partial endpoint lines, a next-line column-zero endpoint,
  preserved duplicates, final-newline presence/absence and Undo/Redo.
- Sorting only the selected characters and disabling sort without a selection.
- Ctrl+G, Ctrl+F and Escape through actual keyboard input; mutual exclusion,
  invalid/overflow input, first/last logical line, wrapped text and scrolling.
- Trimming ASCII spaces/tabs with Unicode offsets and mixed LF/CRLF input.
  Live-buffer undo, BOM/uniform CRLF preservation on disk, autosave's focused-line
  exception, and Save As cancellation without trimming are checked.
- Persistent options, new-tab inheritance and media/unsupported-tab guards.

## Existing regression coverage

All existing tests passed: document/filesystem safety, atomic saves, conflict
protection, autosave/recovery, unsaved prompts, tabs and reordering, native
indentation/minimap/case conversion, Find/Replace, Focus, syntax highlighting,
images/PDF viewing, Save All and Linux resize/maximize/restore.

## Limits and known behavior

- No native Windows build/run, Windows installer execution or Wayland session
  was available. Cross-platform code review and Linux tests do not prove Windows.
- No release/LTO build or full installation on a clean machine was executed.
  PowerShell helper tests were not run; installer behavior is unchanged.
- Existing Windows live-resize artifacts remain; the native-frame + Cairo +
  DirectComposition mitigation was preserved (see RESIZE.md).
- Auto-close handles direct delimiter key events. IME composition/dead keys,
  Ctrl/Alt combinations and overwrite mode retain literal native input.
- Character sorting treats Unicode scalar values individually, including
  spaces/newlines; combining marks are separate. Both sorts ignore case and use
  the OS locale's collation, so ordering can differ between system locales.
- Sorting checkboxes are execution controls: each activation sorts once and
  resets to unchecked. They do not continuously reorder subsequent selections.
- Trimming is opt-in and happens on save. Autosave preserves the focused line
  during that save; an explicit save trims all lines. Undo history is not restored
  across launches, as in previous versions.
- Xvfb emitted EGL/DRI3 warnings without hardware acceleration; the tests passed.
  Python emitted the existing tarfile future-default warning.

The environment was reset while resuming the conversation. Changes were recovered
from the prepared implementation and all validation listed above was rerun in the
final session. The pinned Ubuntu snapshot server returned HTTP 502/503; native
test dependencies were obtained from the official Ubuntu archive instead.

## Repeating graphical checks

Use a graphical session with a window manager. Run each ignored test in a separate
process to keep GTK on a single test thread. The native keyboard test requires
Linux/X11 and xdotool. Set `LEDITOR_PDFIUM_LIBRARY` to PDFium's full path when needed.

```sh
cargo check --locked --all-targets
cargo build --locked
cargo test --locked -- --test-threads=1
cargo test --locked source_engine_and_tab_navigation -- --ignored --test-threads=1
cargo test --locked native_keyboard_and_map_input -- --ignored --test-threads=1
cargo test --locked find_replace_focus_and_minimap -- --ignored --test-threads=1
cargo test --locked version_150_syntax_media_and_save_all -- --ignored --test-threads=1
cargo test --locked resize_maximize_and_focus_preserve_editor -- --ignored --test-threads=1
cargo test --locked version_160_editing_and_navigation -- --ignored --test-threads=1
python3 -m unittest discover -s tests -p 'test_*.py'
```
