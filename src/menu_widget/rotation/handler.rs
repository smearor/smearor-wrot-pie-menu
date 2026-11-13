use crate::PieMenuWidget;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;
use smearor_commons_geometry::RotationRad;

pub trait RotationHandler {
    fn set_rotation(&self, rotation: RotationRad);

    /// Returns the current rotation.
    fn rotation(&self) -> RotationRad;
}

impl RotationHandler for PieMenuWidget {
    fn set_rotation(&self, rotation: RotationRad) {
        self.imp().set_rotation(rotation);
        self.queue_allocate();
        self.queue_draw();
    }

    fn rotation(&self) -> RotationRad {
        self.imp().rotation()
    }
}
