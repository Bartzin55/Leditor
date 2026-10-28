use super::*;
impl Editor {
    pub(super) fn setup_tab_actions(self: &Rc<Self>, tab: &Rc<Tab>) {
        let focus = gtk::EventControllerFocus::new();
        let weak = Rc::downgrade(self);
        focus.connect_enter(move |_| {
            if let Some(e) = weak.upgrade() {
                e.explorer.focused.set(false);
                e.refresh();
            }
        });
        tab.view.add_controller(focus);
        let id = tab.id;
        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        let weak = Rc::downgrade(self);
        let header = tab.header.downgrade();
        gesture.connect_pressed(move |g, _, x, y| {
            if let (Some(e), Some(header)) = (weak.upgrade(), header.upgrade()) {
                if e.busy.get() {
                    return;
                }
                let mut items = vec![
                    ("close", Operation::CloseTab(id)),
                    ("close-all", Operation::CloseAll),
                ];
                if e.tab(id).is_some_and(|t| t.is_text()) {
                    items.extend([
                        ("save", Operation::SaveTab(id, false)),
                        ("save-as", Operation::SaveTab(id, true)),
                        ("duplicate", Operation::Duplicate(id)),
                    ]);
                }
                items.push(("rename", Operation::RenameTab(id)));
                e.operation_menu(&header, items, x, y);
                g.set_state(gtk::EventSequenceState::Claimed);
            }
        });
        tab.header.add_controller(gesture);
        let source = gtk::DragSource::builder()
            .actions(gtk::gdk::DragAction::MOVE)
            .build();
        let weak = Rc::downgrade(self);
        source.connect_prepare(move |_, _, _| {
            let e = weak.upgrade()?;
            if e.busy.get() {
                return None;
            }
            Some(gtk::gdk::ContentProvider::for_value(
                &format!("leditor-tab:{id}").to_value(),
            ))
        });
        let header = tab.header.downgrade();
        source.connect_drag_begin(move |source, _| {
            if let Some(header) = header.upgrade() {
                source.set_icon(Some(&gtk::WidgetPaintable::new(Some(&header))), 0, 0);
                header.add_css_class("dragging");
            }
        });
        let header = tab.header.downgrade();
        source.connect_drag_end(move |_, _, _| {
            if let Some(header) = header.upgrade() {
                header.remove_css_class("dragging");
            }
        });
        tab.header.add_controller(source);
        let target = gtk::DropTarget::new(String::static_type(), gtk::gdk::DragAction::MOVE);
        let header = tab.header.downgrade();
        target.connect_motion(move |_, x, _| {
            if let Some(header) = header.upgrade() {
                let before = x < f64::from(header.width()) / 2.0;
                header.remove_css_class(if before { "drop-after" } else { "drop-before" });
                header.add_css_class(if before { "drop-before" } else { "drop-after" });
            }
            gtk::gdk::DragAction::MOVE
        });
        let header = tab.header.downgrade();
        target.connect_leave(move |_| {
            if let Some(header) = header.upgrade() {
                header.remove_css_class("drop-before");
                header.remove_css_class("drop-after");
            }
        });
        let weak = Rc::downgrade(self);
        let header = tab.header.downgrade();
        target.connect_drop(move |_, value, x, _| {
            let (Some(e), Some(header)) = (weak.upgrade(), header.upgrade()) else {
                return false;
            };
            header.remove_css_class("drop-before");
            header.remove_css_class("drop-after");
            let Ok(value) = value.get::<String>() else {
                return false;
            };
            let Some(source) = value
                .strip_prefix("leditor-tab:")
                .and_then(|v| v.parse::<u64>().ok())
            else {
                return false;
            };
            if e.busy.get() {
                return false;
            }
            e.reorder_tab(source, id, x >= f64::from(header.width()) / 2.0);
            true
        });
        tab.header.add_controller(target);
    }
    pub(super) fn reorder_tab(&self, source: u64, target: u64, after: bool) {
        if source == target {
            return;
        }
        let mut tabs = self.tabs.borrow_mut();
        let Some(from) = tabs.iter().position(|t| t.id == source) else {
            return;
        };
        if !tabs.iter().any(|t| t.id == target) {
            return;
        }
        let tab = tabs.remove(from);
        let to = tabs.iter().position(|t| t.id == target).unwrap() + usize::from(after);
        tabs.insert(to, tab);
        let mut previous: Option<gtk::Box> = None;
        for tab in tabs.iter() {
            self.tabs_box
                .reorder_child_after(&tab.header, previous.as_ref());
            previous = Some(tab.header.clone());
        }
    }
    pub(super) fn edit_clipboard(self: &Rc<Self>, action: &str) {
        if self.explorer.focused.get() {
            let selected = self.explorer.selected.borrow().clone();
            if let Some(path) = selected {
                let operation = match action {
                    "cut" => Operation::Clipboard(path, true),
                    "copy" => Operation::Clipboard(path, false),
                    _ => Operation::Paste(if path.is_dir() {
                        path
                    } else {
                        path.parent().unwrap_or(&path).to_owned()
                    }),
                };
                self.operation(operation);
            }
            return;
        }
        if let Some(tab) = self.current().filter(|t| t.is_text()) {
            match action {
                "cut" => tab.view.copy_or_cut(true),
                "copy" => tab.view.copy_or_cut(false),
                "paste" => tab
                    .view
                    .buffer()
                    .paste_clipboard(&tab.view.clipboard(), None, true),
                _ => (),
            }
        }
    }
}
