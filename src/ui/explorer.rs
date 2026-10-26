use super::*;
use leditor::file_ops;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(super) enum Operation {
    OpenFolder,
    CloseFolder,
    NewRootFolder,
    Open(PathBuf),
    Create(PathBuf, bool),
    Rename(PathBuf),
    Delete(PathBuf),
    Clipboard(PathBuf, bool),
    Paste(PathBuf),
    Move(PathBuf, PathBuf),
    CopyPath(PathBuf),
    RenameTab(u64),
    Duplicate(u64),
    CloseAll,
    CloseTab(u64),
    SaveTab(u64, bool),
}
pub(super) struct Explorer {
    pub panel: gtk::Box,
    pub focused: Cell<bool>,
    pub creators: RefCell<Vec<gtk::Button>>,
    pub rows: gtk::Box,
    pub root: RefCell<Option<PathBuf>>,
    pub expanded: RefCell<BTreeSet<PathBuf>>,
    pub selected: RefCell<Option<PathBuf>>,
    pub clipboard: RefCell<Option<(PathBuf, bool)>>,
    pub reveal: gtk::Button,
}
impl Explorer {
    pub fn new() -> Self {
        let panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        panel.set_width_request(230);
        panel.add_css_class("explorer");
        let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let reveal = gtk::Button::from_icon_name("pan-end-symbolic");
        reveal.add_css_class("flat");
        reveal.set_visible(false);
        Self {
            panel,
            focused: Cell::new(false),
            creators: RefCell::new(Vec::new()),
            rows,
            root: RefCell::new(None),
            expanded: RefCell::new(BTreeSet::new()),
            selected: RefCell::new(None),
            clipboard: RefCell::new(None),
            reveal,
        }
    }
}
impl Editor {
    pub(super) fn setup_explorer(self: &Rc<Self>) {
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let label = gtk::Label::new(Some(&self.strings.text("explorer")));
        label.set_hexpand(true);
        toolbar.append(&label);
        for (icon, label, command) in [
            ("document-new-symbolic", "new-file", 0),
            ("folder-new-symbolic", "new-folder", 1),
            ("view-refresh-symbolic", "refresh", 2),
            ("pan-start-symbolic", "show-explorer", 3),
        ] {
            let button = gtk::Button::from_icon_name(icon);
            button.add_css_class("flat");
            button.set_tooltip_text(Some(&self.strings.text(label)));
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(e) = weak.upgrade() {
                    if e.busy.get() {
                        return;
                    }
                    match command {
                        0 | 1 => {
                            let root = e.explorer.root.borrow().clone();
                            if let Some(root) = root {
                                e.operation(Operation::Create(root, command == 1));
                            }
                        }
                        2 => e.refresh_explorer(),
                        _ => e.toggle_explorer(),
                    }
                }
            });
            if command < 2 {
                self.explorer.creators.borrow_mut().push(button.clone());
            }
            toolbar.append(&button);
        }
        self.explorer.panel.append(&toolbar);
        let scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&self.explorer.rows)
            .build();
        self.explorer.panel.append(&scroll);
        let weak = Rc::downgrade(self);
        self.explorer
            .reveal
            .set_tooltip_text(Some(&self.strings.text("show-explorer")));
        self.explorer.reveal.connect_clicked(move |_| {
            if let Some(e) = weak.upgrade() {
                e.toggle_explorer();
            }
        });
        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        let weak = Rc::downgrade(self);
        gesture.connect_pressed(move |g, _, x, y| {
            if let Some(e) = weak.upgrade() {
                if let Some(root) = e.explorer.root.borrow().clone() {
                    e.path_menu(&e.explorer.rows, &root, true, x, y);
                    g.set_state(gtk::EventSequenceState::Claimed);
                }
            }
        });
        self.explorer.rows.add_controller(gesture);
        self.explorer.rows.set_vexpand(true);
        self.refresh_explorer();
    }
    pub(super) fn toggle_explorer(&self) {
        let visible = !self.explorer.panel.is_visible();
        self.explorer.panel.set_visible(visible);
        self.explorer.reveal.set_visible(!visible);
        self.actions["show-explorer"].set_state(&visible.to_variant());
        self.save_settings();
        if !visible {
            self.explorer.focused.set(false);
            if let Some(tab) = self.current() {
                tab.view.grab_focus();
            }
        }
    }
    pub(super) fn load_folder(self: &Rc<Self>, path: PathBuf) {
        self.explorer.root.replace(Some(path.clone()));
        self.explorer.expanded.borrow_mut().clear();
        self.explorer.expanded.borrow_mut().insert(path);
        self.explorer.selected.replace(None);
        if !self.explorer.panel.is_visible() {
            self.toggle_explorer();
        }
        self.refresh_explorer();
    }
    pub(super) fn refresh_explorer(self: &Rc<Self>) {
        let restore_focus = self.explorer.focused.get();
        while let Some(child) = self.explorer.rows.first_child() {
            self.explorer.rows.remove(&child);
        }
        let root = self.explorer.root.borrow().clone();
        if let Some(root) = root {
            self.append_row(&root, 0);
        } else {
            let label = gtk::Label::new(Some(&self.strings.text("no-folder")));
            label.set_margin_top(16);
            self.explorer.rows.append(&label);
        }
        if restore_focus {
            let mut child = self.explorer.rows.first_child();
            while let Some(row) = child {
                if row.has_css_class("selected") {
                    row.grab_focus();
                    break;
                }
                child = row.next_sibling();
            }
        }
    }
    fn append_row(self: &Rc<Self>, path: &Path, depth: i32) {
        let folder = path.is_dir();
        let expanded = self.explorer.expanded.borrow().contains(path);
        let row = gtk::Button::new();
        row.add_css_class("explorer-row");
        row.add_css_class("flat");
        row.set_halign(gtk::Align::Fill);
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        content.set_margin_start(depth * 16 + 6);
        content.append(&gtk::Image::from_icon_name(if folder {
            if expanded {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            }
        } else {
            "text-x-generic-symbolic"
        }));
        if folder {
            content.append(&gtk::Image::from_icon_name("folder-symbolic"));
        } else {
            // Replace the generic file icon with the platform MIME icon.
            if let Some(icon) = content.first_child() {
                content.remove(&icon);
            }
            let (content_type, _) = gio::content_type_guess(Some(path), None);
            content.append(&gtk::Image::from_gicon(
                &gio::content_type_get_symbolic_icon(&content_type),
            ));
        }
        let label = gtk::Label::new(Some(
            &path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy(),
        ));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        content.append(&label);
        row.set_child(Some(&content));
        row.set_tooltip_text(Some(&path.display().to_string()));
        if self.explorer.selected.borrow().as_deref() == Some(path) {
            row.add_css_class("selected");
        }
        if self
            .explorer
            .clipboard
            .borrow()
            .as_ref()
            .is_some_and(|(p, cut)| *cut && p == path)
        {
            row.add_css_class("cut-entry");
        }
        let weak = Rc::downgrade(self);
        let p = path.to_owned();
        row.connect_clicked(move |_| {
            if let Some(e) = weak.upgrade() {
                if e.busy.get() {
                    return;
                }
                e.explorer.selected.replace(Some(p.clone()));
                if folder {
                    if !e.explorer.expanded.borrow_mut().remove(&p) {
                        e.explorer.expanded.borrow_mut().insert(p.clone());
                    }
                    e.refresh_explorer();
                } else {
                    e.operation(Operation::Open(p.clone()));
                }
            }
        });
        let focus = gtk::EventControllerFocus::new();
        let weak = Rc::downgrade(self);
        let p = path.to_owned();
        focus.connect_enter(move |_| {
            if let Some(e) = weak.upgrade() {
                e.explorer.selected.replace(Some(p.clone()));
                e.explorer.focused.set(true);
                e.refresh();
            }
        });
        row.add_controller(focus);
        let keys = gtk::EventControllerKey::new();
        let weak = Rc::downgrade(self);
        let p = path.to_owned();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            if let Some(e) = weak.upgrade() {
                let op = if key == gtk::gdk::Key::F2 {
                    Some(Operation::Rename(p.clone()))
                } else if key == gtk::gdk::Key::Delete {
                    Some(Operation::Delete(p.clone()))
                } else if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    match key {
                        gtk::gdk::Key::c => Some(Operation::Clipboard(p.clone(), false)),
                        gtk::gdk::Key::x => Some(Operation::Clipboard(p.clone(), true)),
                        gtk::gdk::Key::v => Some(Operation::Paste(if folder {
                            p.clone()
                        } else {
                            p.parent().unwrap().to_owned()
                        })),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(op) = op {
                    e.operation(op);
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        });
        row.add_controller(keys);
        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        let weak = Rc::downgrade(self);
        let p = path.to_owned();
        let target = row.downgrade();
        gesture.connect_pressed(move |g, _, x, y| {
            if let (Some(e), Some(row)) = (weak.upgrade(), target.upgrade()) {
                e.explorer.selected.replace(Some(p.clone()));
                row.grab_focus();
                e.path_menu(&row, &p, false, x, y);
                g.set_state(gtk::EventSequenceState::Claimed);
            }
        });
        row.add_controller(gesture);
        let drag = gtk::DragSource::builder()
            .actions(gtk::gdk::DragAction::MOVE)
            .build();
        let p = path.to_owned();
        let weak = Rc::downgrade(self);
        drag.connect_prepare(move |_, _, _| {
            let e = weak.upgrade()?;
            if e.busy.get() {
                return None;
            }
            Some(gtk::gdk::ContentProvider::for_value(
                &gio::File::for_path(&p).to_value(),
            ))
        });
        let weak_row = row.downgrade();
        drag.connect_drag_begin(move |source, _| {
            if let Some(row) = weak_row.upgrade() {
                source.set_icon(Some(&gtk::WidgetPaintable::new(Some(&row))), 0, 0);
            }
        });
        row.add_controller(drag);
        let destination = if folder {
            path.to_owned()
        } else {
            path.parent().unwrap().to_owned()
        };
        let drop = gtk::DropTarget::new(gio::File::static_type(), gtk::gdk::DragAction::MOVE);
        let weak_row = row.downgrade();
        drop.connect_enter(move |_, _, _| {
            if let Some(row) = weak_row.upgrade() {
                row.add_css_class("drop-target");
            }
            gtk::gdk::DragAction::MOVE
        });
        let weak_row = row.downgrade();
        drop.connect_leave(move |_| {
            if let Some(row) = weak_row.upgrade() {
                row.remove_css_class("drop-target");
            }
        });
        let weak = Rc::downgrade(self);
        let weak_row = row.downgrade();
        drop.connect_drop(move |_, value, _, _| {
            if let Some(row) = weak_row.upgrade() {
                row.remove_css_class("drop-target");
            }
            let Ok(value) = value.get::<gio::File>() else {
                return false;
            };
            let Some(source) = value.path() else {
                return false;
            };
            if let Some(e) = weak.upgrade() {
                if !e.busy.get() {
                    e.operation(Operation::Move(source, destination.clone()));
                    return true;
                }
            }
            false
        });
        row.add_controller(drop);
        self.explorer.rows.append(&row);
        // Symlink folders are visible but never recursively followed (cycles/escape).
        if folder && expanded && !path.is_symlink() {
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let mut paths = Vec::new();
                    for entry in entries {
                        match entry {
                            Ok(entry) => paths.push(entry.path()),
                            Err(error) => {
                                let label = gtk::Label::new(Some(&error.to_string()));
                                label.set_wrap(true);
                                self.explorer.rows.append(&label);
                            }
                        }
                    }
                    let mut entries = paths;
                    entries.sort_by_key(|p| {
                        (
                            !p.is_dir(),
                            p.file_name().unwrap_or_default().to_ascii_lowercase(),
                        )
                    });
                    for entry in entries {
                        self.append_row(&entry, depth + 1);
                    }
                }
                Err(error) => {
                    let label = gtk::Label::new(Some(&error.to_string()));
                    label.set_wrap(true);
                    self.explorer.rows.append(&label);
                }
            }
        }
    }
    fn path_menu(
        self: &Rc<Self>,
        widget: &impl IsA<gtk::Widget>,
        path: &Path,
        background: bool,
        x: f64,
        y: f64,
    ) {
        if self.busy.get() {
            return;
        }
        let parent = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        let mut items = Vec::new();
        if background || path.is_dir() {
            items.push(("new-file", Operation::Create(parent.to_owned(), false)));
            items.push(("new-folder", Operation::Create(parent.to_owned(), true)));
        }
        if !background {
            items.extend([
                ("rename", Operation::Rename(path.to_owned())),
                ("cut", Operation::Clipboard(path.to_owned(), true)),
                ("copy", Operation::Clipboard(path.to_owned(), false)),
            ]);
        }
        items.push(("paste", Operation::Paste(parent.to_owned())));
        if !background {
            items.extend([
                ("delete", Operation::Delete(path.to_owned())),
                ("copy-path", Operation::CopyPath(path.to_owned())),
            ]);
        }
        self.operation_menu(widget, items, x, y);
    }
    pub(super) fn operation_menu(
        self: &Rc<Self>,
        widget: &impl IsA<gtk::Widget>,
        items: Vec<(&str, Operation)>,
        x: f64,
        y: f64,
    ) {
        let popover = gtk::Popover::new();
        popover.set_parent(widget);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        for (label, operation) in items {
            let button = gtk::Button::with_label(&self.strings.text(label));
            button.add_css_class("flat");
            if matches!(&operation, Operation::SaveTab(id, _) | Operation::Duplicate(id) if self.tab(*id).is_some_and(|t| t.unsupported.get()))
            {
                button.set_sensitive(false);
            }
            if label == "paste" {
                button.set_sensitive(self.explorer.clipboard.borrow().is_some());
            }
            let weak = Rc::downgrade(self);
            let weak_popover = popover.downgrade();
            button.connect_clicked(move |_| {
                if let Some(p) = weak_popover.upgrade() {
                    p.popdown();
                }
                if let Some(e) = weak.upgrade() {
                    e.operation(operation.clone());
                }
            });
            content.append(&button);
        }
        popover.set_child(Some(&content));
        popover.connect_closed(|p| p.unparent());
        popover.popup();
    }
    pub(super) fn operation(self: &Rc<Self>, operation: Operation) {
        self.request(Intent::Operation(operation));
    }
    // GtkAlertDialog has no text-entry API. Keep the small GTK Dialog boundary
    // until GTK provides a replacement, without adding libadwaita on Windows.
    #[allow(deprecated)]
    async fn name_dialog(&self, title: &str, initial: &str) -> Option<String> {
        let dialog = gtk::Dialog::builder()
            .transient_for(&self.window)
            .modal(true)
            .title(title)
            .build();
        dialog.add_button(&self.strings.text("cancel"), gtk::ResponseType::Cancel);
        dialog.add_button(&self.strings.text("ok"), gtk::ResponseType::Ok);
        dialog.set_default_response(gtk::ResponseType::Ok);
        let entry = gtk::Entry::builder()
            .text(initial)
            .activates_default(true)
            .width_chars(36)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();
        dialog.content_area().append(&entry);
        entry.grab_focus();
        entry.select_region(0, -1);
        let response = dialog.run_future().await;
        let name = entry.text().to_string();
        dialog.close();
        (response == gtk::ResponseType::Ok).then_some(name)
    }
    async fn choose_folder(&self) -> Option<PathBuf> {
        match gtk::FileDialog::builder()
            .title(self.strings.text("open-folder"))
            .modal(true)
            .build()
            .select_folder_future(Some(&self.window))
            .await
        {
            Ok(file) => {
                if let Some(path) = file.path() {
                    Some(path)
                } else {
                    self.error(&self.strings.text("local-only")).await;
                    None
                }
            }
            Err(error) => {
                self.dialog_error(error).await;
                None
            }
        }
    }
    pub(super) async fn open_path(self: &Rc<Self>, path: &Path) {
        if let Some(id) = self.open_tab(path, None) {
            self.select(id);
            return;
        }
        match Document::open(path) {
            Ok(doc) => {
                self.add_tab(doc);
            }
            Err(OpenError::Encoding | OpenError::Binary) => {
                let mut doc = Document::default();
                doc.path = Some(path.to_owned());
                let id = self.add_tab(doc);
                if let Some(tab) = self.tab(id) {
                    tab.mark_unsupported(&self.strings);
                }
                self.refresh();
            }
            Err(error) => self.error(&error.to_string()).await,
        }
    }
    fn affected(&self, path: &Path) -> Vec<(Rc<Tab>, PathBuf)> {
        self.tabs
            .borrow()
            .iter()
            .filter_map(|tab| {
                let relative = file_ops::relative(tab.document.borrow().path.as_deref()?, path)?;
                Some((tab.clone(), relative))
            })
            .collect()
    }
    async fn relocate(self: &Rc<Self>, source: PathBuf, target: PathBuf) {
        let affected = self.affected(&source);
        let from = source.clone();
        let to = target.clone();
        let result = gio::spawn_blocking(move || file_ops::move_entry(&from, &to))
            .await
            .expect("filesystem worker panicked");
        match result {
            Ok(()) => {
                for (tab, relative) in affected {
                    tab.document.borrow_mut().path = Some(if relative.as_os_str().is_empty() {
                        target.clone()
                    } else {
                        target.join(relative)
                    });
                }
                let root = self.explorer.root.borrow().clone();
                if let Some(root) = root {
                    if let Ok(relative) = root.strip_prefix(&source) {
                        self.explorer.root.replace(Some(target.join(relative)));
                    }
                }
                let expanded = self
                    .explorer
                    .expanded
                    .borrow()
                    .iter()
                    .map(|p| {
                        p.strip_prefix(&source)
                            .map(|relative| target.join(relative))
                            .unwrap_or_else(|_| p.clone())
                    })
                    .collect();
                self.explorer.expanded.replace(expanded);
                let clipboard = self.explorer.clipboard.borrow().clone();
                if let Some((path, cut)) = clipboard {
                    if let Ok(relative) = path.strip_prefix(&source) {
                        self.explorer
                            .clipboard
                            .replace(Some((target.join(relative), cut)));
                    }
                }
                self.explorer.selected.replace(Some(target));
            }
            Err(error) => self.error(&error.to_string()).await,
        }
    }
    pub(super) async fn run_operation(self: &Rc<Self>, operation: Operation) {
        match operation {
            Operation::CloseTab(id) => {
                if let Some(tab) = self.tab(id) {
                    if self.confirm_close(&tab).await {
                        self.remove_tab(id);
                    }
                }
            }
            Operation::SaveTab(id, save_as) => {
                if let Some(tab) = self.tab(id) {
                    self.save(&tab, save_as).await;
                }
            }
            Operation::OpenFolder => {
                if let Some(path) = self.choose_folder().await {
                    self.load_folder(path);
                }
            }
            Operation::CloseFolder => {
                self.explorer.root.replace(None);
                self.explorer.selected.replace(None);
            }
            Operation::NewRootFolder => {
                if let Some(parent) = self.choose_folder().await {
                    if let Some(name) = self.name_dialog(&self.strings.text("new-folder"), "").await
                    {
                        match file_ops::create(&parent, &name, true) {
                            Ok(path) => self.load_folder(path),
                            Err(error) => self.error(&error.to_string()).await,
                        }
                    }
                }
            }
            Operation::Open(path) => self.open_path(&path).await,
            Operation::Create(parent, folder) => {
                if let Some(name) = self
                    .name_dialog(
                        &self
                            .strings
                            .text(if folder { "new-folder" } else { "new-file" }),
                        "",
                    )
                    .await
                {
                    match file_ops::create(&parent, &name, folder) {
                        Ok(path) => {
                            self.explorer.expanded.borrow_mut().insert(parent);
                            if !folder {
                                self.open_path(&path).await;
                            }
                        }
                        Err(error) => self.error(&error.to_string()).await,
                    }
                }
            }
            Operation::Rename(path) => {
                if let Some(name) = self
                    .name_dialog(
                        &self.strings.text("rename"),
                        &path.file_name().unwrap_or_default().to_string_lossy(),
                    )
                    .await
                {
                    match file_ops::child(path.parent().unwrap_or(Path::new("/")), &name) {
                        Ok(target) if target != path => self.relocate(path, target).await,
                        Ok(_) => (),
                        Err(error) => self.error(&error.to_string()).await,
                    }
                }
            }
            Operation::RenameTab(id) => {
                if let Some(tab) = self.tab(id) {
                    let path = tab.document.borrow().path.clone();
                    if let Some(path) = path {
                        Box::pin(self.run_operation(Operation::Rename(path))).await;
                    } else if let Some(name) = self
                        .name_dialog(&self.strings.text("rename"), &tab.name(&self.strings))
                        .await
                    {
                        match file_ops::child(Path::new("."), &name) {
                            Ok(_) => {
                                tab.display_name.replace(Some(name));
                            }
                            Err(error) => self.error(&error.to_string()).await,
                        }
                    }
                }
            }
            Operation::Duplicate(id) => {
                if let Some(tab) = self.tab(id).filter(|t| !t.unsupported.get()) {
                    let mut document = Document::default();
                    document.text = tab.document.borrow().text.clone();
                    let id = self.add_tab(document);
                    self.tab(id)
                        .unwrap()
                        .display_name
                        .replace(Some("Untitled.txt".into()));
                }
            }
            Operation::Clipboard(path, cut) => {
                self.explorer.clipboard.replace(Some((path, cut)));
            }
            Operation::CopyPath(path) => {
                self.window
                    .clipboard()
                    .set_text(&path.display().to_string());
            }
            Operation::Paste(parent) => {
                let clipboard = self.explorer.clipboard.borrow().clone();
                if let Some((source, cut)) = clipboard {
                    if let Some(name) = source.file_name() {
                        let target = parent.join(name);
                        if cut {
                            self.relocate(source.clone(), target).await;
                            if !source.exists() {
                                self.explorer.clipboard.replace(None);
                            }
                        } else {
                            let result =
                                gio::spawn_blocking(move || file_ops::copy_entry(&source, &target))
                                    .await
                                    .expect("filesystem worker panicked");
                            if let Err(error) = result {
                                self.error(&error.to_string()).await;
                            }
                        }
                    }
                }
            }
            Operation::Move(source, parent) => {
                if let Some(name) = source.file_name() {
                    let target = parent.join(name);
                    self.relocate(source, target).await;
                }
            }
            Operation::Delete(path) => {
                let affected = self.affected(&path);
                // Preserve dirty buffers as independent untitled documents on deletion.
                // No buffer is discarded, including if filesystem deletion is partial.
                let dirty = affected
                    .iter()
                    .any(|(tab, _)| tab.document.borrow().is_dirty());
                let detail = format!(
                    "{}\n\n{}",
                    path.display(),
                    self.strings.text(if dirty {
                        "delete-dirty-detail"
                    } else {
                        "delete-detail"
                    })
                );
                let dialog = gtk::AlertDialog::builder()
                    .message(self.strings.text("delete-confirm"))
                    .detail(detail)
                    .buttons([self.strings.text("cancel"), self.strings.text("delete")])
                    .cancel_button(0)
                    .default_button(0)
                    .modal(true)
                    .build();
                if dialog.choose_future(Some(&self.window)).await == Ok(1) {
                    for (tab, _) in &affected {
                        if tab.unsupported.get() {
                            continue;
                        }
                        let name = tab.name(&self.strings);
                        tab.display_name.replace(Some(name));
                        tab.document.borrow_mut().detach();
                    }
                    let target = path.clone();
                    let result = gio::spawn_blocking(move || file_ops::delete(&target))
                        .await
                        .expect("filesystem worker panicked");
                    if let Err(error) = result {
                        self.error(&error.to_string()).await;
                    }
                    for (tab, _) in &affected {
                        if tab.unsupported.get()
                            && tab
                                .document
                                .borrow()
                                .path
                                .as_ref()
                                .is_some_and(|p| !p.exists())
                        {
                            self.remove_tab(tab.id);
                        }
                    }
                    if !path.exists() && self.explorer.root.borrow().as_deref() == Some(&path) {
                        self.explorer.root.replace(None);
                    }
                }
            }
            Operation::CloseAll => {
                let tabs = self.tabs.borrow().clone();
                for tab in &tabs {
                    if !self.confirm_close(tab).await {
                        return;
                    }
                }
                for tab in tabs {
                    self.remove_tab(tab.id);
                }
            }
        }
        self.refresh_explorer();
        self.refresh();
    }
}
