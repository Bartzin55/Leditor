use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

fn widgets<T: IsA<gtk::Widget> + glib::types::StaticType>(widget: &gtk::Widget) -> Vec<T> {
    let mut result = Vec::new();
    if let Ok(found) = widget.clone().downcast::<T>() {
        result.push(found);
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        result.extend(widgets::<T>(&widget));
        child = widget.next_sibling();
    }
    result
}

#[test]
#[ignore = "Run separately under X11 with xdotool and a window manager"]
fn version_190_clipboard_project_network_and_shortcuts() {
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp.path());
    std::env::set_var("XDG_DATA_HOME", temp.path());
    let _ = std::fs::remove_dir_all(session::directory());
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("io.github.leditor.Features190Test")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(gio::Cancellable::NONE).unwrap();
    let e = create(&app);
    pump_until(|| e.window.is_active());
    let root = temp.path().join("project");
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::create_dir(root.join("assets")).unwrap();
    let image =
        gtk::gdk_pixbuf::Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, 4, 4).unwrap();
    image.fill(0x0088ffff);
    let png = image.save_to_bufferv("png", &[]).unwrap();
    std::fs::write(root.join("assets/image.png"), &png).unwrap();
    std::fs::write(root.join("assets/file.txt"), "target").unwrap();
    let source = root.join("docs/test.md");
    std::fs::write(&source, "initial\n").unwrap();
    e.load_folder(root.clone());
    run(&e, Intent::Operation(Operation::Open(source.clone())));
    let tab = e.current().unwrap();
    tab.view
        .clipboard()
        .set_texture(&gtk::gdk::Texture::for_pixbuf(&image));
    tab.view
        .buffer()
        .place_cursor(&tab.view.buffer().end_iter());
    tab.view.grab_focus();
    assert!(std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+v"])
        .status()
        .unwrap()
        .success());
    pump_until(|| tab.document.borrow().text.contains("pasted-image-"));
    pump_until(|| !e.busy.get());
    let first = tab.document.borrow().text.clone();
    assert!(first.contains("![Pasted Image](pasted-image-"));
    let files = || {
        std::fs::read_dir(root.join("docs"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|f| f.path().extension().is_some_and(|x| x == "png"))
            .map(|f| f.path())
            .collect::<Vec<_>>()
    };
    assert_eq!(files().len(), 1);
    assert_eq!(
        std::fs::read(&files()[0]).unwrap().get(..8),
        Some(b"\x89PNG\r\n\x1a\n".as_slice())
    );
    toggle(&e, "undo");
    assert_eq!(tab.document.borrow().text, "initial\n");
    toggle(&e, "redo");
    assert_eq!(tab.document.borrow().text, first);
    tab.view.emit_by_name::<()>("paste-clipboard", &[]);
    pump_until(|| !e.busy.get());
    assert_eq!(files().len(), 2);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/image.png", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let server_count = count.clone();
    let server_stop = stop.clone();
    let server = std::thread::spawn(move || {
        while !server_stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    socket
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = [0; 4096];
                    let _ = socket.read(&mut request);
                    server_count.fetch_add(1, Ordering::Relaxed);
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        png.len()
                    );
                    let _ = socket.write_all(header.as_bytes());
                    let _ = socket.write_all(&png);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(error) => panic!("{error}"),
            }
        }
    });
    tab.view.buffer().set_text(&format!(
        "![local](/assets/image.png)\n\n[document](../assets/file.txt)\n\n![remote]({url})"
    ));
    toggle(&e, "disable-internet");
    toggle(&e, "only-view");
    drain();
    let preview = tab.preview_scroll.child().unwrap();
    assert!(labels(&preview)
        .iter()
        .any(|s| s.contains("Internet images disabled")));
    assert_eq!(count.load(Ordering::Relaxed), 0);
    assert_eq!(widgets::<gtk::Picture>(&preview).len(), 1);
    capture(&e, "190-preview-disabled.png");
    let link = widgets::<gtk::Label>(&preview)
        .into_iter()
        .find(|l| l.text() == "document")
        .unwrap();
    assert!(link.label().contains(
        &gio::File::for_path(root.join("assets/file.txt"))
            .uri()
            .to_string()
    ));
    let bounds = link.compute_bounds(&e.window).unwrap();
    let window_id = std::process::Command::new("xdotool")
        .arg("getactivewindow")
        .output()
        .unwrap();
    let window_id = String::from_utf8(window_id.stdout).unwrap();
    assert!(std::process::Command::new("xdotool")
        .args([
            "mousemove",
            "--window",
            window_id.trim(),
            &(bounds.x() as i32 + 10).to_string(),
            &(bounds.y() as i32 + 8).to_string()
        ])
        .status()
        .unwrap()
        .success());
    drain();
    assert!(link.current_uri().is_some_and(|u| u.starts_with("file:")));
    assert!(std::process::Command::new("xdotool")
        .args(["click", "3"])
        .status()
        .unwrap()
        .success());
    pump_until(|| button(link.upcast_ref(), "Open with…").is_some());
    capture(&e, "190-open-with.png");
    for popover in widgets::<gtk::Popover>(link.upcast_ref()) {
        popover.popdown();
    }
    drain();
    let before = tab.document.borrow().text.clone();
    tab.view.paste();
    drain();
    assert_eq!(before, tab.document.borrow().text);
    toggle(&e, "disable-internet");
    pump_until(|| widgets::<gtk::Picture>(&tab.preview_scroll.child().unwrap()).len() == 2);
    assert_eq!(count.load(Ordering::Relaxed), 1);
    capture(&e, "190-preview-online.png");
    toggle(&e, "disable-internet");
    drain();
    assert!(labels(&tab.preview_scroll.child().unwrap())
        .iter()
        .any(|s| s.contains("Internet images disabled")));
    assert_eq!(count.load(Ordering::Relaxed), 1);
    stop.store(true, Ordering::Relaxed);
    server.join().unwrap();

    let old_minimap = e.minimap.get();
    assert!(std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+m"])
        .status()
        .unwrap()
        .success());
    pump_until(|| e.minimap.get() != old_minimap);
    assert_ne!(old_minimap, e.minimap.get());
    assert!(!tab.minimap.is_visible());
    toggle(&e, "only-view");
    drain();
    e.set_zoom(100.0);
    drain();
    let gesture = controller::<gtk::GestureZoom>(&tab.content);
    gesture.emit_by_name::<()>("begin", &[&None::<gtk::gdk::EventSequence>]);
    gesture.emit_by_name::<()>("scale-changed", &[&1.125f64]);
    assert!((e.zoom.get() - 112.5).abs() < 0.01);
    gesture.emit_by_name::<()>("end", &[&None::<gtk::gdk::EventSequence>]);
    drain();
    assert_eq!(
        app.accels_for_action("win.minimap")
            .first()
            .map(|s| s.as_str()),
        Some("<Control>m")
    );
    assert_eq!(
        app.accels_for_action("win.rename-active")
            .first()
            .map(|s| s.as_str()),
        Some("F12")
    );
    let target = root.join("docs/renamed.md");
    assert!(std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", "F12"])
        .status()
        .unwrap()
        .success());
    answer_name("renamed.md");
    pump_until(|| !e.busy.get());
    assert_eq!(tab.document.borrow().path.as_ref(), Some(&target));
    assert!(!source.exists());

    run(&e, Intent::New);
    let untitled = e.current().unwrap();
    queue(&e, true, None);
    toggle(&e, "rename-active");
    pump_until(|| !e.busy.get());
    assert!(untitled.document.borrow().path.is_none());
    queue(&e, true, Some(&root.join("new.md")));
    toggle(&e, "rename-active");
    pump_until(|| !e.busy.get());
    assert!(root.join("new.md").is_file());
    run(&e, Intent::New);
    let unsaved = e.current().unwrap();
    unsaved.source_buffer().set_language(
        sourceview5::LanguageManager::default()
            .language("markdown")
            .as_ref(),
    );
    queue(&e, true, None);
    unsaved.view.paste();
    pump_until(|| !e.busy.get());
    assert!(unsaved.document.borrow().path.is_none());
    assert!(unsaved.document.borrow().text.is_empty());
    queue(&e, true, Some(&root.join("pasted.md")));
    unsaved.view.paste();
    pump_until(|| !e.busy.get());
    assert!(unsaved.document.borrow().text.contains("pasted-image-"));
    e.save_settings();
    e.window.destroy();
    drain();
    let restored = create(&app);
    assert!(restored.internet_disabled.get());
    restored.window.destroy();
}
