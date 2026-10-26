use super::*;
use crate::session;
use std::time::{Duration, Instant};

impl Editor {
    pub(super) fn apply_text_options(&self) {
        for tab in self.tabs.borrow().iter() {
            tab.view
                .configure(self.spaces.get(), self.convert.get(), self.zoom.get());
        }
    }
    pub(super) fn set_zoom(&self, value: f64) {
        self.zoom.set(value.clamp(5.0, 800.0));
        self.zoom_input.set_value(self.zoom.get());
        self.apply_text_options();
    }
    pub(super) fn scroll_zoom(&self, delta: f64) {
        let now = Instant::now();
        if self
            .zoom_snap
            .get()
            .is_some_and(|t| now.duration_since(t) < Duration::from_millis(220))
        {
            return;
        }
        let old = self.zoom.get();
        let new = (old - delta * 5.0).clamp(5.0, 800.0);
        if (old < 100.0 && new >= 100.0) || (old > 100.0 && new <= 100.0) {
            self.zoom_snap.set(Some(now));
            self.set_zoom(100.0);
        } else {
            self.set_zoom(new);
        }
    }
    pub(super) fn recovery_key(&self) -> glib::KeyFile {
        let key = glib::KeyFile::new();
        key.set_integer("session", "version", 1);
        let tabs = self.tabs.borrow();
        key.set_integer("session", "count", tabs.len() as i32);
        key.set_integer(
            "session",
            "active",
            tabs.iter()
                .position(|t| Some(t.id) == self.active.get())
                .unwrap_or(0) as i32,
        );
        if let Some(path) = &*self.explorer.root.borrow() {
            session::set_path(&key, "session", "folder", path);
        }
        for (index, tab) in tabs.iter().enumerate() {
            let group = format!("tab-{index}");
            tab.document.borrow().write_recovery(&key, &group);
            key.set_string(&group, "name", &tab.name(&self.strings));
            key.set_boolean(&group, "unsupported", tab.unsupported.get());
            key.set_integer(&group, "cursor", tab.view.buffer().cursor_position());
        }
        key
    }
    pub(super) fn save_recovery(&self) -> std::io::Result<()> {
        session::write(
            &session::directory().join("session.ini"),
            &self.recovery_key(),
        )
    }
    pub(super) fn save_settings(&self) {
        let key = glib::KeyFile::new();
        key.set_boolean("settings", "wrap", self.wrap.get());
        key.set_boolean("settings", "numbers", self.line_numbers.get());
        key.set_boolean("settings", "explorer", self.explorer.panel.is_visible());
        key.set_boolean("settings", "convert", self.convert.get());
        key.set_boolean("settings", "light", self.light.get());
        key.set_integer("settings", "spaces", self.spaces.get() as i32);
        key.set_double("settings", "zoom", self.zoom.get());
        key.set_integer("settings", "sidebar-width", self.paned.position());
        key.set_string("settings", "language", &self.language.borrow());
        key.set_boolean("settings", "autosave", self.autosave.get());
        key.set_boolean("settings", "restore", self.restore.get());
        if let Err(e) = session::write(&session::directory().join("settings.ini"), &key) {
            self.show_background_error(&e.to_string());
        }
    }
    pub(super) fn show_background_error(&self, reason: &str) {
        let mut args = FluentArgs::new();
        args.set("reason", reason);
        self.notice
            .set_text(&self.strings.format("background-save-error", Some(&args)));
        self.notice.set_visible(true);
    }
    pub(super) fn setup_preferences(self: &Rc<Self>) {
        let settings = session::read(&session::directory().join("settings.ini")).ok();
        self.explorer.panel.set_visible(false);
        self.explorer.reveal.set_visible(true);
        if let Some(key) = settings {
            self.wrap
                .set(key.boolean("settings", "wrap").unwrap_or(false));
            self.line_numbers
                .set(key.boolean("settings", "numbers").unwrap_or(true));
            self.convert
                .set(key.boolean("settings", "convert").unwrap_or(false));
            self.light
                .set(key.boolean("settings", "light").unwrap_or(false));
            self.spaces
                .set(key.integer("settings", "spaces").unwrap_or(4).clamp(1, 8) as u32);
            let zoom = key.double("settings", "zoom").unwrap_or(100.0);
            self.zoom.set(if zoom.is_finite() {
                zoom.clamp(5.0, 800.0)
            } else {
                100.0
            });
            let visible = key.boolean("settings", "explorer").unwrap_or(false);
            self.explorer.panel.set_visible(visible);
            self.explorer.reveal.set_visible(!visible);
            self.paned.set_position(
                key.integer("settings", "sidebar-width")
                    .unwrap_or(250)
                    .clamp(80, 2000),
            );
            self.autosave
                .set(key.boolean("settings", "autosave").unwrap_or(false));
            self.restore
                .set(self.autosave.get() && key.boolean("settings", "restore").unwrap_or(false));
        }
        self.actions["enable-autosave"].set_state(&self.autosave.get().to_variant());
        self.actions["restore-session"].set_state(&self.restore.get().to_variant());
        for (name, value) in [
            ("word-wrap", self.wrap.get()),
            ("line-numbers", self.line_numbers.get()),
            ("convert-tabs", self.convert.get()),
            ("show-explorer", self.explorer.panel.is_visible()),
        ] {
            self.actions[name].set_state(&value.to_variant());
        }
        self.zoom_input.set_value(self.zoom.get());
        self.spaces_input.set_value(self.spaces.get() as f64);
        for name in ["word-wrap", "line-numbers", "convert-tabs"] {
            let weak = Rc::downgrade(self);
            self.actions[name].connect_activate(move |_, _| {
                if let Some(e) = weak.upgrade() {
                    e.save_settings();
                }
            });
        }
        let weak = Rc::downgrade(self);
        self.paned.connect_position_notify(move |_| {
            if let Some(e) = weak.upgrade() {
                e.save_settings();
            }
        });
        for (input, is_zoom) in [(&self.zoom_input, true), (&self.spaces_input, false)] {
            let weak = Rc::downgrade(self);
            input.connect_value_changed(move |input| {
                if let Some(e) = weak.upgrade() {
                    if is_zoom {
                        e.zoom.set(input.value());
                    } else {
                        e.spaces.set(input.value_as_int() as u32);
                    }
                    e.apply_text_options();
                    e.update_footer();
                    e.save_settings();
                }
            });
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let Some(e) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if e.window.is_visible()
                && e.autosave.get()
                && !e.busy.get()
                && e.pending
                    .get()
                    .is_some_and(|t| t.elapsed() >= Duration::from_millis(600))
            {
                e.pending.set(None);
                e.autosave_now();
            }
            glib::ControlFlow::Continue
        });
    }
    pub(super) fn autosave_now(self: &Rc<Self>) {
        // Snapshot first: even a failed file write leaves recoverable text.
        if let Err(error) = self.save_recovery() {
            self.show_background_error(&error.to_string());
        }
        let jobs: Vec<_> = self
            .tabs
            .borrow()
            .iter()
            .filter_map(|tab| {
                let doc = tab.document.borrow();
                (doc.is_dirty()
                    && doc.path.is_some()
                    && !tab.unsupported.get()
                    && !self.autosave_failed.borrow().contains(&tab.id))
                .then(|| (tab.id, doc.clone()))
            })
            .collect();
        if jobs.is_empty() {
            return;
        }
        self.saving
            .borrow_mut()
            .extend(jobs.iter().map(|(id, _)| *id));
        self.update_footer();
        self.busy.set(true);
        let editor = self.clone();
        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || {
                jobs.into_iter()
                    .map(|(id, mut doc)| {
                        let result = doc.autosave();
                        (id, doc, result)
                    })
                    .collect::<Vec<_>>()
            })
            .await;
            match result {
                Ok(results) => {
                    for (id, doc, result) in results {
                        match result {
                            Ok(()) => {
                                if let Some(tab) = editor.tab(id) {
                                    tab.document.borrow_mut().accept_saved_snapshot(&doc);
                                }
                            }
                            Err(error) => {
                                editor.autosave_failed.borrow_mut().insert(id);
                                editor.show_background_error(&error.to_string());
                            }
                        }
                    }
                }
                Err(_) => editor.show_background_error("Autosave worker failed"),
            }
            editor.saving.borrow_mut().clear();
            editor.busy.set(false);
            editor.refresh();
            if let Err(error) = editor.save_recovery() {
                editor.show_background_error(&error.to_string());
            }
        });
    }
    pub(super) fn restore_startup(self: &Rc<Self>) {
        if !self.restore.get() {
            return;
        }
        let path = session::directory().join("session.ini");
        if !path.exists() {
            return;
        }
        let result = (|| -> Result<_, glib::Error> {
            let key = session::read(&path)?;
            if key.integer("session", "version")? != 1 {
                return Err(glib::Error::new(
                    glib::KeyFileError::InvalidValue,
                    "Unsupported recovery format",
                ));
            }
            let count = key.integer("session", "count")?;
            if !(0..=10000).contains(&count) {
                return Err(glib::Error::new(
                    glib::KeyFileError::InvalidValue,
                    "Invalid tab count",
                ));
            }
            let tabs = (0..count)
                .map(|i| session::recover(&key, &format!("tab-{i}")))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((key, tabs))
        })();
        match result {
            Ok((key, tabs)) => {
                if let Some(folder) =
                    session::path(&key, "session", "folder").filter(|p| p.is_dir())
                {
                    self.load_folder(folder);
                }
                let mut ids = Vec::new();
                for recovered in tabs {
                    let id = self.add_tab(recovered.document);
                    let tab = self.tab(id).unwrap();
                    tab.display_name.replace(recovered.name);
                    if recovered.unsupported {
                        tab.mark_unsupported(&self.strings);
                    }
                    let buffer = tab.view.buffer();
                    buffer.place_cursor(
                        &buffer.iter_at_offset(recovered.cursor.clamp(0, buffer.char_count())),
                    );
                    ids.push(id);
                }
                if let Some(id) =
                    ids.get(key.integer("session", "active").unwrap_or(0).max(0) as usize)
                {
                    self.select(*id);
                }
            }
            Err(error) => {
                // Preserve unreadable data: no background overwrite of this snapshot.
                self.autosave.set(false);
                self.restore.set(false);
                self.actions["enable-autosave"].set_state(&false.to_variant());
                self.actions["restore-session"].set_state(&false.to_variant());
                self.show_background_error(&error.to_string());
            }
        }
    }
}
