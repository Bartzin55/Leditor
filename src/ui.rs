use crate::{popups, shortcuts::SHORTCUTS, tab::Tab};
use fluent_bundle::FluentArgs;
use gtk::{gio, glib, prelude::*};
use leditor::{
    document::{same_path, Document, OpenError},
    i18n::I18n,
};
#[cfg(test)]
use std::path::PathBuf;
#[path = "ui/explorer.rs"]
mod explorer;
use explorer::{Explorer, Operation};
#[path = "ui/tab_actions.rs"]
mod tab_actions;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

#[derive(Clone, Debug)]
enum Intent {
    New,
    Open,
    Close(u64),
    Save(u64),
    SaveAs(u64),
    Quit,
    Operation(Operation),
}
struct Editor {
    window: gtk::ApplicationWindow,
    explorer: Explorer,
    stack: gtk::Stack,
    tab_bar: gtk::Box,
    tabs_box: gtk::Box,
    plus: gtk::Button,
    tabs: RefCell<Vec<Rc<Tab>>>,
    active: Cell<Option<u64>>,
    next_id: Cell<u64>,
    busy: Cell<bool>,
    wrap: Cell<bool>,
    line_numbers: Cell<bool>,
    strings: I18n,
    actions: BTreeMap<&'static str, gio::SimpleAction>,
    #[cfg(test)]
    file_choices: RefCell<std::collections::VecDeque<(bool, Option<PathBuf>)>>,
}
pub fn build(application: &gtk::Application) {
    create(application);
}
fn create(application: &gtk::Application) -> Rc<Editor> {
    if let Some(settings) = gtk::Settings::default() {
        if let Some(property) = settings.find_property("gtk-interface-color-scheme") {
            if let Some(value) = glib::EnumClass::with_type(property.value_type())
                .and_then(|class| class.to_value_by_nick("dark"))
            {
                settings.set_property_from_value("gtk-interface-color-scheme", &value);
            }
        } else {
            settings.set_gtk_application_prefer_dark_theme(true);
        }
    }
    let css = gtk::CssProvider::new();
    css.load_from_data(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let strings = I18n::default();
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Leditor")
        .default_width(1100)
        .default_height(720)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let model = gio::Menu::new();
    let mut actions = BTreeMap::new();
    for (menu_name, entries) in [
        (
            "file",
            vec![
                ("new", "new-file"),
                ("open", "open-file"),
                ("open-folder", "open-folder"),
                ("close-folder", "close-folder"),
                ("new-folder", "new-folder"),
                ("close", "close-file"),
                ("save", "save"),
                ("save-as", "save-as"),
            ],
        ),
        (
            "edit",
            vec![
                ("undo", "undo"),
                ("redo", "redo"),
                ("cut", "cut"),
                ("copy", "copy"),
                ("paste", "paste"),
            ],
        ),
        (
            "view",
            vec![
                ("word-wrap", "word-wrap"),
                ("line-numbers", "line-counter"),
                ("show-explorer", "show-explorer"),
            ],
        ),
        (
            "other",
            vec![("keyboard", "keyboard-shortcuts"), ("about", "about")],
        ),
    ] {
        let submenu = gio::Menu::new();
        for (name, label) in entries {
            let action = match name {
                "word-wrap" => gio::SimpleAction::new_stateful(name, None, &false.to_variant()),
                "line-numbers" | "show-explorer" => {
                    gio::SimpleAction::new_stateful(name, None, &true.to_variant())
                }
                _ => gio::SimpleAction::new(name, None),
            };
            window.add_action(&action);
            actions.insert(name, action);
            submenu.append(Some(&strings.text(label)), Some(&format!("win.{name}")));
        }
        model.append_submenu(Some(&strings.text(menu_name)), &submenu);
    }
    for shortcut in SHORTCUTS {
        application
            .set_accels_for_action(&format!("win.{}", shortcut.action), shortcut.accelerators);
    }
    root.append(&gtk::PopoverMenuBar::from_model(Some(&model)));
    let explorer = Explorer::new();
    let editor_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let tab_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tab_bar.add_css_class("tab-bar");
    let tabs_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let tab_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_width(true)
        .child(&tabs_box)
        .build();
    let plus = gtk::Button::with_label("+");
    plus.add_css_class("flat");
    plus.set_tooltip_text(Some(&strings.text("new-file")));
    tab_bar.append(&explorer.reveal);
    tab_bar.append(&tab_scroll);
    tab_bar.append(&plus);
    editor_area.append(&tab_bar);
    let stack = gtk::Stack::builder().hexpand(true).vexpand(true).build();
    stack.add_css_class("editor-surface");
    editor_area.append(&stack);
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&explorer.panel));
    paned.set_end_child(Some(&editor_area));
    paned.set_resize_start_child(false);
    paned.set_shrink_start_child(false);
    paned.set_position(250);
    root.append(&paned);
    window.set_child(Some(&root));
    let editor = Rc::new(Editor {
        window,
        explorer,
        stack,
        tab_bar,
        tabs_box,
        plus,
        tabs: RefCell::new(Vec::new()),
        active: Cell::new(None),
        next_id: Cell::new(1),
        busy: Cell::new(false),
        wrap: Cell::new(false),
        line_numbers: Cell::new(true),
        strings,
        actions,
        #[cfg(test)]
        file_choices: RefCell::new(std::collections::VecDeque::new()),
    });
    for (&name, action) in &editor.actions {
        let weak = Rc::downgrade(&editor);
        action.connect_activate(move |action, _| {
            if let Some(editor) = weak.upgrade() {
                editor.activate(name, action);
            }
        });
    }
    let weak = Rc::downgrade(&editor);
    editor.plus.connect_clicked(move |_| {
        if let Some(editor) = weak.upgrade() {
            editor.request(Intent::New);
        }
    });
    // The window owns the controller until destruction. Other callbacks are weak.
    let owner = editor.clone();
    let handler = editor.window.connect_close_request(move |_| {
        owner.request(Intent::Quit);
        glib::Propagation::Stop
    });
    let handler = RefCell::new(Some(handler));
    editor.window.connect_destroy(move |window| {
        if let Some(handler) = handler.borrow_mut().take() {
            window.disconnect(handler);
        }
    });
    editor.setup_explorer();
    editor.add_tab(Document::default());
    editor.window.present();
    editor
}
impl Editor {
    fn tab(&self, id: u64) -> Option<Rc<Tab>> {
        self.tabs.borrow().iter().find(|tab| tab.id == id).cloned()
    }
    fn current(&self) -> Option<Rc<Tab>> {
        self.active.get().and_then(|id| self.tab(id))
    }
    fn activate(self: &Rc<Self>, name: &str, action: &gio::SimpleAction) {
        if self.busy.get() {
            return;
        }
        match name {
            "open-folder" => self.operation(Operation::OpenFolder),
            "close-folder" => self.operation(Operation::CloseFolder),
            "new-folder" => self.operation(Operation::NewRootFolder),
            "show-explorer" => self.toggle_explorer(),
            "cut" | "copy" | "paste" => self.edit_clipboard(name),
            "new" => self.request(Intent::New),
            "open" => self.request(Intent::Open),
            "close" => {
                if let Some(id) = self.active.get() {
                    self.request(Intent::Close(id));
                }
            }
            "save" => {
                if let Some(id) = self.active.get() {
                    self.request(Intent::Save(id));
                }
            }
            "save-as" => {
                if let Some(id) = self.active.get() {
                    self.request(Intent::SaveAs(id));
                }
            }
            "undo" | "redo" => {
                if let Some(tab) = self.current() {
                    let buffer = tab.view.buffer();
                    if name == "undo" && buffer.can_undo() {
                        buffer.undo();
                    }
                    if name == "redo" && buffer.can_redo() {
                        buffer.redo();
                    }
                    self.refresh();
                }
            }
            "word-wrap" => {
                let enabled = !self.wrap.get();
                self.wrap.set(enabled);
                action.set_state(&enabled.to_variant());
                for tab in self.tabs.borrow().iter() {
                    tab.set_wrap(enabled);
                }
            }
            "line-numbers" => {
                let enabled = !self.line_numbers.get();
                self.line_numbers.set(enabled);
                action.set_state(&enabled.to_variant());
                for tab in self.tabs.borrow().iter() {
                    tab.view.set_line_numbers(enabled);
                }
            }
            "keyboard" => {
                popups::keyboard(&self.window, &self.strings);
            }
            "about" => {
                popups::about(&self.window, &self.strings);
            }
            _ => {}
        }
    }
    fn select(&self, id: u64) {
        if let Some(tab) = self.tab(id) {
            self.explorer.focused.set(false);
            self.active.set(Some(id));
            self.stack.set_visible_child_name(&id.to_string());
            self.refresh();
            tab.view.grab_focus();
        }
    }
    fn add_tab(self: &Rc<Self>, document: Document) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let tab = Tab::new(id, document, &self.strings);
        tab.set_wrap(self.wrap.get());
        tab.view.set_line_numbers(self.line_numbers.get());
        let weak = Rc::downgrade(self);
        tab.select.connect_clicked(move |_| {
            if let Some(editor) = weak.upgrade() {
                if !editor.busy.get() {
                    editor.select(id);
                }
            }
        });
        let weak = Rc::downgrade(self);
        tab.close.connect_clicked(move |_| {
            if let Some(editor) = weak.upgrade() {
                editor.request(Intent::Close(id));
            }
        });
        let weak = Rc::downgrade(self);
        tab.view.buffer().connect_changed(move |_| {
            if let Some(editor) = weak.upgrade() {
                if let Some(tab) = editor.tab(id) {
                    tab.synchronize();
                }
                editor.refresh();
            }
        });
        for property in ["can-undo", "can-redo"] {
            let weak = Rc::downgrade(self);
            tab.view
                .buffer()
                .connect_notify_local(Some(property), move |_, _| {
                    if let Some(editor) = weak.upgrade() {
                        editor.refresh();
                    }
                });
        }
        self.setup_tab_actions(&tab);
        self.tabs_box.append(&tab.header);
        self.stack.add_named(&tab.scroll, Some(&id.to_string()));
        self.tabs.borrow_mut().push(tab);
        self.select(id);
        id
    }
    fn remove_tab(self: &Rc<Self>, id: u64) {
        let mut tabs = self.tabs.borrow_mut();
        let Some(index) = tabs.iter().position(|tab| tab.id == id) else {
            return;
        };
        let tab = tabs.remove(index);
        let next = tabs.get(index).or_else(|| tabs.last()).map(|tab| tab.id);
        drop(tabs);
        self.tabs_box.remove(&tab.header);
        self.stack.remove(&tab.scroll);
        if self.active.get() == Some(id) {
            self.active.set(None);
            if let Some(id) = next {
                self.select(id);
            } else {
                self.add_tab(Document::default());
            }
        }
        self.refresh();
    }
    fn refresh(&self) {
        let current = self.current();
        let title = current
            .as_ref()
            .map(|tab| {
                let doc = tab.document.borrow();
                format!(
                    "{}{} — Leditor",
                    if doc.is_dirty() { "*" } else { "" },
                    doc.path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| self.strings.text("untitled"))
                )
            })
            .unwrap_or_else(|| "Leditor".to_owned());
        self.window.set_title(Some(&title));
        for (&name, action) in &self.actions {
            let enabled = match name {
                "close" => current.is_some(),
                "cut" | "copy" | "paste" if self.explorer.focused.get() => {
                    self.explorer.selected.borrow().is_some()
                }
                "save" | "save-as" | "cut" | "copy" | "paste" => {
                    current.as_ref().is_some_and(|t| !t.unsupported.get())
                }
                "undo" => current
                    .as_ref()
                    .is_some_and(|tab| tab.view.buffer().can_undo()),
                "redo" => current
                    .as_ref()
                    .is_some_and(|tab| tab.view.buffer().can_redo()),
                _ => true,
            };
            action.set_enabled(!self.busy.get() && enabled);
        }
        self.tab_bar.set_sensitive(!self.busy.get());
        for button in self.explorer.creators.borrow().iter() {
            button.set_sensitive(!self.busy.get() && self.explorer.root.borrow().is_some());
        }
        for tab in self.tabs.borrow().iter() {
            tab.refresh(&self.strings);
            tab.view
                .set_editable(!self.busy.get() && !tab.unsupported.get());
            if self.active.get() == Some(tab.id) {
                tab.header.add_css_class("active");
            } else {
                tab.header.remove_css_class("active");
            }
        }
    }
    fn request(self: &Rc<Self>, intent: Intent) {
        if self.busy.replace(true) {
            return;
        }
        self.refresh();
        let focus_editor = !matches!(&intent, Intent::Operation(op) if !matches!(op, Operation::Open(_) | Operation::Duplicate(_)));
        let editor = self.clone();
        glib::spawn_future_local(async move {
            editor.run(intent).await;
            editor.busy.set(false);
            editor.refresh();
            if focus_editor {
                if let Some(tab) = editor.current() {
                    tab.view.grab_focus();
                }
            }
        });
    }
    async fn run(self: &Rc<Self>, intent: Intent) {
        let save_as = matches!(intent, Intent::SaveAs(_));
        match intent {
            Intent::Operation(operation) => self.run_operation(operation).await,
            Intent::New => {
                self.add_tab(Document::default());
            }
            Intent::Open => match self.select_file(false, None, None).await {
                Ok(file) => {
                    if let Some(path) = file.path() {
                        self.open_path(&path).await;
                    } else {
                        self.error(&self.strings.text("local-only")).await;
                    }
                }
                Err(error) => self.dialog_error(error).await,
            },
            Intent::Close(id) => {
                if let Some(tab) = self.tab(id) {
                    if self.confirm_close(&tab).await {
                        self.remove_tab(id);
                    }
                }
            }
            Intent::Save(id) | Intent::SaveAs(id) => {
                if let Some(tab) = self.tab(id) {
                    self.save(&tab, save_as).await;
                }
            }
            Intent::Quit => {
                // Do not remove any tabs until every warning has been accepted.
                // A later Cancel therefore keeps even earlier discarded tabs open.
                let tabs = self.tabs.borrow().clone();
                for tab in tabs {
                    if !self.confirm_close(&tab).await {
                        return;
                    }
                }
                self.window.destroy();
            }
        }
    }
    fn open_tab(&self, path: &std::path::Path, excluding: Option<u64>) -> Option<u64> {
        self.tabs
            .borrow()
            .iter()
            .find(|tab| {
                Some(tab.id) != excluding
                    && tab
                        .document
                        .borrow()
                        .path
                        .as_ref()
                        .is_some_and(|p| same_path(p, path))
            })
            .map(|tab| tab.id)
    }
    async fn confirm_close(self: &Rc<Self>, tab: &Tab) -> bool {
        if !tab.document.borrow().is_dirty() {
            return true;
        }
        self.select(tab.id);
        let mut args = FluentArgs::new();
        args.set("name", tab.name(&self.strings));
        let buttons = [
            self.strings.text("cancel"),
            self.strings.text("dont-save"),
            self.strings.text("save"),
        ];
        let dialog = gtk::AlertDialog::builder()
            .message(self.strings.text("unsaved"))
            .detail(self.strings.format("unsaved-detail", Some(&args)))
            .modal(true)
            .buttons(buttons.iter().map(String::as_str).collect::<Vec<_>>())
            .cancel_button(0)
            .default_button(2)
            .build();
        match dialog.choose_future(Some(&self.window)).await {
            Ok(1) => true,
            Ok(2) => self.save(tab, false).await,
            _ => false,
        }
    }
    async fn save(self: &Rc<Self>, tab: &Tab, save_as: bool) -> bool {
        if tab.unsupported.get() {
            return false;
        }
        let current = tab.document.borrow().path.clone();
        let path = match current.clone().filter(|_| !save_as) {
            Some(path) => path,
            None => match self
                .select_file(true, current.as_deref(), Some(&tab.name(&self.strings)))
                .await
            {
                Ok(file) => match file.path() {
                    Some(path) => path,
                    None => {
                        self.error(&self.strings.text("local-only")).await;
                        return false;
                    }
                },
                Err(error) => {
                    self.dialog_error(error).await;
                    return false;
                }
            },
        };
        if self.open_tab(&path, Some(tab.id)).is_some() {
            self.error(&self.strings.text("destination-open")).await;
            return false;
        }
        let result = tab.document.borrow_mut().save_to(&path);
        match result {
            Ok(()) => {
                self.refresh_explorer();
                self.refresh();
                true
            }
            Err(error) => {
                self.error(&error.to_string()).await;
                false
            }
        }
    }
    /// The OS selector is a boundary: controller tests supply its result instead
    /// of relying on translated buttons or widgets in another portal process.
    #[cfg(not(test))]
    async fn select_file(
        &self,
        save: bool,
        current: Option<&std::path::Path>,
        suggested_name: Option<&str>,
    ) -> Result<gio::File, glib::Error> {
        let dialog = gtk::FileDialog::builder()
            .title(
                self.strings
                    .text(if save { "save-as" } else { "open-file" }),
            )
            .modal(true)
            .build();
        if save {
            if let Some(path) = current {
                dialog.set_initial_file(Some(&gio::File::for_path(path)));
            } else {
                dialog.set_initial_name(Some(suggested_name.unwrap_or("Untitled.txt")));
            }
            dialog.save_future(Some(&self.window)).await
        } else {
            dialog.open_future(Some(&self.window)).await
        }
    }

    #[cfg(test)]
    async fn select_file(
        &self,
        save: bool,
        _current: Option<&std::path::Path>,
        _suggested_name: Option<&str>,
    ) -> Result<gio::File, glib::Error> {
        let (expected_save, path) = self
            .file_choices
            .borrow_mut()
            .pop_front()
            .expect("test must queue a response for the file selector");
        assert_eq!(save, expected_save, "unexpected file selector operation");
        path.map(gio::File::for_path).ok_or_else(|| {
            glib::Error::new(
                gtk::DialogError::Dismissed,
                "File selection cancelled by test",
            )
        })
    }

    async fn dialog_error(&self, error: glib::Error) {
        if !error.matches(gtk::DialogError::Dismissed)
            && !error.matches(gtk::DialogError::Cancelled)
        {
            self.error(&error.to_string()).await;
        }
    }
    async fn error(&self, reason: &str) {
        let mut args = FluentArgs::new();
        args.set("reason", reason);
        let dialog = gtk::AlertDialog::builder()
            .message(self.strings.text("error-title"))
            .detail(self.strings.format("error-detail", Some(&args)))
            .buttons([self.strings.text("close").as_str()])
            .cancel_button(0)
            .modal(true)
            .build();
        let _ = dialog.choose_future(Some(&self.window)).await;
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "ui_tests.rs"]
mod tests;
