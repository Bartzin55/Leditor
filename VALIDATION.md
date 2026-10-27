# Validation — Leditor 1.4.0

Date: 2026-10-26. Base: GitHub main at 6916fdb797281d55a042ded7db921e422567759c, checked before implementation. No push made.

## Passed

- Linux `cargo check --all-targets` and release build.
- Formatting and Clippy with `--all-targets -- -D warnings`.
- 22 library/integration tests: preference migration, document/localization, filesystem safety.
- 10 non-graphical binary tests: installation, session recovery, Unicode conversion/indentation and minimap representation.
- Existing GTK regression suite: passed, including tabs/history/layout/dialogs/file safety and previous-version behaviors.
- New GTK suite: passed, including Unicode case-insensitive search, case reset, next/previous/wrap navigation, single/all replacement, undo/redo, absent/empty/multiline terms, replacement containing the search term, tab switching, minimap click, Focus visibility/top-edge reveal/restoration and settings isolation.
- Total: **34 passing tests** across separate invocations.
- Screenshots rendered and visually inspected in dark and light themes. Both attached mockups were used as references for the search bar.

## Environment and limits

Rust 1.99.1, Linux x64, GTK 4.14. Graphical tests ran under Xvfb over TCP with the Cairo renderer and `G_DEBUG=fatal-criticals`. Initial separate-process display setup failed; launching Xvfb and the test in the same execution environment resolved connectivity. An initial widget-parenting integration issue was found and fixed; the final graphical suite passed without GTK criticals. EGL reported lack of DRI3 acceleration; hardware-accelerated rendering was not tested.

Windows build/runtime and native Wayland runtime: not tested for 1.4.0. The Windows implementation was reviewed against the native binding signatures. The previously delivered Windows 1.3.0 executable does not contain these changes.

Always-on-top stacking and fullscreen window-manager behavior require real desktop testing: Xvfb alone has no desktop window manager. Wayland's Always on top option is intentionally disabled; compositor-specific pinning can be used externally. Focus fullscreen requests, mouse-edge handlers and widget restoration are covered, but a real compositor must verify the resulting fullscreen/stacking state.

Keyboard accelerators/controllers are registered; search navigation and menu actions are exercised programmatically. Physical keyboard/IME, touch, mixed-DPI, extreme file sizes, and compositor-specific behavior still need local checks. The minimap samples dense documents and shows the first 240 visual columns, with clicks targeting logical line starts. Search refresh is synchronous, so unusually large files with many matches can take longer.

No claim of bug-free operation. The deliverable is a complete source ZIP with manifests, lockfile, assets and documentation, not a Windows binary package.
