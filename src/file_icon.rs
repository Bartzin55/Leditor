//! The same platform MIME icon in the explorer and document tabs.
use gtk::gio;
use std::path::Path;

pub fn for_path(path: &Path) -> gio::Icon {
    let (content_type, _) = gio::content_type_guess(Some(path), None);
    gio::content_type_get_symbolic_icon(&content_type)
}
