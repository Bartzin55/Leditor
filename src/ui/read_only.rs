use super::*;
use sourceview5::prelude::ViewExt;

impl Editor {
    pub(super) fn toggle_only_view(&self) {
        self.only_view.set(!self.only_view.get());
        self.actions["only-view"].set_state(&self.only_view.get().to_variant());
        for tab in self.tabs.borrow().iter() {
            tab.view.completion().hide();
        }
        self.refresh();
        if self.current().is_some_and(|tab| tab.is_preview()) {
            self.close_search();
            self.close_goto();
        }
        if let Some(tab) = self.current() {
            if tab.is_preview() {
                tab.preview_scroll.grab_focus();
            } else {
                tab.view.grab_focus();
            }
        }
        self.save_settings();
    }
    pub(super) fn refresh_read_only(&self) {
        let active = self.only_view.get();
        self.view_button.set_icon_name(if active {
            "document-edit-symbolic"
        } else {
            "view-reveal-symbolic"
        });
        let tooltip = self.strings.text(if active {
            "resume-editing"
        } else {
            "only-view"
        });
        self.view_button.set_tooltip_text(Some(&tooltip));
        self.view_button
            .update_property(&[gtk::accessible::Property::Label(&tooltip)]);
        for tab in self.tabs.borrow().iter() {
            let path = tab.document.borrow().path.clone();
            let markdown = active && tab.is_text() && crate::markdown::is_markdown(path.as_deref());
            if markdown {
                let rebuild = tab
                    .preview
                    .borrow()
                    .as_ref()
                    .is_none_or(|preview| preview.source_path != path);
                if rebuild {
                    if !tab.is_preview() {
                        tab.edit_position.set((
                            tab.scroll.hadjustment().value(),
                            tab.scroll.vadjustment().value(),
                        ));
                    }
                    let preview = crate::markdown::MarkdownPreview::new(
                        &tab.document.borrow().text,
                        path,
                        &tab.preview_scroll,
                        &self.strings,
                    );
                    preview.set_zoom(self.zoom.get());
                    preview.set_theme(self.preview_light());
                    tab.preview_scroll.set_child(Some(&preview.content));
                    tab.preview.replace(Some(preview));
                }
                // Retain the source view's parent and scroll adjustments. A
                // GtkSourceMap tracks those adjustments even while hidden.
                tab.editor_area.set_visible(false);
                tab.preview_scroll.set_visible(true);
            } else {
                let old_preview = tab.preview.borrow_mut().take();
                if old_preview.is_none() {
                    continue;
                }
                tab.preview_scroll.set_visible(false);
                tab.preview_scroll.set_child(gtk::Widget::NONE);
                tab.editor_area.set_visible(true);
                tab.set_wrap(self.wrap.get());
                let scroll = tab.scroll.downgrade();
                let (x, y) = tab.edit_position.get();
                glib::idle_add_local_once(move || {
                    if let Some(scroll) = scroll.upgrade() {
                        scroll.hadjustment().set_value(x);
                        scroll.vadjustment().set_value(y);
                    }
                });
            }
            tab.minimap
                .set_visible(self.minimap.get() && tab.is_text() && !markdown);
        }
    }
    pub(super) fn preview_light(&self) -> bool {
        let theme = self.theme.borrow();
        theme.as_str() == "light" || (theme.as_str() == "system" && !self.system_dark.get())
    }
}
