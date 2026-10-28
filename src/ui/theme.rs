use super::*;

fn gtk_system_dark(settings: &gtk::Settings) -> bool {
    if settings
        .find_property("gtk-interface-color-scheme")
        .is_some()
    {
        let value = settings.property_value("gtk-interface-color-scheme");
        if let Some((_, value)) = glib::EnumValue::from_value(&value) {
            if value.nick() == "dark" || value.nick() == "light" {
                return value.nick() == "dark";
            }
        }
    }
    settings.is_gtk_application_prefer_dark_theme()
        || settings
            .gtk_theme_name()
            .is_some_and(|n| n.to_ascii_lowercase().contains("dark"))
}
impl Editor {
    fn set_system_dark(&self, dark: bool) {
        if self.system_dark.replace(dark) != dark && self.theme.borrow().as_str() == "system" {
            self.apply_theme();
        }
    }
    pub(super) fn setup_theme(self: &Rc<Self>) {
        if let Some(settings) = gtk::Settings::default() {
            self.system_dark.set(gtk_system_dark(&settings));
            for name in [
                "gtk-interface-color-scheme",
                "gtk-theme-name",
                "gtk-application-prefer-dark-theme",
            ] {
                if settings.find_property(name).is_none() {
                    continue;
                }
                let weak = Rc::downgrade(self);
                settings.connect_notify_local(Some(name), move |settings, _| {
                    if let Some(e) = weak.upgrade().filter(|e| !e.applying_theme.get()) {
                        let dark = if name == "gtk-theme-name"
                            && settings
                                .find_property("gtk-interface-color-scheme")
                                .is_none()
                        {
                            settings
                                .gtk_theme_name()
                                .is_some_and(|n| n.to_ascii_lowercase().contains("dark"))
                        } else {
                            gtk_system_dark(settings)
                        };
                        e.set_system_dark(dark);
                    }
                });
            }
        }
        #[cfg(target_os = "linux")]
        {
            let weak = Rc::downgrade(self);
            glib::spawn_future_local(async move {
                let Ok(proxy) = gio::DBusProxy::for_bus_future(
                    gio::BusType::Session,
                    gio::DBusProxyFlags::DO_NOT_LOAD_PROPERTIES,
                    None,
                    "org.freedesktop.portal.Desktop",
                    "/org/freedesktop/portal/desktop",
                    "org.freedesktop.portal.Settings",
                )
                .await
                else {
                    return;
                };
                let changed = weak.clone();
                proxy.connect_local("g-signal", false, move |values| {
                    let name = values[2].get::<String>().ok()?;
                    if name != "SettingChanged" {
                        return None;
                    }
                    let parameters = values[3].get::<glib::Variant>().ok()?;
                    if let Some((namespace, key, value)) =
                        parameters.get::<(String, String, glib::Variant)>()
                    {
                        if namespace == "org.freedesktop.appearance" && key == "color-scheme" {
                            if let (Some(e), Some(value)) = (changed.upgrade(), value.get::<u32>())
                            {
                                e.set_system_dark(value == 1);
                            }
                        }
                    }
                    None
                });
                let result = proxy
                    .call_future(
                        "Read",
                        Some(&("org.freedesktop.appearance", "color-scheme").to_variant()),
                        gio::DBusCallFlags::NONE,
                        2000,
                    )
                    .await;
                if let Some(e) = weak.upgrade() {
                    if let Ok(result) = result {
                        if let Some((value,)) = result.get::<(glib::Variant,)>() {
                            let value = value.as_variant().unwrap_or(value);
                            if let Some(value) = value.get::<u32>() {
                                e.set_system_dark(value == 1);
                            }
                        }
                    }
                    e.portal.replace(Some(proxy));
                }
            });
        }
        #[cfg(target_os = "windows")]
        {
            let weak = Rc::downgrade(self);
            glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                let Some(e) = weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                if let Some(dark) = windows_dark() {
                    e.set_system_dark(dark);
                }
                glib::ControlFlow::Continue
            });
            if let Some(dark) = windows_dark() {
                self.system_dark.set(dark);
            }
        }
    }
    pub(super) fn apply_theme(&self) {
        self.applying_theme.set(true);
        let theme = self.theme.borrow().clone();
        let light = theme == "light" || (theme == "system" && !self.system_dark.get());
        self.actions["theme"].set_state(&theme.to_variant());
        // Never replace the OS preference with the user's explicit application choice.
        self.theme_css.load_from_data(if light {
            include_str!("../light.css")
        } else {
            ""
        });
        if let Some(property) = self.theme_css.find_property("prefers-color-scheme") {
            if let Some(value) = glib::EnumClass::with_type(property.value_type())
                .and_then(|class| class.to_value_by_nick(if light { "light" } else { "dark" }))
            {
                self.theme_css
                    .set_property_from_value(property.name(), &value);
            }
        }
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(!light);
        }
        gtk::style_context_add_provider_for_display(
            &gtk::prelude::WidgetExt::display(&self.window),
            &self.theme_css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2,
        );
        self.applying_theme.set(false);
    }
}
#[cfg(target_os = "windows")]
fn windows_dark() -> Option<bool> {
    use windows_sys::Win32::System::Registry::*;
    let key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
    let mut value = 1u32;
    let mut size = 4u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut value as *mut u32).cast(),
            &mut size,
        )
    };
    (status == 0).then_some(value == 0)
}
