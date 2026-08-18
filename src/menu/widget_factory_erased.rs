use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use crate::menu::widget_factory::MenuItemWidgetFactory;
use gtk4::Widget;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Type-erased factory trait for registry storage.
///
/// This trait allows the registry to store factories with different
/// `Config` types in a single `HashMap<String, Box<dyn MenuItemWidgetFactoryErased>>`.
///
/// A blanket implementation automatically converts any
/// `MenuItemWidgetFactory` into a `MenuItemWidgetFactoryErased` by
/// deserializing `item.widget_config` into the factory's `Config` type.
pub trait MenuItemWidgetFactoryErased {
    /// Returns the unique type name for this factory.
    fn type_name(&self) -> &str;

    /// Builds and returns a GTK4 widget, deserializing the config automatically.
    fn build(&self, item: &MenuItem, context: &MenuItemContext) -> Widget;
}

impl<F> MenuItemWidgetFactoryErased for F
where
    F: MenuItemWidgetFactory,
    F::Config: Serialize + DeserializeOwned + Default,
{
    fn type_name(&self) -> &str {
        <Self as MenuItemWidgetFactory>::type_name(self)
    }

    fn build(&self, item: &MenuItem, context: &MenuItemContext) -> Widget {
        let config: F::Config = item
            .widget_config
            .as_ref()
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        <Self as MenuItemWidgetFactory>::build(self, item, &config, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::widget_factory::MenuItemWidgetFactory;
    use gtk4::prelude::Cast;
    use serde::Deserialize;

    struct DummyFactory;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
    fn test_erased_type_name() {
        let factory = DummyFactory;
        let erased: &dyn MenuItemWidgetFactoryErased = &factory;
        assert_eq!(erased.type_name(), "dummy");
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_erased_build_deserializes_config() {
        crate::test_util::ensure_gtk_init();
        let factory = DummyFactory;
        let item = MenuItem::builder()
            .id("test")
            .angle(0.0)
            .event("event")
            .widget_config(serde_json::json!({ "value": 99.0 }))
            .build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let erased: &dyn MenuItemWidgetFactoryErased = &factory;
        let widget = erased.build(&item, &context);
        let label = widget.downcast_ref::<gtk4::Label>().expect("expected Label");
        assert_eq!(label.text(), "99");
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_erased_build_uses_default_when_none() {
        crate::test_util::ensure_gtk_init();
        let factory = DummyFactory;
        let item = MenuItem::builder().id("test").angle(0.0).event("event").build();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "event".to_string(),
            trigger_event: std::boxed::Box::new(|| {}),
        };
        let erased: &dyn MenuItemWidgetFactoryErased = &factory;
        let widget = erased.build(&item, &context);
        let label = widget.downcast_ref::<gtk4::Label>().expect("expected Label");
        assert_eq!(label.text(), "0");
    }
}
