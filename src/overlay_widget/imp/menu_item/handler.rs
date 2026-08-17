use crate::PieMenuOverlayWidgetImpl;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::error::SetMenuItemEnabledError;
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

    fn remove_all_menu_items(&self) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.remove_all_menu_items();
        }
    }

    fn menu_item_count(&self) -> usize {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.menu_item_count()
        } else {
            0
        }
    }

    fn set_menu_item_enabled(&self, id: &str, enabled: bool) -> Result<(), SetMenuItemEnabledError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(SetMenuItemEnabledError::NotFound { id: id.to_string() });
        };
        pie_menu_widget.set_menu_item_enabled(id, enabled)
    }

    fn add_menu_item_auto(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(AddMenuItemError::MenuWidgetNotAvailable);
        };
        pie_menu_widget.add_menu_item_auto(menu_item)
    }
}
