//! App-local GTK input methods. No process environment mutation or restart.
use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc};

// Widget overrides require the canonical GIO extension ID; `simple` is
// an alias accepted by GTK_IM_MODULE/GtkSettings, not by IMMulticontext.
pub const SIMPLE_CONTEXT: &str = "gtk-im-context-simple";

pub struct InputMethod {
    simple: Cell<bool>,
    default_module: Option<glib::GString>,
}
impl Drop for InputMethod {
    fn drop(&mut self) {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_im_module(self.default_module.as_deref());
        }
    }
}
impl InputMethod {
    pub fn new() -> Rc<Self> {
        let this = Rc::new(Self {
            simple: Cell::new(false),
            default_module: gtk::Settings::default().and_then(|s| s.gtk_im_module()),
        });
        // GTK dialogs may create their entries after window construction. Apply
        // explicit per-widget overrides when they map, including GtkText inside
        // Entry/SpinButton. This also covers an inherited GTK_IM_MODULE override.
        let windows = gtk::Window::toplevels();
        let weak = Rc::downgrade(&this);
        windows.connect_items_changed(move |windows, position, _, added| {
            for i in position..position + added {
                if let Some(window) = windows.item(i).and_downcast::<gtk::Window>() {
                    let weak = weak.clone();
                    window.connect_map(move |window| {
                        if let Some(this) = weak.upgrade() {
                            this.apply_to(window);
                        }
                    });
                }
            }
        });
        this
    }
    pub fn is_simple(&self) -> bool {
        self.simple.get()
    }
    pub fn set_simple(&self, simple: bool) {
        self.simple.set(simple);
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_im_module(if simple {
                Some("simple")
            } else {
                self.default_module.as_deref()
            });
        }
        for window in gtk::Window::list_toplevels() {
            self.apply_to(&window);
        }
    }
    pub fn apply_to(&self, widget: &impl IsA<gtk::Widget>) {
        let widget = widget.as_ref();
        let module = self.simple.get().then_some(SIMPLE_CONTEXT);
        if let Some(view) = widget.downcast_ref::<gtk::TextView>() {
            if view.im_module().as_deref() != module {
                view.set_im_module(module);
            }
        } else if let Some(text) = widget.downcast_ref::<gtk::Text>() {
            if text.im_module().as_deref() != module {
                text.set_im_module(module);
            }
        }
        let mut child = widget.first_child();
        while let Some(widget) = child {
            self.apply_to(&widget);
            child = widget.next_sibling();
        }
    }
}
