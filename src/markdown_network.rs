//! Asynchronous, bounded HTTP image loading on GTK's main context.
use glib::translate::IntoGlib;
use gtk::{glib, prelude::*};
use soup::prelude::*;

const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

pub fn is_remote(url: &str) -> bool {
    !url.chars().any(char::is_control)
        && glib::uri_parse_scheme(url)
            .is_some_and(|s| s.eq_ignore_ascii_case("http") || s.eq_ignore_ascii_case("https"))
}

pub fn session() -> soup::Session {
    soup::Session::builder()
        .timeout(15)
        .max_conns(4)
        .max_conns_per_host(2)
        .user_agent(concat!("Leditor/", env!("CARGO_PKG_VERSION")))
        .build()
}

pub async fn download(session: &soup::Session, url: &str) -> Result<Vec<u8>, String> {
    if !is_remote(url) {
        return Err("Only HTTP(S) images are supported".into());
    }
    let message = soup::Message::new("GET", url).map_err(|e| e.to_string())?;
    let stream = session
        .send_future(&message, glib::Priority::DEFAULT)
        .await
        .map_err(|e| e.to_string())?;
    let result = async {
        if !(200..300).contains(&(message.status().into_glib())) {
            return Err(format!("HTTP {}", message.status().into_glib()));
        }
        if message
            .response_headers()
            .is_some_and(|h| h.content_length() > MAX_IMAGE_BYTES as i64)
        {
            return Err("Image exceeds 16 MiB".into());
        }
        let mut bytes = Vec::new();
        loop {
            let chunk = stream
                .read_bytes_future(64 * 1024, glib::Priority::DEFAULT)
                .await
                .map_err(|e| e.to_string())?;
            if chunk.is_empty() {
                break;
            }
            if bytes.len() + chunk.len() > MAX_IMAGE_BYTES {
                return Err("Image exceeds 16 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
    .await;
    let _ = stream.close_future(glib::Priority::DEFAULT).await;
    result
}

pub fn decode(bytes: &[u8]) -> Result<gtk::gdk_pixbuf::Pixbuf, String> {
    // Match the existing PNG/JPEG viewer; never send HTML/SVG to a renderer.
    let kind = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "jpeg"
    } else {
        return Err("Unsupported image format (PNG/JPEG required)".into());
    };
    let loader = gtk::gdk_pixbuf::PixbufLoader::with_type(kind).map_err(|e| e.to_string())?;
    loader.connect_size_prepared(|loader, width, height| {
        if width > 0 && height > 0 {
            let scale = (900.0 / f64::from(width))
                .min(600.0 / f64::from(height))
                .min(1.0);
            loader.set_size(
                (f64::from(width) * scale).max(1.0) as i32,
                (f64::from(height) * scale).max(1.0) as i32,
            );
        }
    });
    let written = loader.write(bytes);
    let closed = loader.close();
    written.and(closed).map_err(|e| e.to_string())?;
    loader.pixbuf().ok_or_else(|| "Invalid image".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    #[test]
    fn http_images_redirect_errors_and_limits() {
        let image =
            gtk::gdk_pixbuf::Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, 2, 2).unwrap();
        image.fill(0x336699ff);
        let png = image.save_to_bufferv("png", &[]).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for _ in 0..5 {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut request = [0; 4096];
                let n = socket.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..n]);
                let path = request.split_whitespace().nth(1).unwrap();
                let (header, body) = match path {
                    "/image" => (format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", png.len()), png.as_slice()),
                    "/redirect" => ("HTTP/1.1 302 Found\r\nLocation: /image\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), &[][..]),
                    "/large" => (format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", MAX_IMAGE_BYTES + 1), &[][..]),
                    _ => ("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), &[][..]),
                };
                socket.write_all(header.as_bytes()).unwrap();
                socket.write_all(body).unwrap();
            }
        });
        glib::MainContext::default().block_on(async {
            let session = session();
            session.set_proxy_resolver(Some(&gtk::gio::SimpleProxyResolver::new(
                None,
                [] as [&str; 0],
            )));
            for path in ["/image", "/redirect"] {
                let bytes = download(&session, &format!("{url}{path}")).await.unwrap();
                assert_eq!(decode(&bytes).unwrap().width(), 2);
            }
            for path in ["/missing", "/large"] {
                assert!(download(&session, &format!("{url}{path}")).await.is_err());
            }
            assert!(download(&session, "file:///tmp/x.png").await.is_err());
            assert!(decode(b"<svg/>").is_err());
        });
        server.join().unwrap();
    }
    #[test]
    #[ignore = "Requires internet access and a working platform TLS backend"]
    fn https_image_smoke() {
        assert!(gtk::gio::TlsBackend::default().supports_tls());
        glib::MainContext::default().block_on(async {
            let url = "https://raw.githubusercontent.com/Bartzin55/Leditor/afeff3d9e9f9fe8d62d5bbef9854afd092a5ca58/assets/leditor.png";
            let bytes = download(&session(), url).await.unwrap();
            let pixbuf = decode(&bytes).unwrap();
            assert!(pixbuf.width() > 0 && pixbuf.height() > 0);
        });
    }
}
