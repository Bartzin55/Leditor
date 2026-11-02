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
    configure_platform();
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

fn configure_platform() {
    // Set before GTK initialization. User-tested Windows compatibility profile;
    // reduces resize artifacts but does not fully fix live resize (RESIZE.md).
    // Explicit environment overrides remain available for diagnosis.
    #[cfg(target_os = "windows")]
    for (name, value) in [
        ("GTK_CSD", "0"),
        ("GSK_RENDERER", "cairo"),
        ("GDK_DEBUG", "dcomp"),
    ] {
        if std::env::var_os(name).is_none() {
            std::env::set_var(name, value);
        }
    }
}

mod text_ops;
