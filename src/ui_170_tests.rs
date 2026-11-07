use super::*;

fn editor() -> (tempfile::TempDir, Rc<Editor>) {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp.path());
    std::env::set_var("XDG_DATA_HOME", temp.path());
    std::env::set_var("GSETTINGS_BACKEND", "memory");
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.Features170Test")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(gio::Cancellable::NONE).unwrap();
    let e = create(&app);
    (temp, e)
}

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

#[test]
#[ignore = "Run separately with a graphical display"]
fn version_170_tabs_and_bracket_styles() {
    let (temp, e) = editor();
    let first = e.current().unwrap();
    assert!(first.icon.gicon().is_some());
    let names = [
        "notes.txt",
        "main.rs",
        "picture.png",
        "manual.pdf",
        "unknown.leditor-test-extension",
        "ação_東京.txt",
        "this_file_contains_a_very_very_very_very_long_title.txt",
    ];
    let mut tabs = Vec::new();
    for name in names {
        let path = temp.path().join(name);
        std::fs::write(&path, "(one) [two] {three}").unwrap();
        tabs.push(e.tab(e.add_tab(Document::open(&path).unwrap())).unwrap());
    }
    e.load_folder(temp.path().to_owned());
    drain();
    for tab in &tabs {
        let path = tab.document.borrow().path.clone().unwrap();
        let row = explorer_row(&e, &path);
        let explorer_icon = widgets::<gtk::Image>(&row)
            .into_iter()
            .find_map(|image| image.gicon())
            .unwrap();
        assert!(tab.icon.gicon().unwrap().equal(Some(&explorer_icon)));
        assert_eq!(tab.icon.pixel_size(), 16);
        assert!(tab.title.width() >= tab.title.width_request());
        assert!(tab.close.width() > 0 && tab.dirty.width() > 0);
        assert_eq!(tab.header.tooltip_text().as_deref(), path.to_str());
    }
    assert!(!tabs[0].title.layout().is_ellipsized());
    assert!(!tabs[5].title.layout().is_ellipsized());
    assert!(tabs.last().unwrap().title.layout().is_ellipsized());
    assert_eq!(tabs[0].title.ellipsize(), gtk::pango::EllipsizeMode::Middle);
    let widths: Vec<_> = tabs.iter().map(|tab| tab.header.width()).collect();
    let window_width = e.window.width();
    for i in 0..30 {
        let id = e.add_tab(Document::default());
        e.tab(id)
            .unwrap()
            .display_name
            .replace(Some(format!("file_{i}.txt")));
    }
    e.refresh();
    drain();
    assert_eq!(e.window.width(), window_width);
    for (tab, width) in tabs.iter().zip(&widths) {
        assert_eq!(
            tab.header.width(),
            *width,
            "tabs must not collapse under pressure"
        );
    }
    let scroller = widgets::<gtk::ScrolledWindow>(e.tab_bar.upcast_ref())
        .into_iter()
        .next()
        .unwrap();
    assert!(scroller.hadjustment().upper() > scroller.hadjustment().page_size());
    let long = tabs.last().unwrap();
    let position = long
        .header
        .compute_point(&e.tabs_box, &gtk::graphene::Point::zero())
        .unwrap();
    scroller.hadjustment().set_value(position.x() as f64);
    capture(&e, "170-long-title.png");
    scroller.hadjustment().set_value(0.0);
    append(&tabs[0], " edited");
    drain();
    assert_eq!(tabs[0].dirty.opacity(), 1.0);
    assert_eq!(tabs[0].header.width(), widths[0]);

    // Save As refreshes both the icon and the title using the new extension.
    let renamed = temp.path().join("renamed.html");
    queue(&e, true, Some(&renamed));
    run(&e, Intent::Operation(Operation::SaveTab(tabs[0].id, true)));
    assert_eq!(tabs[0].title.text(), "renamed.html");
    assert!(tabs[0]
        .icon
        .gicon()
        .unwrap()
        .equal(Some(&crate::file_icon::for_path(&renamed))));
    e.select(tabs[1].id);
    let buffer = tabs[1].source_buffer();
    e.choose_syntax("plain");
    let matched = Rc::new(Cell::new(false));
    let result = matched.clone();
    buffer.connect_bracket_matched(move |_, _, kind| {
        result.set(kind == sourceview5::BracketMatchType::Found);
    });
    for theme in ["dark", "light"] {
        e.theme.replace(theme.into());
        e.apply_theme();
        let style = buffer
            .style_scheme()
            .unwrap()
            .style("bracket-match")
            .unwrap();
        assert!(style.is_bold());
        assert_eq!(style.pango_underline(), gtk::pango::Underline::Single);
        let color = gtk::gdk::RGBA::parse(style.underline_color().unwrap()).unwrap();
        assert_eq!(color, gtk::gdk::RGBA::parse("#86ee77").unwrap());
        for text in ["(one)", "[two]", "{three}"] {
            matched.set(false);
            buffer.set_text(text);
            buffer.place_cursor(&buffer.iter_at_offset(1));
            pump_until(|| matched.get());
            for offset in [0, text.len() as i32 - 1] {
                assert!(buffer.iter_at_offset(offset).tags().iter().any(|tag| {
                    tag.property::<gtk::pango::Underline>("underline")
                        == gtk::pango::Underline::Single
                        && tag.property::<i32>("weight") >= 700
                }));
            }
        }
        capture(&e, &format!("170-tabs-brackets-{theme}.png"));
    }
    e.actions["bracket-matching"].activate(None);
    drain();
    assert!(!buffer.is_highlight_matching_brackets());
    assert!(!buffer
        .start_iter()
        .tags()
        .iter()
        .any(|tag| tag.property::<gtk::pango::Underline>("underline")
            == gtk::pango::Underline::Single));
    for (_, label) in crate::shortcuts::TEXT_SHORTCUTS {
        assert_ne!(e.strings.text(label), *label);
    }
    e.window.destroy();
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "Run separately under X11 with xdotool and a window manager"]
fn version_170_native_completion() {
    fn key(value: &str) {
        assert!(std::process::Command::new("xdotool")
            .args(["key", "--clearmodifiers", value])
            .status()
            .unwrap()
            .success());
        drain();
    }
    fn popup(tab: &Tab) -> Option<gtk::Popover> {
        widgets::<gtk::Popover>(tab.view.upcast_ref())
            .into_iter()
            .find(|popover| popover.is_mapped())
    }
    fn request(tab: &Tab) {
        // Re-indexing an edited line is asynchronous too. Ask again after the
        // native provider has had a chance to scan; an empty request is valid.
        let mut retry = Instant::now();
        pump_until(|| {
            if popup(tab).is_some() {
                return true;
            }
            if Instant::now() >= retry {
                key("ctrl+space");
                retry = Instant::now() + Duration::from_millis(250);
            }
            false
        });
    }
    let (_temp, e) = editor();
    let first = e.current().unwrap();
    let buffer = first.source_buffer();
    buffer.set_text("cedaralpha cedaromega\n");
    let other = e.tab(e.add_tab(Document::default())).unwrap();
    other.source_buffer().set_text("zebrabridge coração\n");
    e.select(first.id);
    pump_until(|| e.window.is_active());
    first.view.grab_focus();
    // GtkSourceCompletionWords indexes asynchronously after an idle delay.
    let scanned = Instant::now() + Duration::from_secs(6);
    pump_until(|| Instant::now() >= scanned);
    buffer.place_cursor(&buffer.end_iter());
    for letter in ["c", "e", "d"] {
        key(letter);
    }
    pump_until(|| popup(&first).is_some());
    let suggestions = labels(popup(&first).unwrap().upcast_ref());
    assert!(
        suggestions.iter().any(|label| label == "cedaralpha"),
        "{suggestions:?}"
    );
    assert!(!suggestions.iter().any(|label| label == "zebrabridge"));
    if let Ok(directory) = std::env::var("LEDITOR_QA_DIR") {
        let popover = popup(&first).unwrap();
        let paintable = gtk::WidgetPaintable::new(Some(&popover));
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, popover.width() as f64, popover.height() as f64);
        if let Some(node) = snapshot.to_node() {
            popover
                .renderer()
                .unwrap()
                .render_texture(&node, None)
                .save_to_png(std::path::Path::new(&directory).join("170-completion.png"))
                .unwrap();
        }
    }
    key("Down");
    key("Up");
    key("Return");
    assert_eq!(
        first.document.borrow().text,
        "cedaralpha cedaromega\ncedaralpha"
    );
    assert!(popup(&first).is_none());
    key("ctrl+z");
    assert_eq!(first.document.borrow().text, "cedaralpha cedaromega\nced");
    key("ctrl+shift+z");
    assert_eq!(
        first.document.borrow().text,
        "cedaralpha cedaromega\ncedaralpha"
    );

    buffer.delete(&mut buffer.iter_at_line(1).unwrap(), &mut buffer.end_iter());
    buffer.insert_at_cursor("ced");
    request(&first);
    key("Tab");
    assert_eq!(
        first.document.borrow().text,
        "cedaralpha cedaromega\ncedaralpha"
    );
    key("ctrl+z");
    request(&first);
    key("Escape");
    assert!(popup(&first).is_none());
    assert_eq!(first.document.borrow().text, "cedaralpha cedaromega\nced");

    // The other document has its own provider; words cannot leak between tabs.
    e.select(other.id);
    other
        .source_buffer()
        .place_cursor(&other.source_buffer().end_iter());
    other.source_buffer().insert_at_cursor("zebr");
    request(&other);
    key("Return");
    assert_eq!(
        other.document.borrow().text,
        "zebrabridge coração\nzebrabridge"
    );
    other.source_buffer().insert_at_cursor("\ncora");
    request(&other);
    key("Return");
    assert!(other.document.borrow().text.ends_with("\ncoração"));
    e.select(first.id);
    buffer.delete(&mut buffer.iter_at_line(1).unwrap(), &mut buffer.end_iter());
    buffer.insert_at_cursor("zebr");
    key("ctrl+space");
    drain();
    assert!(popup(&first).is_none());
    assert_eq!(first.document.borrow().text, "cedaralpha cedaromega\nzebr");

    // Enter/Tab retain indentation when no suggestion is being accepted.
    buffer.set_text("\t  standalone");
    buffer.place_cursor(&buffer.end_iter());
    key("Return");
    assert_eq!(first.document.borrow().text, "\t  standalone\n\t  ");
    key("Tab");
    assert_eq!(first.document.borrow().text, "\t  standalone\n\t  \t");
    key("ctrl+g");
    assert!(e.goto.bar.is_visible());
    key("Escape");
    assert!(!e.goto.bar.is_visible());
    key("ctrl+f");
    assert!(e.search.bar.is_visible());
    key("Escape");
    assert!(!e.search.bar.is_visible());
    let closed_id = other.id;
    let content = other.content.downgrade();
    e.remove_tab(closed_id);
    drop(other);
    drain();
    assert!(e.tab(closed_id).is_none());
    assert!(content.upgrade().is_none());
    assert_eq!(e.active.get(), Some(first.id));
    e.window.destroy();
}
