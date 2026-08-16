use crate::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use gtk4::prelude::WidgetExt;
use std::sync::atomic::Ordering;

impl PieMenuControlHandler for PieMenuOverlayWidgetImpl {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(ShowPieMenuError::MenuWidgetNotAvailable);
        };
        self.visible.store(true, Ordering::Relaxed);
        pie_menu_widget.set_visible(true);
        Ok(())
    }

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(HidePieMenuError::MenuWidgetNotAvailable);
        };
        self.visible.store(false, Ordering::Relaxed);
        pie_menu_widget.set_visible(false);
        Ok(())
    }

    fn is_pie_menu_open(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }
}
