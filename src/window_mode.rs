//! Platform window-manager requests. Wayland intentionally has no fallback that steals focus.
use gtk::prelude::*;
pub fn supported(window: &gtk::ApplicationWindow) -> bool {
    let Some(surface) = window.surface() else {
        return false;
    };
    #[cfg(target_os = "linux")]
    {
        surface.is::<gdk4_x11::X11Surface>()
    }
    #[cfg(target_os = "windows")]
    {
        surface.is::<gdk4_win32::Win32Surface>()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = surface;
        false
    }
}
pub fn set_above(
    window: &gtk::ApplicationWindow,
    enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let surface = window.surface().ok_or("Window has no surface")?;
    #[cfg(target_os = "linux")]
    {
        use x11rb::{
            connection::Connection,
            protocol::xproto::{ClientMessageEvent, ConnectionExt, EventMask},
        };
        let surface = surface
            .downcast::<gdk4_x11::X11Surface>()
            .map_err(|_| "Unsupported window system")?;
        let (connection, screen) = x11rb::connect(None)?;
        let state = connection
            .intern_atom(false, b"_NET_WM_STATE")?
            .reply()?
            .atom;
        let above = connection
            .intern_atom(false, b"_NET_WM_STATE_ABOVE")?
            .reply()?
            .atom;
        let event = ClientMessageEvent::new(
            32,
            surface.xid() as u32,
            state,
            [u32::from(enabled), above, 0, 1, 0],
        );
        connection
            .send_event(
                false,
                connection.setup().roots[screen].root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            )?
            .check()?;
        connection.flush()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        };
        let surface = surface
            .downcast::<gdk4_win32::Win32Surface>()
            .map_err(|_| "Unsupported window system")?;
        // GTK owns the native window; this call only changes its Z order.
        let result = unsafe {
            SetWindowPos(
                surface.handle().0 as _,
                if enabled {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        };
        if result == 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (surface, enabled);
        Err("Unsupported window system".into())
    }
}
