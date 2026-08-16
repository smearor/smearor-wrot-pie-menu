use crate::PieMenuOverlayWidgetImpl;
use crate::menu_widget::rotation::handler::RotationHandler;

impl RotationHandler for PieMenuOverlayWidgetImpl {
    fn set_rotation(&self, rotation: f32) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return;
        };
        pie_menu_widget.set_rotation(rotation);
    }
}
