use crate::PieMenuWidget;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::error::SetMenuItemEnabledError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use glib::subclass::prelude::ObjectSubclassIsExt;

impl PieMenuMenuItemHandler for PieMenuWidget {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        self.imp().add_menu_item(menu_item)
    }

    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError> {
        self.imp().remove_menu_item(id)
    }

    fn remove_all_menu_items(&self) {
        self.imp().remove_all_menu_items()
    }

    fn menu_item_count(&self) -> usize {
        self.imp().menu_item_count()
    }

    fn set_menu_item_enabled(&self, id: &str, enabled: bool) -> Result<(), SetMenuItemEnabledError> {
        self.imp().set_menu_item_enabled(id, enabled)
    }

    fn add_menu_item_auto(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        self.imp().add_menu_item_auto(menu_item)
    }
}
