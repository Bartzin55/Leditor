# Validation — Leditor 1.3.0

Date: 2026-10-26. Base: GitHub main at bda3ee3ea8d3f7f0f53dfd361372e82e5f87a5e9, fetched before editing. No newer upstream commits were present.

## Completed

- `cargo build --locked --release`: passed (Linux, Rust 1.99.1, GTK 4.14 development/runtime libraries).

- `cargo check --locked --all-targets`: passed.
- `cargo fmt` and `cargo clippy --locked --all-targets -- -D warnings`: passed on the final Rust changes.
- `cargo test --locked --lib --test document --test file_ops`: 22 passed (1 preference migration, 12 document/localization, 9 filesystem).
- `cargo test --locked --bin leditor -- --skip ui::tests --test-threads=1`: 9 passed (2 Linux installation/desktop escaping, 4 recovery, 2 indentation, 1 Unicode case conversion). One graphical integration test excluded.
- Total: **31 non-graphical tests passed**.
- Catalog coverage includes matching keys and successful Fluent formatting in English, Portuguese and Spanish.

Initial test linking failed because the scratch GTK development environment had missing library links. Restoring development-library links resolved the failure; the tests above then passed. No project workaround or dependency change was needed.

## Remaining validation

- GTK integration assertions compile, including the new autoindent undo/redo and guide preference assertions, but were **not executed**. Xvfb failed to open Unix/local listening sockets; no working graphical display was available. Theme selection, menu appearance, guide rendering, actual Enter keyboard dispatch and launcher appearance require local graphical testing.
- Windows build and execution: not performed. Automatic installation is gated with `cfg(target_os = "linux")`.
- Linux installation was exercised in isolated temporary directories (initial install, repeat install, executable update, command conflict, spaces/percent in desktop Exec paths). It was not registered in a real desktop session here.
- `~/.local/bin` must be in the user's PATH. The app cannot modify its parent terminal's environment. Global `/usr/bin` installation requires an explicit administrator command; see BUILD.md.
- System theme behavior depends on GTK and desktop integration. Guides cover leading indentation levels and the first visual row of wrapped indentation; blank lines do not infer nesting from neighboring lines.

No claim of bug-free operation. This ZIP contains source, not Windows binaries. No GitHub push was made.
