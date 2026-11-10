# Validation — Leditor 1.8.0-alpha

Validated on 2026-11-09/10 against published main commit
`0484f9ee017e47f2d7899548b0976e3f98f63c47` (1.7.0-alpha and installer/documentation
fixes). No push, tag or GitHub release was performed for this update.

## Environment and results

Linux x86_64, Rust/Cargo 1.99.1, GTK 4.24.1, GtkSourceView 5.18.0,
GLib 2.90.1, PDFium 7881, Xvfb 21.1.16/Openbox 3.6.1, X11 and the Cairo renderer.
The private build environment used conda-forge packages; X11 test utilities came
from Debian trixie. Keyboard composition tests used a Brazilian keymap.

- `cargo check --locked --all-targets`: passed.
- `cargo build --locked`: passed, development profile with debug information disabled.
- `cargo test --locked -- --test-threads=1`: **39 passed**; 9 graphical tests
  intentionally ignored in this invocation.
- All 9 graphical suites passed in separate processes: **48 Rust tests total**.
  The final 1.8 suite also passed with `G_DEBUG=fatal-criticals`.
- `python3 -m unittest discover -s tests -p 'test_*.py'`: **20 passed**.
  Installer download/build operations are mocked, not clean-machine installations.
- `cargo fmt --all -- --check`, `bash -n install.sh` and `git diff --check`: passed.
- en-US, pt-BR and es catalogs have **147 identical, unique message keys**.
- All **152** third-party Cargo.lock entries match the bundled license inventory
  and checksums. The 134 existing entries are unchanged; 18 versions were added.
- Inspected screenshots of the Markdown preview in light/dark themes and the
  restored editor/toolbar. Existing graphical regression suites also passed.

## 1.8.0-alpha coverage

- Toolbar and menu toggle the same saved global view-only state. Both enabled and
  disabled preferences survive recreation; restored dirty Markdown opens in preview.
- Markdown replaces the editing area/minimap only for a final `.md` extension,
  case-insensitively. The source text, caret, undo history and dirty state survive
  toggling. Non-Markdown text remains visible and selectable in view-only mode.
- Tables, code highlighting, explicit heading anchors, local images, remote-image
  placeholders, light/dark themes and zoom. Internal links scroll to the target;
  selected rendered text is copied through the application action.
- Escaping of user markup and permitted URL schemes; local-image path boundaries,
  including percent-encoded paths and attempts to escape the document directory.
- Real keyboard typing, Backspace, Return, Tab, Cut/Paste and Undo/Redo cannot edit
  read-only text. Sorting, case conversion and replacement are blocked. Saving
  existing unsaved changes does not apply trailing-whitespace cleanup in this mode.
- Accent compatibility applies to existing/new tabs, the search field and a later
  dialog. Real X11 dead-key events produced **á**, **õ** and **é**. Default restores
  the original GTK setting and removes widget overrides without modifying text.
- Explorer duplication copies on-disk bytes without changing an unsaved source tab.
  Filesystem tests cover numbered collisions, Unicode names, file extensions,
  dotted folder names, recursive copies, symlink refusal and failed-copy cleanup.
  Four concurrent copies obtain distinct names without replacing another copy.

## Existing regression coverage

Document/filesystem errors, atomic saves, overwrite/conflict protection, autosave,
recovery, unsaved prompts, tabs/reordering, native indentation/minimap/case
conversion, Find/Replace, Focus, syntax highlighting, image/PDF viewing, Save All,
resize/maximize/restore, 1.6 editing/sorting/navigation, 1.7 tab icons/title widths,
bracket styling and native word completion passed.

## Findings corrected during validation

- GIO's preflight/no-overwrite move was insufficient under concurrent publication.
  Native no-replace rename operations now make the publication atomic; the
  previously failing concurrent-copy test passes.
- Unparenting the source view while entering preview invalidated GtkSourceMap
  adjustment handlers. The source view now stays parented in a hidden editor area,
  with a separate preview scroller. Final transitions pass with fatal criticals.
- `simple` works as an environment/GtkSettings alias but not as the explicit GTK
  widget context ID. Widgets now use `gtk-im-context-simple`; the previously
  failing dead-key checks pass with actual keyboard events.

## Known limitations and unverified platforms

- No native Windows compile/run, native Wayland/COSMIC session or Japanese IME
  (IBus/Fcitx/Windows IME) validation was available. Restoring default properties
  is tested; that does not prove every OS, compositor or input-service combination.
- Rust 1.89 is the declared minimum required by Comrak; this run used Rust 1.99.1.
  No release/LTO build, clean-machine installer run or PowerShell execution was
  performed. Both installers were updated for the minimum compiler requirement;
  there is no new native runtime dependency.
- GTK 4.24 warns that the existing dark-theme setting should use its newer
  color-scheme property. The existing Find/Focus test also emits a GtkBox width
  measurement warning. Neither caused an assertion failure; no toolkit-wide
  warning-free claim is made.
- Preview uses native GTK widgets, not a browser. Raw HTML is literal; arbitrary
  HTML/CSS/JS, remote images, SVG, math/Mermaid and relative document navigation
  are not rendered. Local PNG/JPEG images are restricted to the document's
  directory tree and appear below their paragraph. Selection is per block.
  Very large Markdown documents render synchronously and can pause the UI.
- Duplication uses saved disk contents. Symbolic links and special files are
  refused. Filesystems without native no-replace rename support fail safely;
  existing cross-volume moves still have no copy/delete fallback.
- The 1.7 diagnostic concerning immediate native-view finalization was not rerun.
  Functional tab removal passes; long-running memory growth and toolkit-held
  references remain unverified.
- Existing Windows live-resize artifacts remain subject to the native-frame,
  Cairo and DirectComposition mitigation described in RESIZE.md. Existing
  completion indexing delays and sorting/trim limits remain documented in README.md.

## Repeating the checks

Use a desktop session even for the default test set. Run ignored graphical suites
separately to keep GTK on one test thread. Keyboard tests require Linux/X11,
xdotool and a window manager; the 1.8 composition checks need `setxkbmap br`.
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
cargo test --locked version_180_preview_read_only_input_and_duplicate -- --ignored --test-threads=1
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all -- --check
bash -n install.sh
```

## Ctrl+L follow-up (same version)

Added Ctrl+L through the shared shortcut table and updated the three translations.
Basic validation: `cargo check --locked --all-targets`, `cargo test --locked --lib`,
formatting and translation-key parity checks passed. The full graphical suites
reported above were run before this shortcut-only follow-up, not repeated for it.
