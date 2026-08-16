use crate::PieMenuOverlayWidgetImpl;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

impl PieMenuMenuItemHandler for PieMenuOverlayWidgetImpl {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(AddMenuItemError::MenuWidgetNotAvailable);
        };
        pie_menu_widget.add_menu_item(menu_item)
    }

    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(RemoveMenuItemError::MenuWidgetNotAvailable);
        };
        pie_menu_widget.remove_menu_item(id)
    }
}
