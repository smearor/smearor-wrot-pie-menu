use crate::color::RgbaColor;
use crate::menu::MenuItem;
use crate::menu::circle_item_widget::CircleItemWidget;
use crate::menu::context::MenuItemContext;
use crate::menu::widget_factory::MenuItemWidgetFactory;
use gtk4::Widget;
use gtk4::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

/// Typed configuration for the `"circle"` widget type.
///
/// All visual properties (icon, label, colors) are defined here —
/// `MenuItem` no longer carries presentation fields.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CircleConfig {
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

impl Default for CircleConfig {
    fn default() -> Self {
        Self {
            icon_name: String::new(),
            label: String::new(),
            color: None,
            label_color: None,
            icon_size: None,
            show_label: None,
        }
    }
}

/// Factory for creating circular menu item widgets.
///
/// Produces a `CircleItemWidget` — a custom GTK4 widget subclass
/// that draws a circular background, icon, and label in its own
/// `snapshot` method. This is the default widget type — when
/// `MenuItem::widget_type` is `None`, the registry resolves
/// `"circle"`.
pub struct CircleWidgetFactory;

impl MenuItemWidgetFactory for CircleWidgetFactory {
    type Config = CircleConfig;

    fn type_name(&self) -> &str {
        "circle"
    }

    fn build(&self, item: &MenuItem, config: &CircleConfig, _context: &MenuItemContext) -> Widget {
        let bg_color = config.color.unwrap_or(crate::menu::item::DEFAULT_ICON_COLOR);

        let label_color = config.label_color.unwrap_or(crate::menu::item::DEFAULT_LABEL_COLOR);

        let widget = CircleItemWidget::new(
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
    fn test_circle_config_default() {
        let config = CircleConfig::default();
        assert!(config.icon_name.is_empty());
        assert!(config.label.is_empty());
        assert_eq!(config.color, None);
        assert_eq!(config.label_color, None);
        assert!(config.icon_size.is_none());
        assert!(config.show_label.is_none());
    }

    #[test]
    fn test_circle_config_serialize() {
        let config = CircleConfig::builder()
            .icon_name("test")
            .label("Label")
            .icon_size(Some(48))
            .show_label(Some(true))
            .build();
        let json = serde_json::to_string(&config).expect("serialize should succeed");
        assert!(json.contains("test"));
        assert!(json.contains("Label"));
    }

    #[test]
    fn test_circle_config_deserialize() {
        let json = "{\"icon_name\":\"test\",\"label\":\"Label\",\"color\":\"#FF0000FF\",\"label_color\":\"#FFFFFFFF\",\"icon_size\":48}";
        let config: CircleConfig = serde_json::from_str(json).expect("deserialize should succeed");
        assert_eq!(config.icon_name, "test");
        assert_eq!(config.label, "Label");
        assert_eq!(config.icon_size, Some(48));
    }

    #[test]
    fn test_circle_factory_type_name() {
        let factory = CircleWidgetFactory;
        assert_eq!(factory.type_name(), "circle");
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_circle_factory_build_returns_widget() {
        crate::test_util::ensure_gtk_init();
        let factory = CircleWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = CircleConfig::default();
        let widget = factory.build(&item, &config, &context);
        assert!(widget.is::<CircleItemWidget>());
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_circle_factory_reads_icon_from_config() {
        crate::test_util::ensure_gtk_init();
        let factory = CircleWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = CircleConfig::builder().icon_name("config-icon").build();
        let _widget = factory.build(&item, &config, &context);
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_circle_factory_reads_icon_from_item() {
        crate::test_util::ensure_gtk_init();
        let factory = CircleWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = CircleConfig::default();
        let _widget = factory.build(&item, &config, &context);
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_circle_factory_reads_label_from_config() {
        crate::test_util::ensure_gtk_init();
        let factory = CircleWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = CircleConfig::builder().icon_name("test-icon").label("ConfigLabel").build();
        let _widget = factory.build(&item, &config, &context);
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_circle_factory_reads_label_from_item() {
        crate::test_util::ensure_gtk_init();
        let factory = CircleWidgetFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = CircleConfig::default();
        let _widget = factory.build(&item, &config, &context);
    }
}
