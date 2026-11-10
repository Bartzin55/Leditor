//! Comrak parses Markdown; GTK renders its AST without HTML, scripts or a WebView.
use comrak::{
    nodes::{AstNode, ListType, NodeValue, TableAlignment},
    Arena, Options,
};
use gtk::{gio, glib, prelude::*};
use leditor::i18n::I18n;
use sourceview5::prelude::BufferExt;
use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

type Anchors = Rc<RefCell<HashMap<String, glib::WeakRef<gtk::Widget>>>>;
const MAX_DEPTH: usize = 64;

pub fn is_markdown(path: Option<&Path>) -> bool {
    path.and_then(Path::extension)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}
fn options() -> Options<'static> {
    let mut options = Options::default();
    let ext = &mut options.extension;
    ext.table = true;
    ext.strikethrough = true;
    ext.autolink = true;
    ext.tasklist = true;
    ext.footnotes = true;
    ext.inline_footnotes = true;
    ext.description_lists = true;
    ext.shortcodes = true;
    ext.superscript = true;
    ext.subscript = true;
    ext.highlight = true;
    ext.insert = true;
    ext.multiline_block_quotes = true;
    ext.alerts = true;
    ext.header_attributes = true;
    // Keep CommonMark's __bold__. Raw HTML and math are not interpreted by GTK.
    options
}
fn escape(text: &str) -> String {
    glib::markup_escape_text(text).to_string()
}
fn plain<'a>(node: &'a AstNode<'a>) -> String {
    let mut output = String::new();
    for node in node.descendants() {
        match &node.data.borrow().value {
            NodeValue::Text(text) => output.push_str(text),
            NodeValue::Code(code) => output.push_str(&code.literal),
            NodeValue::ShortCode(code) => output.push_str(&code.emoji),
            NodeValue::SoftBreak | NodeValue::LineBreak => output.push(' '),
            _ => (),
        }
    }
    output
}
fn safe_link(url: &str) -> bool {
    if url.chars().any(char::is_control) {
        return false;
    }
    if url.starts_with('#') {
        return true;
    }
    glib::uri_parse_scheme(url).is_some_and(|scheme| {
        ["https", "http", "mailto"]
            .iter()
            .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
    })
}
fn inline_children<'a>(node: &'a AstNode<'a>, depth: usize) -> String {
    if depth >= MAX_DEPTH {
        return escape(&plain(node));
    }
    node.children().map(|n| inline(n, depth + 1)).collect()
}
fn inline<'a>(node: &'a AstNode<'a>, depth: usize) -> String {
    let data = node.data.borrow();
    let children = || inline_children(node, depth);
    match &data.value {
        NodeValue::Text(text) => escape(text),
        NodeValue::Code(code) => format!("<tt>{}</tt>", escape(&code.literal)),
        NodeValue::SoftBreak => " ".into(),
        NodeValue::LineBreak => "\n".into(),
        NodeValue::Emph => format!("<i>{}</i>", children()),
        NodeValue::Strong => format!("<b>{}</b>", children()),
        NodeValue::Strikethrough => format!("<s>{}</s>", children()),
        NodeValue::Insert => format!("<u>{}</u>", children()),
        NodeValue::Superscript => format!("<sup>{}</sup>", children()),
        NodeValue::Subscript => format!("<sub>{}</sub>", children()),
        NodeValue::Highlight => format!(
            "<span background=\"#f4d750\" foreground=\"#202020\">{}</span>",
            children()
        ),
        NodeValue::ShortCode(code) => escape(&code.emoji),
        NodeValue::Link(link) if safe_link(&link.url) => {
            format!(
                "<a href=\"{}\" title=\"{}\">{}</a>",
                escape(&link.url),
                escape(&link.title),
                children()
            )
        }
        NodeValue::Image(_) => escape(&plain(node)),
        NodeValue::FootnoteReference(reference) => format!(
            "<a href=\"#fn-{}\"><sup>{}</sup></a>",
            escape(&reference.name),
            reference.ix
        ),
        // Render HTML literally, never inject it into Pango or execute it.
        NodeValue::HtmlInline(text) => escape(text),
        _ => children(),
    }
}

pub struct MarkdownPreview {
    pub content: gtk::Box,
    pub source_path: Option<PathBuf>,
    css: gtk::CssProvider,
    css_class: String,
    code_buffers: Vec<sourceview5::Buffer>,
}
impl MarkdownPreview {
    pub fn new(
        text: &str,
        source_path: Option<PathBuf>,
        scroll: &gtk::ScrolledWindow,
        strings: &I18n,
    ) -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
        content.add_css_class("markdown-preview");
        content.set_valign(gtk::Align::Start);
        content.set_hexpand(true);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        let anchors = Rc::new(RefCell::new(HashMap::new()));
        let mut renderer = Renderer {
            scroll: scroll.downgrade(),
            root: content.clone(),
            anchors,
            anchorizer: comrak::Anchorizer::new(),
            strings,
            source_path: source_path.as_deref(),
            code_buffers: Vec::new(),
            footnotes: HashMap::new(),
        };
        let arena = Arena::new();
        let options = options();
        let root = comrak::parse_document(&arena, text, &options);
        for node in root.descendants() {
            if let NodeValue::FootnoteReference(reference) = &node.data.borrow().value {
                renderer
                    .footnotes
                    .insert(reference.name.clone(), reference.ix);
            }
        }
        renderer.blocks(root, &content, 0);
        let css = gtk::CssProvider::new();
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let css_class = format!(
            "markdown-{}",
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        content.add_css_class(&css_class);
        gtk::style_context_add_provider_for_display(
            &content.display(),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
        let code_buffers = std::mem::take(&mut renderer.code_buffers);
        drop(renderer);
        Self {
            content,
            source_path,
            css,
            css_class,
            code_buffers,
        }
    }
    pub fn set_zoom(&self, zoom: f64) {
        self.css.load_from_data(&format!(
            ".{} {{ font-size: {}px; }} .{} textview {{ font-size: {}px; }}",
            self.css_class,
            16.0 * zoom / 100.0,
            self.css_class,
            14.0 * zoom / 100.0
        ));
    }
    pub fn set_theme(&self, light: bool) {
        let manager = sourceview5::StyleSchemeManager::default();
        let scheme = manager.scheme(if light { "Adwaita" } else { "Adwaita-dark" });
        for buffer in &self.code_buffers {
            buffer.set_style_scheme(scheme.as_ref());
        }
    }
}
impl Drop for MarkdownPreview {
    fn drop(&mut self) {
        gtk::style_context_remove_provider_for_display(&self.content.display(), &self.css);
    }
}
struct Renderer<'s> {
    scroll: glib::WeakRef<gtk::ScrolledWindow>,
    root: gtk::Box,
    anchors: Anchors,
    anchorizer: comrak::Anchorizer,
    strings: &'s I18n,
    source_path: Option<&'s Path>,
    code_buffers: Vec<sourceview5::Buffer>,
    footnotes: HashMap<String, u32>,
}
impl Renderer<'_> {
    fn label(&self, markup: &str) -> gtk::Label {
        let label = gtk::Label::new(None);
        label.set_markup(markup);
        label.set_selectable(true);
        label.set_wrap(true);
        label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        label.set_xalign(0.0);
        label.set_yalign(0.0);
        label.set_hexpand(true);
        label.set_width_chars(1);
        let anchors = self.anchors.clone();
        let root = self.root.downgrade();
        let scroll = self.scroll.clone();
        label.connect_activate_link(move |label, url| {
            if let Some(fragment) = url.strip_prefix('#') {
                let fragment = glib::uri_unescape_string(fragment, None::<&str>)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| fragment.into());
                let target = anchors
                    .borrow()
                    .get(&fragment)
                    .and_then(glib::WeakRef::upgrade);
                if let (Some(target), Some(root), Some(scroll)) =
                    (target, root.upgrade(), scroll.upgrade())
                {
                    if let Some(bounds) = target.compute_bounds(&root) {
                        scroll
                            .vadjustment()
                            .set_value(f64::from(bounds.y()) + f64::from(root.margin_top()));
                    }
                }
            } else if safe_link(url) {
                // Explicit clicks only; no automatic network requests.
                let parent = label.root().and_downcast::<gtk::Window>();
                gtk::UriLauncher::new(url).launch(parent.as_ref(), gio::Cancellable::NONE, |_| {});
            }
            glib::Propagation::Stop
        });
        label
    }
    fn anchor(&self, name: String, widget: &impl IsA<gtk::Widget>) {
        self.anchors
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| widget.as_ref().downgrade());
    }
    fn paragraph<'a>(&self, node: &'a AstNode<'a>, parent: &gtk::Box, prefix: &str, suffix: &str) {
        parent.append(&self.label(&format!("{prefix}{}{suffix}", inline_children(node, 0))));
        // Images are laid out below their containing paragraph. Text remains
        // selectable; unsupported/remote images have a visible fallback.
        for image in node.descendants() {
            if let NodeValue::Image(link) = &image.data.borrow().value {
                self.image(&link.url, &plain(image), parent);
            }
        }
    }
    fn image(&self, url: &str, alt: &str, parent: &gtk::Box) {
        let path = local_image_path(url, self.source_path);
        if let Some(path) = path {
            if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(&path, 900, 600, true) {
                let picture = gtk::Picture::for_pixbuf(&pixbuf);
                picture.set_can_shrink(true);
                picture.set_content_fit(gtk::ContentFit::Contain);
                picture.set_alternative_text(Some(alt));
                picture.set_halign(gtk::Align::Start);
                picture.set_tooltip_text(Some(alt));
                parent.append(&picture);
                return;
            }
        }
        let key = if glib::uri_parse_scheme(url).is_some() || url.starts_with("//") {
            "markdown-remote-image"
        } else {
            "markdown-image-unavailable"
        };
        let label = self.label(&format!(
            "<i>{}: {}</i>",
            escape(&self.strings.text(key)),
            escape(alt)
        ));
        label.set_tooltip_text(Some(url));
        parent.append(&label);
    }
    fn blocks<'a>(&mut self, node: &'a AstNode<'a>, parent: &gtk::Box, depth: usize) {
        if depth >= MAX_DEPTH {
            parent.append(&self.label(&escape(&plain(node))));
            return;
        }
        for child in node.children() {
            self.block(child, parent, depth + 1);
        }
    }
    fn block<'a>(&mut self, node: &'a AstNode<'a>, parent: &gtk::Box, depth: usize) {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Paragraph => self.paragraph(node, parent, "", ""),
            NodeValue::Heading(heading) => {
                let size = match heading.level {
                    1 => "xx-large",
                    2 => "x-large",
                    3 => "large",
                    _ => "medium",
                };
                let label = self.label(&format!(
                    "<span size=\"{size}\" weight=\"bold\">{}</span>",
                    inline_children(node, 0)
                ));
                label.set_margin_top(8);
                let id = data
                    .attrs
                    .as_ref()
                    .and_then(|a| a.id.clone())
                    .unwrap_or_else(|| self.anchorizer.anchorize(&plain(node)));
                self.anchor(id, &label);
                parent.append(&label);
            }
            NodeValue::ThematicBreak => {
                parent.append(&gtk::Separator::new(gtk::Orientation::Horizontal))
            }
            NodeValue::List(list) => {
                for (index, item) in node.children().enumerate() {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                    row.set_margin_start(8);
                    let marker = match &item.data.borrow().value {
                        NodeValue::TaskItem(task) => {
                            if task.symbol.is_some() {
                                "☑".into()
                            } else {
                                "☐".into()
                            }
                        }
                        _ if list.list_type == ListType::Ordered => {
                            format!("{}.", list.start + index)
                        }
                        _ => "•".into(),
                    };
                    let label = gtk::Label::new(Some(&marker));
                    label.set_valign(gtk::Align::Start);
                    label.set_width_chars(3);
                    label.set_xalign(1.0);
                    row.append(&label);
                    let body =
                        gtk::Box::new(gtk::Orientation::Vertical, if list.tight { 4 } else { 12 });
                    body.set_hexpand(true);
                    self.blocks(item, &body, depth);
                    row.append(&body);
                    parent.append(&row);
                }
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) | NodeValue::Alert(_) => {
                let quote = gtk::Box::new(gtk::Orientation::Vertical, 8);
                quote.add_css_class("markdown-quote");
                if let NodeValue::Alert(alert) = &data.value {
                    let title = alert.title.clone().unwrap_or_else(|| {
                        self.strings.text(&format!(
                            "markdown-{}",
                            format!("{:?}", alert.alert_type).to_lowercase()
                        ))
                    });
                    quote.append(&self.label(&format!("<b>{}</b>", escape(&title))));
                }
                self.blocks(node, &quote, depth);
                parent.append(&quote);
            }
            NodeValue::CodeBlock(code) => {
                let buffer = sourceview5::Buffer::new(None);
                buffer.set_enable_undo(false);
                buffer.set_text(code.literal.strip_suffix('\n').unwrap_or(&code.literal));
                let id = code.info.split_whitespace().next().unwrap_or("");
                let id = match id {
                    "js" => "javascript",
                    "ts" => "typescript",
                    "py" => "python",
                    "sh" | "bash" => "sh",
                    "c++" => "cpp",
                    "yml" => "yaml",
                    other => other,
                };
                let manager = sourceview5::LanguageManager::default();
                buffer.set_language(manager.language(id).as_ref());
                let view = sourceview5::View::with_buffer(&buffer);
                view.set_editable(false);
                view.set_cursor_visible(false);
                view.set_monospace(true);
                view.set_wrap_mode(gtk::WrapMode::WordChar);
                view.set_left_margin(12);
                view.set_right_margin(12);
                view.set_top_margin(10);
                view.set_bottom_margin(10);
                view.add_css_class("markdown-code");
                parent.append(&view);
                self.code_buffers.push(buffer);
            }
            NodeValue::Table(table) => {
                let grid = gtk::Grid::new();
                grid.add_css_class("markdown-table");
                grid.set_hexpand(true);
                for (row_index, row) in node.children().enumerate() {
                    let header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
                    for (column, cell) in row.children().enumerate() {
                        let markup = inline_children(cell, 0);
                        let label = self.label(&if header {
                            format!("<b>{markup}</b>")
                        } else {
                            markup
                        });
                        label.add_css_class("markdown-cell");
                        label.set_width_chars(8);
                        let align = match table.alignments.get(column) {
                            Some(TableAlignment::Center) => 0.5,
                            Some(TableAlignment::Right) => 1.0,
                            _ => 0.0,
                        };
                        label.set_xalign(align);
                        grid.attach(&label, column as i32, row_index as i32, 1, 1);
                    }
                }
                parent.append(&grid);
            }
            NodeValue::FootnoteDefinition(note) => {
                let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
                self.anchor(format!("fn-{}", note.name), &body);
                let number = self
                    .footnotes
                    .get(&note.name)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| note.name.clone());
                body.append(&self.label(&format!("<b>[{}]</b>", escape(&number))));
                self.blocks(node, &body, depth);
                parent.append(&body);
            }
            NodeValue::DescriptionTerm => self.paragraph(node, parent, "<b>", "</b>"),
            NodeValue::DescriptionDetails => {
                let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
                body.set_margin_start(24);
                self.blocks(node, &body, depth);
                parent.append(&body);
            }
            NodeValue::HtmlBlock(html) => parent.append(&self.label(&escape(&html.literal))),
            _ => self.blocks(node, parent, depth),
        }
    }
}
fn local_image_path(url: &str, source: Option<&Path>) -> Option<PathBuf> {
    // No remote fetch, UNC share, data URI, or SVG/HTML scripting surface.
    if url.starts_with(['/', '\\']) || glib::uri_parse_scheme(url).is_some() {
        return None;
    }
    let decoded = glib::uri_unescape_string(url, None::<&str>)?;
    if decoded.starts_with(['/', '\\']) {
        return None;
    }
    let base = source?.parent()?.canonicalize().ok()?;
    let path = base.join(decoded.as_str()).canonicalize().ok()?;
    if !path.starts_with(&base) || !path.is_file() {
        return None;
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    ["png", "jpg", "jpeg"]
        .contains(&extension.as_str())
        .then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn markdown_extensions_are_safe_and_escape_user_markup() {
        let arena = Arena::new();
        let options = options();
        let root = comrak::parse_document(&arena, "**bold** __bold__ *italic* ~~gone~~ ==mark== H~2~O x^2^ :smile: ++new++ [link](https://example.com?a=1&b=2) <script>alert(1)</script>", &options);
        let markup = inline_children(root.first_child().unwrap(), 0);
        assert!(markup.contains("<b>bold</b> <b>bold</b>"));
        assert!(markup.contains("<s>gone</s>"));
        assert!(markup.contains("<sub>2</sub>"));
        assert!(markup.contains("<sup>2</sup>"));
        assert!(markup.contains("😄"));
        assert!(markup.contains("&lt;script&gt;"));
        assert!(markup.contains("a=1&amp;b=2"));
        assert!(gtk::pango::parse_markup(
            &inline_children(
                comrak::parse_document(&arena, "**bold** H~2~O x^2^ ==mark==", &options)
                    .first_child()
                    .unwrap(),
                0
            ),
            '\0'
        )
        .is_ok());
        assert!(!safe_link("javascript:alert(1)"));
        assert!(!safe_link("file:///tmp/app.exe"));
        assert!(!safe_link("https://example.com\n"));
        assert!(safe_link("#heading"));
        assert!(safe_link("HTTPS://example.com"));
    }
    #[test]
    fn markdown_filename_and_local_image_boundaries() {
        assert!(is_markdown(Some(Path::new("README.MD"))));
        assert!(!is_markdown(Some(Path::new("README.md.txt"))));
        assert!(!is_markdown(None));
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("docs");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("my image.png"), []).unwrap();
        std::fs::write(temp.path().join("secret.png"), []).unwrap();
        let source = root.join("readme.md");
        assert_eq!(
            local_image_path("my%20image.png", Some(&source)),
            Some(root.join("my image.png"))
        );
        for url in [
            "https://example.com/a.png",
            "//server/a.png",
            "../secret.png",
            "data:image/png;base64,...",
            "image.svg",
            "%2Ftmp%2Fimage.png",
        ] {
            assert!(local_image_path(url, Some(&source)).is_none(), "{url}");
        }
    }
}
