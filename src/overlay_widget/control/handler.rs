use crate::PieMenuOverlayWidget;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use glib::subclass::prelude::ObjectSubclassIsExt;

pub trait PieMenuControlHandler {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError>;

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError>;

    fn is_pie_menu_open(&self) -> bool;
}

impl PieMenuControlHandler for PieMenuOverlayWidget {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError> {
        self.imp().show_pie_menu()
    }

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError> {
        self.imp().hide_pie_menu()
    }

    fn is_pie_menu_open(&self) -> bool {
        self.imp().is_pie_menu_open()
    }
}
