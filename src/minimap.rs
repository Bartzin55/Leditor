//! The native map shares the source view's buffer, syntax tags and scrolling.
use crate::editor_view::EditorView;
use gtk::prelude::*;
use sourceview5::prelude::MapExt;
pub fn create(view: &EditorView) -> sourceview5::Map {
    let map = sourceview5::Map::new();
    map.set_view(view);
    map.set_width_request(120);
    map.set_vexpand(true);
    map.set_hexpand(false);
    map.add_css_class("minimap");
    // Keep the native map font; the editor's zoom CSS must not enlarge it.
    map
}
