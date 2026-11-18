//! A native picture with a nonzero height derived from its available width.
//! GtkPicture's shrinkable minimum alone lets long documents squeeze it away.
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Default)]
    pub struct MarkdownImage {
        pub picture: OnceCell<gtk::Picture>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
        pub zoom: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MarkdownImage {
        const NAME: &'static str = "LeditorMarkdownImage";
        type Type = super::MarkdownImage;
        type ParentType = gtk::Widget;
    }
    impl ObjectImpl for MarkdownImage {
        fn dispose(&self) {
            if let Some(picture) = self.picture.get() {
                picture.unparent();
            }
        }
    }
    impl MarkdownImage {
        fn natural_width(&self) -> i32 {
            (f64::from(self.width.get()) * self.zoom.get() / 100.0)
                .round()
                .max(1.0) as i32
        }
    }
    impl WidgetImpl for MarkdownImage {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let natural_width = self.natural_width();
            if orientation == gtk::Orientation::Horizontal {
                return (1, natural_width, -1, -1);
            }
            let width = if for_size < 0 {
                natural_width
            } else {
                for_size.min(natural_width).max(1)
            };
            let height = (f64::from(width) * f64::from(self.height.get())
                / f64::from(self.width.get().max(1)))
            .round()
            .max(1.0) as i32;
            (height, height, -1, -1)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(picture) = self.picture.get() {
                picture.allocate(
                    width.min(self.natural_width()).max(0),
                    height,
                    baseline,
                    None,
                );
            }
        }
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if let Some(picture) = self.picture.get() {
                self.obj().snapshot_child(picture, snapshot);
            }
        }
    }
}

glib::wrapper! {
    pub struct MarkdownImage(ObjectSubclass<imp::MarkdownImage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MarkdownImage {
    pub fn new(pixbuf: &gtk::gdk_pixbuf::Pixbuf, alt: &str, zoom: f64) -> Self {
        let image: Self = glib::Object::new();
        let imp = image.imp();
        imp.width.set(pixbuf.width());
        imp.height.set(pixbuf.height());
        imp.zoom.set(zoom.clamp(5.0, 800.0));
        let picture = gtk::Picture::for_paintable(&gtk::gdk::Texture::for_pixbuf(pixbuf));
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_can_shrink(true);
        picture.set_alternative_text(Some(alt));
        picture.set_tooltip_text(Some(alt));
        picture.set_parent(&image);
        imp.picture.set(picture).unwrap();
        image.set_hexpand(true);
        image
    }

    pub fn set_zoom(&self, zoom: f64) {
        self.imp().zoom.set(zoom.clamp(5.0, 800.0));
        self.queue_resize();
    }
}
