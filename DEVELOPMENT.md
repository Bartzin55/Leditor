# Development Notes — Leditor 1.6.0-alpha

## 1.6.0-alpha integration

- `editor_view.rs`: native SmartHomeEndType::Before and SpaceDrawer spaces/tabs.
  Typed-delimiter pairing groups edits in the native undo stack. Selection wrapping,
  escaped quotes and word apostrophes are handled. Pairing is bypassed during IME
  preedit/overwrite mode; composed/dead-key and Ctrl/Alt input stays literal.
- `ui/editing.rs`: native Buffer::sort_lines with preservation of an absent final
  newline (GtkSourceView 5.12 adds one). Character sorting uses GLib casefold and
  CollationKey, retaining all Unicode scalar values. Checkboxes execute once and
  reset rather than mutating future selections. Sorting requires a text selection.
- `text_ops.rs`: trailing whitespace ranges in GTK character offsets. Delete them
  backwards in a single undo group in the live buffer before the write snapshot.
  Save As cancellation does not trim. Failed saves retain edits and undo history.
  Autosave excludes the focused line so an idle pause cannot erase a typed space.
- `ui/goto.rs`: bottom navigation bar, numeric validation, cursor and viewport.
  Search mutual exclusion is in `ui/search.rs`; Escape handling in `ui/focus.rs`.
  Tab changes refresh the valid range; media/unsupported tabs hide navigation.
- `ui/preferences.rs`: four persistent booleans in the existing settings file.
  Bracket matching defaults on; whitespace, auto-close and trim default off.
  Smart Home/End is always enabled. Sorting and navigation state are not saved.
- `ui/syntax.rs`: matching brackets independently of syntax-color visibility,
  using native language contexts. No new session format, undo stack or dependency.

Native API references:
- https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/method.Buffer.sort_lines.html
- https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/enum.SmartHomeEndType.html
- https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/method.SpaceDrawer.set_types_for_locations.html

## Editing engine

The existing application already used GtkSourceView View and Buffer for syntax
highlighting. This update delegates the requested editing features to that same
engine without rebuilding the application or adding dependencies.

- `editor_view.rs`: `EditorView` subclasses GtkSourceView. Native line numbers,
  tab width, indentation width, Tab/Shift+Tab and insert-spaces-instead-of-tabs
  replace manual gutter drawing and tab handling. The remaining custom code
  preserves clipboard-without-selection behavior, exact-whitespace auto-indent,
  current-line background, indentation guides and zoom.
- `minimap.rs`: constructs a native GtkSourceMap bound to the editor view. It
  shares syntax tags and owns scroll navigation, at the same right-hand location.
  Its small font is independent of editor zoom; it is read-only and hidden for
  media/unsupported tabs. Visibility remains a persisted preference.
- `tab.rs`: owns the source view/buffer, scroll container and tab state. Word Wrap
  already used the native inherited TextView API and still does. Undo/Redo uses
  the native source buffer history inherited from GTK; there is no parallel stack.
- `ui/enhancements.rs`: calls GtkSourceBuffer `change_case`. First-letter modes
  adapt only the ranges passed to the native uppercase operation, preserving the
  rest of each word. Selection marks handle Unicode length changes; a single
  user-action group makes each complete command undoable in one step.

With conversion enabled, Tab now advances to the native indentation stop, so
inside a line it may insert fewer spaces than the configured width. Existing text
is not rewritten when changing this preference. Multiline selection indentation
and Shift+Tab are native GtkSourceView behavior.

## Appearance and compatibility

`assets/leditor-light.xml` and `assets/leditor-dark.xml` inherit Adwaita syntax
styles and override only editor/gutter colors and the yellow current-line number.
They are embedded with include_str and written once to a private TempDir retained
by the GUI thread. This supports GtkSourceView 5.12, whose style parser requires
filesystem paths. The directory is removed on normal thread shutdown; after an
abrupt process termination temporary files may remain. A preparation failure is
reported and falls back to the existing native schemes. No theme option is added.

The Windows startup profile remains `GTK_CSD=0`, `GSK_RENDERER=cairo`,
`GDK_DEBUG=dcomp`, respecting explicit environment overrides. It is a mitigation,
not a complete fix for live resize; see RESIZE.md. Icon assets are preserved.

## Shortcuts and actions

`shortcuts.rs` supplies both accelerators and the Keyboard Shortcuts dialog.
English, Portuguese and Spanish descriptions are in `locales/`.
Ctrl+Tab/Ctrl+Shift+Tab navigate the existing tab vector, including its drag-reordered
order, and wrap at either end. They do nothing with only one tab. Ctrl+Shift+O/W/N
invoke the existing folder open/close/create actions. No new menu entries are
needed for tab navigation. Find retains its special Enter/Shift+Enter handling.

## Existing modules and data safety

- `ui.rs`, `ui/menu.rs`, `ui/tab_actions.rs`: action dispatch, lifecycle, menus,
  tab selection and reordering.
- `document.rs`, `file_ops.rs`: UTF-8 I/O, atomic replacement, dirty state,
  conflict checks and safe explorer filesystem operations.
- `session.rs`, `ui/preferences.rs`: recovery, autosave scheduling and settings.
  The session format is unchanged; multiple windows are outside this update.
- `ui/search.rs`: existing literal search and replace with yellow/green tags.
- `ui/focus.rs`, `ui/theme.rs`, `ui/syntax.rs`: Focus, system/light/dark resolution,
  language detection and native scheme integration.
- `viewer.rs`: image decoding and PDFium worker; media stays read-only and is
  excluded from text mutation/save paths. Save All retains existing cancellation.
- `popups.rs`, `i18n.rs`, `locales/`: dialogs, version presentation and translations.

Document storage, autosave conflict handling, recovery and save prompts are
preserved. Dirty recovered text whose on-disk baseline changed remains recoverable
as an independent unsaved tab. Undo history is still not persisted across launches.

## Build, installers and licensing

Cargo.lock retains the same dependency versions; only Leditor becomes 1.6.0-alpha.
Both install.sh (through scripts/install_linux.py) and install.ps1 already provide
GtkSourceView 5 and its native data. No new packages or asset-copy steps are
required. About reads Cargo's package version. The Linux installer User-Agent
was updated to 1.6.0-alpha.

The application remains MIT. Native GTK/GtkSourceView are dynamically linked
LGPL libraries; their notices and redistribution obligations remain applicable.
See THIRD_PARTY.md and LICENSES/. Build commands are in BUILD.md and measured
validation results and limits in VALIDATION.md. No push was performed.
