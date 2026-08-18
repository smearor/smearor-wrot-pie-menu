use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use crate::menu::widget_factory::MenuItemWidgetFactory;
use gtk4::Button;
use gtk4::Widget;
use gtk4::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

/// Typed configuration for the `"button"` widget type.
///
/// A simple debug widget that renders a GTK4 `Button` with a label.
/// Useful for verifying that the child widget positioning pipeline works.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder, Default)]
pub struct ButtonConfig {
    /// Label text displayed on the button.
    #[builder(setter(into))]
    pub label: String,
}

/// Factory for creating simple button menu item widgets.
///
/// Produces a GTK4 `Button` displaying the item label. This is primarily
/// a debug widget type to verify that child widget positioning and
/// visibility work correctly.
pub struct ButtonWidgetFactory;

impl MenuItemWidgetFactory for ButtonWidgetFactory {
    type Config = ButtonConfig;

    fn type_name(&self) -> &str {
        "button"
    }

    fn build(&self, _item: &MenuItem, config: &ButtonConfig, _context: &MenuItemContext) -> Widget {
        let button = Button::with_label(&config.label);
        button.set_halign(gtk4::Align::Center);
        button.set_valign(gtk4::Align::Center);
        button.upcast::<Widget>()
    }
}
