use crate::overlay_widget::imp::PieMenuOverlayWidgetImpl;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::glib::Object;
use gtk4::subclass::prelude::*;
use tracing::error;

glib::wrapper! {
    pub struct PieMenuOverlayWidget(ObjectSubclass<PieMenuOverlayWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl PieMenuOverlayWidget {
    pub fn new(child: Option<&Widget>) -> Self {
        let widget: Self = Object::builder().build();
        if let Some(child_widget) = child {
            let imp = widget.imp();
            imp.overlay.set_child(Some(child_widget));
        } else {
            error!("PieMenuWidget::new failed to find child widget");
        }
        widget
    }
}
