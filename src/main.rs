#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app_icon;
mod editor_view;
#[cfg(target_os = "linux")]
mod linux_install;
mod minimap;
mod popups;
mod session;
mod shortcuts;
mod tab;
mod ui;
mod viewer;
use gtk::prelude::*;
fn main() -> gtk::glib::ExitCode {
    // Set before GTK initialization or any application threads are started.
    // The software renderer avoids GPU composition artifacts during Win32 resize.
    // Respect an explicit renderer override for troubleshooting/performance tests.
    #[cfg(target_os = "windows")]
    if std::env::var_os("GSK_RENDERER").is_none() {
        std::env::set_var("GSK_RENDERER", "cairo");
    }
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
