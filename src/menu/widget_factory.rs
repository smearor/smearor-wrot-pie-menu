use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use gtk4::Widget;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Factory trait for creating menu item widgets.
///
/// Each factory is registered in the [`MenuItemWidgetRegistry`](crate::menu::widget_registry::MenuItemWidgetRegistry)
/// under a unique type name. When a menu item is rendered, the registry
/// resolves the factory by the item's `widget_type` field and calls
/// `build` to create the GTK4 widget.
///
/// The associated `Config` type provides type-safe, serializable
/// configuration for the widget. The registry automatically
/// deserializes `MenuItem::widget_config` (`serde_json::Value`) into
/// `Config` before calling `build`. This gives consumers full type
/// safety without manual JSON extraction.
///
/// Implementations are `!Send` and `!Sync` because `gtk4::Widget` is
/// bound to the GLib main thread. All registration and rendering occurs
/// on the GTK main thread.
pub trait MenuItemWidgetFactory {
    /// Type-safe configuration for this widget type.
    ///
    /// Must implement `Serialize`, `DeserializeOwned`, and `Default`.
    /// The `Default` value is used when `widget_config` is `None`.
    type Config: Serialize + DeserializeOwned + Default;

    /// Returns the unique type name for this factory.
    ///
    /// This name is used by `MenuItem::widget_type` to resolve the
    /// factory from the registry. Examples: `"circle"`, `"square"`.
    fn type_name(&self) -> &str;

    /// Builds and returns a GTK4 widget for the given menu item.
    ///
    /// The widget is registered as a child of `PieMenuWidget` via
    /// `set_parent` by the rendering pipeline after construction.
    /// The `config` parameter is the typed configuration,
    /// automatically deserialized from `item.widget_config`.
    fn build(&self, item: &MenuItem, config: &Self::Config, context: &MenuItemContext) -> Widget;
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::Cast;
    use gtk4::prelude::ObjectExt;
    use serde::Deserialize;

    struct DummyFactory;

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct DummyConfig {
        value: f64,
    }

    impl MenuItemWidgetFactory for DummyFactory {
        type Config = DummyConfig;

        fn type_name(&self) -> &str {
            "dummy"
        }

        fn build(&self, _item: &MenuItem, config: &DummyConfig, _context: &MenuItemContext) -> Widget {
            let label = gtk4::Label::new(Some(&format!("{}", config.value)));
            label.upcast::<Widget>()
        }
    }

    #[test]
    fn test_factory_type_name() {
        let factory = DummyFactory;
        assert_eq!(factory.type_name(), "dummy");
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_factory_build_returns_widget() {
        crate::test_util::ensure_gtk_init();
        let factory = DummyFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let config = DummyConfig { value: 42.0 };
        let widget = factory.build(&item, &config, &context);
        assert!(widget.is::<gtk4::Label>());
    }
}
