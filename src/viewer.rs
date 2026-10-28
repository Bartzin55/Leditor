//! Read-only media; a single worker owns PDFium, documents and rasterization.
//! GTK stays on the UI thread. Visible pages use a bounded raster cache.
use gtk::{gio, glib, prelude::*};
use pdfium_render::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{mpsc, OnceLock},
};
type Dimensions = Vec<(f64, f64)>;
type OpenReply = Result<(u64, Dimensions), String>;
struct Raster {
    page: usize,
    width: i32,
    height: i32,
    pixels: Vec<u8>,
    requested: i32,
}
type RenderReply = Result<Raster, (usize, i32, String)>;
enum Command {
    Open(PathBuf, mpsc::Sender<OpenReply>),
    Render(u64, usize, i32, mpsc::Sender<RenderReply>),
    Close(u64),
}
fn bind_pdfium() -> Result<Pdfium, String> {
    let explicit = std::env::var_os("LEDITOR_PDFIUM_LIBRARY").map(PathBuf::from);
    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Pdfium::pdfium_platform_library_name_at_path));
    let bindings = if let Some(path) = explicit {
        Pdfium::bind_to_library(path)
    } else {
        beside_exe
            .map(Pdfium::bind_to_library)
            .unwrap_or_else(Pdfium::bind_to_system_library)
            .or_else(|_| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                    &glib::user_data_dir().join("leditor/lib"),
                ))
            })
            .or_else(|_| Pdfium::bind_to_system_library())
    };
    bindings.map(Pdfium::new).map_err(|e| format!("PDFium: {e}. See BUILD.md; LEDITOR_PDFIUM_LIBRARY can specify the runtime library's full path."))
}
fn worker() -> &'static mpsc::Sender<Command> {
    static WORKER: OnceLock<mpsc::Sender<Command>> = OnceLock::new();
    WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let pdfium = bind_pdfium();
            let mut documents = HashMap::new();
            let mut next = 0;
            for command in rx {
                match command {
                    Command::Open(path, reply) => {
                        let result = (|| {
                            let engine = pdfium.as_ref().map_err(Clone::clone)?;
                            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
                            let document = engine
                                .load_pdf_from_byte_vec(bytes, None)
                                .map_err(|e| e.to_string())?;
                            let sizes: Dimensions = document
                                .pages()
                                .iter()
                                .map(|p| (f64::from(p.width().value), f64::from(p.height().value)))
                                .collect();
                            if sizes.is_empty()
                                || sizes.iter().any(|(w, h)| {
                                    !w.is_finite()
                                        || !h.is_finite()
                                        || *w <= 0.0
                                        || *h <= 0.0
                                        || *w > 100_000.0
                                        || *h > 100_000.0
                                })
                            {
                                return Err("Invalid or empty PDF page geometry".into());
                            }
                            next += 1;
                            documents.insert(next, document);
                            Ok((next, sizes))
                        })();
                        if let Err(mpsc::SendError(Ok((id, _)))) = reply.send(result) {
                            documents.remove(&id);
                        }
                    }
                    Command::Render(id, page, requested, reply) => {
                        let result = (|| {
                            let document = documents.get(&id).ok_or("PDF closed")?;
                            let page_object = document
                                .pages()
                                .get(page as i32)
                                .map_err(|e| e.to_string())?;
                            let config = PdfRenderConfig::new()
                                .set_target_width(requested)
                                .set_maximum_width(4096)
                                .set_maximum_height(4096);
                            let bitmap = page_object
                                .render_with_config(&config)
                                .map_err(|e| e.to_string())?;
                            let mut pixels = bitmap.as_rgba_bytes();
                            for p in pixels.as_chunks_mut::<4>().0 {
                                let a = u32::from(p[3]);
                                let color = (a << 24)
                                    | ((u32::from(p[0]) * a / 255) << 16)
                                    | ((u32::from(p[1]) * a / 255) << 8)
                                    | (u32::from(p[2]) * a / 255);
                                p.copy_from_slice(&color.to_ne_bytes());
                            }
                            Ok(Raster {
                                page,
                                requested,
                                width: bitmap.width(),
                                height: bitmap.height(),
                                pixels,
                            })
                        })();
                        let _ =
                            reply.send(result.map_err(|error: String| (page, requested, error)));
                    }
                    Command::Close(id) => {
                        documents.remove(&id);
                    }
                }
            }
        });
        tx
    })
}
pub fn supported(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| ["pdf", "png", "jpg", "jpeg"].contains(&s.to_ascii_lowercase().as_str()))
}
pub struct Viewer {
    pub content: gtk::Box,
    pages: Vec<gtk::DrawingArea>,
    dimensions: Dimensions,
    zoom: Cell<f64>,
    pdf: Option<u64>,
    cache: RefCell<VecDeque<(usize, i32, gtk::cairo::ImageSurface)>>,
    pending: RefCell<HashSet<(usize, i32)>>,
    failures: RefCell<HashMap<(usize, i32), String>>,
    response: mpsc::Sender<RenderReply>,
}
impl Drop for Viewer {
    fn drop(&mut self) {
        if let Some(id) = self.pdf {
            let _ = worker().send(Command::Close(id));
        }
    }
}
impl Viewer {
    pub async fn load(path: PathBuf, zoom: f64) -> Result<Rc<Self>, String> {
        let is_pdf = path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("pdf"));
        let (pdf, dimensions, image) = if is_pdf {
            let (tx, rx) = mpsc::channel();
            worker()
                .send(Command::Open(path, tx))
                .map_err(|e| e.to_string())?;
            let (id, sizes) = gio::spawn_blocking(move || rx.recv())
                .await
                .map_err(|_| "PDF worker failed")?
                .map_err(|e| e.to_string())??;
            (Some(id), sizes, None)
        } else {
            let bytes = gio::spawn_blocking(move || std::fs::read(path))
                .await
                .map_err(|_| "Image worker failed")?
                .map_err(|e| e.to_string())?;
            let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_owned(bytes));
            let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE)
                .map_err(|e| e.to_string())?;
            (
                None,
                vec![(f64::from(pixbuf.width()), f64::from(pixbuf.height()))],
                Some(pixbuf),
            )
        };
        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Start);
        content.add_css_class("media-viewer");
        let (tx, rx) = mpsc::channel();
        let pages = dimensions
            .iter()
            .map(|_| {
                let area = gtk::DrawingArea::new();
                area.set_halign(gtk::Align::Center);
                content.append(&area);
                area
            })
            .collect();
        let viewer = Rc::new(Self {
            content,
            pages,
            dimensions,
            zoom: Cell::new(zoom),
            pdf,
            cache: RefCell::new(VecDeque::new()),
            pending: RefCell::new(HashSet::new()),
            failures: RefCell::new(HashMap::new()),
            response: tx,
        });
        for (index, area) in viewer.pages.iter().enumerate() {
            let weak = Rc::downgrade(&viewer);
            let image = image.clone();
            area.set_draw_func(move |area, context, width, height| {
                let Some(v) = weak.upgrade() else {
                    return;
                };
                if let Some(image) = &image {
                    context.scale(
                        f64::from(width) / f64::from(image.width()),
                        f64::from(height) / f64::from(image.height()),
                    );
                    context.set_source_pixbuf(image, 0.0, 0.0);
                    let _ = context.paint();
                    return;
                }
                context.set_source_rgb(1.0, 1.0, 1.0);
                let _ = context.paint();
                let requested = width.clamp(1, 4096);
                let cache = v.cache.borrow();
                if let Some((_, _, surface)) = cache
                    .iter()
                    .find(|(page, w, _)| *page == index && *w == requested)
                {
                    context.scale(
                        f64::from(width) / f64::from(surface.width()),
                        f64::from(height) / f64::from(surface.height()),
                    );
                    let _ = context.set_source_surface(surface, 0.0, 0.0);
                    let _ = context.paint();
                } else if let Some(error) = v.failures.borrow().get(&(index, requested)) {
                    context.set_source_rgb(0.7, 0.1, 0.1);
                    context.move_to(16.0, 24.0);
                    let _ = context.show_text(error);
                } else if Self::visible(area)
                    && v.pending.borrow().len() < 4
                    && v.pending.borrow_mut().insert((index, requested))
                {
                    let _ = worker().send(Command::Render(
                        v.pdf.unwrap(),
                        index,
                        requested,
                        v.response.clone(),
                    ));
                }
            });
        }
        let weak = Rc::downgrade(&viewer);
        glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
            let Some(v) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let mut received = false;
            while let Ok(result) = rx.try_recv() {
                received = true;
                match result {
                    Ok(raster) => {
                        v.pending
                            .borrow_mut()
                            .remove(&(raster.page, raster.requested));
                        if let Ok(surface) = gtk::cairo::ImageSurface::create_for_data(
                            raster.pixels,
                            gtk::cairo::Format::ARgb32,
                            raster.width,
                            raster.height,
                            raster.width * 4,
                        ) {
                            let mut cache = v.cache.borrow_mut();
                            cache.retain(|(page, _, _)| *page != raster.page);
                            cache.push_front((raster.page, raster.requested, surface));
                            let mut bytes = 0usize;
                            cache.retain(|(_, _, surface)| {
                                bytes += surface.stride() as usize * surface.height() as usize;
                                bytes <= 128 * 1024 * 1024
                            });
                        }
                    }
                    Err((page, width, error)) => {
                        v.pending.borrow_mut().remove(&(page, width));
                        v.failures.borrow_mut().insert((page, width), error);
                    }
                }
            }
            if received {
                v.redraw_visible();
            }
            glib::ControlFlow::Continue
        });
        viewer.set_zoom(zoom);
        Ok(viewer)
    }
    fn visible(area: &gtk::DrawingArea) -> bool {
        area.ancestor(gtk::ScrolledWindow::static_type())
            .and_then(|scroll| area.compute_bounds(&scroll).map(|r| (scroll, r)))
            .is_some_and(|(scroll, r)| r.y() < scroll.height() as f32 && r.y() + r.height() > 0.0)
    }
    fn redraw_visible(&self) {
        for area in &self.pages {
            if Self::visible(area) {
                area.queue_draw();
            }
        }
    }
    pub fn attach_scroll(self: &Rc<Self>, scroll: &gtk::ScrolledWindow) {
        let weak = Rc::downgrade(self);
        scroll.vadjustment().connect_value_changed(move |_| {
            if let Some(v) = weak.upgrade() {
                for area in &v.pages {
                    area.queue_draw();
                }
            }
        });
    }
    pub fn set_zoom(&self, zoom: f64) {
        self.zoom.set(zoom.clamp(5.0, 800.0));
        for (area, (width, height)) in self.pages.iter().zip(&self.dimensions) {
            let scale =
                self.zoom.get() / 100.0 * if self.pdf.is_some() { 96.0 / 72.0 } else { 1.0 };
            area.set_content_width((width * scale).round().clamp(1.0, 1_000_000.0) as i32);
            area.set_content_height((height * scale).round().clamp(1.0, 1_000_000.0) as i32);
            area.queue_draw();
        }
    }
    #[cfg(test)]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    #[cfg(test)]
    pub fn rendered_pages(&self) -> usize {
        self.cache.borrow().len()
    }
}
