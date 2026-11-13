use crate::PieMenuOverlayWidgetImpl;
use crate::menu_widget::rotation::handler::RotationHandler;
use smearor_commons_geometry::RotationRad;

impl RotationHandler for PieMenuOverlayWidgetImpl {
    fn set_rotation(&self, rotation: RotationRad) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return;
        };
        pie_menu_widget.set_rotation(rotation);
    }

    fn rotation(&self) -> RotationRad {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return RotationRad::ZERO;
        };
        pie_menu_widget.rotation()
    }
}
