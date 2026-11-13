use crate::PieMenuWidgetImpl;
use crate::menu_widget::rotation::handler::RotationHandler;
use smearor_commons_geometry::RotationRad;
use std::sync::atomic::Ordering;

impl RotationHandler for PieMenuWidgetImpl {
    fn set_rotation(&self, rotation: RotationRad) {
        self.rotation.store(rotation.to_degrees_f32(), Ordering::Relaxed);
    }

    fn rotation(&self) -> RotationRad {
        self.rotation()
    }
}
