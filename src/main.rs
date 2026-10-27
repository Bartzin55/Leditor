#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod editor_view;
#[cfg(target_os = "linux")]
mod linux_install;
mod minimap;
mod popups;
mod session;
mod shortcuts;
mod tab;
mod ui;
mod window_mode;
use gtk::prelude::*;
fn main() -> gtk::glib::ExitCode {
    #[cfg(target_os = "linux")]
    if let Err(error) = linux_install::integrate() {
        eprintln!("Leditor: desktop integration failed: {error}");
    }
    gtk::glib::set_application_name("Leditor");
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.Leditor")
        .build();
    app.connect_activate(ui::build);
    app.run()
}

mod text_ops;
