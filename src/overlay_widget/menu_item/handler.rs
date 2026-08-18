use crate::PieMenuWidget;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::error::SetMenuItemEnabledError;
use crate::menu_widget::menu_item::error::UpdateMenuItemError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use crate::menu_widget::menu_item::widget_config_error::SetWidgetConfigError;
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

    fn redistribute(&self) {
        self.imp().redistribute();
    }

    fn get_menu_item(&self, id: &str) -> Option<MenuItem> {
        self.imp().get_menu_item(id)
    }

    fn update_menu_item(&self, menu_item: MenuItem) -> Result<(), UpdateMenuItemError> {
        self.imp().update_menu_item(menu_item)
    }

    fn refresh_widgets(&self) {
        self.imp().refresh_widgets()
    }

    fn set_widget_config(&self, id: &str, config: serde_json::Value) -> Result<(), SetWidgetConfigError> {
        self.imp().set_widget_config(id, config)
    }
}
