# Validation — Leditor 1.7.0-alpha

Validated on 2026-11-07 against published main commit
`ce69188b3ab9165c38847a0cf38b71428f1e4242` (1.6.0-alpha).
Validation was completed before publication; no tag or release was created.

## Environment and results

Linux x86_64, Rust/Cargo 1.99.1, GTK 4.18.6, GtkSourceView 5.16.0,
GLib 2.84.4, PDFium 7881, Xvfb/Openbox, X11 and the Cairo renderer.
Native test dependencies came from the official Debian trixie archive.

- `cargo check --locked --all-targets`: passed.
- `cargo build --locked`: passed, development profile with debug information disabled.
- `cargo test --locked -- --test-threads=1`: 33 passed; 8 graphical tests
  intentionally ignored in this invocation.
- All 8 graphical suites passed in separate processes: **41 Rust tests total**.
- `python3 -m unittest discover -s tests -p 'test_*.py'`: **16 passed**.
  Network and build operations in the installer tests are mocked.
- `cargo fmt --all -- --check`, `bash -n install.sh` and `git diff --check`: passed.
- en-US, pt-BR and es catalogs have 133 identical, unique message keys.
- All 134 third-party Cargo.lock entries are unchanged. Their versions/checksums
  match the bundled license inventory. No dependency was introduced or upgraded.
- Native completion, long tab titles and light/dark bracket screenshots inspected.

## 1.7.0-alpha coverage

- Automatic word suggestions and manual Ctrl+Space invocation through real X11
  keyboard events, including reopening the popup after previous acceptance.
- Up/Down selection, Enter and Tab acceptance, Esc dismissal, Undo/Redo, accented
  words and independent vocabularies in two documents.
- Ordinary Enter retains exact mixed whitespace indentation; ordinary Tab and
  Ctrl+F/Ctrl+G/Esc retain their existing behavior outside completion.
- Closing a tab removes it from the editor and releases its content container.
- Icons match the Explorer for text, source, image, PDF, unknown and Unicode
  filenames. Save As updates the icon and title for the new extension.
- 38 simultaneous tabs: previously allocated title/control widths remain stable,
  the window does not grow, and the strip scrolls horizontally. Normal and short
  Unicode filenames are not ellipsized; long names use middle ellipsis.
- Dirty-state changes preserve width. Full paths/names remain available in tooltips.
- Both themes expose the native bold/single-underline/#86ee77 matching style;
  native tags apply to both endpoints of (), [] and {}. The existing View toggle
  disables matching. Native language-context behavior is retained.

## Existing regression coverage

Document/filesystem errors, atomic saves, overwrite/conflict protection, autosave,
recovery, unsaved prompts, tabs/reordering, native indentation/minimap/case
conversion, Find/Replace, Focus, syntax highlighting, image/PDF viewing, Save All,
resize/maximize/restore, 1.6 editing options, sorting and Go to Line all passed.

## Findings and limitations

- Initial completion testing exposed missing `sourceview5::init()`: native popup
  CSS was not loaded, and reopening could request zero width. Initialization is
  now explicit. The final native keyboard test passes without that warning.
- An additional diagnostic requiring immediate destruction of the native GTK view
  after closing a tab did not pass: this X11 harness retained one reference to an
  unparented view, including a blank tab where completion had not been opened.
  Tab removal and content-container release passed. This observation does not
  establish a completion-specific leak; long-running memory growth and the
  lifetime of toolkit-held references remain unverified. No unsafe forced-disposal
  workaround was added. The functional test checks tab/container removal rather
  than assuming immediate finalization of every toolkit object.
- Native Windows compilation/execution and a native Wayland session were not
  available. Linux results do not establish behavior on those platforms.
- No release/LTO build, clean-machine installer execution or PowerShell test run
  was performed. Installers retain their existing dependency provisions.
- Native word indexing is asynchronous; recently opened/changed text may take a
  few seconds to appear. Very short prefixes can have no suggestions. There is
  no cross-document, semantic or LSP completion.
- File icon artwork depends on the platform MIME database and GTK icon theme,
  exactly as in Leditor's Explorer.
- Existing Windows live-resize artifacts remain under the native-frame + Cairo +
  DirectComposition mitigation described in RESIZE.md.
- Existing sorting/trim/IME limits documented in README.md and DEVELOPMENT.md
  remain. Python installer tests emit the existing tarfile deprecation warning.

## Repeating the checks

Run graphical suites separately in a desktop session to keep GTK on one test
thread. The keyboard tests require Linux/X11, xdotool and a window manager.
Set `LEDITOR_PDFIUM_LIBRARY` to the full PDFium library path when needed.

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
cargo test --locked version_170_tabs_and_bracket_styles -- --ignored --test-threads=1
cargo test --locked version_170_native_completion -- --ignored --test-threads=1
python3 -m unittest discover -s tests -p 'test_*.py'
```
