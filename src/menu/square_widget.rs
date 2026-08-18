use crate::color::RgbaColor;
use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use crate::menu::square_item_widget::SquareItemWidget;
use crate::menu::widget_factory::MenuItemWidgetFactory;
use gtk4::Widget;
use gtk4::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

/// Typed configuration for the `"square"` widget type.
///
/// All visual properties (icon, label, colors) are defined here -
/// `MenuItem` no longer carries presentation fields.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, Default)]
pub struct SquareConfig {
    /// Icon name for GTK4 icon theme lookup.
    #[builder(setter(into))]
    pub icon_name: String,
    /// Text label displayed alongside the icon.
    #[builder(default, setter(into))]
    pub label: String,
    /// Background color as an `RgbaColor`. When `None`, uses the factory default.
    #[builder(default, setter(into, strip_option))]
    pub color: Option<RgbaColor>,
    /// Label text color as an `RgbaColor`. When `None`, uses the factory default.
    #[builder(default, setter(into, strip_option))]
    pub label_color: Option<RgbaColor>,
    /// Icon size in pixels. When `None`, uses the default icon size.
    #[builder(default)]
    pub icon_size: Option<u32>,
    /// Whether to show the label. Defaults to `true`.
    #[builder(default)]
    pub show_label: Option<bool>,
}

/// Factory for creating square menu item widgets.
///
/// Produces a `SquareItemWidget` - a custom GTK4 widget subclass
/// that draws a square background, icon, and label in its own
/// `snapshot` method. Registered under the `"square"` type name.
pub struct SquareWidgetFactory;

impl MenuItemWidgetFactory for SquareWidgetFactory {
    type Config = SquareConfig;

    fn type_name(&self) -> &str {
        "square"
    }

    fn build(&self, item: &MenuItem, config: &SquareConfig, _context: &MenuItemContext) -> Widget {
        let bg_color = config.color.unwrap_or(crate::menu::item::DEFAULT_ICON_COLOR);

        let label_color = config.label_color.unwrap_or(crate::menu::item::DEFAULT_LABEL_COLOR);

        let widget = SquareItemWidget::new(
            &config.icon_name,
            if config.show_label.unwrap_or(true) { &config.label } else { "" },
            bg_color,
            label_color,
            item.radius(),
            item.enabled,
        );

        widget.upcast::<Widget>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::Cast;
    use gtk4::prelude::ObjectExt;

    #[test]
    fn test_square_config_default() {
        let config = SquareConfig::default();
        assert!(config.icon_name.is_empty());
        assert!(config.label.is_empty());
    }

    #[test]
    fn test_square_config_serialize() {
        let config = SquareConfig::builder()
            .icon_name("test")
            .label("Label")
            .icon_size(Some(48))
            .show_label(Some(true))
            .build();
        let json = serde_json::to_string(&config).expect("serialize should succeed");
        assert!(json.contains("test"));
    }

    #[test]
    fn test_square_config_deserialize() {
        let json = "{\"icon_name\":\"test\",\"label\":\"Label\",\"color\":\"#FF0000FF\",\"label_color\":\"#FFFFFFFF\"}";
        let config: SquareConfig = serde_json::from_str(json).expect("deserialize should succeed");
        assert_eq!(config.icon_name, "test");
    }

    #[test]
    fn test_square_factory_type_name() {
        let factory = SquareWidgetFactory;
        assert_eq!(factory.type_name(), "square");
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_square_factory_build_returns_widget() {
        crate::test_util::ensure_gtk_init();
        let factory = SquareWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = SquareConfig::default();
        let widget = factory.build(&item, &config, &context);
        assert!(widget.is::<SquareItemWidget>());
    }
}
