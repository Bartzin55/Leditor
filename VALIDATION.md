# Validation — Leditor 1.9.0-alpha

Validated on 2026-11-15, based on published main commit
`afeff3d9e9f9fe8d62d5bbef9854afd092a5ca58` (1.8.0-alpha, including Ctrl+L).
No push, tag or release was performed for this update.

## Environment

Linux x86_64; Rust/Cargo 1.100.0; GTK 4.24.1; GtkSourceView 5.18.0;
GLib 2.90.1; libsoup 3.8.1; GLib networking 2.80.0 with the OpenSSL TLS backend;
PDFium 7881. GUI checks used Xvfb 21.1.7, Openbox 3.6.1, a Brazilian keymap,
xdotool and GTK's Cairo renderer. Native dependencies were supplied by a private
conda-forge environment, with X11 tools from Debian packages.

## Results

| Check | Result |
| --- | --- |
| `cargo check --locked --all-targets` | Passed |
| `cargo build --locked` | Passed, development profile |
| `cargo test --locked -- --test-threads=1` under X11 | 41 passed; 11 deliberately ignored |
| Ten GUI suites, each run in a separate process | 10 passed |
| Total distinct passing Rust tests | 51 |
| `python3 -m unittest discover -s tests -p 'test_*.py'` | 21 passed |
| `cargo fmt --all -- --check` | Passed |
| `bash -n install.sh` | Passed |
| `git diff --check` | Passed |
| Translation catalog consistency | 158 identical, unique keys in en-US, pt-BR and es |
| Rust dependency license inventory | All 154 locked third-party packages match inventory/checksums |
| Optional external `https_image_smoke` | Failed: environment DNS could not resolve raw.githubusercontent.com |

The optional HTTPS test successfully detected the platform TLS backend, but did
not download an external image: **no successful public HTTPS request is claimed**.
It remains ignored by default and can be rerun in an environment with internet
access. Local HTTP requests and real image decoding/rendering were exercised.

Installer tests use mocks for installation/network/administrator operations;
these results do not mean clean-machine installations were performed. A release
profile build, Windows compilation/runtime and physical touchpad input were not
validated in this environment.

## New-feature coverage

- Actual X11 Ctrl+V and the native paste signal insert a Markdown reference and
  create a PNG beside the document. Repeated pastes create distinct files.
  Undo/redo restores the insertion; undo retains the PNG. View-only blocks paste.
- Untitled Markdown requests Save As; cancellation preserves the unsaved buffer
  without creating an image. A successful save permits image insertion.
- Project-root `/` and Windows-style `\` paths, document-relative `../`, percent
  decoding, out-of-project traversal refusal and symlink escape refusal.
- Local image rendering and canonical local file links in the preview. An actual
  right-click opens the translated Open with menu. The OS's external application
  launch and app chooser completion were not exercised.
- A local HTTP server supplies real PNG bytes. The preview displays downloaded
  images; disabled internet starts zero requests and shows a browser link.
  Disabling it again removes the downloaded preview and the preference survives
  editor recreation. Unit tests cover redirects, HTTP 404, the download-size cap,
  non-HTTP refusal and rejection of unsupported image bytes.
- Actual X11 Ctrl+M toggles the minimap; F12 renames the active file. Untitled F12
  uses Save As, including cancellation. Shared accelerators/dialog entries checked.
- Synthetic native GtkGestureZoom begin/scale/end signals verify continuous zoom
  (100% → 112.5%) and persistence. This does not prove delivery of physical gestures
  by every Windows/Wayland driver or compositor.
- Screenshots inspected for the updated preview; existing media, text, completion,
  theme, focus, search, session, input-method and file-safety suites were retained.
- Windows packaging test verifies that dynamically loaded GIO TLS modules, their
  indirect DLL dependencies and the CA bundle are copied into the installation.

## Failures investigated and remaining limits

- The first resize run failed because the test window manager had mismatched
  libraries; a second setup lacked its theme path. With matching Openbox libraries
  and data paths, resize/maximize/fullscreen tests passed. No application resize
  behavior was changed to accommodate the environment.
- Rebuilding a focused Markdown link initially caused a GTK focus warning. Focus
  is now moved to the retained preview scroller before replacing its children;
  the final new-feature test no longer emits that warning.
- **Remaining native minimap warning:** GtkSourceView 5.18.0 can emit
  `gtk_adjustment_set_value: assertion 'isfinite (value)' failed` when the minimap
  is revealed for a short document. The captured stack reaches
  `gtk_source_map_do_update`; this version's upstream implementation divides by
  `upper - page_size` without guarding a zero range. It was not patched or
  suppressed in Leditor. The test's text, history, zoom and shortcut assertions
  passed, but a run with fatal GTK criticals would fail on this warning.
  Upstream source: https://github.com/GNOME/gtksourceview/blob/5.18.0/gtksourceview/gtksourcemap.c
- GTK 4.24 also printed its existing color-scheme deprecation warning; some
  screenshot/input runs printed a frame-timing warning. No claim of warning-free
  operation is made.
- Markdown images remain PNG/JPEG. SVG, GIF/WebP, arbitrary HTML/CSS, math and
  Mermaid rendering are not added. Remote images are limited to 16 MiB each and
  decoded to at most 900×600; slow/offline/invalid resources display a fallback.
- No persistent remote-image cache; reopening/rebuilding a preview may download
  its images again. Large Markdown AST rendering remains synchronous.
- Windows, Wayland/COSMIC, real touchpad gestures, system file associations and
  native app-chooser behavior require platform testing. The pre-existing Windows
  live-resize limitation remains documented in RESIZE.md.

## Reproduction

Install the dependencies described in BUILD.md. Use an isolated configuration
and a graphical desktop for UI tests; GTK tests must run on one thread and each
ignored graphical suite in its own process. X11 keyboard tests need xdotool and
an active window manager. The media test needs PDFium 7881 via
`LEDITOR_PDFIUM_LIBRARY` or the normal runtime search path.

```sh
cargo check --locked --all-targets
cargo build --locked
cargo test --locked -- --test-threads=1
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all -- --check
bash -n install.sh

for test in \
  version_150_syntax_media_and_save_all \
  source_engine_and_tab_navigation \
  native_keyboard_and_map_input \
  version_160_editing_and_navigation \
  resize_maximize_and_focus_preserve_editor \
  find_replace_focus_and_minimap \
  version_170_tabs_and_bracket_styles \
  version_170_native_completion \
  version_180_preview_read_only_input_and_duplicate \
  version_190_clipboard_project_network_and_shortcuts
do
  cargo test --locked "$test" -- --ignored --test-threads=1
done

# Optional public HTTPS check; needs working DNS, internet, TLS modules and CAs.
cargo test --locked https_image_smoke -- --ignored --test-threads=1
```
