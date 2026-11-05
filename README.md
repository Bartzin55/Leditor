# Leditor

## 1.6.0-alpha — Editing and navigation tools

- View → Match brackets: native highlighting for `()`, `[]`, `{}` (on by default).
- Smart Home/End: first press goes to the first/last text, the second to the line
  edge. Shift extends selection using the same native GtkSourceView behavior.
- View → Show spaces and tabs: native whitespace markers (off by default).
- Edit → Auto-close brackets and quotes: pairs `()`, `[]`, `{}`, `''`, `""`, wraps
  selected text and skips an existing closing delimiter. Escaped quotes and
  apostrophes in words remain literal. Off by default; paste is unchanged.
- Edit → Trim trailing spaces on save: removes ASCII spaces/tabs at line ends
  on Save, Save As, Save All and autosave (off by default). Autosave preserves the
  focused line so pausing between words does not erase the current space.
  Explicit saves trim every line. Cleanup is undoable; BOM/line endings are retained.
- Edit → Sort selected lines: native GtkSourceView sorting of whole touched lines.
  A selection ending at the next line's column zero excludes that next line.
- Edit → Sort selected characters: GLib locale collation of all selected Unicode
  scalar characters, including whitespace. Both sorts ignore case and retain
  duplicates. The checkboxes execute once and reset; each sort is undoable.
  Combining marks are separate characters; collation follows the OS locale.
- View → Go to line / Ctrl+G: bottom bar with a 1-based logical line number.
  Enter or Go to line moves the cursor/view. Invalid input leaves the cursor
  unchanged. Esc closes the bar; Go to Line and Find/Replace hide one another.
- Permanent options are remembered and apply to existing/new text tabs.

All historical application version labels now use `-alpha`. Existing Git
commits/tags and third-party versions are unchanged. No push was performed.

## Retained from 1.5.3-alpha — Native GtkSourceView editing

- Native GtkSourceMap at the right, sharing syntax colors and scroll navigation.
- Native logical line numbers, preserving the yellow current-line number.
- Native Tab/Shift+Tab indentation and configurable tab stops/spaces.
- Native GtkSourceBuffer case conversion, preserving first-letter-only commands.
- Word Wrap and Undo/Redo continue through the native GTK/GtkSourceView APIs.
- Ctrl+Tab / Ctrl+Shift+Tab navigate tabs in display order, wrapping at the ends.
- Ctrl+Shift+O opens a folder; Ctrl+Shift+W closes it; Ctrl+Shift+N creates one.

Use `bash install.sh --source .` or `powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Source .` from this extracted ZIP. This ZIP contains the unpublished 1.6.0-alpha update.
See BUILD.md, DEVELOPMENT.md and VALIDATION.md for setup, design and test results.


Windows retains the 1.5.2-alpha compatibility profile: native title bar/borders,
Cairo and DirectComposition. It reduced the reported artifacts but did not fully
resolve live resizing. See RESIZE.md and VALIDATION.md.

Build/install from the extracted source with `bash install.sh --source .` on
Linux or `powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Source .`
on Windows. Choose a per-user installation without administrator requests or a
system installation. Native dependencies and temporary missing build tools are
prepared automatically on supported x86_64 platforms. See [BUILD.md](BUILD.md)
for prerequisites, updates, uninstall, and manual fallback instructions.

The revised 1.5.1-alpha scripts preserve an existing installation marker and report
temporary-file cleanup, including cleanup after a failed installation.
PDFium notices from the downloaded runtime are merged with the bundled notices,
including when their directories already exist.
Windows extractions explicitly use `%SystemRoot%\System32\tar.exe` to avoid
MSYS2 PATH conflicts with Windows drive-letter paths.

Third-party texts are included in [LICENSES](LICENSES/README.md); Leditor remains
MIT. See [VALIDATION.md](VALIDATION.md) for the actual validation limits.

### A modern text editor for Linux and Windows.

**Version:** 1.6.0-alpha
**Release date:** November 5, 2026

Leditor brings text editing and file organization together in a clean, minimalist workspace. Write, edit, and browse your files with independent tabs, an integrated file explorer, and a distraction-free interface.

Built with **Rust and GTK4**, Leditor works entirely offline and is growing step by step into a versatile editor for everyday writing and development.

## New in 1.5.0-alpha

- **Syntax highlighting:** View → Syntax highlighting enables/disables GtkSourceView highlighting (enabled by default). Detect language uses the filename and GIO content metadata. Plain Text is first; all installed GtkSourceView language definitions follow alphabetically, with a radio indicator for the active tab. The enable preference is saved; per-tab selections are included in optional session recovery.
- **PDF viewer:** open PDFs in read-only tabs, with continuous vertical pages and the existing 5–800% zoom controls. Requires the PDFium runtime described in BUILD.md.
- **Image viewer:** open PNG, JPG and JPEG in read-only tabs, using the same zoom controls. Media tabs hide line numbers/minimap, and show only zoom in the footer.
- **Save all:** saves text tabs in their current order, asking for a destination for each untitled document. Cancellation or failure stops the operation; remaining edits are retained. Read-only viewers are skipped.
- **Reorganized menus:** File, Edit, View, Window, Other. Indentation settings now share an Edit submenu; minimap, focus, line counter, explorer and theme are under Window.

PDF and image tabs are viewers only: no text editing, Save/Save as, find/replace or duplication-as-text. Renaming, closing and reordering still work. PDFs requiring passwords are not supported in this basic viewer. Syntax names come from the installed GtkSourceView catalog; the surrounding UI is translated into English, Portuguese and Spanish.

## Retained from 1.4.1-alpha

This maintenance release continues the GTK 1.4.0-alpha codebase. The GTK codebase is retained; historical application versions now carry the alpha suffix.

- Removed Always on top completely, including its menu item and dedicated platform bindings.
- System default selects the existing light or dark palette from the OS preference, instead of a separate third style.
- Ctrl+F toggles the search bar. Shift+Enter navigates backward while it is visible; inserting a line above resumes after closing it.
- Indentation guides are dashed, half a character before the text, and continue through blank lines.
- The supplied Leditor icon appears in About, Linux application launchers and the Windows executable resource.

## Retained from 1.4.0-alpha

- **Clickable minimap:** enable Window → Minimap to see a compact overview at the right. Click or drag to navigate the viewport using GtkSourceMap. Off by default; your choice is remembered.
- **Find and replace:** Ctrl+F toggles the two-row search bar and focuses Find. Matches are yellow; the active match is green. Navigate with ↑/↓ or Enter in Find or Shift+Enter while the bar is visible. Tab switches between Find and Replace. Replace one or all with Undo/Redo support.
- **Case-sensitive search:** optional for each search session, reset whenever Find is invoked; never saved.
- **Focus mode:** Shift+F11 enters fullscreen and hides menus, tabs, explorer, line numbers and footer. Move to the top edge to reveal menus/tabs. Shift+F11 or Esc restores the previous layout. Never saved.

Search is literal and applies to the active document. Empty searches produce no matches. Navigation wraps at the beginning/end. Esc also closes the search bar outside Focus mode.

## Included since 1.3.0-alpha

- **Linux integration:** running the built app (including `cargo run`) installs a per-user launcher and a stable executable, accessible through `~/.local/bin/leditor`.
- **Automatic indentation:** Enter and Edit → Insert line below preserve leading spaces and tabs. Enabled by default; toggle Edit → Indentation options → Automatic indentation.
- **Indentation guides:** optional vertical guides in View. Off by default; follows the configured tab width and editor zoom.
- **Theme selector:** Window → Theme offers System default, Light theme and Dark theme, with the active option marked.
- **XDG preferences:** Linux settings now live in `~/.config/leditor/settings.ini`; existing preferences are migrated without deleting their old copy.

See [BUILD.md](BUILD.md) for installation paths, PATH setup and optional system-wide installation.

## Also included

- **A useful status bar:** see line and character counts, caret position, save status, encoding, file format, and zoom at a glance.
- **English, Português (Brasil), and Español:** choose your interface language in Other > Language and restart to apply it.
- **Your preferences remembered:** keep your theme, zoom, wrapping, line numbers, indentation, explorer visibility and width, autosave, and session choices between launches.
- **Light and dark themes:** choose System default, Light theme or Dark theme in Window → Theme.
- **Quick text transformations:** change case for a selection or the word at the caret. Insert a line above with Shift+Enter.
- **A quieter start:** the explorer starts hidden for new configurations.

## Current Features

### Your files, one workspace

- **Independent tabs:** work with multiple documents, including several unsaved files, each with its own editing history.
- **Flexible tab management:** drag tabs to reorder them, duplicate documents, rename files, and save or close tabs through context menus.
- **Integrated file explorer:** browse expandable folder trees, create files and folders, and open documents directly from the sidebar.
- **File organization:** copy, move, rename, and delete files and folders, with drag-and-drop support and contextual actions.

### Pick up where you left off

- **Optional autosave:** save changes to named files after a short pause in typing.
- **Session restoration:** reopen your folder and tabs, including recovered untitled documents.
- **Recovery without filenames:** keep unsaved drafts without choosing a destination or creating files in your project.

Autosave and session restoration start disabled. Enable them in **Other**; restoration requires autosave. Disabling autosave also disables restoration.

### Comfortable text editing

- **UTF-8 editing:** open, create, edit, and save text files, with support for UTF-8 BOMs and preservation of uniform Windows line endings.
- **Word wrap and line numbers:** adjust how text is displayed without changing the document's contents.
- **Flexible indentation:** insert literal tabs or 1–8 spaces, adjust tab width, and outdent with `Shift+Tab`.
- **Text zoom:** choose 5–800% magnification for the editor and line numbers, with keyboard and Ctrl+scroll controls.
- **Current-line highlighting:** follow the caret with a subtle gray highlight and a yellow line number.
- **Undo and redo:** keep an independent editing history for every document.
- **Convenient clipboard actions:** cut or copy selected text—or the current logical line when nothing is selected.
- **Keyboard shortcuts:** access common actions quickly and consult the built-in shortcuts reference.

### A focused experience

- **Minimalist dark interface:** keep your text at the center of your workspace.
- **Collapsible explorer:** show or hide the sidebar with `Ctrl+B`.
- **Unsaved-change protection:** choose whether to save before closing modified documents.
- **File-operation safeguards:** name-conflict checks and deletion confirmations help protect your files.
- **Offline operation:** edit local files without an internet connection.

The interface is available in **English, Brazilian Portuguese, and Spanish**. Binary files and unsupported text encodings open in a clearly marked, non-editable tab.

## What's Next

Leditor's roadmap focuses on richer editing, broader file support, and a more customizable workspace:

- **Language Server Protocol (LSP) support** for language-aware editing tools.
- **Advanced text-editing features** for more demanding workflows.
- **An integrated Markdown viewer** for reading formatted documents.
- **CSV viewer** within the editor.
- **Additional interface languages** and localization.
- **Support for more text encodings.**

These features are planned for future releases and are not included in version 1.6.0-alpha.

## Get Started

See [BUILD.md](BUILD.md) for Linux and Windows build instructions, [DEVELOPMENT.md](DEVELOPMENT.md) for technical notes, and [VALIDATION.md](VALIDATION.md) for test results and current limitations.

## License

[MIT license](LICENSE). See [THIRD_PARTY.md](THIRD_PARTY.md) for dependency licenses and runtime redistribution requirements.

## Development Transparency

Most of Leditor's code is generated with the assistance of artificial intelligence, under human supervision.



Windows resize follow-up (1.5.2-alpha): default to native frame + Cairo +
DirectComposition, the best of the user-tested configurations. Blank areas
during live resize remain a known issue; this is a mitigation, not a complete
fix. Existing environment overrides take precedence. See RESIZE.md.
