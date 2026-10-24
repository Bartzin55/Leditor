use super::*;
use std::time::{Duration, Instant};

#[track_caller]
fn pump_until(mut condition: impl FnMut() -> bool) {
    let caller = std::panic::Location::caller();
    let deadline = Instant::now() + Duration::from_secs(8);
    let context = glib::MainContext::default();
    while !condition() {
        assert!(Instant::now() < deadline, "GTK timed out at {caller}");
        for _ in 0..100 {
            if !context.pending() {
                break;
            }
            context.iteration(false);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn drain() {
    let until = Instant::now() + Duration::from_millis(100);
    pump_until(|| Instant::now() >= until);
}
fn labels(widget: &gtk::Widget) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        result.push(label.text().to_string());
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        result.extend(labels(&widget));
        child = widget.next_sibling();
    }
    result
}
fn button(widget: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Some(button) = widget.downcast_ref::<gtk::Button>() {
        if button
            .label()
            .is_some_and(|text| text.replace('_', "") == label)
        {
            return Some(button.clone());
        }
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        if let Some(found) = button(&widget, label) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}
#[track_caller]
fn click(label: &str) {
    let mut found = None;
    pump_until(|| {
        let context = glib::MainContext::default();
        for _ in 0..100 {
            if !context.pending() {
                break;
            }
            context.iteration(false);
        }
        found = gtk::Window::list_toplevels()
            .iter()
            .filter(|w| w.is_visible())
            .find_map(|w| button(w, label));
        found.is_some()
    });
    found.unwrap().emit_clicked();
}
fn run(editor: &Rc<Editor>, intent: Intent) {
    editor.request(intent);
    pump_until(|| !editor.busy.get());
}
fn append(tab: &Tab, text: &str) {
    let buffer = tab.view.buffer();
    buffer.begin_user_action();
    buffer.insert(&mut buffer.end_iter(), text);
    buffer.end_user_action();
}
fn queue(editor: &Editor, save: bool, path: Option<&std::path::Path>) {
    editor
        .file_choices
        .borrow_mut()
        .push_back((save, path.map(ToOwned::to_owned)));
}
fn toggle(editor: &Rc<Editor>, name: &str) {
    editor.actions[name].activate(None);
}

/// The editor, warnings and popups are real GTK widgets. Only OS file choices
/// are scripted, so this test does not depend on a portal or translated controls.
#[test]
fn tabs_history_layout_dialogs_and_file_safety() {
    let test_home = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_DATA_HOME", test_home.path());
    std::env::set_var("XDG_CONFIG_HOME", test_home.path());
    std::env::set_var("GSETTINGS_BACKEND", "memory");
    glib::set_prgname(Some("leditor-tests"));
    glib::set_application_name("Leditor tests");
    gtk::init().unwrap();
    let application = gtk::Application::builder()
        .application_id("io.github.leditor.Test")
        .build();
    application.register(None::<&gio::Cancellable>).unwrap();
    let editor = create(&application);
    assert_eq!(editor.tabs.borrow().len(), 1);
    let a = editor.current().unwrap();
    assert_eq!(
        editor.stack.visible_child_name().as_deref(),
        Some(a.id.to_string().as_str())
    );
    assert!(a.document.borrow().text.is_empty());
    assert!(a.view.is_editable());
    assert!(editor.actions["save"].is_enabled());
    assert!(!editor.actions["undo"].is_enabled());
    append(&a, "first");
    editor.plus.emit_clicked();
    pump_until(|| !editor.busy.get());
    let b = editor.current().unwrap();
    append(&b, "second");
    assert_ne!(a.id, b.id);
    assert_eq!(a.name(&editor.strings), "Untitled");
    assert_eq!(b.name(&editor.strings), "Untitled");
    assert_eq!(editor.tabs.borrow().len(), 2);
    assert_eq!(a.document.borrow().text, "first");
    assert_eq!(b.document.borrow().text, "second");
    assert_eq!(b.dirty.opacity(), 1.0);
    toggle(&editor, "undo");
    assert_eq!(b.document.borrow().text, "");
    assert_eq!(b.dirty.opacity(), 0.0);
    assert_eq!(a.document.borrow().text, "first");
    editor.select(a.id);
    toggle(&editor, "undo");
    assert_eq!(a.document.borrow().text, "");
    toggle(&editor, "redo");
    assert_eq!(a.document.borrow().text, "first");
    editor.select(b.id);
    toggle(&editor, "redo");
    assert_eq!(b.document.borrow().text, "second");
    // Closing a background tab must act on its ID, not the active index.
    a.close.emit_clicked();
    click("Cancel");
    pump_until(|| !editor.busy.get());
    assert!(editor.tab(a.id).is_some());
    assert_eq!(editor.active.get(), Some(a.id));
    queue(&editor, true, None);
    editor.request(Intent::Close(a.id));
    click("Save");
    pump_until(|| !editor.busy.get());
    assert_eq!(a.document.borrow().text, "first");
    assert!(editor.tab(a.id).is_some());
    editor.request(Intent::Close(a.id));
    click("Don't save");
    pump_until(|| !editor.busy.get());
    assert!(editor.tab(a.id).is_none());
    assert_eq!(b.document.borrow().text, "second");
    let path = test_home.path().join("document.txt");
    queue(&editor, true, Some(&path));
    run(&editor, Intent::Save(b.id));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
    assert_eq!(b.name(&editor.strings), "document.txt");
    assert_eq!(b.dirty.opacity(), 0.0);
    // Undo across a save must still compare against the saved snapshot.
    toggle(&editor, "undo");
    assert!(b.document.borrow().is_dirty());
    toggle(&editor, "redo");
    assert!(!b.document.borrow().is_dirty());
    let copy = test_home.path().join("copy.txt");
    append(&b, " copy");
    queue(&editor, true, Some(&copy));
    run(&editor, Intent::SaveAs(b.id));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
    assert_eq!(std::fs::read_to_string(&copy).unwrap(), "second copy");
    queue(&editor, false, Some(&path));
    run(&editor, Intent::Open);
    let c = editor.current().unwrap();
    assert_ne!(c.id, b.id);
    append(&c, " unsaved");
    editor.select(b.id);
    queue(
        &editor,
        false,
        Some(&test_home.path().join("./document.txt")),
    );
    run(&editor, Intent::Open);
    assert_eq!(editor.active.get(), Some(c.id));
    assert_eq!(editor.tabs.borrow().len(), 2);
    assert_eq!(c.document.borrow().text, "second unsaved");
    #[cfg(unix)]
    {
        let link = test_home.path().join("alias.txt");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        queue(&editor, false, Some(&link));
        run(&editor, Intent::Open);
        assert_eq!(editor.active.get(), Some(c.id));
        assert_eq!(editor.tabs.borrow().len(), 2);
    }
    // Saving another tab over an open destination is rejected without data loss.
    queue(&editor, true, Some(&path));
    editor.request(Intent::SaveAs(b.id));
    click("Close");
    pump_until(|| !editor.busy.get());
    assert_eq!(b.document.borrow().path.as_deref(), Some(copy.as_path()));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
    // Open failures leave all existing buffers and the active tab intact.
    let invalid = test_home.path().join("invalid.bin");
    std::fs::write(&invalid, [0xff]).unwrap();
    queue(&editor, false, Some(&invalid));
    run(&editor, Intent::Open);
    let unsupported = editor.current().unwrap();
    assert!(unsupported.unsupported.get());
    assert!(unsupported.title.has_css_class("unsupported-title"));
    assert!(labels(unsupported.scroll.upcast_ref()).contains(&"Unsupported file".into()));
    assert!(!editor.actions["save"].is_enabled());
    capture(&editor, "unsupported.png");
    run(&editor, Intent::Close(unsupported.id));
    assert_eq!(editor.tabs.borrow().len(), 2);
    assert_eq!(c.document.borrow().text, "second unsaved");
    queue(&editor, false, None);
    run(&editor, Intent::Open);
    assert_eq!(editor.active.get(), Some(c.id));
    // Actual layout must wrap within one logical line, and hide horizontal scroll.
    drain();
    assert!(c.scroll.hadjustment().upper() <= c.scroll.hadjustment().page_size() + 1.0);
    append(&c, &format!(" {}\nline two\n\n", "longword".repeat(140)));
    pump_until(|| c.view.width() > 200);
    drain();
    let logical_lines = c.view.buffer().line_count();
    assert_eq!(logical_lines, 4);
    pump_until(|| c.scroll.hadjustment().upper() > c.scroll.hadjustment().page_size());
    let before = c.view.line_yrange(&c.view.buffer().start_iter()).1;
    toggle(&editor, "word-wrap");
    drain();
    assert_eq!(c.view.wrap_mode(), gtk::WrapMode::WordChar);
    assert_eq!(b.view.wrap_mode(), gtk::WrapMode::WordChar);
    assert_eq!(c.scroll.hscrollbar_policy(), gtk::PolicyType::Never);
    assert_eq!(c.view.buffer().line_count(), logical_lines);
    let after = c.view.line_yrange(&c.view.buffer().start_iter()).1;
    assert!(after > before);
    assert!(
        c.view
            .line_yrange(&c.view.buffer().iter_at_line(1).unwrap())
            .0
            >= after
    );
    toggle(&editor, "line-numbers");
    assert!(!c
        .view
        .gutter(gtk::TextWindowType::Left)
        .unwrap()
        .is_visible());
    assert!(!b
        .view
        .gutter(gtk::TextWindowType::Left)
        .unwrap()
        .is_visible());
    toggle(&editor, "line-numbers");
    assert!(c
        .view
        .gutter(gtk::TextWindowType::Left)
        .unwrap()
        .is_visible());
    toggle(&editor, "word-wrap");
    assert_eq!(c.scroll.hscrollbar_policy(), gtk::PolicyType::Automatic);
    assert!(
        !b.document.borrow().is_dirty(),
        "view changes must not dirty a saved document"
    );
    // New tabs inherit current view settings and have no undo history.
    toggle(&editor, "word-wrap");
    toggle(&editor, "line-numbers");
    run(&editor, Intent::New);
    let d = editor.current().unwrap();
    assert_eq!(d.view.wrap_mode(), gtk::WrapMode::WordChar);
    assert!(!d
        .view
        .gutter(gtk::TextWindowType::Left)
        .unwrap()
        .is_visible());
    assert!(!d.view.buffer().can_undo());
    run(&editor, Intent::Close(d.id));
    // Popups share the package version and shortcut definitions with the app.
    let about = popups::about(&editor.window, &editor.strings);
    let contents = labels(about.upcast_ref());
    assert!(contents.contains(&"1.0.0".to_owned()));
    assert!(contents.contains(&popups::RELEASE_DATE.to_owned()));
    assert!(!contents.iter().any(|s| s.contains("github.com")));
    click("OK");
    drain();
    let keyboard = popups::keyboard(&editor.window, &editor.strings);
    let contents = labels(keyboard.upcast_ref());
    for shortcut in SHORTCUTS {
        assert!(contents.contains(&shortcut.display.to_owned()));
        assert!(!application
            .accels_for_action(&format!("win.{}", shortcut.action))
            .is_empty());
    }
    click("Close");
    drain();
    // Failed save during close keeps that exact tab and its contents open.
    let old_path = c.document.borrow().path.clone();
    c.document.borrow_mut().path = Some(test_home.path().join("missing/file.txt"));
    editor.request(Intent::Close(c.id));
    click("Save");
    click("Close");
    pump_until(|| !editor.busy.get());
    assert!(editor.tab(c.id).is_some());
    assert!(c.document.borrow().is_dirty());
    c.document.borrow_mut().path = old_path;
    append(&b, " last change");
    // Cancel at the second quit warning must not remove the first tab either.
    editor.request(Intent::Quit);
    click("Don't save");
    click("Cancel");
    pump_until(|| !editor.busy.get());
    assert!(editor.window.is_visible());
    assert_eq!(editor.tabs.borrow().len(), 2);
    assert!(b.document.borrow().is_dirty());
    // Closing the final tab replaces it only after the warning is accepted.
    editor.request(Intent::Close(b.id));
    click("Don't save");
    pump_until(|| !editor.busy.get());
    editor.request(Intent::Close(c.id));
    click("Cancel");
    pump_until(|| !editor.busy.get());
    assert_eq!(editor.tabs.borrow().len(), 1);
    assert_eq!(editor.active.get(), Some(c.id));
    editor.request(Intent::Close(c.id));
    click("Don't save");
    pump_until(|| !editor.busy.get());
    assert_eq!(editor.tabs.borrow().len(), 1);
    let replacement = editor.current().unwrap();
    assert_ne!(replacement.id, c.id);
    assert_eq!(replacement.name(&editor.strings), "Untitled");
    assert!(replacement.document.borrow().text.is_empty());
    assert!(!replacement.document.borrow().is_dirty());
    assert!(!replacement.view.buffer().can_undo());
    // A clean final tab is replaced too; the X and menu share this close path.
    replacement.close.emit_clicked();
    pump_until(|| !editor.busy.get());
    assert_eq!(editor.tabs.borrow().len(), 1);
    let e = editor.current().unwrap();
    assert_ne!(e.id, replacement.id);
    assert!(e.document.borrow().text.is_empty());
    assert!(e.id > d.id);
    version_one_features(&editor, test_home.path());
    editor.select(e.id);
    append(&e, "quit save");
    let quit_path = test_home.path().join("quit.txt");
    queue(&editor, true, Some(&quit_path));
    editor.request(Intent::Quit);
    click("Save");
    pump_until(|| !editor.busy.get());
    assert!(!editor.window.is_visible());
    assert_eq!(std::fs::read_to_string(quit_path).unwrap(), "quit save");
    assert!(editor.file_choices.borrow().is_empty());
}

fn version_one_features(editor: &Rc<Editor>, root: &std::path::Path) {
    let original = editor.current().unwrap();
    editor.load_folder(root.to_owned());
    assert!(labels(editor.explorer.rows.upcast_ref())
        .iter()
        .any(|s| s == "document.txt"));
    toggle(editor, "show-explorer");
    assert!(!editor.explorer.panel.is_visible());
    assert!(editor.explorer.reveal.is_visible());
    toggle(editor, "show-explorer");
    assert!(editor.explorer.panel.is_visible());
    editor.operation(Operation::Create(root.to_owned(), true));
    answer_name("created-folder");
    pump_until(|| !editor.busy.get());
    assert!(root.join("created-folder").is_dir());
    editor.operation(Operation::Create(root.join("created-folder"), false));
    answer_name("created.txt");
    pump_until(|| !editor.busy.get());
    let created = editor.current().unwrap();
    assert!(root.join("created-folder/created.txt").exists());
    editor.select(original.id);
    let row = explorer_row(editor, &root.join("created-folder/created.txt"));
    row.grab_focus();
    let keys = controller::<gtk::EventControllerKey>(&row);
    assert!(keys.emit_by_name::<bool>(
        "key-pressed",
        &[&gtk::gdk::Key::F2, &0u32, &gtk::gdk::ModifierType::empty()]
    ));
    answer_name("renamed.txt");
    pump_until(|| !editor.busy.get());
    assert!(root.join("created-folder/renamed.txt").exists());
    assert_eq!(created.name(&editor.strings), "renamed.txt");
    assert_eq!(editor.active.get(), Some(original.id));
    run(editor, Intent::Close(created.id));
    editor.operation(Operation::RenameTab(original.id));
    answer_name("Draft.txt");
    pump_until(|| !editor.busy.get());
    assert_eq!(original.name(&editor.strings), "Draft.txt");
    assert!(original.document.borrow().path.is_none());
    let folder = root.join("tree");
    std::fs::create_dir(&folder).unwrap();
    let nested = folder.join("nested");
    std::fs::create_dir(&nested).unwrap();
    let path = nested.join("file.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();
    run(editor, Intent::Operation(Operation::Open(path.clone())));
    let text = editor.current().unwrap();
    editor
        .explorer
        .expanded
        .borrow_mut()
        .extend([folder.clone(), nested.clone()]);
    editor.refresh_explorer();
    capture(editor, "explorer.png");
    let buffer = text.view.buffer();
    buffer.place_cursor(&buffer.start_iter());
    // Native context action, keyboard signal and Edit menu share line semantics.
    text.view.activate_action("clipboard.copy", None).unwrap();
    let copied = glib::MainContext::default()
        .block_on(text.view.clipboard().read_text_future())
        .unwrap()
        .unwrap();
    assert_eq!(copied, "alpha\n");
    text.view.emit_by_name::<()>("cut-clipboard", &[]);
    assert_eq!(text.document.borrow().text, "beta\n");
    toggle(editor, "undo");
    assert_eq!(text.document.borrow().text, "alpha\nbeta\n");
    buffer.select_range(&buffer.start_iter(), &buffer.iter_at_offset(2));
    toggle(editor, "cut");
    assert_eq!(text.document.borrow().text, "pha\nbeta\n");
    toggle(editor, "undo");
    buffer.place_cursor(&buffer.iter_at_offset(7));
    toggle(editor, "cut");
    assert_eq!(text.document.borrow().text, "alpha\n");
    toggle(editor, "undo");
    buffer.select_range(&buffer.start_iter(), &buffer.iter_at_offset(5));
    text.view.clipboard().set_text("Ω");
    text.view.activate_action("clipboard.paste", None).unwrap();
    pump_until(|| text.document.borrow().text == "Ω\nbeta\n");
    toggle(editor, "undo");
    assert_eq!(text.document.borrow().text, "alpha\nbeta\n");
    append(&text, "dirty");
    buffer.place_cursor(&buffer.end_iter());
    text.view.copy_or_cut(true);
    assert_eq!(text.document.borrow().text, "alpha\nbeta\n");
    toggle(editor, "undo");
    assert!(text.document.borrow().text.ends_with("dirty"));
    editor.select(original.id);
    run(editor, Intent::Operation(Operation::Duplicate(text.id)));
    let duplicate = editor.current().unwrap();
    assert_eq!(duplicate.name(&editor.strings), "Untitled.txt");
    assert!(duplicate.document.borrow().path.is_none());
    assert!(duplicate.document.borrow().is_dirty());
    assert_eq!(
        duplicate.document.borrow().text,
        text.document.borrow().text
    );
    append(&duplicate, " independent");
    assert_ne!(
        duplicate.document.borrow().text,
        text.document.borrow().text
    );
    let active = editor.active.get();
    let target = controller::<gtk::DropTarget>(&original.header);
    assert!(target.emit_by_name::<bool>(
        "drop",
        &[
            &glib::BoxedValue(format!("leditor-tab:{}", duplicate.id).to_value()),
            &0.0f64,
            &0.0f64
        ]
    ));
    assert_eq!(editor.tabs.borrow()[0].id, duplicate.id);
    assert_eq!(editor.active.get(), active);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "alpha\nbeta\n");
    let destination = root.join("destination");
    std::fs::create_dir(&destination).unwrap();
    editor.refresh_explorer();
    let destination_row = explorer_row(editor, &destination);
    let target = controller::<gtk::DropTarget>(&destination_row);
    assert!(target.emit_by_name::<bool>(
        "drop",
        &[
            &glib::BoxedValue(gio::File::for_path(&folder).to_value()),
            &0.0f64,
            &0.0f64
        ]
    ));
    pump_until(|| !editor.busy.get());
    let moved = destination.join("tree/nested/file.txt");
    assert_eq!(
        text.document.borrow().path.as_deref(),
        Some(moved.as_path())
    );
    assert!(text.document.borrow().is_dirty());
    assert!(!folder.exists());
    run(editor, Intent::Operation(Operation::Open(moved.clone())));
    assert_eq!(editor.active.get(), Some(text.id));
    run(
        editor,
        Intent::Operation(Operation::Clipboard(moved.clone(), false)),
    );
    run(editor, Intent::Operation(Operation::Paste(root.to_owned())));
    assert_eq!(
        std::fs::read_to_string(root.join("file.txt")).unwrap(),
        "alpha\nbeta\n"
    );
    // Close All cancellation retains every document, even previously accepted ones.
    let count = editor.tabs.borrow().len();
    editor.operation(Operation::CloseAll);
    click("Don't save");
    click("Cancel");
    pump_until(|| !editor.busy.get());
    assert_eq!(editor.tabs.borrow().len(), count);
    editor.operation(Operation::Delete(moved.clone()));
    click("Cancel");
    pump_until(|| !editor.busy.get());
    assert!(moved.exists());
    assert!(text.document.borrow().path.is_some());
    editor.operation(Operation::Delete(moved.clone()));
    click("Delete");
    pump_until(|| !editor.busy.get());
    assert!(!moved.exists());
    assert!(text.document.borrow().path.is_none());
    assert!(text.document.borrow().is_dirty());
    assert!(text.document.borrow().text.contains("dirty"));
    run(editor, Intent::Operation(Operation::CloseFolder));
    assert!(editor.explorer.root.borrow().is_none());
    assert_eq!(editor.tabs.borrow().len(), count);
    for id in [duplicate.id, text.id] {
        editor.request(Intent::Close(id));
        click("Don't save");
        pump_until(|| !editor.busy.get());
    }
    assert_eq!(editor.tabs.borrow().len(), 1);
}

#[allow(deprecated)]
fn answer_name(name: &str) {
    let mut dialog = None;
    pump_until(|| {
        dialog = gtk::Window::list_toplevels()
            .iter()
            .filter(|w| w.is_visible())
            .find_map(|w| w.clone().downcast::<gtk::Dialog>().ok());
        dialog.is_some()
    });
    let dialog = dialog.unwrap();
    let entry = dialog
        .content_area()
        .first_child()
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap();
    entry.set_text(name);
    dialog.response(gtk::ResponseType::Ok);
}
fn capture(editor: &Editor, name: &str) {
    let Ok(directory) = std::env::var("LEDITOR_QA_DIR") else {
        return;
    };
    drain();
    let paintable = gtk::WidgetPaintable::new(Some(&editor.window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(
        &snapshot,
        f64::from(editor.window.width()),
        f64::from(editor.window.height()),
    );
    if let Some(node) = snapshot.to_node() {
        let texture = editor
            .window
            .renderer()
            .unwrap()
            .render_texture(&node, None);
        texture
            .save_to_png(std::path::Path::new(&directory).join(name))
            .unwrap();
    }
}

fn controller<T: IsA<gtk::EventController> + IsA<glib::Object> + glib::types::StaticType>(
    widget: &impl IsA<gtk::Widget>,
) -> T {
    let controllers = widget.observe_controllers();
    (0..controllers.n_items())
        .find_map(|i| controllers.item(i).unwrap().downcast::<T>().ok())
        .unwrap()
}
fn explorer_row(editor: &Editor, path: &std::path::Path) -> gtk::Widget {
    let mut child = editor.explorer.rows.first_child();
    while let Some(row) = child {
        if row.tooltip_text().as_deref() == Some(path.to_string_lossy().as_ref()) {
            return row;
        }
        child = row.next_sibling();
    }
    panic!("Missing explorer row: {}", path.display());
}
