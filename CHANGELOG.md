# Release 1.5.1 — 2026-10-29

- Fixed the Windows installer's redundant `.leditor-managed` write after tests;
  existing markers are now preserved, as they already were on Linux.
- Added post-test preparation and temporary-file cleanup messages on both
  platforms, including the Linux Python bootstrap. Installation errors are
  displayed before the potentially lengthy temporary-workspace cleanup.
- Added Linux and Windows installation/update scripts with user/system scope,
  temporary missing toolchains, native GTK/GtkSourceView provisioning and a
  checksum-verified PDFium 7881 runtime.
- Added uninstall scripts with confirmation and a separate opt-in to delete
  preferences and recovered documents. Existing tools and other users are kept.
- Added LICENSES with exact Cargo dependency legal texts and native-library
  notices; installers retain notices for the runtime they actually install.
- Script-managed Linux launches bypass the legacy automatic binary copy, keeping
  the private runtime and launcher together.
- Updated build/manual fallback documentation and version metadata. Keyboard
  shortcuts and UI translations are unchanged; this release adds no GUI actions.
- See VALIDATION.md for checks performed and platform limitations.

# Release 1.5.0 — 2026-10-28

- Integrated GtkSourceView 5 syntax highlighting, automatic extension/content-type detection, per-tab radio language selection and persisted enable preference.
- Added read-only PDF tabs using PDFium and PNG/JPG/JPEG tabs using GTK/GdkPixbuf. Existing zoom controls work with both; media footers contain only zoom and omit minimap/line numbers.
- PDF rasterization runs in one background worker with lazy visible-page rendering and a bounded cache. Unsupported/corrupt/password-protected documents report errors without modifying the source file.
- Added sequential Save all, including Save as for untitled documents; stops safely on cancellation or failure and skips viewers.
- Reorganized top menus into File / Edit / View / Window / Other, including an indentation submenu.
- Retained GTK editing, autosave, recovery, themes, minimap, focus and find/replace. Session recovery reopens media paths as read-only tabs.
- Updated About metadata, existing zoom shortcut descriptions, EN/PT-BR/ES catalogs and Linux/Windows build documentation. No new keyboard binding was introduced.
- Added sourceview5 (MIT), pdfium-render (MIT OR Apache-2.0); native GtkSourceView is LGPL-2.1-or-later and PDFium is BSD-style with third-party notices. Leditor remains MIT.

# Release 1.4.1 — 2026-10-27

GTK maintenance release based on 1.4.0. Historical versions and Git tags are unchanged.

### Changes
- Removed Always on top, its menu action, translations and exclusive X11/Win32 bindings.
- Embedded the supplied icon for About, Linux launchers and Windows executable resources.

### Bug fixes
- System default selects the existing light/dark palettes and follows appearance changes, preserving explicit manual choices.
- Ctrl+F shows/hides search; Shift+Enter goes to the previous match while search is visible without inserting an unintended line.
- Indentation guides are dashed and offset from text; blank lines continue adjacent indentation levels.
- Updated shortcuts, English/Portuguese/Spanish catalogs and build instructions.

# Release 1.4.0 — 2026-10-26

- Added an optional clickable minimap following the active theme; its visibility preference is saved.
- Added Ctrl+F / Edit → Find and replace, a two-row bar matching the supplied mockups, yellow/green match tags, wrapping navigation, replacement messages and grouped undo for Replace/Replace all.
- Added per-invocation case sensitivity (never persisted), Find-field Enter/Shift+Enter and Tab navigation between fields.
- Added Shift+F11 Focus mode with top-edge menu/tab reveal and Esc exit, preserving explorer/line-counter preferences and the previous fullscreen state.
- Added Always on top through native Windows and X11 window-manager requests. Native Wayland does not expose a portable GTK equivalent; the option is disabled there. Focus and Always on top never persist.
- Updated all three catalogs, shortcuts, package/About version, documentation and Linux graphical tests.

# Release 1.3.0 — 2026-10-26

- Added Linux-only automatic per-user executable/launcher installation, atomic updates and conflict protection. Optional global installation is documented.
- Moved preferences to the XDG config directory with legacy fallback; recovery stays in the user data directory.
- Added default-enabled automatic indentation with spaces/tabs and one undo operation for newline plus indentation.
- Added optional indentation guides below View → Line counter.
- Replaced the light-theme checkbox with a radio-style Theme submenu: System default, Light theme, Dark theme. Existing explicit light/dark choices are preserved.
- Updated all three Fluent catalogs, shortcuts reference, package metadata and build/development documentation. No dependencies added; MIT license retained.

# Changelog

All notable changes to Leditor from version 0.1.0 through version 1.2.0 are recorded below, newest first. Changes are grouped into **Major Additions**, **Minor Changes**, and **Bug Fixes**.

Dates use `YYYY-MM-DD`. Versions 0.2.0, 0.2.1, and 1.0.0 use the release dates recorded in their README and About metadata. Version 0.1.0 has no explicit release-date field; retained filesystem and archive timestamps span 2026-10-21 and 2026-10-22, so its exact release date remains unconfirmed. Maintenance changes delivered without a version increment are included under their existing version.

## [1.2.0] — 2026-10-25

Pre-publication UI revision, 2026-10-26: replaced language buttons with grouped radio-style choices, preselected the configured language, and added an explicit label identifying the currently running interface language. Restart remains required.

### Major Additions

- Added a footer with logical line and Unicode character counts, caret line/column, pending/saving indicators, UTF-8 encoding, extension-based format, and zoom.
- Added Brazilian Portuguese and Spanish Fluent catalogs and a language selector (applied on restart).
- Persisted all exposed editor preferences, including theme and explorer width/visibility.
- Added case conversion for selections or the word touching the caret.
- Added Insert line above (Shift+Enter) and Insert line below (Enter behavior).

### Minor Changes

- Added light-theme styling alongside the dark theme.
- Hide the explorer by default for new configurations.
- Updated About, shortcut reference, README, build and development documentation.
- Incorporated upstream README changes from commit 864dc99498ad35fc1f41be298a6ca96a2d024c76.

### Bug Fixes

- Moved manual document writes to a worker so the footer can animate while saving.
- Made current-line shading translucent for legibility in both themes.

## [1.1.0] — 2026-10-24

### Major Additions

- Added optional debounced autosave for named files and recovery snapshots for untitled documents.
- Added optional session restoration of the explorer folder, tab order, active tab, text, and caret positions; restoration requires autosave.
- Added editor-only zoom from 5% to 800%, editable in View, with keyboard shortcuts and Ctrl+scroll snapping at 100%.
- Added configurable 1–8-space indentation, literal-tab display width, and undoable Shift+Tab outdent for the current line or selected lines.

### Minor Changes

- Added caret-line shading and a yellow current-line number.
- Added a clickable GitHub repository link in About.
- Updated About, package metadata, the shortcuts reference, and the user-focused README.
- Added separate build, development, and validation documentation.

### Bug Fixes

- No separately reported 1.0.0 defect is claimed as fixed in this release. New autosave safeguards preserve dirty content on write failure and refuse detected external file conflicts.

## [1.0.0] — 2026-10-23

### Major Additions

- Added an integrated file explorer on the left side of the editor, with a hierarchical directory tree, expandable and collapsible folders, and file-type icons.
- Added **File > Open Folder**, using the platform folder selector, and **Close Folder**, which leaves open tabs intact.
- Added **File > New Folder**, allowing the user to select a parent directory, create a folder, and open it as the explorer root.
- Added explorer toolbar buttons for **New File**, **New Folder**, **Refresh**, and **Show/Hide Explorer**.
- Added file opening from the explorer. Already-open files focus their existing tabs instead of creating duplicates.
- Added non-editable tabs for unsupported binary files and text encodings, with a red tab title and a centered **Unsupported file** message. Unsupported entries remain visible in the explorer.
- Added drag-and-drop movement of files and folders within the explorer, with destination highlighting. Dropping onto a file targets its parent directory.
- Added context menus for the explorer background, folders, and files:
  - Background: **New File**, **New Folder**, and **Paste** in the root directory.
  - Folders: **New File**, **New Folder**, **Rename**, **Cut**, **Copy**, **Paste**, **Delete**, and **Copy Path**.
  - Files: **Rename**, **Cut**, **Copy**, **Paste**, **Delete**, and **Copy Path**; Paste targets the file's parent directory.
- Added safe filesystem operations: exclusive creation, name validation, conflict rejection, staged recursive copies, and protection against moving or copying a folder into itself or a descendant.
- Added automatic path updates for open documents affected by a file or folder rename or move, while retaining unsaved buffer contents.
- Added permanent-deletion confirmation. Open editable documents affected by deletion are retained as unsaved tabs without an associated path, preserving their text for Save As.
- Added tab context menus with **Close**, **Close All**, **Save**, **Save As**, **Duplicate**, and **Rename**. Actions target the tab that was right-clicked.
- Added **Close All** handling that checks every unsaved document before removing any tabs. Cancelling a later warning retains all tabs; completed saves remain saved.
- Added independent document duplication into a new `Untitled.txt` tab, without associating it with the original file.
- Added renaming of saved files from their tabs and display-name changes for unsaved tabs. Unsaved names are suggested during the next Save As.
- Added horizontal drag-and-drop tab reordering, with insertion indicators, while preserving document state, undo history, cursor position, and active-tab selection.

### Minor Changes

- Added **Cut**, **Copy**, and **Paste** to the Edit menu.
- Changed Cut and Copy to operate on the entire logical line when no text is selected, including its terminating newline when present. Selected text retains the normal behavior; Paste replaces the selection.
- Kept the native editor context menu and applied the same clipboard behavior to its actions and keyboard shortcuts.
- Added **View > Show/Hide Explorer** and `Ctrl+B`, plus a reveal button when the explorer is hidden.
- Added `F2` for renaming and `Delete` for deleting the focused explorer item, without intercepting normal text editing.
- Made Cut, Copy, and Paste respect explorer focus. File operations use an internal application clipboard; **Copy Path** writes the absolute path to the system clipboard.
- Synchronized the explorer after operations performed by the application. Changes made by external programs can be loaded with Refresh.
- Disabled text saving and duplication for unsupported-file tabs.
- Refactored filesystem operations and explorer/tab interactions into dedicated modules, retaining the existing Rust/GTK codebase and adding no dependencies.
- Updated About to version **1.0.0**, with release date **2026-10-23**, and expanded the Keyboard Shortcuts dialog and English Fluent catalog.
- Added filesystem and GTK integration coverage. The Linux suite contains **19 passing tests**: nine document tests, nine filesystem-operation tests, and one GTK integration test covering multiple workflows.
- Produced a Windows x64 release package containing the executable, GTK runtime dependencies, resources, licenses, source archive, launcher, and compiled document/filesystem test executables.
- Documented operational limits: cross-volume moves requiring copy/delete are rejected; recursive copies containing symbolic links are rejected; symbolic-link folders are not recursively expanded; deletion is permanent and filesystem failures can leave a partial deletion.

### Bug Fixes

- Corrected the native GTK clipboard-action integration so Cut and Copy remain available without a selection, enabling the requested whole-line behavior in the existing context menu.

## [0.2.1] — 2026-10-22

### Major Additions

- No major feature additions; this release focused on the initial and last-tab workflow.

### Minor Changes

- Opened a blank, immediately editable `Untitled` tab at startup.
- Removed the **Open or create a file** empty screen.
- Changed the default new-document display name from `Untitled.txt` to `Untitled`.
- Updated About to version **0.2.1**, retaining release date **2026-10-22**, and extended the existing GTK tests for the new lifecycle behavior.
- Produced a Windows x64 release package with the executable, GTK runtime dependencies, resources, source archive, and a compiled document-test executable.

### Bug Fixes

- Prevented the editor from remaining without a document after closing its last tab; a fresh, empty `Untitled` tab is created automatically.
- Retained the original last tab when the user cancels its unsaved-changes warning. Replacement occurs only after closing is accepted.
- Kept window shutdown separate from last-tab replacement, allowing the application to exit without creating another tab.

## [0.2.0] — 2026-10-22

### Major Additions

- Added multiple independent document tabs, including multiple unsaved documents sharing the display name `Untitled.txt`.
- Changed **New file** to create a new tab and added a **+** button after the tabs.
- Added a close button that appears when a tab is hovered or its controls receive keyboard focus, and a yellow dot indicating unsaved changes.
- Opened files in separate tabs and focused an existing tab when an equivalent path was already open, including canonicalized relative paths and existing symbolic-link aliases.
- Added logical line numbers in the editor gutter, including empty lines, and **View > Line counter** to toggle them.
- Added **View > Word wrap**. Enabled wrapping fits text to the editor width and disables horizontal scrolling; disabled wrapping shows the horizontal scrollbar only when needed.
- Added the **Edit**, **View**, and **Other** menus.
- Added per-document Undo and Redo through the Edit menu, `Ctrl+Z`, `Ctrl+Y`, and the `Ctrl+Shift+Z` Redo alternative.
- Added the read-only **Keyboard shortcuts** dialog and the **About** dialog, showing the application name, version, release date, and an empty repository field.

### Minor Changes

- Applied Save, Save As, unsaved-changes warnings, and independent edit history to individual tabs. Closing a background tab targets that tab rather than the active document.
- Preserved undo history across saves and compared edited text with the saved snapshot to update the unsaved indicator correctly after Undo or Redo.
- Kept all tabs available if a later unsaved-changes warning is cancelled during window shutdown.
- Rejected Save As destinations already associated with another open tab to prevent document conflicts.
- Applied word-wrap and line-counter settings across all tabs, including newly created ones, without marking documents as modified.
- Reused GTK text layout for gutter positioning so line numbers follow wrapping, resizing, and vertical scrolling.
- Split tab widgets, line-number rendering, popups, and shortcut definitions into dedicated modules.
- Updated the dark interface and English Fluent messages to support the new controls and supplied mockups.
- Set About to version **0.2.0**, with release date **2026-10-22**. Shared shortcut definitions between action registration and the shortcuts table.
- Expanded validation to **10 passing Linux tests**: nine document tests and one GTK integration test covering multiple workflows.

### Bug Fixes

- Corrected gutter iteration so a trailing empty logical line receives a line number instead of being skipped.

## [0.1.0] — Release date unconfirmed

The initial MVP and its maintenance corrections were delivered under version 0.1.0. Available timestamps span **2026-10-21–2026-10-22**; no authoritative release date was recorded in the project.

### Major Additions

- Introduced Leditor as a Rust desktop application using GTK4, with a minimalist dark interface and fully offline operation after compilation.
- Added single-document text editing, including writing and deleting text, with native text-selection and clipboard controls.
- Added the initial centered **Open or create a file** message.
- Added the File menu with **New file**, **Open file**, **Close file**, **Save**, and **Save as**.
- Integrated platform file-opening and file-saving dialogs through `GtkFileDialog`.
- Made the first save of a new document open Save As and returned to the initial screen when the current document was closed.
- Added vertical and horizontal scrolling without automatic word wrapping.
- Added **Save**, **Don't save**, and **Cancel** warnings before discarding changes when closing a document or window, creating a document, or opening another file.
- Added graceful file-I/O error handling, preserving the current document when saving fails or a dialog is cancelled.
- Added safe saving through a temporary file in the destination directory, flushing and atomic replacement, with document state committed only after success.
- Added English interface messages through an embedded Fluent catalog, establishing the internationalization foundation for future translations.

### Minor Changes

- Added window-title updates showing the current file path and an asterisk for unsaved changes.
- Supported UTF-8 text with or without a byte-order mark. Unsupported encodings and NUL-containing files were rejected without replacing the current document.
- Preserved UTF-8 BOMs, uniform CRLF line endings, mixed line endings, existing symbolic links during saving, and traditional Unix file permissions.
- Added `Ctrl+N`, `Ctrl+O`, `Ctrl+W`, `Ctrl+S`, and `Ctrl+Shift+S` for the File menu actions.
- Blocked editing and overlapping document actions while asynchronous dialogs were active.
- Established separate document, interface, and localization modules, pinned dependencies, and Linux/Windows build instructions.
- Added document-integrity and GTK lifecycle tests. The corrected MVP archive records **nine passing Linux tests**: eight document tests and one GTK integration test covering multiple workflows.

### Bug Fixes

- Removed the GUI test's dependency on translated buttons and internal widgets of external file selectors, addressing the reported `GTK operation timed out` failure.
- Replaced external-selector inspection with controlled test responses while continuing to exercise real editor widgets and unsaved-changes dialogs. Production file selection continued to use the platform dialog.
- Added runtime detection of the newer GTK dark-theme setting, with a fallback for older GTK versions, to avoid the deprecated-setting warning on newer GTK releases.

## Windows Validation Note

The Windows x64 packages for versions 0.2.1 and 1.0.0 were cross-compiled on Linux. Their test executables were compiled, but execution through Wine was blocked before tests could start by `wineserver: socket: Operation not permitted`. No successful native Windows execution is claimed. These packaging updates did not change the editor's source version.


