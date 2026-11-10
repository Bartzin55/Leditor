use super::*;

fn widgets<T: IsA<gtk::Widget> + glib::types::StaticType>(widget: &gtk::Widget) -> Vec<T> {
    let mut result = Vec::new();
    if let Ok(found) = widget.clone().downcast::<T>() {
        result.push(found);
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        result.extend(widgets::<T>(&widget));
        child = widget.next_sibling();
    }
    result
}
fn capture(e: &Editor, name: &str) {
    let Ok(dir) = std::env::var("LEDITOR_QA_DIR") else {
        return;
    };
    drain();
    let paintable = gtk::WidgetPaintable::new(Some(&e.window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, e.window.width() as f64, e.window.height() as f64);
    if let Some(node) = snapshot.to_node() {
        e.window
            .renderer()
            .unwrap()
            .render_texture(&node, None)
            .save_to_png(std::path::Path::new(&dir).join(name))
            .unwrap();
    }
}
fn key(key: &str) {
    assert!(std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", key])
        .status()
        .unwrap()
        .success());
    drain();
}

#[test]
#[ignore = "Run separately with a graphical X11 display and xdotool"]
fn version_180_preview_read_only_input_and_duplicate() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp.path());
    std::env::set_var("XDG_DATA_HOME", temp.path());
    std::env::set_var("GSETTINGS_BACKEND", "memory");
    let _ = std::fs::remove_dir_all(session::directory());
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.Features180Test")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(gio::Cancellable::NONE).unwrap();
    let e = create(&app);
    let original_default = gtk::Settings::default().unwrap().gtk_im_module();
    let path = temp.path().join("Welcome.md");
    let content = "# Leditor 1.8.0\n\n**Markdown preview** with *Comrak* and ==highlighted text==.\n\n[Go to details](#details) · :smile: · H~2~O · x^2^\n\n| Feature | Status |\n| :--- | ---: |\n| Tables | Ready |\n| Native GTK | Ready |\n\n- [x] Read-only preview\n- [ ] Continue editing\n\n> [!NOTE]\n> Your source text stays unchanged.\n\n```rust\nfn main() {\n    println!(\"Olá, mundo!\");\n}\n```\n\n## Details {#details}\n\nA footnote[^one].\n\n[^one]: Preserved text.\n\nTerm\n\n: A definition.\n";
    let content = format!(
        "{content}\n![Leditor icon](leditor.png)\n\n![Remote](https://example.invalid/image.png)\n"
    );
    std::fs::write(
        temp.path().join("leditor.png"),
        include_bytes!("../assets/leditor.png"),
    )
    .unwrap();
    std::fs::write(&path, &content).unwrap();
    let tab = e.tab(e.add_tab(Document::open(&path).unwrap())).unwrap();
    let b = tab.source_buffer();
    append(&tab, "\nunsaved text  \n");
    b.place_cursor(&b.iter_at_offset(12));
    let text = tab.document.borrow().text.clone();
    let caret = b.cursor_position();
    assert!(b.can_undo());
    e.minimap.set(true);
    e.trim_trailing.set(true);
    e.open_search();
    e.view_button.emit_clicked();
    assert!(e.only_view.get());
    assert!(tab.is_preview());
    assert!(!tab.view.is_editable());
    assert!(!tab.minimap.is_visible());
    assert!(!e.search.bar.is_visible());
    assert_eq!(
        e.view_button.icon_name().as_deref(),
        Some("document-edit-symbolic")
    );
    assert!(!e.actions["undo"].is_enabled());
    assert!(!e.actions["find-replace"].is_enabled());
    assert!(!e.actions["case-upper"].is_enabled());
    let preview_borrow = tab.preview.borrow();
    let preview = preview_borrow.as_ref().unwrap();
    let preview_labels = labels(preview.content.upcast_ref());
    assert!(
        preview_labels.iter().any(|label| label == "Leditor 1.8.0"),
        "{preview_labels:?}"
    );
    assert!(preview_labels
        .iter()
        .any(|label| label.contains("Markdown preview")));
    assert!(preview_labels.iter().any(|label| label == "Details"));
    assert_eq!(widgets::<gtk::Grid>(preview.content.upcast_ref()).len(), 1);
    assert_eq!(
        widgets::<sourceview5::View>(preview.content.upcast_ref()).len(),
        1
    );
    for view in widgets::<sourceview5::View>(preview.content.upcast_ref()) {
        assert!(!view.is_editable());
        assert!(view
            .buffer()
            .downcast::<sourceview5::Buffer>()
            .unwrap()
            .language()
            .is_some());
    }
    assert_eq!(
        widgets::<gtk::Picture>(preview.content.upcast_ref()).len(),
        1
    );
    assert!(preview_labels
        .iter()
        .any(|label| label.contains("Remote image")));
    let heading = widgets::<gtk::Label>(preview.content.upcast_ref())
        .into_iter()
        .find(|label| label.text() == "Leditor 1.8.0")
        .unwrap();
    let link = widgets::<gtk::Label>(preview.content.upcast_ref())
        .into_iter()
        .find(|label| label.text().contains("Go to details"))
        .unwrap();
    // Drop the preview borrow before actions that can replace it.
    drop(preview_borrow);
    // Source buffer commands must not modify hidden Markdown.
    for name in [
        "undo",
        "redo",
        "cut",
        "paste",
        "case-upper",
        "insert-above",
        "insert-below",
        "sort-lines",
        "sort-characters",
    ] {
        e.actions[name].activate(None);
    }
    e.edit_transform("case-upper");
    e.sort_selection(true);
    e.trim_for_save(&tab, false);
    assert_eq!(tab.document.borrow().text, text);
    assert_eq!(b.cursor_position(), caret);
    e.theme.replace("dark".into());
    e.apply_theme();
    drain();
    capture(&e, "v180-preview-dark.png");
    e.theme.replace("light".into());
    e.apply_theme();
    capture(&e, "v180-preview-light.png");
    link.emit_by_name::<bool>("activate-link", &[&"#details"]);
    drain();
    assert!(tab.preview_scroll.vadjustment().value() > 0.0);
    tab.preview_scroll.vadjustment().set_value(0.0);
    heading.grab_focus();
    heading.select_region(0, -1);
    e.actions["copy"].activate(None);
    let copied = glib::MainContext::default()
        .block_on(e.window.clipboard().read_text_future())
        .unwrap();
    assert_eq!(copied.as_deref(), Some("Leditor 1.8.0"));
    heading.select_region(0, 0);
    e.set_zoom(125.0);
    assert!(tab.is_preview());
    e.set_zoom(100.0);
    // Undo history, exact text, caret and dirty state survive a preview roundtrip.
    toggle(&e, "only-view");
    assert!(!tab.is_preview());
    assert!(tab.view.is_editable());
    assert!(tab.minimap.is_visible());
    assert_eq!(tab.document.borrow().text, text);
    assert_eq!(b.cursor_position(), caret);
    assert!(tab.document.borrow().is_dirty());
    assert!(b.can_undo());
    capture(&e, "v180-editor-toolbar.png");

    // Plain text stays visible but cannot be edited through native or app paths.
    let plain = temp.path().join("plain.txt");
    std::fs::write(&plain, "hello  \nworld").unwrap();
    let plain_tab = e.tab(e.add_tab(Document::open(&plain).unwrap())).unwrap();
    append(&plain_tab, "!");
    let plain_before = plain_tab.document.borrow().text.clone();
    toggle(&e, "only-view");
    assert!(!plain_tab.is_preview());
    assert!(!plain_tab.view.is_editable());
    pump_until(|| e.window.is_active());
    plain_tab.view.grab_focus();
    for k in [
        "a",
        "BackSpace",
        "Return",
        "Tab",
        "ctrl+z",
        "ctrl+y",
        "ctrl+x",
        "ctrl+v",
    ] {
        key(k);
    }
    assert_eq!(plain_tab.document.borrow().text, plain_before);
    e.open_search();
    e.search.find.set_text("hello");
    e.search.replacement.set_text("bad");
    e.search.replace_all.emit_clicked();
    assert!(!e.search.replace.is_sensitive());
    assert_eq!(plain_tab.document.borrow().text, plain_before);
    e.close_search();

    // Saving existing edits in view-only mode must not trim or alter them.
    run(&e, Intent::Save(plain_tab.id));
    assert_eq!(std::fs::read_to_string(&plain).unwrap(), plain_before);
    toggle(&e, "only-view");

    // Runtime IM switch affects existing/new tabs, search and dialog entries.
    e.actions["input-method"].activate(Some(&"simple".to_variant()));
    assert!(e.input_method.is_simple());
    assert_eq!(
        plain_tab.view.im_module().as_deref(),
        Some(crate::input_method::SIMPLE_CONTEXT)
    );
    for entry in widgets::<gtk::Text>(e.search.bar.upcast_ref()) {
        assert_eq!(
            entry.im_module().as_deref(),
            Some(crate::input_method::SIMPLE_CONTEXT)
        );
    }
    let fresh = e.tab(e.add_tab(Document::default())).unwrap();
    assert_eq!(
        fresh.view.im_module().as_deref(),
        Some(crate::input_method::SIMPLE_CONTEXT)
    );
    fresh.view.grab_focus();
    key("dead_acute");
    key("a");
    key("dead_tilde");
    key("o");
    assert_eq!(fresh.document.borrow().text, "áõ");
    e.open_search();
    e.search.find.set_text("");
    e.search.find.grab_focus();
    key("dead_acute");
    key("e");
    assert_eq!(e.search.find.text(), "é");
    e.close_search();
    let dialog = gtk::Window::builder().transient_for(&e.window).build();
    let entry = gtk::Entry::new();
    dialog.set_child(Some(&entry));
    dialog.present();
    drain();
    let delegate = widgets::<gtk::Text>(entry.upcast_ref()).remove(0);
    assert_eq!(
        delegate.im_module().as_deref(),
        Some(crate::input_method::SIMPLE_CONTEXT)
    );
    dialog.destroy();
    let edits = fresh.document.borrow().text.clone();
    e.actions["input-method"].activate(Some(&"default".to_variant()));
    assert!(fresh.view.im_module().is_none());
    assert_eq!(
        gtk::Settings::default().unwrap().gtk_im_module(),
        original_default
    );
    assert_eq!(fresh.document.borrow().text, edits);
    e.actions["input-method"].activate(Some(&"simple".to_variant()));

    // Explorer duplication writes bytes to disk; existing unsaved tabs survive.
    e.load_folder(temp.path().to_owned());
    run(
        &e,
        Intent::Operation(Operation::DuplicateEntry(path.clone())),
    );
    let copy = temp.path().join("Welcome (Copy 1).md");
    assert_eq!(std::fs::read_to_string(copy).unwrap(), content);
    assert_eq!(tab.document.borrow().text, text);

    // Preference persists independently of session restore; recovery retains text.
    e.select(tab.id);
    toggle(&e, "only-view");
    e.save_settings();
    let settings = session::load_settings().unwrap();
    assert!(settings.boolean("settings", "only-view").unwrap());
    assert_eq!(
        settings.string("settings", "input-method").unwrap(),
        "simple"
    );
    e.autosave.set(true);
    e.restore.set(true);
    e.save_settings();
    e.save_recovery().unwrap();
    e.window.destroy();
    drop(e);
    let reopened = create(&app);
    assert!(reopened.only_view.get());
    assert!(reopened.input_method.is_simple());
    let restored = reopened
        .tabs
        .borrow()
        .iter()
        .find(|t| t.document.borrow().path.as_deref() == Some(path.as_path()))
        .unwrap()
        .clone();
    assert!(restored.is_preview());
    assert_eq!(restored.document.borrow().text, text);
    toggle(&reopened, "only-view");
    reopened.actions["input-method"].activate(Some(&"default".to_variant()));
    reopened.window.destroy();
    drop(reopened);
    let reopened = create(&app);
    assert!(!reopened.only_view.get());
    assert!(!reopened.input_method.is_simple());
    assert!(!reopened.current().unwrap().is_preview());
    reopened.window.destroy();
}
