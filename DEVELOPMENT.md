# Development Notes — 1.1.0

This release updates the existing 1.0.0 Rust/GTK4 project. No dependency was added.

## Structure

- `document.rs`: UTF-8 file I/O, atomic replacement, dirty-state tracking, autosave conflict checks, and recovery serialization.
- `file_ops.rs`: existing explorer filesystem safety rules.
- `editor_view.rs`: GTK TextView subclass, logical line gutter, caret highlight, indentation, and editor font scaling.
- `tab.rs`: independent document buffers, tab headers, and unsupported-file display.
- `ui.rs`: application controller, menus, action state, document lifecycle.
- `ui/explorer.rs`, `ui/tab_actions.rs`: explorer operations and tab context menus/reordering.
- `ui/preferences.rs`: autosave scheduling, session lifecycle, zoom, and numeric controls.
- `session.rs`: versioned recovery files, atomic persistence, and conflict-aware restoration.
- `popups.rs`, `shortcuts.rs`, `locales/en-US.ftl`: About, shortcut reference, and English strings.

## Autosave and recovery

Both switches default to off; their choices persist. A 600 ms debounce (checked every 100 ms) follows edits and workspace changes. Named-file writes run on a worker; operations that could change paths are serialized with saving. Failed tabs stop automatic retry until a successful manual save or an autosave toggle. Errors appear in a visible message without discarding the buffer.

Recovery snapshots are written before named-file autosave and after it completes. They contain tab text, saved baselines, names, paths, BOM/newline flags, caret positions, tab order, active tab, and the explorer root. Temporary files are flushed and replaced atomically. Snapshots are not encrypted. They reside under GLib's per-user data directory in `leditor/settings.ini` and `leditor/session.ini` (typically `~/.local/share/leditor` on Linux and the local application-data directory on Windows).

With restoration enabled, quitting first preserves a snapshot, flushes eligible pending named-file writes, and records the final session. Untitled drafts and failed saves remain recoverable without a Save As dialog. Explicitly closing an individual dirty tab retains the existing Save/Don't Save/Cancel behavior. With restoration disabled, quitting retains the existing unsaved-change prompts.

On restart, clean named files reload from disk. Dirty recovered text retains its path only when its saved baseline matches the current file; otherwise it becomes an independent unsaved tab. Missing text files also recover as unsaved documents. Unsupported tabs retain their non-editable display. Corrupt recovery data disables automatic writes for that launch and reports the problem instead of immediately overwriting it.

Conflict checks are best-effort: another process could change a file between checking and replacing it. Recovery is not a substitute for backups. The last debounce interval may be lost after a crash; orderly quit flushes the recovery snapshot. Snapshot serialization/writes currently happen on the main thread and may briefly pause very large sessions. Folder expansion, scroll offsets, undo history, and non-autosave preferences are not persisted.

## Editing and presentation

Tab inserts a literal tab by default. Convert Tabs to Spaces changes future insertions; it does not rewrite existing text. Number of spaces defaults to 4 and controls both inserted spaces and literal-tab display stops. Selected logical lines can be indented/outdented in one undo action; a selection ending at the next line's column zero excludes that line. Outdent only removes leading spaces or a leading literal tab.

Zoom defaults to 100%, clamps to 5–800%, and changes only the TextView font and gutter metrics. Ctrl++ / Ctrl+- change it by five percentage points; Ctrl+= resets it. Ctrl+scroll supports fractional deltas and pauses for 220 ms when crossing 100%. The existing line counter stays on the left. The current line means the text caret's logical line, including its wrapped visual rows, rather than the line merely hovered by the mouse.

## Existing limitations retained

Only UTF-8 text is editable. Explorer changes made outside Leditor require Refresh. Cross-volume moves and copies containing symbolic links retain the previous restrictions. No syntax highlighting, LSP, extra file viewers, or new language was added.
