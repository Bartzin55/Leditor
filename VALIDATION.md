# Validation — Leditor 1.2.0

Date: 2026-10-25. Base: GitHub main at 864dc99498ad35fc1f41be298a6ca96a2d024c76, preserving the user's latest README edit.

- Linux release compilation: passed.
- Clippy with all targets and warnings denied: passed.
- Formatting: passed.
- Non-graphical tests: 26 passed (12 document/localization, 9 filesystem, 4 session, 1 Unicode case conversion).
- GTK integration tests: compile, but cannot execute here. Xvfb failed to create listening sockets; no functional display is available.
- Visual runtime inspection of 1.2.0: not performed. Both supplied mockups were inspected as implementation references.
- Windows build/runtime: not performed.

The new catalog test checks key parity and resolves every message with Fluent variables in all three languages. Unicode case conversion verifies accented Portuguese/Spanish and uppercase expansions. The compiled GTK integration suite now includes caret-word transformations, undo, line insertion, footer pending/saving states, and preference restoration. Those graphical assertions remain unexecuted; a passing non-graphical suite is not evidence that every UI behavior has been verified.

Manual local checks are still needed for footer layout at narrow widths and high DPI, light/dark rendering, menu controls, all three languages after restart, native dialogs, keyboard input, autosave indications, and preference persistence. Very narrow windows may require more space for all footer fields. Counts use Unicode scalar values and format detection uses extensions, as documented in DEVELOPMENT.md.

No claim of bug-free operation. The source ZIP includes no Windows executable. No changes were pushed to GitHub.

## Language selector revision — 2026-10-26

Replaced the three buttons with grouped GTK CheckButtons (radio-style exclusive selection). The configured language is selected before attaching persistence callbacks; only activation of a choice saves a preference. A separate localized label identifies the running interface language, which may differ until restart. Updated the catalog test arguments for the new placeholder.

This small revision was inspected statically and catalog keys/placeholders were checked. The Rust toolchain is unavailable in the current workspace runtime, so compilation and graphical testing were not repeated. Results above refer to the original 1.2.0 build, not this revision. No GitHub push was made.
