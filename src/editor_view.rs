//! TextView's native gutter and layout are reused so numbering follows logical
//! lines, including wrapping, vertical scrolling, empty lines and final newlines.
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct NumberedView;
    #[glib::object_subclass]
    impl ObjectSubclass for NumberedView {
        const NAME: &'static str = "LeditorNumberedView";
        type Type = super::NumberedView;
        type ParentType = gtk::TextView;
    }
    impl ObjectImpl for NumberedView {}
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
                    snapshot.append_layout(&layout, &color);
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
        view.update_gutter();
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
