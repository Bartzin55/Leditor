use super::*;
pub(super) struct GoToBar {
    pub bar: gtk::Box,
    pub entry: gtk::Entry,
    status: gtk::Label,
    pub go: gtk::Button,
    close: gtk::Button,
}
impl GoToBar {
    pub fn new(strings: &I18n) -> Self {
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        bar.add_css_class("search-bar");
        let entry = gtk::Entry::builder()
            .placeholder_text(strings.text("line-number"))
            .input_purpose(gtk::InputPurpose::Digits)
            .width_chars(12)
            .build();
        entry.set_tooltip_text(Some(&strings.text("line-number")));
        let go = gtk::Button::with_label(&strings.text("go-to-line"));
        let status = gtk::Label::builder()
            .hexpand(true)
            .xalign(0.0)
            .wrap(true)
            .build();
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.set_tooltip_text(Some(&strings.text("close-go-to-line")));
        bar.append(&entry);
        bar.append(&go);
        bar.append(&status);
        bar.append(&close);
        bar.set_visible(false);
        Self {
            bar,
            entry,
            status,
            go,
            close,
        }
    }
}
impl Editor {
    pub(super) fn setup_goto(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.goto.entry.connect_activate(move |_| {
            if let Some(e) = weak.upgrade() {
                e.go_to_line();
            }
        });
        let weak = Rc::downgrade(self);
        self.goto.go.connect_clicked(move |_| {
            if let Some(e) = weak.upgrade() {
                e.go_to_line();
            }
        });
        let weak = Rc::downgrade(self);
        self.goto.close.connect_clicked(move |_| {
            if let Some(e) = weak.upgrade() {
                e.close_goto();
            }
        });
        let weak = Rc::downgrade(self);
        self.goto.entry.connect_changed(move |_| {
            if let Some(e) = weak.upgrade() {
                e.refresh_goto();
            }
        });
    }
    pub(super) fn open_goto(&self) {
        let Some(tab) = self.current().filter(|t| t.is_text()) else {
            return;
        };
        if self.search.bar.is_visible() {
            self.close_search();
        }
        self.goto.bar.set_visible(true);
        self.actions["go-to-line"].set_state(&true.to_variant());
        let b = tab.view.buffer();
        self.goto
            .entry
            .set_text(&(b.iter_at_offset(b.cursor_position()).line() + 1).to_string());
        self.refresh_goto();
        self.goto.entry.grab_focus();
        self.goto.entry.select_region(0, -1);
    }
    pub(super) fn close_goto(&self) {
        self.goto.bar.set_visible(false);
        self.actions["go-to-line"].set_state(&false.to_variant());
        if let Some(tab) = self.current().filter(|t| t.is_text()) {
            tab.view.grab_focus();
        }
    }
    pub(super) fn refresh_goto(&self) {
        if !self.goto.bar.is_visible() {
            return;
        }
        let Some(tab) = self.current().filter(|t| t.is_text()) else {
            self.close_goto();
            return;
        };
        let mut args = FluentArgs::new();
        args.set("total", tab.view.buffer().line_count());
        self.goto
            .status
            .set_text(&self.strings.format("go-to-line-range", Some(&args)));
        self.goto.status.remove_css_class("search-error");
        self.goto.go.set_sensitive(!self.busy.get());
    }
    pub(super) fn go_to_line(&self) {
        if self.busy.get() {
            return;
        }
        let Some(tab) = self.current().filter(|t| t.is_text()) else {
            return;
        };
        let b = tab.view.buffer();
        let input = self.goto.entry.text();
        let line = input
            .trim()
            .parse::<i32>()
            .ok()
            .filter(|n| (1..=b.line_count()).contains(n));
        let Some(mut iter) = line.and_then(|n| b.iter_at_line(n - 1)) else {
            let mut args = FluentArgs::new();
            args.set("total", b.line_count());
            self.goto
                .status
                .set_text(&self.strings.format("go-to-line-invalid", Some(&args)));
            self.goto.status.add_css_class("search-error");
            self.goto.entry.grab_focus();
            return;
        };
        b.place_cursor(&iter);
        tab.view.scroll_to_iter(&mut iter, 0.1, true, 0.0, 0.5);
        self.refresh_goto();
        tab.view.grab_focus();
    }
}
