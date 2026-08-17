use crate::menu::MenuItem;
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

    /// Adds a menu item and returns self for chaining.
    /// Returns `Err` if the item could not be added.
    pub fn with_menu_item(self, item: MenuItem) -> Result<Self, AddMenuItemError> {
        self.add_menu_item(item)?;
        Ok(self)
    }
}
