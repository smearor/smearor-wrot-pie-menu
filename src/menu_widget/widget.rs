use crate::menu_widget::imp::widget::PieMenuWidgetImpl;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::glib;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::*;
use std::sync::atomic::Ordering;

glib::wrapper! {
    pub struct PieMenuWidget(ObjectSubclass<PieMenuWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl PieMenuWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_close_callback<F: Fn() + 'static>(&self, callback: F) {
        self.imp().close_callback.replace(Some(Box::new(callback)));
    }

    /// Enables or disables drawing of inner and outer ring markings.
    /// Default: `true`.
    pub fn set_markings_enabled(&self, enabled: bool) {
        self.imp().markings_enabled.store(enabled, Ordering::Relaxed);
        self.queue_draw();
    }

    /// Returns whether ring markings are currently enabled.
    pub fn markings_enabled(&self) -> bool {
        self.imp().markings_enabled.load(Ordering::Relaxed)
    }
}

impl Default for PieMenuWidget {
    fn default() -> Self {
        Self::new()
    }
}
