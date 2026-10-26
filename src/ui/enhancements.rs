use super::*;
impl Editor {
    pub(super) fn update_footer(&self) {
        let Some(tab) = self.current() else {
            return;
        };
        let buffer = tab.view.buffer();
        let cursor = buffer.iter_at_offset(buffer.cursor_position());
        let mut args = FluentArgs::new();
        args.set("lines", buffer.line_count());
        args.set("chars", buffer.char_count());
        args.set("line", cursor.line() + 1);
        args.set("column", cursor.line_offset() + 1);
        let position = if tab.unsupported.get() {
            "—".to_owned()
        } else {
            self.strings.format("footer-position", Some(&args))
        };
        self.footer_left.set_text(&position);
        let name = tab.name(&self.strings);
        let format = match std::path::Path::new(&name)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "rs" => "Rust",
            "c" | "h" => "C",
            "cpp" | "hpp" | "cc" => "C++",
            "md" | "markdown" => "Markdown",
            "py" => "Python",
            "json" => "JSON",
            "html" => "HTML",
            "css" => "CSS",
            "js" => "JavaScript",
            "toml" => "TOML",
            _ => "",
        };
        let mut args = FluentArgs::new();
        args.set(
            "encoding",
            if tab.unsupported.get() {
                "—"
            } else {
                "UTF-8"
            },
        );
        args.set(
            "format",
            if tab.unsupported.get() {
                self.strings.text("unsupported-file")
            } else if format.is_empty() {
                self.strings.text("plain-text")
            } else {
                format.to_owned()
            },
        );
        args.set("zoom", format!("{:.0}", self.zoom.get()));
        self.footer_right
            .set_text(&self.strings.format("footer-format", Some(&args)));
        let saving = self.saving.borrow().contains(&tab.id);
        self.footer_spinner.set_visible(saving);
        self.footer_spinner.set_spinning(saving);
        self.footer_pending
            .set_visible(!saving && tab.document.borrow().is_dirty());
    }
    pub(super) fn apply_theme(&self) {
        let theme = self.theme.borrow();
        let light = theme.as_str() == "light";
        self.actions["theme"].set_state(&theme.as_str().to_variant());
        if let Some(settings) = gtk::Settings::default() {
            if theme.as_str() == "system" {
                settings.reset_property("gtk-application-prefer-dark-theme");
                if settings
                    .find_property("gtk-interface-color-scheme")
                    .is_some()
                {
                    settings.reset_property("gtk-interface-color-scheme");
                }
            } else if let Some(property) = settings.find_property("gtk-interface-color-scheme") {
                if let Some(value) = glib::EnumClass::with_type(property.value_type())
                    .and_then(|class| class.to_value_by_nick(if light { "light" } else { "dark" }))
                {
                    settings.set_property_from_value("gtk-interface-color-scheme", &value);
                }
            } else {
                settings.set_gtk_application_prefer_dark_theme(!light);
            }
        }
        self.theme_css
            .load_from_data(if theme.as_str() == "system" {
                include_str!("../system.css")
            } else if light {
                include_str!("../light.css")
            } else {
                ""
            });
        gtk::style_context_add_provider_for_display(
            &gtk::prelude::WidgetExt::display(&self.window),
            &self.theme_css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2,
        );
    }
    pub(super) fn language_dialog(self: &Rc<Self>) {
        let dialog = gtk::Window::builder()
            .title(self.strings.text("language"))
            .transient_for(&self.window)
            .modal(true)
            .build();
        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.set_margin_start(16);
        content.set_margin_end(16);
        dialog.set_child(Some(&content));
        content.append(&gtk::Label::new(Some(
            &self.strings.text("language-restart"),
        )));
        let mut args = FluentArgs::new();
        args.set(
            "language",
            match self.strings.language() {
                "pt-BR" => "Português (Brasil)",
                "es" => "Español",
                _ => "English",
            },
        );
        let current = gtk::Label::new(Some(&self.strings.format("language-current", Some(&args))));
        current.set_xalign(0.0);
        content.append(&current);
        let mut first: Option<gtk::CheckButton> = None;
        for (id, name) in [
            ("en-US", "English"),
            ("pt-BR", "Português (Brasil)"),
            ("es", "Español"),
        ] {
            let button = gtk::CheckButton::with_label(name);
            button.set_group(first.as_ref());
            if first.is_none() {
                first = Some(button.clone());
            }
            button.set_active(self.language.borrow().as_str() == id);
            let weak = Rc::downgrade(self);
            button.connect_toggled(move |button| {
                if !button.is_active() {
                    return;
                }
                if let Some(e) = weak.upgrade() {
                    e.language.replace(id.to_owned());
                    e.save_settings();
                }
            });
            content.append(&button);
        }
        let close = gtk::Button::with_label(&self.strings.text("close"));
        let weak = dialog.downgrade();
        close.connect_clicked(move |_| {
            if let Some(d) = weak.upgrade() {
                d.close();
            }
        });
        content.append(&close);
        dialog.present();
    }
    pub(super) fn edit_transform(&self, name: &str) {
        let Some(tab) = self.current().filter(|t| !t.unsupported.get()) else {
            return;
        };
        let buffer = tab.view.buffer();
        if name == "insert-below" {
            tab.view.insert_newline();
            return;
        }
        if name == "insert-above" {
            let mut start = buffer.iter_at_offset(buffer.cursor_position());
            start.set_line_offset(0);
            let offset = start.offset();
            buffer.begin_user_action();
            buffer.insert(&mut start, "\n");
            buffer.place_cursor(&buffer.iter_at_offset(offset));
            buffer.end_user_action();
            return;
        }
        let (mut start, mut end) = buffer.selection_bounds().unwrap_or_else(|| {
            let cursor = buffer.iter_at_offset(buffer.cursor_position());
            let mut start = cursor;
            let mut end = cursor;
            let is_word = |c: char| c.is_alphanumeric() || c == '_';
            while start.offset() > 0 {
                let mut prev = start;
                prev.backward_char();
                if !is_word(prev.char()) {
                    break;
                }
                start = prev;
            }
            while !end.is_end() && is_word(end.char()) {
                end.forward_char();
            }
            (start, end)
        });
        if start == end {
            return;
        }
        let text = buffer.text(&start, &end, true);
        let output = crate::text_ops::convert(&text, name);
        let offset = start.offset();
        buffer.begin_user_action();
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, &output);
        buffer.select_range(&buffer.iter_at_offset(offset), &start);
        buffer.end_user_action();
    }
}
