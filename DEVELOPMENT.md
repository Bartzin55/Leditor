# Development Notes — Leditor 1.9.2-alpha


## 1.9.2-alpha integration

Explorer rows retain keyboard focus and selection on click. Files use a GTK double-click gesture to open. The panel tracks pointer entry and exit so file shortcuts can target the selected row even when the text view has keyboard focus. F2 uses the existing rename and Save As dialogs.

## 1.9.1-alpha integration

- `markdown_image.rs` is a small GTK widget around `GtkPicture`. It requests
  height for width from the decoded image ratio and zoom, with equal minimum
  and natural heights. Long documents can no longer compress an image to zero.
  Horizontal minimum stays small, so narrow windows remain usable. Images never
  exceed the available width. Both local and asynchronous network paths share
  this widget; late downloads use the current zoom. Existing PNG/JPEG and
  900×600 decode bounds are unchanged.
- `test_support.rs` runs each GTK test in a child of the test executable with
  `--exact` and a recursion guard. This isolates GTK initialization, callbacks
  and process-wide test configuration. Network tests create their own
  thread-default GLib main context. The original default UI test is retained,
  and optional graphical tests still document their display/tool requirements.
  Tests driving the mouse/keyboard should run separately to avoid competing for
  focus even though their GTK contexts are now isolated.
- `ui/theme.rs` detects properties at runtime: GTK 4.20+ receives
  `GtkCssProvider:prefers-color-scheme` and the modern process-local
  `GtkSettings:gtk-interface-color-scheme` override, keeping native controls and
  custom surfaces consistent. The OS preference is recorded separately and
  updates from the Linux portal/Windows registry remain supported. Older GTK
  retains the existing setting; the native minimum stays GTK 4.12.
- `file_ops::project_relative_path` formats visible Explorer paths using `/`
  separators and a leading project-root slash. It rejects outside-root paths
  and parent traversal and preserves visible symlink names. Clipboard actions
  do not read, write, rename or canonicalize the selected file.
- Tab icons are 14 px instead of 16, title text is 0.92 em, button padding is
  2×7 px and the bar minimum is 30 px instead of 34. Existing title reservation,
  middle ellipsis, unsaved indicator and close control remain in place.
- No new dependencies or shortcuts. The app remains MIT and keeps the existing
  dependency notices and installer dependency lists.

## 1.9.0-alpha integration

- `markdown_paths.rs` centralizes percent decoding, project-root semantics,
  lexical parent bounds and canonical/symlink containment for images and files.
  Link opening rechecks containment to handle a changed target. No arbitrary
  filesystem root or network share is inferred from a Markdown path.
- `markdown_network.rs` uses libsoup 3's asynchronous GLib/GIO APIs, platform TLS
  verification, redirects, a 15-second timeout, at most four connections per preview
  and a 16 MiB response cap. PNG/JPEG signatures select the image decoder explicitly;
  decoded output is scaled to at most 900×600. There is no browser engine, scripting,
  cookie jar or persistent cache. Dropping a preview aborts its session.
- `markdown.rs` retains Comrak AST/GTK rendering. `GtkFileLauncher` handles file
  associations and its GTK 4.12 `always-ask` property requests the OS chooser.
  Internet state and project-folder changes invalidate previews. Focus moves to the retained scroller before replacing a focused link. Disabled internet
  is checked before starting each download; the replacement link uses UriLauncher.
- `ui/markdown_actions.rs` installs the image handler on the existing EditorView.
  Menu, native paste signal and context action share it. Existing serialized editor
  operations freeze edits while clipboard/file dialogs and PNG writes complete.
  A named tempfile in the document directory provides atomic unique creation;
  it is retained before insertion and automatically cleaned on write failure.
  Selection replacement is one undo operation; PNGs are not deleted by Undo.
- A capture-phase `GtkGestureZoom` on the tab content uses the gesture-start zoom
  times its current scale. The same zoom preference applies to text/media/preview;
  the native gesture is not approximated with a custom touchpad event parser.
- `shortcuts.rs` remains the single accelerator/dialog table. F12 dispatches the
  existing file rename operation; Untitled now dispatches the existing Save As.
- `soup3` and `soup3-sys` are the only new Rust packages. GTK/sourceview and Comrak
  remain in use. Native libraries are dynamically linked; Leditor stays MIT.
- Installers include libsoup/TLS dependencies and locate GIO modules/certificates
  in private runtimes. Windows bundling walks the TLS DLL dependency closure.

The following sections describe retained implementation from earlier versions.


## 1.8.0-alpha integration

- `markdown.rs`: Comrak 0.59.0 parses the AST with CLI/Syntect defaults disabled.
  Only `shortcodes` and `attributes` crate features are enabled. GTK labels render
  escaped inline markup; boxes/grids render blocks, and noneditable GtkSourceViews
  highlight code using the already installed language catalog. Comrak's Anchorizer
  supplies unique heading IDs; only explicit header IDs are used from attributes.
  No user CSS or other attribute pairs are applied. Nesting is capped at 64 during
  rendering. Raw HTML is literal text; remote/resource URLs cannot execute scripts.
  Local images are PNG/JPEG only, canonicalized within the active project,
  decoded to at most 900×600 pixels and shown below the paragraph. Remote images now load asynchronously; blocked paths have placeholders. Native widgets need no additional web runtime.
- `ui/read_only.rs`: one global saved view-only preference controls all text tabs.
  It switches the visible scroller for eligible `.md` paths without replacing
  the source buffer or unparenting the source view. Keeping its scroll adjustments
  stable also preserves GtkSourceMap signal connections. `Tab` owns preview state and the source scroll position. Returning to edit
  mode restores the same view, undo history and caret. Theme/zoom also update previews.
  Source navigation is disabled only when its surface is hidden. File operations
  and saving pre-existing changes remain available; trim-on-save never modifies a
  read-only buffer. Dirty Markdown uses current buffer contents, not stale disk text.
- `input_method.rs`: app-local GTK settings plus explicit `im-module` overrides on
  TextViews and GtkText delegates. The widget override uses the canonical
  `gtk-im-context-simple` ID: `simple` is only an alias in GTK_IM_MODULE/settings. Mapping hooks apply the current mode to later
  dialogs; tab creation and mode changes also handle source views hidden
  by previews. `None` restores normal GTK input selection. No runtime environment
  mutation, forced restart or automatic document save occurs. Preferences load
  before session tabs are restored. Third-party input services are not installed
  or configured by Leditor.
- `file_ops::duplicate`: recursively copy to a temporary sibling directory, then
  atomically publish to the first available translated numbered name. Regular
  files retain their last extension; dotted folder names remain whole. Symlinks
  and special files are refused, consistently with the existing copy policy.
  A new concurrent-copy test exposed that GIO's preflight/no-overwrite move could
  race. `move_entry` now uses `renameat2(RENAME_NOREPLACE)` through rustix on Linux
  and `MoveFileExW` without replacement/copy flags on Windows. Cross-volume moves
  and filesystems lacking the atomic operation fail safely without fallback.
  Rustix/windows-sys were already in the lockfile and are reused directly.
- `ui_180_tests.rs`: UI transitions, blocked native keyboard edits, buffer/history
  preservation, actual dead-key input in simple mode, search/dialog delegates,
  persistence/session recovery and Explorer duplication. `tests/file_ops.rs`
  adds concurrency, collisions, Unicode names, extension/folder handling and cleanup.
- Existing modules and visual layout are retained. The toolbar is hidden/revealed
  along with the menus/tabs in focus mode. Ctrl+L toggles the existing only-view action through the shared shortcut table,
  which also supplies the translated Keyboard Shortcuts dialog.

Preview deliberately omits browser-only layout, arbitrary HTML/CSS,
SVG/GIF/WebP and math/Mermaid engines. Block selection is
native GTK selection rather than a single continuous web document. Windows and
Wayland validation limits are recorded in VALIDATION.md; property-based switching
on Linux/X11 does not prove every system IME/compositor combination.


## 1.7.0-alpha integration

- `editor_view.rs`: one native CompletionWords provider per source view, registered
  only against that view's buffer. GtkSourceView owns scanning, filtering, the
  popup, keyboard selection/acceptance and the undoable replacement. The first
  suggestion is selected on show. Providers are owned by native completion and
  released with the view; no shared/global vocabulary or custom scanner exists.
- `ui.rs` explicitly calls `sourceview5::init()` before constructing native source
  objects. This loads the library's CSS, translations and internal icons. Without
  it, repeated completion popups could have zero requested width and stay hidden.
- The existing exact-whitespace Enter handler and typed-delimiter logic remain.
  Native completion is attached after those handlers and consumes acceptance
  keys while its popup is open; real keyboard tests cover both paths.
- `file_icon.rs`: shared GIO filename-to-symbolic-icon function for Explorer and
  tabs, using the platform MIME database without reading the file contents.
- `tab.rs`: icon before the title; actual Pango text measurement reserves each
  title's minimum width, capped at about 32 average font characters. Middle
  ellipsis limits unusually long titles. Dot/close space remains reserved. Full
  filename/path is available as a tooltip, including named unsaved documents.
  Renames and Save As update the title/icon; ordinary text edits reuse them.
- `assets/leditor-{light,dark}.xml`: native `bracket-match` style with bold text,
  a single underline and `#86ee77` underline color. The matching algorithm and
  existing View preference are unchanged, including native language contexts.
- `shortcuts.rs`, `locales/`: native completion controls are documented in all
  three UI languages. No new global accelerator, preference or menu is added.
- `ui_170_tests.rs`: UI allocation/icon/style tests and native X11 completion
  input tests. Existing regression suites remain in place.

Native references:
- https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/class.CompletionWords.html
- https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/style-reference.html

Completion follows the native asynchronous scanner (including its idle delay and
word rules). Very short prefixes may yield no proposals. The vocabulary is not
persisted and never aggregates other tabs. It does not understand language symbols.

## Retained 1.6.0-alpha integration

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
styles and override editor/gutter colors, the yellow current-line number and
the bold, light-green bracket underline.
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

Cargo.lock pins Comrak and the enabled dependency graph for 1.9.0-alpha; existing
package versions are retained. Both install.sh (through scripts/install_linux.py)
and install.ps1 require Rust 1.89+ and already provide GtkSourceView 5 and its
native data. Version 1.9 adds GTK 4.12, libsoup 3, GLib TLS modules and CA certificates; private launchers set the corresponding runtime paths. About reads
Cargo's package version. The Linux installer User-Agent is 1.9.1-alpha.

The application remains MIT. Native GTK/GtkSourceView are dynamically linked
LGPL libraries; their notices and redistribution obligations remain applicable.
See THIRD_PARTY.md and LICENSES/. Build commands are in BUILD.md and measured
validation results and limits in VALIDATION.md.
