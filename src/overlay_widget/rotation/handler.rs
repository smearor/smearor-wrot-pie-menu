use crate::PieMenuOverlayWidget;
use crate::menu_widget::rotation::handler::RotationHandler;
use glib::subclass::prelude::ObjectSubclassIsExt;
use smearor_commons_geometry::RotationRad;

impl RotationHandler for PieMenuOverlayWidget {
    fn set_rotation(&self, rotation: RotationRad) {
        self.imp().set_rotation(rotation)
    }

    fn rotation(&self) -> RotationRad {
        self.imp().rotation()
    }
}
