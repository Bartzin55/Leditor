pub fn register() {
    static REGISTER: std::sync::Once = std::sync::Once::new();
    REGISTER.call_once(|| {
        gtk::gio::resources_register_include!("leditor.gresource").expect("embedded icon resource");
    });
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_resource_path("/io/github/leditor/Leditor/icons");
    }
    gtk::Window::set_default_icon_name("io.github.leditor.Leditor");
}
