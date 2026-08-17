use crate::PieMenuWidget;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;

pub trait RotationHandler {
    fn set_rotation(&self, rotation: f32);

    /// Returns the current rotation in degrees.
    fn rotation(&self) -> f32;
}

impl RotationHandler for PieMenuWidget {
    fn set_rotation(&self, rotation: f32) {
        self.imp().set_rotation(rotation);
        self.queue_draw();
    }

    fn rotation(&self) -> f32 {
        self.imp().rotation()
    }
}
