use crate::color::RgbaColor;
use serde::Deserialize;
use serde::Serialize;
use std::hash::Hash;
use std::hash::Hasher;
use typed_builder::TypedBuilder;

/// Default radius for a menu item in pixels
pub const DEFAULT_MENU_ITEM_RADIUS: f32 = 40.0;

/// Default label color (white, fully opaque)
pub const DEFAULT_LABEL_COLOR: RgbaColor = RgbaColor::with_rgb(1.0, 1.0, 1.0, 1.0);

/// Default icon color (grey, ~47% transparent)
pub const DEFAULT_ICON_COLOR: RgbaColor = RgbaColor::with_rgb(0.467, 0.467, 0.467, 0.467);

/// A single item in a pie menu
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct MenuItem {
    /// The unique identifier
    #[builder(setter(into))]
    pub id: String,
    /// The label of the menu item
    #[builder(setter(into))]
    pub label: String,
    /// The color of the label
    #[builder(default = DEFAULT_LABEL_COLOR, setter(into))]
    pub label_color: RgbaColor,
    /// The icon name of the menu item
    #[builder(setter(into))]
    pub icon_name: String,
    /// The color of the icon
    #[builder(default = DEFAULT_ICON_COLOR, setter(into))]
    pub color: RgbaColor,
    /// The angle of the menu item in degrees
    pub angle: f32,
    /// The radius of the menu item in pixels (optional, uses default if not set)
    #[builder(default, setter(into, strip_option))]
    pub radius: Option<f32>,
    /// The name of the event to be triggered when the menu item is selected
    #[builder(setter(into))]
    pub event: String,
    /// Whether the menu item is enabled (clickable). Defaults to `true`.
    #[builder(default = true)]
    pub enabled: bool,
    /// When `true`, the item's `angle` is treated as a fixed semantic position.
    /// Auto-distribution will not re-assign this item's angle.
    /// The remaining items are distributed in the gaps between fixed items.
    #[builder(default = false)]
    pub fixed_position: bool,
    /// Whether the pie menu closes after this item is clicked. Defaults to `true`.
    #[builder(default = true)]
    pub close_on_click: bool,
    /// Optional submenu items. When present, selecting this item
    /// opens a nested ring instead of sending an event.
    ///
    /// Submenu items follow the same `fixed_position` / flexible angle
    /// distribution rules as top-level items.
    ///
    /// **ID uniqueness**: The `id` field must be globally unique across the
    /// entire menu tree (all levels). This simplifies lookup operations
    /// (`open_submenu`, `get_submenu_items`, `set_submenu_items`) to a flat
    /// search instead of a tree traversal from the root via `submenu_stack`.
    /// Duplicate IDs at any level are undefined behavior.
    #[builder(default, setter(strip_option))]
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    /// Returns the radius, falling back to the default if not set
    pub fn radius(&self) -> f32 {
        self.radius.unwrap_or(DEFAULT_MENU_ITEM_RADIUS)
    }

    /// Recursively searches for an item with the given id in this item's subtree.
    /// Returns a clone of the found item, or `None` if not found.
    pub fn find_recursive(&self, id: &str) -> Option<MenuItem> {
        if self.id == id {
            return Some(self.clone());
        }
        if let Some(submenu) = &self.submenu {
            for item in submenu {
                if let Some(found) = item.find_recursive(id) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Recursively replaces the submenu of the item with `parent_id`.
    /// Returns `true` if the parent was found and updated.
    pub fn replace_submenu_recursive(&mut self, parent_id: &str, new_submenu: &[MenuItem]) -> bool {
        if self.id == parent_id {
            self.submenu = Some(new_submenu.to_vec());
            return true;
        }
        if let Some(submenu) = &mut self.submenu {
            for item in submenu.iter_mut() {
                if item.replace_submenu_recursive(parent_id, new_submenu) {
                    return true;
                }
            }
        }
        false
    }
}

impl Hash for MenuItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for MenuItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for MenuItem {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::RgbColor;

    #[test]
    fn test_menu_item_builder() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test Item")
            .icon_name("test-icon")
            .angle(90.0)
            .event("test-event")
            .build();

        assert_eq!(item.id, "test");
        assert_eq!(item.label, "Test Item");
        assert_eq!(item.icon_name, "test-icon");
        assert_eq!(item.angle, 90.0);
        assert_eq!(item.event, "test-event");
        assert_eq!(item.label_color, DEFAULT_LABEL_COLOR);
        assert_eq!(item.color, DEFAULT_ICON_COLOR);
    }

    #[test]
    fn test_menu_item_builder_with_hex_colors() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .label_color("#FF0000FF")
            .icon_name("icon")
            .color("#00FF00FF")
            .angle(0.0)
            .event("event")
            .build();

        assert_eq!(item.label_color, RgbaColor::with_rgb(1.0, 0.0, 0.0, 1.0));
        assert_eq!(item.color, RgbaColor::with_rgb(0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn test_menu_item_builder_with_rgb_color() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .label_color(RgbColor::new(0.5, 0.5, 0.5))
            .icon_name("icon")
            .color(RgbColor::new(0.2, 0.8, 0.4))
            .angle(0.0)
            .event("event")
            .build();

        assert_eq!(item.label_color, RgbaColor::with_rgb(0.5, 0.5, 0.5, 1.0));
        assert_eq!(item.color, RgbaColor::with_rgb(0.2, 0.8, 0.4, 1.0));
    }

    #[test]
    fn test_menu_item_radius_default() {
        let item = MenuItem::builder().id("test").label("Test").icon_name("icon").angle(0.0).event("event").build();
        assert_eq!(item.radius(), DEFAULT_MENU_ITEM_RADIUS);
    }

    #[test]
    fn test_menu_item_radius_custom() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .icon_name("icon")
            .angle(0.0)
            .radius(60.0)
            .event("event")
            .build();
        assert_eq!(item.radius(), 60.0);
    }

    #[test]
    fn test_menu_item_eq_by_id() {
        let item1 = MenuItem::builder()
            .id("same")
            .label("Label1")
            .icon_name("icon1")
            .angle(0.0)
            .event("event1")
            .build();
        let item2 = MenuItem::builder()
            .id("same")
            .label("Label2")
            .icon_name("icon2")
            .angle(90.0)
            .event("event2")
            .build();
        assert_eq!(item1, item2);
    }

    #[test]
    fn test_menu_item_enabled_default() {
        let item = MenuItem::builder().id("test").label("Test").icon_name("icon").angle(0.0).event("event").build();
        assert!(item.enabled);
    }

    #[test]
    fn test_menu_item_disabled() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .icon_name("icon")
            .angle(0.0)
            .event("event")
            .enabled(false)
            .build();
        assert!(!item.enabled);
    }

    #[test]
    fn test_menu_item_close_on_click_default() {
        let item = MenuItem::builder().id("test").label("Test").icon_name("icon").angle(0.0).event("event").build();
        assert!(item.close_on_click);
    }

    #[test]
    fn test_menu_item_close_on_click_false() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .icon_name("icon")
            .angle(0.0)
            .event("event")
            .close_on_click(false)
            .build();
        assert!(!item.close_on_click);
    }

    #[test]
    fn test_submenu_field_default_none() {
        let item = MenuItem::builder().id("test").label("Test").icon_name("icon").angle(0.0).event("event").build();
        assert!(item.submenu.is_none());
    }

    #[test]
    fn test_submenu_with_items() {
        let child = MenuItem::builder()
            .id("child")
            .label("Child")
            .icon_name("icon")
            .angle(0.0)
            .event("child-event")
            .build();
        let parent = MenuItem::builder()
            .id("parent")
            .label("Parent")
            .icon_name("icon")
            .angle(0.0)
            .event("parent-event")
            .submenu(vec![child])
            .build();
        assert!(parent.submenu.is_some());
        assert_eq!(parent.submenu.as_ref().unwrap().len(), 1);
        assert_eq!(parent.submenu.as_ref().unwrap()[0].id, "child");
    }

    #[test]
    fn test_submenu_nested() {
        let grandchild = MenuItem::builder().id("gc").label("GC").icon_name("icon").angle(0.0).event("gc-event").build();
        let child = MenuItem::builder()
            .id("child")
            .label("Child")
            .icon_name("icon")
            .angle(0.0)
            .event("child-event")
            .submenu(vec![grandchild])
            .build();
        let parent = MenuItem::builder()
            .id("parent")
            .label("Parent")
            .icon_name("icon")
            .angle(0.0)
            .event("parent-event")
            .submenu(vec![child])
            .build();
        let submenu = parent.submenu.as_ref().unwrap();
        assert!(submenu[0].submenu.is_some());
        assert_eq!(submenu[0].submenu.as_ref().unwrap()[0].id, "gc");
    }
}
