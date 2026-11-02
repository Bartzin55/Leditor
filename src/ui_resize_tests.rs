//! Run alone with a display and a window manager: GTK must stay on one thread.
use super::*;
use std::time::{Duration, Instant};

fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    let context = glib::MainContext::default();
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "Window did not reach the requested state"
        );
        for _ in 0..100 {
            if !context.pending() {
                break;
            }
            context.iteration(false);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "Run separately with a graphical display and window manager"]
fn resize_maximize_and_focus_preserve_editor() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp.path());
    std::env::set_var("XDG_DATA_HOME", temp.path());
    std::env::set_var("GSETTINGS_BACKEND", "memory");
    let _ = std::fs::remove_dir_all(crate::session::directory());
    crate::configure_platform();
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.ResizeTest")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(gio::Cancellable::NONE).unwrap();
    let editor = create(&app);
    let tab = editor.current().unwrap();
    let buffer = tab.view.buffer();
    buffer.set_text("fn main() {\n    println!(\"Olá, mundo!\");\n}\n");
    buffer.place_cursor(&buffer.iter_at_offset(16));
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
    let caret = buffer.cursor_position();
    wait_until(|| editor.window.is_mapped() && tab.view.width() > 0);

    #[cfg(target_os = "windows")]
    if std::env::var("GTK_CSD").as_deref() == Ok("0") {
        assert!(editor.window.titlebar().is_none());
        assert!(!editor.window.has_css_class("csd"));
        assert!(editor.window.is_decorated());
        assert!(editor.window.is_resizable());
    }

    for theme in ["light", "dark"] {
        editor.theme.replace(theme.into());
        editor.apply_theme();
        editor.open_search();
        for (width, height) in [(1100, 720), (900, 600), (1250, 800), (980, 660)] {
            editor.window.set_default_size(width, height);
            wait_until(|| editor.window.width() == width && editor.window.height() == height);
            // Let the frame clock allocate the editor after the native surface.
            let after_frame = Instant::now() + Duration::from_millis(100);
            wait_until(|| Instant::now() >= after_frame);
            assert!(tab.view.width() > 0 && tab.view.height() > 0);
            assert!(tab.view.width() <= editor.window.width());
            assert!(tab.view.height() <= editor.window.height());
            assert!(editor.search.bar.is_visible());
            assert_eq!(
                buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
                text
            );
            assert_eq!(buffer.cursor_position(), caret);
        }
        editor.close_search();
    }

    editor.window.maximize();
    wait_until(|| editor.window.is_maximized());
    editor.window.unmaximize();
    wait_until(|| !editor.window.is_maximized());
    let line_numbers = editor.line_numbers.get();
    let explorer = editor.explorer.panel.is_visible();
    editor.toggle_focus();
    wait_until(|| editor.window.is_fullscreen());
    assert!(!editor.footer.is_visible());
    editor.toggle_focus();
    wait_until(|| !editor.window.is_fullscreen());
    assert!(editor.footer.is_visible());
    assert!(editor.menubar.is_visible() && editor.tab_bar.is_visible());
    assert_eq!(editor.line_numbers.get(), line_numbers);
    assert_eq!(editor.explorer.panel.is_visible(), explorer);
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        text
    );
    assert_eq!(buffer.cursor_position(), caret);
    assert_eq!(editor.zoom.get(), 100.0);
    editor.window.destroy();
    let _ = std::fs::remove_dir_all(crate::session::directory());
}
