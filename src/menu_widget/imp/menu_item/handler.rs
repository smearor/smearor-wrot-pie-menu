use crate::PieMenuWidgetImpl;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

impl PieMenuMenuItemHandler for PieMenuWidgetImpl {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        self.menu_items.insert(menu_item.id.clone(), menu_item);
        Ok(())
    }

    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError> {
        self.menu_items.remove(id);
        Ok(())
    }
}
