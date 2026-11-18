use super::*;
#[derive(Clone)]
pub(super) struct FocusSnapshot {
    pub explorer: bool,
    pub sidebar_width: i32,
    fullscreen: bool,
}
impl Editor {
    pub(super) fn pointer_over_explorer(&self) -> bool {
        self.explorer.panel.is_mapped() && self.explorer.pointer_inside.get()
    }
    pub(super) fn setup_focus(self: &Rc<Self>, root: &gtk::Box) {
        let motion = gtk::EventControllerMotion::new();
        let weak = Rc::downgrade(self);
        motion.connect_motion(move |_, _, y| {
            if let Some(e) = weak.upgrade().filter(|e| e.focus_mode.get()) {
                if y <= 4.0 {
                    e.menubar.set_visible(true);
                    e.toolbar.set_visible(true);
                    e.tab_bar.set_visible(true);
                } else if y
                    > (e.menubar.height() + e.toolbar.height() + e.tab_bar.height() + 16) as f64
                {
                    e.menubar.set_visible(false);
                    e.toolbar.set_visible(false);
                    e.tab_bar.set_visible(false);
                }
            }
        });
        root.add_controller(motion);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, state| {
            use gtk::gdk::{Key, ModifierType};
            let Some(editor) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let modifiers = state & gtk::accelerator_get_default_mod_mask();
            let explorer = editor.pointer_over_explorer();
            let row_focused = gtk::prelude::GtkWindowExt::focus(&editor.window)
                .is_some_and(|focus| focus.is_ancestor(&editor.explorer.panel));
            if explorer || row_focused {
                let clipboard = match (key, modifiers) {
                    (Key::c, ModifierType::CONTROL_MASK) => Some("copy"),
                    (Key::x, ModifierType::CONTROL_MASK) => Some("cut"),
                    (Key::v, ModifierType::CONTROL_MASK) => Some("paste"),
                    _ => None,
                };
                if let Some(action) = clipboard {
                    if !editor.busy.get()
                        && (explorer || action == "copy" || !editor.only_view.get())
                    {
                        editor.edit_clipboard(action, explorer);
                    }
                    return glib::Propagation::Stop;
                }
                if key == Key::Delete && modifiers.is_empty() {
                    if explorer {
                        let selected = editor.explorer.selected.borrow().clone();
                        if let Some(path) = selected {
                            editor.operation(Operation::Delete(path));
                        }
                    } else if let Some(tab) = editor
                        .current()
                        .filter(|t| t.is_text() && t.view.is_editable() && !editor.busy.get())
                    {
                        tab.view.emit_delete_from_cursor(gtk::DeleteType::Chars, 1);
                    }
                    return glib::Propagation::Stop;
                }
            }
            if key == gtk::gdk::Key::Escape {
                if let Some(e) = weak.upgrade() {
                    if e.focus_mode.get() {
                        e.toggle_focus();
                        return glib::Propagation::Stop;
                    }
                    if e.goto.bar.is_visible() {
                        e.close_goto();
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
            self.toolbar.set_visible(false);
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
            self.toolbar.set_visible(true);
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
}
