use crate::shortcuts::{SHORTCUTS, TEXT_SHORTCUTS};
use gtk::prelude::*;
use leditor::i18n::I18n;
pub const RELEASE_DATE: &str = "10/27/2026";

fn window(parent: &gtk::ApplicationWindow, title: &str, width: i32, height: i32) -> gtk::Window {
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(width)
        .default_height(height)
        .build();
    window.add_css_class("popup-surface");
    window
}
fn close_button(window: &gtk::Window, title: &str) -> gtk::Button {
    let button = gtk::Button::with_label(title);
    let weak = window.downgrade();
    button.connect_clicked(move |_| {
        if let Some(window) = weak.upgrade() {
            window.close();
        }
    });
    button
}
pub fn about(parent: &gtk::ApplicationWindow, strings: &I18n) -> gtk::Window {
    let window = window(parent, &strings.text("about"), 430, 310);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    let icon = gtk::Image::from_resource(
        "/io/github/leditor/Leditor/icons/512x512/apps/io.github.leditor.Leditor.png",
    );
    icon.set_pixel_size(96);
    content.append(&icon);
    let name = gtk::Label::new(Some("Leditor"));
    name.add_css_class("app-name");
    content.append(&name);
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(18)
        .halign(gtk::Align::Center)
        .build();
    for (row, (label, value)) in [
        ("version", env!("CARGO_PKG_VERSION")),
        ("release-date", RELEASE_DATE),
    ]
    .iter()
    .enumerate()
    {
        let caption = gtk::Label::new(Some(&strings.text(label)));
        caption.set_xalign(1.0);
        let value = gtk::Label::new(Some(value));
        value.set_xalign(0.0);
        value.set_selectable(true);
        grid.attach(&caption, 0, row as i32, 1, 1);
        grid.attach(&value, 1, row as i32, 1, 1);
    }
    content.append(&grid);
    content.append(&gtk::LinkButton::with_label(
        "https://github.com/Bartzin55/Leditor",
        "GitHub — Bartzin55/Leditor",
    ));
    let ok = close_button(&window, &strings.text("ok"));
    content.append(&ok);
    window.set_default_widget(Some(&ok));
    window.set_child(Some(&content));
    window.present();
    ok.grab_focus();
    window
}
pub fn keyboard(parent: &gtk::ApplicationWindow, strings: &I18n) -> gtk::Window {
    let window = window(parent, &strings.text("keyboard-shortcuts"), 430, 520);
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);
    let grid = gtk::Grid::builder().hexpand(true).build();
    grid.add_css_class("shortcut-table");
    let mut rows = vec![(strings.text("keybind"), strings.text("function"))];
    rows.extend(
        TEXT_SHORTCUTS
            .iter()
            .map(|(keys, label)| (keys.to_string(), strings.text(label))),
    );
    rows.extend(
        SHORTCUTS
            .iter()
            .map(|shortcut| (shortcut.display.to_string(), strings.text(shortcut.label))),
    );
    for (row, (keys, function)) in rows.iter().enumerate() {
        for (column, text) in [keys, function].iter().enumerate() {
            let label = gtk::Label::new(Some(text));
            label.set_hexpand(true);
            label.set_wrap(true);
            label.add_css_class("shortcut-cell");
            if row == 0 {
                label.add_css_class("heading");
            }
            grid.attach(&label, column as i32, row as i32, 1, 1);
        }
    }
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&grid)
        .build();
    root.append(&scroll);
    root.append(&close_button(&window, &strings.text("close")));
    window.set_child(Some(&root));
    window.present();
    window
}
