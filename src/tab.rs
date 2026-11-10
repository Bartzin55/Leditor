use crate::editor_view::EditorView;
use gtk::prelude::*;
use leditor::{document::Document, i18n::I18n};
use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::Rc,
};

/// IDs, rather than filenames or notebook positions, identify independent tabs.
pub struct Tab {
    pub preview: RefCell<Option<crate::markdown::MarkdownPreview>>,
    pub edit_position: Cell<(f64, f64)>,
    pub id: u64,
    pub unsupported: Cell<bool>,
    pub media: Cell<bool>,
    pub viewer: RefCell<Option<Rc<crate::viewer::Viewer>>>,
    pub syntax_detected: Cell<bool>,
    pub display_name: RefCell<Option<String>>,
    pub document: RefCell<Document>,
    pub view: EditorView,
    pub scroll: gtk::ScrolledWindow,
    pub editor_area: gtk::Box,
    pub preview_scroll: gtk::ScrolledWindow,
    pub content: gtk::Box,
    pub minimap: sourceview5::Map,
    pub header: gtk::Box,
    pub select: gtk::Button,
    pub icon: gtk::Image,
    pub title: gtk::Label,
    pub dirty: gtk::Label,
    pub close: gtk::Button,
}
impl Tab {
    pub fn new(id: u64, document: Document, strings: &I18n) -> Rc<Self> {
        let view = EditorView::new(&document.text);
        let scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .overlay_scrolling(false)
            .child(&view)
            .build();
        let minimap = crate::minimap::create(&view);
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let editor_area = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        editor_area.set_hexpand(true);
        editor_area.append(&scroll);
        editor_area.append(&minimap);
        let preview_scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .overlay_scrolling(false)
            .visible(false)
            .build();
        content.append(&editor_area);
        content.append(&preview_scroll);
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        header.add_css_class("document-tab");
        let select = gtk::Button::new();
        select.add_css_class("flat");
        let header_content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let icon = gtk::Image::new();
        icon.set_pixel_size(16);
        let title = gtk::Label::new(None);
        title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        title.set_max_width_chars(32);
        title.set_single_line_mode(true);
        let dirty = gtk::Label::new(Some("●"));
        dirty.add_css_class("dirty-dot");
        header_content.append(&icon);
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
            preview: RefCell::new(None),
            edit_position: Cell::new((0.0, 0.0)),
            id,
            unsupported: Cell::new(false),
            media: Cell::new(false),
            viewer: RefCell::new(None),
            syntax_detected: Cell::new(false),
            display_name: RefCell::new(None),
            document: RefCell::new(document),
            view,
            scroll,
            editor_area,
            preview_scroll,
            content,
            minimap,
            header,
            select,
            icon,
            title,
            dirty,
            close,
        });
        tab.refresh(strings);
        tab
    }
    pub fn is_text(&self) -> bool {
        !self.unsupported.get() && !self.media.get()
    }
    pub fn is_preview(&self) -> bool {
        self.preview.borrow().is_some()
    }
    pub fn source_buffer(&self) -> sourceview5::Buffer {
        self.view.buffer().downcast().expect("source buffer")
    }
    pub fn set_viewer(&self, viewer: Rc<crate::viewer::Viewer>) {
        self.media.set(true);
        self.minimap.set_visible(false);
        self.scroll
            .set_hscrollbar_policy(gtk::PolicyType::Automatic);
        self.scroll.set_child(Some(&viewer.content));
        viewer.attach_scroll(&self.scroll);
        self.viewer.replace(Some(viewer));
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
        let name = self.name(strings);
        if self.title.text().as_str() != name {
            self.title.set_text(&name);
            // Ellipsized labels normally have a near-zero minimum width. In a
            // scrolling tab strip each title needs its own reserved space.
            // Measure the real glyphs, including wide Unicode characters, and
            // cap unusually long titles at about 32 average font characters.
            let natural = self.title.create_pango_layout(Some(&name)).pixel_size().0;
            let metrics = self.title.pango_context().metrics(None, None);
            let limit = (32 * metrics.approximate_char_width()) / gtk::pango::SCALE;
            self.title.set_width_request(natural.min(limit.max(1)));
            self.icon.set_from_gicon(&crate::file_icon::for_path(
                self.document
                    .borrow()
                    .path
                    .as_deref()
                    .unwrap_or_else(|| Path::new(&name)),
            ));
        }
        // Opacity preserves the tab width while a saved document has no dot.
        self.dirty
            .set_opacity(if self.document.borrow().is_dirty() {
                1.0
            } else {
                0.0
            });
        let document = self.document.borrow();
        let tooltip = document.path.as_ref().map(|p| p.to_string_lossy());
        self.header
            .set_tooltip_text(Some(tooltip.as_deref().unwrap_or(&name)));
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
        if !self.is_text() {
            return;
        }
        let buffer = self.view.buffer();
        self.document.borrow_mut().text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
    }
    pub fn set_wrap(&self, wrap: bool) {
        if !self.is_text() {
            return;
        }
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
