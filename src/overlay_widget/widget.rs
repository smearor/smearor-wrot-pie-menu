use crate::menu::MenuItem;
use crate::menu::widget_factory_erased::MenuItemWidgetFactoryErased;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::imp::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use crate::overlay_widget::message::message::PieMenuMessage;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::glib::Object;
use gtk4::subclass::prelude::*;
use std::sync::mpsc::Sender;
use tracing::error;

glib::wrapper! {
    pub struct PieMenuOverlayWidget(ObjectSubclass<PieMenuOverlayWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl PieMenuOverlayWidget {
    pub fn new(child: Option<&Widget>) -> Self {
        let widget: Self = Object::builder().build();
        if let Some(child_widget) = child {
            let imp = widget.imp();
            imp.overlay.set_child(Some(child_widget));
        } else {
            error!("PieMenuWidget::new failed to find child widget");
        }
        widget
    }

    /// Sets the message sender and returns self for chaining.
    pub fn with_message_sender(self, sender: Sender<PieMenuMessage>) -> Self {
        self.set_message_sender(sender);
        self
    }

    /// Sets the activation threshold and returns self for chaining.
    pub fn with_activation_threshold(self, threshold: f64) -> Self {
        self.set_activation_threshold(threshold);
        self
    }

    /// Sets the deactivation threshold and returns self for chaining.
    pub fn with_deactivation_threshold(self, threshold: f64) -> Self {
        self.set_deactivation_threshold(threshold);
        self
    }

    /// Enables or disables the rotation gesture and returns self for chaining.
    /// Default: `true`.
    pub fn with_rotation_gesture_enabled(self, enabled: bool) -> Self {
        self.set_rotation_gesture_enabled(enabled);
        self
    }

    /// Enables or disables ring markings and returns self for chaining.
    /// Default: `true`.
    pub fn with_markings_enabled(self, enabled: bool) -> Self {
        self.set_markings_enabled(enabled);
        self
    }

    /// Sets the scroll rotation sensitivity multiplier and returns self for chaining.
    /// Default: `5.0`.
    pub fn with_scroll_rotation_step(self, sensitivity: f64) -> Self {
        self.set_scroll_rotation_step(sensitivity);
        self
    }

    /// Sets the radius of the main pie menu ring and returns self for chaining.
    /// Default: `160.0`.
    pub fn with_pie_menu_radius(self, radius: f32) -> Self {
        self.set_pie_menu_radius(radius);
        self
    }

    /// Sets the inner radius of the pie menu ring (transparent center)
    /// and returns self for chaining. Default: `64.0`.
    pub fn with_pie_menu_center_radius(self, radius: f32) -> Self {
        self.set_pie_menu_center_radius(radius);
        self
    }

    /// Sets the step width between consecutive submenu ring levels and returns self for chaining.
    /// Each submenu level's radius is computed as: `main_radius + level * step`.
    /// Default: `80.0`.
    pub fn with_submenu_radius_step(self, step: f32) -> Self {
        self.set_submenu_radius_step(step);
        self
    }

    /// Sets the radius for a specific submenu level and returns self for chaining.
    /// Level 0 is the main ring, level 1 is the first submenu, etc.
    pub fn with_submenu_radius(self, level: u32, radius: f32) -> Self {
        self.set_submenu_radius(level, radius);
        self
    }

    /// Adds a menu item and returns self for chaining.
    /// Returns `Err` if the item could not be added.
    pub fn with_menu_item(self, item: MenuItem) -> Result<Self, AddMenuItemError> {
        self.add_menu_item(item)?;
        Ok(self)
    }

    /// Registers a custom widget factory under its type name.
    ///
    /// If a factory with the same type name already exists, it is
    /// replaced. Call `refresh_widgets` after registering new
    /// factories to rebuild existing item widgets.
    pub fn register_widget_factory(&self, factory: Box<dyn MenuItemWidgetFactoryErased>) {
        let pie_menu_widget_borrow = self.imp().pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.register_widget_factory(factory);
        }
    }
}
