# Validation — Leditor 1.1.0

Date: **2026-10-24**. Updated from the existing Leditor 1.0.0 source.

## Results

| Check | Result |
|---|---|
| Formatting | Passed |
| Clippy, all targets, warnings denied | Passed |
| Linux release build | Passed |
| Document tests | 11 passed |
| Filesystem-operation tests | 9 passed |
| Session recovery tests | 4 passed |
| GTK integration test compilation | Passed |
| Full test suite execution | **Not passed: GTK integration test cannot initialize GTK without a display** |
| Visual inspection / physical keyboard and mouse interaction | Not performed for 1.1.0 |
| Windows compilation and execution | Not performed for 1.1.0 |

There are **24 passing non-graphical tests**. Do not interpret this as a passing full suite. The GTK test failed at `gtk::init()`, before its behavioral assertions could run. Xvfb could not establish listening sockets in this environment, so it could not provide a virtual display.

Rust 1.99.1 and GTK 4.14 development libraries were used. Missing toolchain files and local library links were repaired before successful compilation. Initial compile errors were corrected before the final checks.

## Verified behavior

The non-graphical tests cover the existing file and explorer safeguards plus successful autosave, external-edit/missing-file refusal, keeping new edits dirty while accepting an earlier saved snapshot, UTF-8 BOM/CRLF recovery, untitled text round-trips, snapshot replacement, clean-file reload, conflicting-file detachment, malformed-data rejection, and failed snapshot writes preserving existing bytes.

The expanded GTK test includes existing behaviors and new assertions for grouped indentation undo/redo, numeric zoom limits and reset, snap-to-100%, autosave debounce, restore dependencies, untitled recovery, active-tab/folder restoration, and conflict notices. These assertions are **compiled but unexecuted** in this environment.

## Required platform verification

Run the full suite in a working Linux desktop. Manually verify the custom menu fields, tab widths at different zoom levels, line-highlight alignment during wrapping and scrolling, Ctrl+wheel/trackpad behavior, native file dialogs, and the About browser link. Repeat runtime checks on Windows, including GTK dependencies and default-browser launch.

No power-loss simulation, storage exhaustion, concurrent-writer stress test, or large-session performance benchmark was performed. Recovery has the debounce and concurrency limitations described in DEVELOPMENT.md. No claim of bug-free operation is made.
