use crate::PieMenuOverlayWidget;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use glib::subclass::prelude::ObjectSubclassIsExt;

pub trait PieMenuMenuItemHandler {
    /// Adds a menu item to the pie menu
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;

    /// Removes the menu item with the given id from the pie menu
    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError>;
}

impl PieMenuMenuItemHandler for PieMenuOverlayWidget {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        self.imp().add_menu_item(menu_item)
    }

    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError> {
        self.imp().remove_menu_item(id)
    }
}
