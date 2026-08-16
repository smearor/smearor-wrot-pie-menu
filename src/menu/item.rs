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
}

impl MenuItem {
    /// Returns the radius, falling back to the default if not set
    pub fn radius(&self) -> f32 {
        self.radius.unwrap_or(DEFAULT_MENU_ITEM_RADIUS)
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
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .icon_name("icon")
            .angle(0.0)
            .event("event")
            .build();
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
}
