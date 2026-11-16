use super::*;
use std::io::Write;

impl Editor {
    pub(super) fn setup_markdown_input(self: &Rc<Self>, tab: &Rc<Tab>) {
        let weak = Rc::downgrade(self);
        let id = tab.id;
        tab.view.set_paste_handler(move || {
            let Some(editor) = weak.upgrade() else {
                return false;
            };
            let Some(tab) = editor.tab(id) else {
                return false;
            };
            let path = tab.document.borrow().path.clone();
            let markdown = crate::markdown::is_markdown(path.as_deref())
                || (path.is_none()
                    && tab
                        .source_buffer()
                        .language()
                        .is_some_and(|l| l.id() == "markdown"));
            let formats = tab.view.clipboard().formats();
            if !markdown
                || !(formats.contains_type(gtk::gdk::Texture::static_type())
                    || formats.mime_types().iter().any(|m| m.starts_with("image/")))
            {
                return false;
            }
            editor.operation(Operation::PasteImage(id));
            true
        });
        let zoom = gtk::GestureZoom::new();
        zoom.set_propagation_phase(gtk::PropagationPhase::Capture);
        let start = Rc::new(Cell::new(100.0));
        let initial = start.clone();
        let weak = Rc::downgrade(self);
        zoom.connect_begin(move |_, _| {
            if let Some(editor) = weak.upgrade() {
                initial.set(editor.zoom.get());
            }
        });
        let weak = Rc::downgrade(self);
        zoom.connect_scale_changed(move |gesture, scale| {
            if let Some(editor) = weak.upgrade() {
                if !editor.busy.get() && scale.is_finite() && scale > 0.0 {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    editor.set_zoom(start.get() * scale);
                }
            }
        });
        let weak = Rc::downgrade(self);
        zoom.connect_end(move |_, _| {
            if let Some(editor) = weak.upgrade() {
                editor.save_settings();
            }
        });
        tab.content.add_controller(zoom);
    }

    pub(super) async fn paste_markdown_image(self: &Rc<Self>, id: u64) {
        let Some(tab) = self.tab(id).filter(|t| t.is_text()) else {
            return;
        };
        if self.only_view.get() {
            return;
        }
        let texture = match tab.view.clipboard().read_texture_future().await {
            Ok(Some(texture)) => texture,
            Ok(None) => return,
            Err(error) => {
                self.error(&format!(
                    "{}: {error}",
                    self.strings.text("paste-image-failed")
                ))
                .await;
                return;
            }
        };
        if tab.document.borrow().path.is_none() {
            let old_name = tab.display_name.replace(Some("Untitled.md".into()));
            let saved = self.save(&tab, true).await;
            tab.display_name.replace(old_name);
            if !saved {
                return;
            }
        }
        let path = tab.document.borrow().path.clone();
        if !crate::markdown::is_markdown(path.as_deref()) {
            self.error(&self.strings.text("paste-save-markdown")).await;
            return;
        }
        let Some(directory) = path.and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        else {
            return;
        };
        let bytes = texture.save_to_png_bytes();
        let result = gio::spawn_blocking(move || -> std::io::Result<std::path::PathBuf> {
            // Atomic unique creation, including concurrent pastes/processes.
            let mut image = tempfile::Builder::new()
                .prefix("pasted-image-")
                .suffix(".png")
                .tempfile_in(directory)?;
            image.write_all(&bytes)?;
            image.as_file().sync_all()?;
            let (_, path) = image.keep().map_err(|e| e.error)?;
            Ok(path)
        })
        .await;
        match result {
            Ok(Ok(path)) => {
                let name = path.file_name().unwrap().to_string_lossy();
                let markdown = format!("![{}]({name})", self.strings.text("pasted-image"));
                let buffer = tab.source_buffer();
                buffer.begin_user_action();
                if let Some((mut start, mut end)) = buffer.selection_bounds() {
                    buffer.delete(&mut start, &mut end);
                }
                buffer.insert_at_cursor(&markdown);
                buffer.end_user_action();
                tab.view.scroll_mark_onscreen(&buffer.get_insert());
                self.refresh_explorer();
            }
            Ok(Err(error)) => {
                self.error(&format!(
                    "{}: {error}",
                    self.strings.text("paste-image-failed")
                ))
                .await
            }
            Err(_) => self.error(&self.strings.text("paste-image-failed")).await,
        }
    }
}
