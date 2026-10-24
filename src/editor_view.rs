//! TextView's native gutter and layout are reused so numbering follows logical
//! lines, including wrapping, vertical scrolling, empty lines and final newlines.
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct NumberedView {
        pub spaces: std::cell::Cell<u32>,
        pub convert: std::cell::Cell<bool>,
        pub css: gtk::CssProvider,
        pub css_id: std::cell::Cell<u64>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for NumberedView {
        const NAME: &'static str = "LeditorNumberedView";
        type Type = super::NumberedView;
        type ParentType = gtk::TextView;
    }
    impl ObjectImpl for NumberedView {
        fn dispose(&self) {
            gtk::style_context_remove_provider_for_display(&self.obj().display(), &self.css);
        }
    }
    impl WidgetImpl for NumberedView {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            let view = self.obj();
            let Some(gutter) = view
                .gutter(gtk::TextWindowType::Left)
                .filter(|w| w.is_visible())
            else {
                return;
            };
            let width = gutter.width();
            if width <= 0 {
                return;
            }
            let visible = view.visible_rect();
            let (mut iter, _) = view.line_at_y(visible.y());
            let color = gtk::gdk::RGBA::new(0.58, 0.58, 0.58, 1.0);
            snapshot.push_clip(&gtk::graphene::Rect::new(
                0.0,
                0.0,
                width as f32,
                view.height() as f32,
            ));
            loop {
                let rectangle = view.iter_location(&iter);
                let (_, y) =
                    view.buffer_to_window_coords(gtk::TextWindowType::Widget, 0, rectangle.y());
                if y >= view.height() {
                    break;
                }
                let layout = view.create_pango_layout(Some(&(iter.line() + 1).to_string()));
                let (text_width, text_height) = layout.pixel_size();
                if y + rectangle.height() > 0 {
                    snapshot.save();
                    snapshot.translate(&gtk::graphene::Point::new(
                        (width - text_width - 8) as f32,
                        (y + (rectangle.height() - text_height) / 2) as f32,
                    ));
                    let current = view
                        .buffer()
                        .iter_at_offset(view.buffer().cursor_position())
                        .line();
                    let yellow = gtk::gdk::RGBA::new(1.0, 0.83, 0.12, 1.0);
                    snapshot.append_layout(
                        &layout,
                        if iter.line() == current {
                            &yellow
                        } else {
                            &color
                        },
                    );
                    snapshot.restore();
                }
                let next_line = iter.line() + 1;
                let Some(next) = view.buffer().iter_at_line(next_line) else {
                    break;
                };
                iter = next;
            }
            snapshot.pop();
        }
    }
    impl TextViewImpl for NumberedView {
        fn snapshot_layer(&self, layer: gtk::TextViewLayer, snapshot: gtk::Snapshot) {
            self.parent_snapshot_layer(layer, snapshot.clone());
            if layer == gtk::TextViewLayer::BelowText {
                let view = self.obj();
                let iter = view
                    .buffer()
                    .iter_at_offset(view.buffer().cursor_position());
                let (y, height) = view.line_yrange(&iter);
                let visible = view.visible_rect();
                snapshot.append_color(
                    &gtk::gdk::RGBA::new(0.18, 0.18, 0.18, 1.0),
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
    pub struct NumberedView(ObjectSubclass<imp::NumberedView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}
impl NumberedView {
    pub fn configure(&self, spaces: u32, convert: bool, zoom: f64) {
        self.imp().spaces.set(spaces);
        self.imp().convert.set(convert);
        self.imp().css.load_from_data(&format!(
            "textview.leditor-text-{} {{ font-size: {}px; }}",
            self.imp().css_id.get(),
            14.0 * zoom / 100.0
        ));
        self.update_tab_metrics();
        // GTK may resolve changed CSS on the next main-loop iteration.
        let weak = self.downgrade();
        glib::idle_add_local_once(move || {
            if let Some(view) = weak.upgrade() {
                view.update_tab_metrics();
            }
        });
    }
    fn update_tab_metrics(&self) {
        let layout = self.create_pango_layout(Some(&" ".repeat(self.imp().spaces.get() as usize)));
        let mut tabs = gtk::pango::TabArray::new(1, true);
        tabs.set_tab(0, gtk::pango::TabAlign::Left, layout.pixel_size().0.max(1));
        self.set_tabs(&tabs);
        self.update_gutter();
    }
    pub fn indent(&self, outdent: bool) {
        if !self.is_editable() {
            return;
        }
        let buffer = self.buffer();
        let width = self.imp().spaces.get().clamp(1, 8) as usize;
        let insertion = if self.imp().convert.get() {
            " ".repeat(width)
        } else {
            "\t".into()
        };
        let selection = buffer.selection_bounds();
        buffer.begin_user_action();
        if !outdent && selection.is_none() {
            buffer.insert_at_cursor(&insertion);
        } else {
            let (start, end) = selection.unwrap_or_else(|| {
                let cursor = buffer.iter_at_offset(buffer.cursor_position());
                (cursor, cursor)
            });
            let last = if end.line() > start.line() && end.line_offset() == 0 {
                end.line() - 1
            } else {
                end.line()
            };
            for line in (start.line()..=last).rev() {
                if let Some(mut first) = buffer.iter_at_line(line) {
                    if outdent {
                        let mut end = first;
                        if end.char() == '\t' {
                            end.forward_char();
                        } else {
                            for _ in 0..width {
                                if end.char() != ' ' {
                                    break;
                                }
                                end.forward_char();
                            }
                        }
                        buffer.delete(&mut first, &mut end);
                    } else {
                        buffer.insert(&mut first, &insertion);
                    }
                }
            }
        }
        buffer.end_user_action();
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
            if !state
                .intersects(gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::ALT_MASK)
                && (key == gtk::gdk::Key::Tab || key == gtk::gdk::Key::ISO_Left_Tab)
            {
                if let Some(view) = weak.upgrade() {
                    view.indent(
                        key == gtk::gdk::Key::ISO_Left_Tab
                            || state.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                    );
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        });
        view.add_controller(keys);
        view.set_monospace(true);
        view.set_left_margin(8);
        view.set_right_margin(8);
        view.set_top_margin(8);
        view.set_bottom_margin(8);
        view.add_css_class("editor-surface");
        let gutter = gtk::Box::new(gtk::Orientation::Vertical, 0);
        gutter.add_css_class("line-numbers");
        view.set_gutter(gtk::TextWindowType::Left, Some(&gutter));
        let buffer = view.buffer();
        buffer.set_enable_undo(false);
        buffer.set_text(text);
        buffer.set_enable_undo(true);
        view.configure(4, false, 100.0);
        let weak = view.downgrade();
        buffer.connect_mark_set(move |_, _, _| {
            if let Some(view) = weak.upgrade() {
                view.queue_draw();
            }
        });
        let weak = view.downgrade();
        buffer.connect_changed(move |_| {
            if let Some(view) = weak.upgrade() {
                view.update_gutter();
            }
        });
        view
    }
    pub fn update_gutter(&self) {
        let digits = self.buffer().line_count().to_string();
        let layout = self.create_pango_layout(Some(&digits));
        if let Some(gutter) = self.gutter(gtk::TextWindowType::Left) {
            gutter.set_width_request(layout.pixel_size().0 + 16);
        }
        self.queue_draw();
    }
    pub fn set_line_numbers(&self, enabled: bool) {
        if let Some(gutter) = self.gutter(gtk::TextWindowType::Left) {
            gutter.set_visible(enabled);
        }
        self.queue_draw();
    }
}
