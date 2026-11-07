//! GtkSourceView editor with Leditor clipboard, auto-indent and guide behavior.
use gtk::{glib, prelude::*, subclass::prelude::*};
use sourceview5::prelude::{CompletionWordsExt, ViewExt};

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct EditorView {
        pub spaces: std::cell::Cell<u32>,
        pub auto_indent: std::cell::Cell<bool>,
        pub auto_close: std::cell::Cell<bool>,
        pub composing: std::cell::Cell<bool>,
        pub guides: std::cell::Cell<bool>,
        pub guide_columns: std::cell::RefCell<Vec<usize>>,
        pub css: gtk::CssProvider,
        pub css_id: std::cell::Cell<u64>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for EditorView {
        const NAME: &'static str = "LeditorEditorView";
        type Type = super::EditorView;
        type ParentType = sourceview5::View;
    }
    impl ObjectImpl for EditorView {
        fn dispose(&self) {
            gtk::style_context_remove_provider_for_display(&self.obj().display(), &self.css);
        }
    }
    impl WidgetImpl for EditorView {}
    impl sourceview5::subclass::prelude::ViewImpl for EditorView {}
    impl TextViewImpl for EditorView {
        fn snapshot_layer(&self, layer: gtk::TextViewLayer, snapshot: gtk::Snapshot) {
            self.parent_snapshot_layer(layer, snapshot.clone());
            if layer == gtk::TextViewLayer::BelowText {
                let view = self.obj();
                if self.guides.get() {
                    view.draw_guides(&snapshot);
                }
                let iter = view
                    .buffer()
                    .iter_at_offset(view.buffer().cursor_position());
                let (y, height) = view.line_yrange(&iter);
                let visible = view.visible_rect();
                snapshot.append_color(
                    &gtk::gdk::RGBA::new(0.5, 0.5, 0.5, 0.18),
                    &gtk::graphene::Rect::new(
                        visible.x() as f32,
                        y as f32,
                        visible.width() as f32,
                        height as f32,
                    ),
                );
            }
        }
        fn copy_clipboard(&self) {
            self.obj().copy_or_cut(false);
        }
        fn cut_clipboard(&self) {
            self.obj().copy_or_cut(true);
        }
    }
}
glib::wrapper! {
    pub struct EditorView(ObjectSubclass<imp::EditorView>)
        @extends sourceview5::View, gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}
impl EditorView {
    pub fn set_editing_options(&self, auto_close: bool, whitespace: bool) {
        self.imp().auto_close.set(auto_close);
        let drawer = self.space_drawer();
        drawer.set_types_for_locations(
            sourceview5::SpaceLocationFlags::ALL,
            sourceview5::SpaceTypeFlags::SPACE | sourceview5::SpaceTypeFlags::TAB,
        );
        drawer.set_enable_matrix(whitespace);
    }
    /// Only typed delimiters are paired. Paste and programmatic changes remain literal.
    pub(crate) fn type_delimiter(&self, ch: char) -> bool {
        if !self.imp().auto_close.get()
            || !self.is_editable()
            || self.imp().composing.get()
            || self.overwrites()
        {
            return false;
        }
        let b = self.buffer();
        let cursor = b.iter_at_offset(b.cursor_position());
        let selection = b.selection_bounds();
        let mut previous = cursor;
        let before = previous.backward_char().then(|| previous.char());
        if matches!(ch, '\'' | '"') && selection.is_none() {
            let mut iter = cursor;
            let mut slashes = 0;
            while iter.backward_char() && iter.char() == '\\' {
                slashes += 1;
            }
            if slashes % 2 == 1 {
                return false;
            }
        }
        if selection.is_none() && matches!(ch, ')' | ']' | '}' | '\'' | '"') && cursor.char() == ch
        {
            let mut next = cursor;
            next.forward_char();
            b.place_cursor(&next);
            self.scroll_mark_onscreen(&b.get_insert());
            return true;
        }
        let close = match ch {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '\'' => '\'',
            '"' => '"',
            _ => return false,
        };
        if selection.is_none()
            && matches!(ch, '\'' | '"')
            && (before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                || cursor.char().is_alphanumeric()
                || cursor.char() == '_')
        {
            return false;
        }
        b.begin_user_action();
        if let Some((start, mut end)) = selection {
            let (s, e) = (start.offset(), end.offset());
            let reversed = b.cursor_position() == s;
            b.insert(&mut end, &close.to_string());
            b.insert(&mut b.iter_at_offset(s), &ch.to_string());
            let (start, end) = (b.iter_at_offset(s + 1), b.iter_at_offset(e + 1));
            if reversed {
                b.select_range(&start, &end);
            } else {
                b.select_range(&end, &start);
            }
        } else {
            let offset = cursor.offset();
            b.insert_at_cursor(&format!("{ch}{close}"));
            b.place_cursor(&b.iter_at_offset(offset + 1));
        }
        b.end_user_action();
        self.scroll_mark_onscreen(&b.get_insert());
        true
    }

    pub fn set_indentation_options(&self, auto_indent: bool, guides: bool) {
        self.imp().auto_indent.set(auto_indent);
        self.imp().guides.set(guides);
        self.queue_draw();
    }
    pub fn insert_newline(&self) {
        if !self.is_editable() {
            return;
        }
        let buffer = self.buffer();
        buffer.begin_user_action();
        buffer.delete_selection(true, true);
        let cursor = buffer.iter_at_offset(buffer.cursor_position());
        let mut start = cursor;
        start.set_line_offset(0);
        let prefix = buffer.text(&start, &cursor, true);
        let indent = if self.imp().auto_indent.get() {
            crate::text_ops::leading_indent(&prefix)
        } else {
            ""
        };
        buffer.insert_at_cursor(&format!("\n{indent}"));
        buffer.end_user_action();
        self.scroll_mark_onscreen(&buffer.get_insert());
    }
    fn draw_guides(&self, snapshot: &gtk::Snapshot) {
        let visible = self.visible_rect();
        let (mut iter, _) = self.line_at_y(visible.y());
        let width = self.imp().spaces.get().clamp(1, 8) as usize;
        let color = gtk::gdk::RGBA::new(0.5, 0.5, 0.5, 0.35);
        snapshot.push_clip(&gtk::graphene::Rect::new(
            visible.x() as f32,
            visible.y() as f32,
            visible.width() as f32,
            visible.height() as f32,
        ));
        loop {
            let (y, height) = self.line_yrange(&iter);
            if y > visible.y() + visible.height() {
                break;
            }
            let columns = self
                .imp()
                .guide_columns
                .borrow()
                .get(iter.line() as usize)
                .copied()
                .unwrap_or(0);
            let origin = self.iter_location(&iter);
            let space = self.create_pango_layout(Some("        ")).pixel_size().0 as f32 / 8.0;
            let row_height = height.min(origin.height());
            for col in (width..=columns).step_by(width) {
                let gx = origin.x() as f32 + col as f32 * space - space * 0.5;
                if gx >= (visible.x() + visible.width()) as f32 {
                    break;
                }
                // Half a character before the text; dash phase follows document coordinates.
                for dy in 0..row_height {
                    if (y + dy).rem_euclid(6) < 3 {
                        snapshot.append_color(
                            &color,
                            &gtk::graphene::Rect::new(gx, (y + dy) as f32, 1.0, 1.0),
                        );
                    }
                }
            }
            if !iter.forward_line() {
                break;
            }
        }
        snapshot.pop();
    }

    fn update_guides(&self) {
        let b = self.buffer();
        self.imp()
            .guide_columns
            .replace(crate::text_ops::guide_columns(
                &b.text(&b.start_iter(), &b.end_iter(), true),
                self.imp().spaces.get().clamp(1, 8) as usize,
            ));
    }
    pub fn configure(&self, spaces: u32, convert: bool, zoom: f64) {
        if self.imp().spaces.replace(spaces) != spaces {
            self.update_guides();
        }
        self.set_tab_width(spaces);
        self.set_indent_width(spaces as i32);
        self.set_insert_spaces_instead_of_tabs(convert);
        self.set_indent_on_tab(true);
        self.imp().css.load_from_data(&format!(
            "textview.leditor-text-{} {{ font-size: {}px; }}",
            self.imp().css_id.get(),
            14.0 * zoom / 100.0
        ));
    }

    pub fn copy_or_cut(&self, cut: bool) {
        if cut && !self.is_editable() {
            return;
        }
        let buffer = self.buffer();
        let (mut start, mut end) = buffer.selection_bounds().unwrap_or_else(|| {
            let mut start = buffer.iter_at_offset(buffer.cursor_position());
            start.set_line_offset(0);
            let mut end = start;
            end.forward_line();
            (start, end)
        });
        self.clipboard().set_text(&buffer.text(&start, &end, true));
        if cut {
            buffer.begin_user_action();
            buffer.delete(&mut start, &mut end);
            buffer.end_user_action();
        }
    }
    pub fn new(text: &str) -> Self {
        let view: Self = glib::Object::builder().build();
        // Shadow the native clipboard actions while preserving GTK's full
        // context menu. GTK otherwise disables Cut/Copy without a selection.
        let actions = gtk::gio::SimpleActionGroup::new();
        for name in ["cut", "copy", "paste"] {
            let action = gtk::gio::SimpleAction::new(name, None);
            let weak = view.downgrade();
            action.connect_activate(move |_, _| {
                if let Some(view) = weak.upgrade() {
                    match name {
                        "cut" => view.copy_or_cut(true),
                        "copy" => view.copy_or_cut(false),
                        _ => view.buffer().paste_clipboard(
                            &view.clipboard(),
                            None,
                            view.is_editable(),
                        ),
                    }
                }
            });
            actions.add_action(&action);
        }
        view.insert_action_group("clipboard", Some(&actions));
        static NEXT_STYLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_STYLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        view.imp().css_id.set(id);
        view.add_css_class(&format!("leditor-text-{id}"));
        gtk::style_context_add_provider_for_display(
            &view.display(),
            &view.imp().css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = view.downgrade();
        keys.connect_key_pressed(move |_, key, _, state| {
            if !state.intersects(
                gtk::gdk::ModifierType::CONTROL_MASK
                    | gtk::gdk::ModifierType::ALT_MASK
                    | gtk::gdk::ModifierType::SUPER_MASK
                    | gtk::gdk::ModifierType::META_MASK,
            ) {
                if let (Some(view), Some(ch)) = (weak.upgrade(), key.to_unicode()) {
                    if view.type_delimiter(ch) {
                        return glib::Propagation::Stop;
                    }
                }
            }
            if (key == gtk::gdk::Key::Return || key == gtk::gdk::Key::KP_Enter)
                && !state.intersects(
                    gtk::gdk::ModifierType::CONTROL_MASK
                        | gtk::gdk::ModifierType::ALT_MASK
                        | gtk::gdk::ModifierType::SHIFT_MASK,
                )
            {
                if let Some(view) = weak.upgrade() {
                    view.insert_newline();
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        });
        view.add_controller(keys);
        view.connect_preedit_changed(|view, text| {
            view.imp().composing.set(!text.is_empty());
        });
        view.set_monospace(true);
        view.set_smart_home_end(sourceview5::SmartHomeEndType::Before);
        view.set_left_margin(8);
        view.set_right_margin(8);
        view.set_top_margin(8);
        view.set_bottom_margin(8);
        view.add_css_class("editor-surface");
        view.set_show_line_numbers(true);
        let buffer = sourceview5::Buffer::new(None);
        view.set_buffer(Some(&buffer));
        buffer.set_enable_undo(false);
        buffer.set_text(text);
        buffer.set_enable_undo(true);
        // One provider per view, with only this document registered. No words
        // from other tabs, external dictionaries or language servers are used.
        let words = sourceview5::CompletionWords::new(None);
        words.register(&buffer);
        let completion = view.completion();
        completion.set_select_on_show(true);
        completion.add_provider(&words);
        view.configure(4, false, 100.0);
        view.set_indentation_options(true, false);
        let weak = view.downgrade();
        buffer.connect_mark_set(move |_, _, _| {
            if let Some(view) = weak.upgrade() {
                view.queue_draw();
            }
        });
        let weak = view.downgrade();
        buffer.connect_changed(move |_| {
            if let Some(view) = weak.upgrade() {
                view.update_guides();
                view.queue_draw();
            }
        });
        view.update_guides();
        view
    }
    pub fn set_line_numbers(&self, enabled: bool) {
        self.set_show_line_numbers(enabled);
    }
}
