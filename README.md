# Leditor

### A modern text editor for Linux and Windows.

**Version:** 1.0.0  
**Release date:** October 24, 2026

Leditor brings text editing and file organization together in a clean, minimalist workspace. Write, edit, and browse your files with independent tabs, an integrated file explorer, and a distraction-free dark interface.

Built with **Rust and GTK4**, Leditor works entirely offline and is growing step by step into a versatile editor for everyday writing and development.

## Current Features

### Your files, one workspace

- **Independent tabs:** work with multiple documents, including several unsaved files, each with its own editing history.
- **Flexible tab management:** drag tabs to reorder them, duplicate documents, rename files, and save or close tabs through context menus.
- **Integrated file explorer:** browse expandable folder trees, create files and folders, and open documents directly from the sidebar.
- **File organization:** copy, move, rename, and delete files and folders, with drag-and-drop support and contextual actions.

### Comfortable text editing

- **UTF-8 editing:** open, create, edit, and save text files, with support for UTF-8 BOMs and preservation of uniform Windows line endings.
- **Word wrap and line numbers:** adjust how text is displayed without changing the document's contents.
- **Undo and redo:** keep an independent editing history for every document.
- **Convenient clipboard actions:** cut or copy selected text—or the current logical line when nothing is selected.
- **Keyboard shortcuts:** access common actions quickly and consult the built-in shortcuts reference.

### A focused experience

- **Minimalist dark interface:** keep your text at the center of your workspace.
- **Collapsible explorer:** show or hide the sidebar with `Ctrl+B`.
- **Unsaved-change protection:** choose whether to save before closing modified documents.
- **File-operation safeguards:** name-conflict checks and deletion confirmations help protect your files.
- **Offline operation:** edit local files without an internet connection.

The current interface is available in **English**. Binary files and unsupported text encodings open in a clearly marked, non-editable tab.

## What's Next

Leditor's roadmap focuses on richer editing, broader file support, and a more customizable workspace:

- **Syntax highlighting** for source code and structured text.
- **Language Server Protocol (LSP) support** for language-aware editing tools.
- **A light theme** alongside the current dark interface.
- **Advanced text-editing features** for more demanding workflows.
- **An integrated Markdown viewer** for reading formatted documents.
- **PDF, image, and CSV viewers** within the editor.
- **Additional interface languages** and localization.
- **Support for more text encodings.**

These features are planned for future releases and are not included in version 1.0.0.

## License

MIT License.

## Development Transparency

Most of Leditor's code is generated with the assistance of artificial intelligence, under human supervision.
