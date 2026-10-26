# Leditor

### A modern text editor for Linux and Windows.

**Version:** 1.2.0  
**Release date:** October 25, 2026

Leditor brings text editing and file organization together in a clean, minimalist workspace. Write, edit, and browse your files with independent tabs, an integrated file explorer, and a distraction-free dark interface.

Built with **Rust and GTK4**, Leditor works entirely offline and is growing step by step into a versatile editor for everyday writing and development.

## New in 1.2.0

- **A useful status bar:** see line and character counts, caret position, save status, encoding, file format, and zoom at a glance.
- **English, Português (Brasil), and Español:** choose your interface language in Other > Language and restart to apply it.
- **Your preferences remembered:** keep your theme, zoom, wrapping, line numbers, indentation, explorer visibility and width, autosave, and session choices between launches.
- **Light and dark themes:** choose Light theme in View, or turn it off for the dark interface.
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

- **Syntax highlighting** for source code and structured text.
- **Language Server Protocol (LSP) support** for language-aware editing tools.
- **Advanced text-editing features** for more demanding workflows.
- **An integrated Markdown viewer** for reading formatted documents.
- **PDF, image, and CSV viewers** within the editor.
- **Additional interface languages** and localization.
- **Support for more text encodings.**

These features are planned for future releases and are not included in version 1.2.0.

## Get Started

See [BUILD.md](BUILD.md) for Linux and Windows build instructions, [DEVELOPMENT.md](DEVELOPMENT.md) for technical notes, and [VALIDATION.md](VALIDATION.md) for test results and current limitations.

## License

[LICENSE](LICENSE)

## Development Transparency

Most of Leditor's code is generated with the assistance of artificial intelligence, under human supervision.

