# Development Notes — 1.4.0

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

Recovery snapshots are written before named-file autosave and after it completes. They contain tab text, saved baselines, names, paths, BOM/newline flags, caret positions, tab order, active tab, and the explorer root. Temporary files are flushed and replaced atomically. Snapshots are not encrypted. Recovery resides under GLib's per-user data directory in `leditor/session.ini`; preferences now use the user config directory in `leditor/settings.ini` (typically `~/.local/share/leditor` on Linux and the local application-data directory on Windows).

With restoration enabled, quitting first preserves a snapshot, flushes eligible pending named-file writes, and records the final session. Untitled drafts and failed saves remain recoverable without a Save As dialog. Explicitly closing an individual dirty tab retains the existing Save/Don't Save/Cancel behavior. With restoration disabled, quitting retains the existing unsaved-change prompts.

On restart, clean named files reload from disk. Dirty recovered text retains its path only when its saved baseline matches the current file; otherwise it becomes an independent unsaved tab. Missing text files also recover as unsaved documents. Unsupported tabs retain their non-editable display. Corrupt recovery data disables automatic writes for that launch and reports the problem instead of immediately overwriting it.

Conflict checks are best-effort: another process could change a file between checking and replacing it. Recovery is not a substitute for backups. The last debounce interval may be lost after a crash; orderly quit flushes the recovery snapshot. Snapshot serialization/writes currently happen on the main thread and may briefly pause very large sessions. Folder expansion, scroll offsets, undo history, are not persisted.

## Editing and presentation

Tab inserts a literal tab by default. Convert Tabs to Spaces changes future insertions; it does not rewrite existing text. Number of spaces defaults to 4 and controls both inserted spaces and literal-tab display stops. Selected logical lines can be indented/outdented in one undo action; a selection ending at the next line's column zero excludes that line. Outdent only removes leading spaces or a leading literal tab.

Zoom defaults to 100%, clamps to 5–800%, and changes only the TextView font and gutter metrics. Ctrl++ / Ctrl+- change it by five percentage points; Ctrl+= resets it. Ctrl+scroll supports fractional deltas and pauses for 220 ms when crossing 100%. The existing line counter stays on the left. The current line means the text caret's logical line, including its wrapped visual rows, rather than the line merely hovered by the mouse.

## Existing limitations retained

Only UTF-8 text is editable. Explorer changes made outside Leditor require Refresh. Cross-volume moves and copies containing symbolic links retain the previous restrictions. No syntax highlighting, LSP, extra file viewers, or new language was added.


## Version 1.2.0 decisions

The base is upstream commit 864dc99498ad35fc1f41be298a6ca96a2d024c76. No remote writes were made.

The footer uses a CenterBox independent of editor zoom. Counts and columns use Unicode scalar values (not UTF-8 bytes or grapheme clusters); columns are one-based and literal tabs count as one character. Lines are logical, so wrapping does not increase their count. Empty documents have one logical line. Unsupported files display unavailable text metrics. Format detection uses the filename extension and does not enable syntax highlighting.

Case conversion preserves unrelated text and forms one undo operation. First-letter modes uppercase the relevant letters without lowercasing the remainder. A word consists of alphanumeric characters and underscores. Insert below follows Enter by splitting at the caret and replacing selected text; insert above inserts at the current logical line's start and places the caret on the new line.

All exposed preferences are stored atomically in settings.ini: language, light/dark theme, wrapping, line numbers, tab conversion/width, zoom, explorer visibility/width, autosave and restore. Language changes take effect on restart; catalog messages are embedded with Fluent. Native GTK dialog labels follow the platform locale, and operating-system error details may remain in the operating system's language. Sidebar visibility overrides the automatic reveal that would otherwise happen during session restoration. New configurations hide it by default.

No new dependency was added. Light-theme CSS overrides the existing dark stylesheet; current-line shading uses transparency. Manual file writes use a worker to permit the saving spinner to animate. The UI remains locked against edits during a manual save, preserving its snapshot.


## Version 1.3.0 decisions

- `linux_install.rs` is compiled only on Linux. It uses existing tempfile and std APIs; it does not execute shell commands or request privilege escalation. Binary replacement is atomic; command/desktop conflicts fail safely. A stable copy avoids launchers pointing to a deleted Cargo target directory. No installer is run at build time or on Windows.
- `config.rs` shares preference loading with localization. New settings win; otherwise the legacy data-directory file is read. The first normal save writes the new location, retaining the old file. Session snapshots are unaffected.
- `NumberedView::insert_newline` handles ordinary Enter and the menu action. It preserves the exact leading whitespace before the split position, so splitting inside indentation does not duplicate whitespace from the suffix. Paste is unaffected. A selection is deleted and the insertion performed inside one GTK undo group. Shift+Enter retains its existing insert-above behavior.
- Guides use visible logical lines only and GTK text geometry. They show complete indentation levels in leading whitespace, including mixed tabs/spaces. Blank lines now continue the deepest indentation of their nearest nonblank neighbors. When indentation wraps, guides are confined to the first visual row rather than crossing wrapped text. Guides default off.
- Theme is a string-valued GAction, providing mutually exclusive native menu indicators. System default selects the existing dark/light CSS, as described below. Explicit dark/light retain the existing visual identity. System color-scheme integration depends on the desktop and GTK version.
- No new keyboard shortcuts: Enter is now listed in the shortcuts dialog to document the affected newline operation. Existing shortcuts remain.
- No dependency added or license changed. Native GTK distribution obligations still apply when bundling GTK binaries.

References: [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry/latest-single/) and [Gtk.Settings.reset_property](https://docs.gtk.org/gtk4/method.Settings.reset_property.html).

## Version 1.4.0 decisions

- `minimap.rs` adds a GTK DrawingArea next to each tab's ScrolledWindow. A cached 240-column bitmap represents non-whitespace text; rendering samples at most one logical line per vertical pixel for large files. The map covers the whole document, marks the visible line range and navigates to line starts. Lines wrap only in the main editor. Colors come from the text widget and existing theme CSS. The map is a structural text overview, not syntax highlighting.
- `ui/search.rs` uses GTK TextIter forward_search, including native Unicode case-insensitive matching, rather than implementing a second search engine. Two TextTags provide yellow and green highlighting without modifying document contents/dirty state. Searches are literal, non-overlapping and local to the active tab. Replacements are applied in reverse offset order within a single user action, preventing inserted text from being searched/replaced again in the same operation. Buffer changes, undo/redo and tab switches refresh matches.
- The bar matches the supplied two-row mockup: Find and arrows above Replace and replacement buttons, followed by status, case sensitivity and a close control. Opening the bar resets case sensitivity and focuses Find; Ctrl+F toggles it. Navigation keeps focus in the search box. Case, query and replacement are not persisted.
- `ui/focus.rs` snapshots explorer visibility, sidebar width and fullscreen state. Line-number preference stays unchanged; only its presentation is suppressed. Settings writes during Focus use the snapshot, so closing the application in Focus cannot persist a hidden explorer by accident. Menu/tab reveal is tied to top-edge pointer motion. The existing fullscreen state is restored on exit.
- No document format, file-saving, recovery or Linux installation behavior was changed.


## Version 1.4.1 decisions

- `ui/theme.rs` separates the stored selection (`system`, `light`, `dark`) from the resolved OS preference. GTK 4.20's runtime color-scheme property is observed without overriding it. Linux uses the desktop Settings portal and change signal, falling back to GTK theme information. Windows reads AppsUseLightTheme and checks every two seconds. Manual choices are preserved when the system changes; missing appearance information falls back to light.
- `system.css` and `window_mode.rs` were removed. Existing `style.css` and `light.css` remain. No Qt code is included.
- The search bar removes the Insert Above accelerator while open and captures Shift+Enter at the window. The action is also disabled/guarded; closing restores its accelerator. This prevents shortcut ordering from modifying the document.
- Guide depths are cached when text or indentation width changes, with a linear two-pass calculation for blank lines. Rendering uses GTK snapshots and monospace metrics: half-character offset, 3px dash/3px gap. Guides never insert spaces and stay on the first visual row when a logical line wraps.
- `app_icon.rs`, `assets/leditor.gresource.xml` and `build.rs` embed the supplied PNG. The ICO is a format conversion of that artwork, used for Windows resources. Linux icon writes use the existing atomic file writer. No new Rust dependency was added; unused platform bindings were removed.
- Source remains MIT; native GTK/library licensing obligations remain applicable to binary redistribution.

Reference: https://docs.gtk.org/gtk4/property.Settings.gtk-interface-color-scheme.html
