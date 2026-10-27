use super::*;

pub(super) struct SearchBar {
    pub bar: gtk::Box,
    pub find: gtk::Entry,
    pub replacement: gtk::Entry,
    case: gtk::CheckButton,
    previous: gtk::Button,
    next: gtk::Button,
    pub replace: gtk::Button,
    pub replace_all: gtk::Button,
    close: gtk::Button,
    status: gtk::Label,
    matches: RefCell<Vec<(i32, i32)>>,
    current: Cell<Option<usize>>,
    updating: Cell<bool>,
}
impl SearchBar {
    pub fn new(strings: &I18n) -> Self {
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        bar.add_css_class("search-bar");
        let grid = gtk::Grid::builder()
            .column_spacing(4)
            .row_spacing(4)
            .build();
        let find = gtk::Entry::builder()
            .placeholder_text(strings.text("find"))
            .width_chars(18)
            .build();
        let replacement = gtk::Entry::builder()
            .placeholder_text(strings.text("replace"))
            .width_chars(18)
            .build();
        find.set_tooltip_text(Some(&strings.text("find")));
        replacement.set_tooltip_text(Some(&strings.text("replace")));
        let previous = gtk::Button::with_label("↑");
        let next = gtk::Button::with_label("↓");
        previous.set_tooltip_text(Some(&strings.text("previous-match")));
        next.set_tooltip_text(Some(&strings.text("next-match")));
        let replace = gtk::Button::with_label(&strings.text("replace"));
        let replace_all = gtk::Button::with_label(&strings.text("replace-all"));
        grid.attach(&find, 0, 0, 1, 1);
        grid.attach(&previous, 1, 0, 1, 1);
        grid.attach(&next, 2, 0, 1, 1);
        grid.attach(&replacement, 0, 1, 1, 1);
        grid.attach(&replace, 1, 1, 1, 1);
        grid.attach(&replace_all, 2, 1, 1, 1);
        let status = gtk::Label::builder()
            .hexpand(true)
            .xalign(0.0)
            .wrap(true)
            .build();
        let case = gtk::CheckButton::with_label(&strings.text("case-sensitive"));
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.set_tooltip_text(Some(&strings.text("close-search")));
        bar.append(&grid);
        bar.append(&status);
        bar.append(&case);
        bar.append(&close);
        bar.set_visible(false);
        Self {
            bar,
            find,
            replacement,
            case,
            previous,
            next,
            replace,
            replace_all,
            close,
            status,
            matches: RefCell::new(Vec::new()),
            current: Cell::new(None),
            updating: Cell::new(false),
        }
    }
}
impl Editor {
    pub(super) fn setup_search(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.search.find.connect_changed(move |_| {
            if let Some(e) = weak.upgrade() {
                e.refresh_search();
                e.paint_current_match(true);
            }
        });
        let weak = Rc::downgrade(self);
        self.search.case.connect_toggled(move |_| {
            if let Some(e) = weak.upgrade() {
                e.refresh_search();
                e.paint_current_match(true);
            }
        });
        for (button, direction) in [(&self.search.next, 1), (&self.search.previous, -1)] {
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(e) = weak.upgrade() {
                    e.navigate_search(direction);
                }
            });
        }
        for (button, all) in [
            (&self.search.replace, false),
            (&self.search.replace_all, true),
        ] {
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(e) = weak.upgrade() {
                    e.replace_matches(all);
                }
            });
        }
        let weak = Rc::downgrade(self);
        self.search.close.connect_clicked(move |_| {
            if let Some(e) = weak.upgrade() {
                e.close_search();
            }
        });
        for (entry, other, is_find) in [
            (&self.search.find, &self.search.replacement, true),
            (&self.search.replacement, &self.search.find, false),
        ] {
            let keys = gtk::EventControllerKey::new();
            keys.set_propagation_phase(gtk::PropagationPhase::Capture);
            let other = other.downgrade();
            let weak = Rc::downgrade(self);
            keys.connect_key_pressed(move |_, key, _, state| {
                use gtk::gdk::{Key, ModifierType};
                if key == Key::Tab || key == Key::ISO_Left_Tab {
                    if let Some(other) = other.upgrade() {
                        other.grab_focus();
                    }
                    return glib::Propagation::Stop;
                }
                if is_find
                    && (key == Key::Return || key == Key::KP_Enter)
                    && !state.intersects(ModifierType::CONTROL_MASK | ModifierType::ALT_MASK)
                {
                    if let Some(e) = weak.upgrade() {
                        e.navigate_search(if state.contains(ModifierType::SHIFT_MASK) {
                            -1
                        } else {
                            1
                        });
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            entry.add_controller(keys);
        }
    }
    pub(super) fn open_search(&self) {
        self.search.case.set_active(false);
        self.search.bar.set_visible(true);
        self.refresh_search();
        self.paint_current_match(true);
        self.search.find.grab_focus();
    }
    pub(super) fn close_search(&self) {
        self.search.bar.set_visible(false);
        self.clear_search_tags();
        self.search.matches.borrow_mut().clear();
        self.search.current.set(None);
        if let Some(tab) = self.current() {
            tab.view.grab_focus();
        }
    }
    fn clear_search_tags(&self) {
        for tab in self.tabs.borrow().iter() {
            let b = tab.view.buffer();
            for name in ["search-match", "search-current"] {
                if let Some(tag) = b.tag_table().lookup(name) {
                    b.remove_tag(&tag, &b.start_iter(), &b.end_iter());
                }
            }
        }
    }
    fn search_status(&self, text: &str, style: &str) {
        for class in ["search-success", "search-error", "search-replaced"] {
            self.search.status.remove_css_class(class);
        }
        if !style.is_empty() {
            self.search.status.add_css_class(style);
        }
        self.search.status.set_text(text);
    }
    pub(super) fn refresh_search(&self) {
        if self.search.updating.get() || !self.search.bar.is_visible() {
            return;
        }
        self.search.updating.set(true);
        self.clear_search_tags();
        let term = self.search.find.text();
        let mut matches = Vec::new();
        if let Some(tab) = self.current().filter(|t| !t.unsupported.get()) {
            let b = tab.view.buffer();
            for (name, color) in [("search-match", "#ffe066"), ("search-current", "#66dd77")] {
                if b.tag_table().lookup(name).is_none() {
                    b.create_tag(
                        Some(name),
                        &[("background", &color), ("foreground", &"#111111")],
                    );
                }
            }
            if !term.is_empty() {
                let flags = if self.search.case.is_active() {
                    gtk::TextSearchFlags::TEXT_ONLY
                } else {
                    gtk::TextSearchFlags::TEXT_ONLY | gtk::TextSearchFlags::CASE_INSENSITIVE
                };
                let mut iter = b.start_iter();
                while let Some((start, end)) = iter.forward_search(&term, flags, None) {
                    if end.offset() <= iter.offset() {
                        break;
                    }
                    b.apply_tag_by_name("search-match", &start, &end);
                    matches.push((start.offset(), end.offset()));
                    iter = end;
                }
            }
        }
        let cursor = self
            .current()
            .map(|t| t.view.buffer().cursor_position())
            .unwrap_or(0);
        let current = if matches.is_empty() {
            None
        } else {
            Some(
                matches
                    .iter()
                    .position(|&(s, e)| s >= cursor || (s <= cursor && cursor < e))
                    .unwrap_or(0),
            )
        };
        self.search.matches.replace(matches);
        self.search.current.set(current);
        self.search.updating.set(false);
        self.paint_current_match(false);
    }
    fn paint_current_match(&self, navigate: bool) {
        let matches = self.search.matches.borrow();
        let current = self.search.current.get();
        if let Some(tab) = self.current() {
            let b = tab.view.buffer();
            if let Some(tag) = b.tag_table().lookup("search-current") {
                b.remove_tag(&tag, &b.start_iter(), &b.end_iter());
            }
            if let Some(&(start, end)) = current.and_then(|i| matches.get(i)) {
                let mut start = b.iter_at_offset(start);
                let end = b.iter_at_offset(end);
                b.apply_tag_by_name("search-current", &start, &end);
                if navigate {
                    self.search.updating.set(true);
                    b.place_cursor(&start);
                    self.search.updating.set(false);
                    tab.view.scroll_to_iter(&mut start, 0.15, true, 0.5, 0.5);
                }
            }
        }
        if self.search.find.text().is_empty() {
            self.search_status(&self.strings.text("search-prompt"), "");
        } else if matches.is_empty() {
            self.search_status(&self.strings.text("no-matches"), "search-error");
        } else {
            let mut args = FluentArgs::new();
            args.set("current", (current.unwrap_or(0) + 1) as i32);
            args.set("total", matches.len() as i32);
            self.search_status(
                &self.strings.format("match-count", Some(&args)),
                "search-success",
            );
        }
    }
    pub(super) fn search_cursor_changed(&self) {
        if self.search.updating.get() || !self.search.bar.is_visible() {
            return;
        }
        let Some(tab) = self.current() else { return };
        let cursor = tab.view.buffer().cursor_position();
        let index = self
            .search
            .matches
            .borrow()
            .iter()
            .position(|&(s, e)| s <= cursor && cursor < e);
        if let Some(index) = index {
            if self.search.current.replace(Some(index)) != Some(index) {
                self.paint_current_match(false);
            }
        }
    }
    fn navigate_search(&self, direction: i32) {
        let count = self.search.matches.borrow().len();
        if count == 0 {
            return;
        }
        let index = (self.search.current.get().unwrap_or(0) as i64 + direction as i64)
            .rem_euclid(count as i64) as usize;
        self.search.current.set(Some(index));
        self.paint_current_match(true);
    }
    fn replace_matches(&self, all: bool) {
        if self.busy.get() {
            return;
        }
        let Some(tab) = self
            .current()
            .filter(|t| !t.unsupported.get() && t.view.is_editable())
        else {
            return;
        };
        let matches = self.search.matches.borrow().clone();
        let selected: Vec<_> = if all {
            matches
        } else {
            self.search
                .current
                .get()
                .and_then(|i| matches.get(i).copied())
                .into_iter()
                .collect()
        };
        if selected.is_empty() {
            return;
        }
        let count = selected.len();
        let text = self.search.replacement.text();
        let b = tab.view.buffer();
        self.search.updating.set(true);
        b.begin_user_action();
        for (start, end) in selected.into_iter().rev() {
            let mut s = b.iter_at_offset(start);
            let mut e = b.iter_at_offset(end);
            b.delete(&mut s, &mut e);
            b.insert(&mut s, &text);
            b.place_cursor(&s);
        }
        b.end_user_action();
        self.search.updating.set(false);
        self.refresh_search();
        let mut args = FluentArgs::new();
        args.set("total", count as i32);
        self.search_status(
            &self.strings.format(
                if all { "all-replaced" } else { "one-replaced" },
                Some(&args),
            ),
            "search-replaced",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pump() {
        let end = std::time::Instant::now() + std::time::Duration::from_millis(120);
        while std::time::Instant::now() < end {
            let context = glib::MainContext::default();
            while context.pending() {
                context.iteration(false);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    fn capture(e: &Editor, name: &str) {
        let Ok(dir) = std::env::var("LEDITOR_QA_DIR") else {
            return;
        };
        pump();
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
    #[test]
    #[ignore = "Run separately with a graphical display to keep GTK on one test thread"]
    fn find_replace_focus_and_minimap() {
        let temp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        std::env::set_var("XDG_DATA_HOME", temp.path());
        std::env::set_var("GSETTINGS_BACKEND", "memory");
        let _ = std::fs::remove_dir_all(crate::session::directory());
        gtk::init().unwrap();
        let app = gtk::Application::builder()
            .application_id("io.github.leditor.V140Tests")
            .flags(gio::ApplicationFlags::NON_UNIQUE)
            .build();
        app.register(None::<&gio::Cancellable>).unwrap();
        let e = create(&app);
        let tab = e.current().unwrap();
        let b = tab.view.buffer();
        assert!(!e.minimap.get());
        assert!(!e.focus_mode.get());
        assert!(!e.always_on_top.get());
        b.set_text("ação AÇÃO ação\nsecond line\n");
        b.place_cursor(&b.start_iter());
        e.open_search();
        e.search.find.set_text("ação");
        assert_eq!(e.search.matches.borrow().len(), 3);
        assert_eq!(e.search.current.get(), Some(0));
        e.navigate_search(1);
        assert_eq!(b.cursor_position(), 5);
        e.navigate_search(-1);
        assert_eq!(b.cursor_position(), 0);
        e.navigate_search(-1);
        assert_eq!(b.cursor_position(), 10); // wrap
        e.search.case.set_active(true);
        assert_eq!(e.search.matches.borrow().len(), 2);
        e.open_search();
        assert!(!e.search.case.is_active());
        assert_eq!(e.search.matches.borrow().len(), 3);
        e.search.replacement.set_text("x");
        e.replace_matches(false);
        assert!(tab.document.borrow().text.contains('x'));
        b.undo();
        assert_eq!(tab.document.borrow().text, "ação AÇÃO ação\nsecond line\n");
        e.replace_matches(true);
        assert_eq!(tab.document.borrow().text, "x x x\nsecond line\n");
        b.undo();
        assert_eq!(tab.document.borrow().text, "ação AÇÃO ação\nsecond line\n");
        b.redo();
        assert_eq!(tab.document.borrow().text, "x x x\nsecond line\n");
        e.search.find.set_text("not-present");
        assert!(e.search.matches.borrow().is_empty());
        assert!(e.search.status.has_css_class("search-error"));
        e.search.find.set_text("");
        assert!(e.search.matches.borrow().is_empty());
        e.search.find.set_text("x\nsecond");
        assert_eq!(e.search.matches.borrow().len(), 1);
        // Replacement contains the search string: only the original matches are replaced.
        e.search.find.set_text("x");
        e.search.replacement.set_text("xx");
        e.replace_matches(true);
        assert_eq!(tab.document.borrow().text, "xx xx xx\nsecond line\n");
        b.undo();
        e.actions["minimap"].activate(None);
        assert!(tab.minimap.is_visible());
        b.set_text(
            &"fn example() {\n    let value = 42;\n    println!(\"value\");\n}\n".repeat(60),
        );
        e.search.find.set_text("value");
        pump();
        let controllers = tab.minimap.observe_controllers();
        let click = (0..controllers.n_items())
            .find_map(|i| {
                controllers
                    .item(i)
                    .unwrap()
                    .downcast::<gtk::GestureClick>()
                    .ok()
            })
            .unwrap();
        click.emit_by_name::<()>("pressed", &[&1i32, &20.0f64, &100.0f64]);
        assert!(b.iter_at_offset(b.cursor_position()).line() > 0);
        e.open_search();
        e.theme.replace("dark".into());
        e.apply_theme();
        pump();
        capture(&e, "v140-dark.png");
        e.theme.replace("light".into());
        e.apply_theme();
        capture(&e, "v140-light.png");
        e.explorer.panel.set_visible(true);
        e.explorer.reveal.set_visible(false);
        e.toggle_focus();
        assert!(!e.menubar.is_visible());
        assert!(!e.tab_bar.is_visible());
        assert!(!e.footer.is_visible());
        assert!(!e.explorer.panel.is_visible());
        let root = e.window.child().unwrap();
        let controllers = root.observe_controllers();
        let motion = (0..controllers.n_items())
            .find_map(|i| {
                controllers
                    .item(i)
                    .unwrap()
                    .downcast::<gtk::EventControllerMotion>()
                    .ok()
            })
            .unwrap();
        motion.emit_by_name::<()>("motion", &[&1.0f64, &1.0f64]);
        assert!(e.menubar.is_visible());
        assert!(e.tab_bar.is_visible());
        motion.emit_by_name::<()>("motion", &[&1.0f64, &600.0f64]);
        assert!(!e.menubar.is_visible());
        assert!(!e.tab_bar.is_visible());
        e.save_settings();
        let settings = crate::session::load_settings().unwrap();
        assert!(settings.boolean("settings", "explorer").unwrap());
        assert!(settings.boolean("settings", "numbers").unwrap());
        assert!(settings.boolean("settings", "minimap").unwrap());
        assert!(!settings.has_key("settings", "focus").unwrap());
        assert!(!settings.has_key("settings", "always-on-top").unwrap());
        assert!(!settings.has_key("settings", "case-sensitive").unwrap());
        e.toggle_focus();
        assert!(e.explorer.panel.is_visible());
        assert!(e.menubar.is_visible());
        assert!(e.footer.is_visible());
        let first = tab.id;
        let second = e.add_tab(Document::default());
        assert!(e.search.matches.borrow().is_empty());
        e.select(first);
        assert_eq!(e.search.matches.borrow().len(), 120);
        e.remove_tab(second);
        e.close_search();
        assert!(!e.search.bar.is_visible());
        let tag = b.tag_table().lookup("search-match").unwrap();
        assert!(!b.start_iter().has_tag(&tag));
        e.window.destroy();
        let _ = std::fs::remove_dir_all(crate::session::directory());
    }
}
