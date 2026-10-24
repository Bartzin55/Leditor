#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod editor_view;
mod popups;
mod session;
mod shortcuts;
mod tab;
mod ui;
use gtk::prelude::*;
fn main() -> gtk::glib::ExitCode {
    gtk::glib::set_application_name("Leditor");
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.Leditor")
        .build();
    app.connect_activate(ui::build);
    app.run()
}
