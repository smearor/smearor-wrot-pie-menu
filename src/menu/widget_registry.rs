use std::collections::HashMap;

use crate::menu::widget_factory_erased::MenuItemWidgetFactoryErased;

/// Registry mapping widget type names to their factory implementations.
///
/// The registry is populated with standard implementations (`"circle"`,
/// `"square"`) by the library. Consumers register custom widget
/// factories via [`MenuItemWidgetRegistry::register`].
///
/// The registry is `!Send` and `!Sync` because factories produce
/// `gtk4::Widget` instances bound to the GLib main thread.
pub struct MenuItemWidgetRegistry {
    factories: HashMap<String, Box<dyn MenuItemWidgetFactoryErased>>,
}

impl MenuItemWidgetRegistry {
    /// Creates a new registry pre-populated with standard implementations.
    ///
    /// Standard implementations:
    /// - `"circle"` - circular item with icon + label (existing behavior)
    /// - `"square"` - square item with icon + label
    /// - `"button"` - simple GTK4 Button with label (debug widget)
    /// - `"gauge"` - tachometer-style gauge with color-coded zones
    pub fn new() -> Self {
        let mut registry = Self { factories: HashMap::new() };
        registry.register(Box::new(crate::menu::circle_widget::CircleWidgetFactory));
        registry.register(Box::new(crate::menu::square_widget::SquareWidgetFactory));
        registry.register(Box::new(crate::menu::button_widget::ButtonWidgetFactory));
        registry.register(Box::new(crate::menu::gauge_widget::GaugeWidgetFactory));
        registry
    }

    /// Registers a custom widget factory under its type name.
    ///
    /// If a factory with the same type name already exists, it is
    /// replaced. This allows consumers to override standard
    /// implementations if desired.
    pub fn register(&mut self, factory: Box<dyn MenuItemWidgetFactoryErased>) {
        self.factories.insert(factory.type_name().to_string(), factory);
    }

    /// Resolves a factory by type name.
    ///
    /// Returns `None` if no factory is registered under the given name.
    pub fn get(&self, type_name: &str) -> Option<&dyn MenuItemWidgetFactoryErased> {
        self.factories.get(type_name).map(|boxed| boxed.as_ref())
    }
}

impl Default for MenuItemWidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::Cast;

    #[test]
    fn test_registry_new_has_circle() {
        let registry = MenuItemWidgetRegistry::new();
        assert!(registry.get("circle").is_some());
    }

    #[test]
    fn test_registry_new_has_square() {
        let registry = MenuItemWidgetRegistry::new();
        assert!(registry.get("square").is_some());
    }

    #[test]
    fn test_registry_register_custom() {
        let mut registry = MenuItemWidgetRegistry::new();

        struct CustomFactory;
        impl MenuItemWidgetFactoryErased for CustomFactory {
            fn type_name(&self) -> &str {
                "custom"
            }
            fn build(&self, _item: &crate::menu::MenuItem, _context: &crate::menu::context::MenuItemContext) -> gtk4::Widget {
                gtk4::Label::new(Some("custom")).upcast::<gtk4::Widget>()
            }
        }

        registry.register(Box::new(CustomFactory));
        assert!(registry.get("custom").is_some());
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_registry_register_overrides_existing() {
        crate::test_util::ensure_gtk_init();
        let mut registry = MenuItemWidgetRegistry::new();

        struct ReplacementCircleFactory;
        impl MenuItemWidgetFactoryErased for ReplacementCircleFactory {
            fn type_name(&self) -> &str {
                "circle"
            }
            fn build(&self, _item: &crate::menu::MenuItem, _context: &crate::menu::context::MenuItemContext) -> gtk4::Widget {
                gtk4::Label::new(Some("replacement")).upcast::<gtk4::Widget>()
            }
        }

        registry.register(Box::new(ReplacementCircleFactory));
        let factory = registry.get("circle").expect("circle factory should exist");
        let item = crate::menu::MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = crate::menu::context::MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: Box::new(|| {}),
        };
        let widget = factory.build(&item, &context);
        let label = widget.downcast_ref::<gtk4::Label>().expect("expected Label");
        assert_eq!(label.text(), "replacement");
    }

    #[test]
    fn test_registry_get_unknown_returns_none() {
        let registry = MenuItemWidgetRegistry::new();
        assert!(registry.get("unknown").is_none());
    }
}
