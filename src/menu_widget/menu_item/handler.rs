use crate::PieMenuOverlayWidget;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::error::SetMenuItemEnabledError;
use glib::subclass::prelude::ObjectSubclassIsExt;

pub trait PieMenuMenuItemHandler {
    /// Adds a menu item to the pie menu
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;

    /// Removes the menu item with the given id from the pie menu
    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError>;

    /// Removes all menu items from the pie menu
    fn remove_all_menu_items(&self);

    /// Returns the number of menu items currently in the pie menu
    fn menu_item_count(&self) -> usize;

    /// Sets the enabled state of a menu item and triggers a redraw.
    /// When `enabled` is `false`, the item is rendered at reduced opacity
    /// and click events are suppressed.
    fn set_menu_item_enabled(&self, id: &str, enabled: bool) -> Result<(), SetMenuItemEnabledError>;

    /// Adds a menu item with an automatically calculated angle.
    /// The angle is distributed evenly across 360° based on the current item count.
    /// If the item has `fixed_position == true`, its angle is kept as-is.
    fn add_menu_item_auto(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;

    /// Redistributes all non-fixed items proportionally in the gaps between fixed items.
    /// Triggers a redraw. Useful after manual `remove_menu_item` calls to re-space
    /// the remaining items.
    fn redistribute(&self);
}

impl PieMenuMenuItemHandler for PieMenuOverlayWidget {
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
}
