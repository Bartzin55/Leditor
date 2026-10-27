use crate::editor_view::NumberedView;
use gtk::prelude::*;
use leditor::{document::Document, i18n::I18n};
use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::Rc,
};

/// IDs, rather than filenames or notebook positions, identify independent tabs.
pub struct Tab {
    pub id: u64,
    pub unsupported: Cell<bool>,
    pub display_name: RefCell<Option<String>>,
    pub document: RefCell<Document>,
    pub view: NumberedView,
    pub scroll: gtk::ScrolledWindow,
    pub content: gtk::Box,
    pub minimap: gtk::DrawingArea,
    pub header: gtk::Box,
    pub select: gtk::Button,
    pub title: gtk::Label,
    pub dirty: gtk::Label,
    pub close: gtk::Button,
}
impl Tab {
    pub fn new(id: u64, document: Document, strings: &I18n) -> Rc<Self> {
        let view = NumberedView::new(&document.text);
        let scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .overlay_scrolling(false)
            .child(&view)
            .build();
        let minimap = crate::minimap::create(&view, &scroll);
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        content.append(&scroll);
        content.append(&minimap);
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        header.add_css_class("document-tab");
        let select = gtk::Button::new();
        select.add_css_class("flat");
        let header_content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let title = gtk::Label::new(None);
        title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        title.set_max_width_chars(26);
        let dirty = gtk::Label::new(Some("●"));
        dirty.add_css_class("dirty-dot");
        header_content.append(&title);
        header_content.append(&dirty);
        select.set_child(Some(&header_content));
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.add_css_class("tab-close");
        close.add_css_class("flat");
        close.set_tooltip_text(Some(&strings.text("close-file")));
        header.append(&select);
        header.append(&close);
        let tab = Rc::new(Self {
            id,
            unsupported: Cell::new(false),
            display_name: RefCell::new(None),
            document: RefCell::new(document),
            view,
            scroll,
            content,
            minimap,
            header,
            select,
            title,
            dirty,
            close,
        });
        tab.refresh(strings);
        tab
    }
    pub fn name(&self, strings: &I18n) -> String {
        self.document
            .borrow()
            .path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                self.display_name
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| strings.text("untitled"))
            })
    }
    pub fn refresh(&self, strings: &I18n) {
        self.title.set_text(&self.name(strings));
        // Opacity preserves the tab width while a saved document has no dot.
        self.dirty
            .set_opacity(if self.document.borrow().is_dirty() {
                1.0
            } else {
                0.0
            });
        self.header.set_tooltip_text(
            self.document
                .borrow()
                .path
                .as_ref()
                .map(|p| p.to_string_lossy())
                .as_deref(),
        );
    }
    pub fn mark_unsupported(&self, strings: &I18n) {
        self.unsupported.set(true);
        self.minimap.set_visible(false);
        self.title.add_css_class("unsupported-title");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_valign(gtk::Align::Center);
        content.set_halign(gtk::Align::Center);
        let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
        icon.set_pixel_size(72);
        icon.add_css_class("unsupported-icon");
        content.append(&icon);
        content.append(&gtk::Label::new(Some(&strings.text("unsupported-file"))));
        self.scroll.set_child(Some(&content));
    }
    pub fn synchronize(&self) {
        let buffer = self.view.buffer();
        self.document.borrow_mut().text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
    }
    pub fn set_wrap(&self, wrap: bool) {
        self.view.set_wrap_mode(if wrap {
            gtk::WrapMode::WordChar
        } else {
            gtk::WrapMode::None
        });
        self.scroll.set_hscrollbar_policy(if wrap {
            gtk::PolicyType::Never
        } else {
            gtk::PolicyType::Automatic
        });
        if wrap {
            self.scroll.hadjustment().set_value(0.0);
        }
        self.view.queue_draw();
    }
}
