use super::*;
impl Editor {
    pub(super) fn update_footer(&self) {
        let Some(tab) = self.current() else {
            return;
        };
        if tab.media.get() {
            self.footer_left.set_text("");
            self.footer_right
                .set_text(&format!("{:.0}%", self.zoom.get()));
            self.footer_pending.set_visible(false);
            self.footer_spinner.set_visible(false);
            self.footer_spinner.stop();
            return;
        }
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
        let format = tab
            .source_buffer()
            .language()
            .map(|l| l.name().to_string())
            .unwrap_or_default();
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
        let Some(tab) = self.current().filter(|t| t.is_text()) else {
            return;
        };
        let buffer = tab.view.buffer();
        if name == "insert-below" {
            tab.view.insert_newline();
            return;
        }
        if name == "insert-above" {
            if self.search.bar.is_visible() {
                return;
            }
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
