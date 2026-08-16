use crate::PieMenuWidgetImpl;
use crate::menu_widget::rotation::handler::RotationHandler;
use std::sync::atomic::Ordering;

impl RotationHandler for PieMenuWidgetImpl {
    fn set_rotation(&self, rotation: f32) {
        self.rotation.store(rotation, Ordering::Relaxed);
    }
}
