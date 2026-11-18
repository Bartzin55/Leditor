# Validation — Leditor 1.9.2-alpha

Linux/X11 basic checks passed with GTK 4.24.1: Cargo compilation of all targets,
formatting, the file safety/UI test, and the targeted native mouse/keyboard
test `explorer_pointer_shortcuts_and_clicks`. The latter exercises folder
expansion, single-click selection, double-click open, Ctrl+C with editor focus,
and F2 on both sides of the Explorer boundary. Native Windows and Wayland were
not tested for this patch. The full test suite was not rerun.

## Previous validation — Leditor 1.9.1-alpha

Date: 2026-11-17. Based on published 1.9.0-alpha commit
`edfb47fc1d5c53fbc7e5fc07d6a31d1e14a1cd1f`. No push performed.

## Environment and results

Linux x86_64; Rust/Cargo 1.100.0, GTK 4.24.1, GtkSourceView 5.18.0,
libsoup 3.8.1. Graphical tests used Xvfb/X11, Openbox, xdotool, the Cairo
renderer and a Brazilian keyboard map. Dependencies were installed in a private
build environment; this is not a clean-machine installer validation.

| Check | Result |
| --- | --- |
| `cargo check --locked --all-targets` | Passed |
| `cargo build --locked` (development profile) | Passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo test --locked`, default parallel runner | 42 passed, 0 failed, 12 ignored |
| Ten optional graphical test cases, run individually | 10 passed, 0 failed |
| Python installer/bootstrap tests | 21 passed (mocked installations) |
| `bash -n install.sh`, `git diff --check` | Passed |
| Translation catalogs | 159 matching keys in en-US, pt-BR and es |

There are **52 distinct passing Rust tests**, not counting the isolated child
process's repeated result line as another test. The default UI test remains
part of the ordinary test run; it has not been hidden or marked ignored.

## Image regression and graphical coverage

The regression `images_keep_height_with_text_zoom_and_narrow_windows` uses a
real PNG in a real GTK scroller. It checks that adding/removing 100 lines below
an image preserves its allocation, that 50%/100% zoom scales its height, and
that a narrow window preserves the aspect ratio without collapsing the image.
The same regression was run against the original renderer, where adding text
collapsed the image to zero, and against the corrected renderer, where it passed.
Local and downloaded images use the same sizing widget and zoom state.

These optional cases were executed separately:

- `images_keep_height_with_text_zoom_and_narrow_windows`
- `source_engine_and_tab_navigation`
- `version_160_editing_and_navigation`
- `find_replace_focus_and_minimap`
- `version_170_tabs_and_bracket_styles`
- `version_170_native_completion`
- `version_180_preview_read_only_input_and_duplicate`
- `version_190_clipboard_project_network_and_shortcuts`
- `resize_maximize_and_focus_preserve_editor`
- `native_keyboard_and_map_input`

The default UI test also checks both clipboard path actions and current GTK
color-scheme properties. File-operation tests cover root `/`, nested Unicode
names, spaces, outside-root rejection, parent traversal and visible symlink
names. Light/dark tab screenshots were inspected. The 1.9.0 link-menu test now
scrolls the link into view before a native right-click: retaining the image's
height correctly puts that link below the initial viewport.

## Warnings and limits

The default test run emitted no deprecated `gtk-interface-color-scheme`
recommendation. The optional runs still emitted native GTK/GtkSourceView
messages in this environment:

- `gtk_adjustment_set_value: assertion 'isfinite (value)' failed` during the
  1.9.0 graphical suite (the existing native minimap path).
- `Trying to measure GtkBox ... but it needs at least ...` in constrained layouts.
- `gdk_frame_timings_submitted() called on submitted frame` in some frame updates.

These messages were not suppressed; assertions and test processes completed
successfully. This is not a claim of warning-free operation. Running with
`G_DEBUG=fatal-warnings` may abort on these remaining native messages.

Not validated here: native Windows, native Wayland/COSMIC, GTK 4.12–4.18 fallback,
physical touchpad gestures, the release build profile, or real clean-machine
Linux/Windows installer runs. The optional PDFium/media suite and the public
HTTPS smoke test were not rerun for this update. Network coverage here used a
local HTTP server and the existing image/redirect/error/size-limit tests.

## Reproduction commands

From a graphical session with the dependencies in BUILD.md installed:

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo build --locked
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py'
```

On headless Linux, the default suite can use `xvfb-run -a cargo test --locked`.
For an individual optional test, start a window manager in the test display
and install xdotool where required, then run, for example:

```sh
cargo test --locked --bin leditor images_keep_height_with_text_zoom_and_narrow_windows -- --ignored
cargo test --locked --bin leditor version_190_clipboard_project_network_and_shortcuts -- --ignored
```

GTK tests automatically isolate themselves in child processes. Mouse/keyboard
suites should still run one at a time because display focus is shared. No
`--test-threads=1` workaround is needed by the default suite. See BUILD.md for
Linux/Windows installation and DEVELOPMENT.md for implementation decisions.
