use crate::PieMenuOverlayWidget;
use crate::menu_widget::rotation::handler::RotationHandler;
use glib::subclass::prelude::ObjectSubclassIsExt;

impl RotationHandler for PieMenuOverlayWidget {
    fn set_rotation(&self, rotation: f32) {
        self.imp().set_rotation(rotation)
    }
}
