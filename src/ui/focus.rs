use super::*;
#[derive(Clone)]
pub(super) struct FocusSnapshot {
    pub explorer: bool,
    pub sidebar_width: i32,
    fullscreen: bool,
}
impl Editor {
    pub(super) fn setup_focus(self: &Rc<Self>, root: &gtk::Box) {
        let motion = gtk::EventControllerMotion::new();
        let weak = Rc::downgrade(self);
        motion.connect_motion(move |_, _, y| {
            if let Some(e) = weak.upgrade().filter(|e| e.focus_mode.get()) {
                if y <= 4.0 {
                    e.menubar.set_visible(true);
                    e.tab_bar.set_visible(true);
                } else if y > (e.menubar.height() + e.tab_bar.height() + 16) as f64 {
                    e.menubar.set_visible(false);
                    e.tab_bar.set_visible(false);
                }
            }
        });
        root.add_controller(motion);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                if let Some(e) = weak.upgrade() {
                    if e.focus_mode.get() {
                        e.toggle_focus();
                        return glib::Propagation::Stop;
                    }
                    if e.search.bar.is_visible() {
                        e.close_search();
                        return glib::Propagation::Stop;
                    }
                }
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(keys);
        let weak = Rc::downgrade(self);
        self.window.connect_fullscreened_notify(move |w| {
            if !w.is_fullscreen() {
                if let Some(e) = weak.upgrade().filter(|e| e.focus_mode.get()) {
                    e.toggle_focus();
                }
            }
        });
    }
    pub(super) fn toggle_focus(&self) {
        if !self.focus_mode.get() {
            self.focus_snapshot.replace(Some(FocusSnapshot {
                explorer: self.explorer.panel.is_visible(),
                sidebar_width: self.paned.position(),
                fullscreen: self.window.is_fullscreen(),
            }));
            self.focus_mode.set(true);
            self.explorer.panel.set_visible(false);
            self.explorer.reveal.set_visible(false);
            self.menubar.set_visible(false);
            self.tab_bar.set_visible(false);
            self.footer.set_visible(false);
            for tab in self.tabs.borrow().iter() {
                tab.view.set_line_numbers(false);
            }
            self.window.fullscreen();
        } else {
            self.focus_mode.set(false);
            let snapshot = self.focus_snapshot.borrow().clone();
            if let Some(s) = snapshot {
                self.explorer.panel.set_visible(s.explorer);
                self.explorer.reveal.set_visible(!s.explorer);
                self.paned.set_position(s.sidebar_width);
                if !s.fullscreen {
                    self.window.unfullscreen();
                }
            }
            self.menubar.set_visible(true);
            self.tab_bar.set_visible(true);
            self.footer.set_visible(true);
            for tab in self.tabs.borrow().iter() {
                tab.view.set_line_numbers(self.line_numbers.get());
            }
            self.focus_snapshot.replace(None);
        }
        self.actions["focus"].set_state(&self.focus_mode.get().to_variant());
        self.refresh();
    }
    pub(super) fn toggle_always_on_top(&self) {
        let enabled = !self.always_on_top.get();
        match crate::window_mode::set_above(&self.window, enabled) {
            Ok(()) => {
                self.always_on_top.set(enabled);
                self.actions["always-on-top"].set_state(&enabled.to_variant());
            }
            Err(_) => {
                self.notice.set_text(&self.strings.text("top-unavailable"));
                self.notice.set_visible(true);
            }
        }
    }
}
