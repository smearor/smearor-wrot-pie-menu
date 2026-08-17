use crate::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::message::PieMenuMessage;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use gtk4::PropagationPhase;
use gtk4::prelude::EventControllerExt;
use gtk4::prelude::WidgetExt;
use std::sync::atomic::Ordering;

impl PieMenuControlHandler for PieMenuOverlayWidgetImpl {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(ShowPieMenuError::MenuWidgetNotAvailable);
        };
        self.visible.store(true, Ordering::Relaxed);
        pie_menu_widget.set_visible(true);
        self.send_message(PieMenuMessage::Opened);
        Ok(())
    }

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.clone() else {
            return Err(HidePieMenuError::MenuWidgetNotAvailable);
        };
        self.visible.store(false, Ordering::Relaxed);
        pie_menu_widget.set_visible(false);
        self.send_message(PieMenuMessage::Closed);
        Ok(())
    }

    fn is_pie_menu_open(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    fn set_activation_threshold(&self, threshold: f64) {
        self.activation_threshold.store(threshold, Ordering::Relaxed);
    }

    fn activation_threshold(&self) -> f64 {
        self.activation_threshold.load(Ordering::Relaxed)
    }

    fn set_deactivation_threshold(&self, threshold: f64) {
        self.deactivation_threshold.store(threshold, Ordering::Relaxed);
    }

    fn deactivation_threshold(&self) -> f64 {
        self.deactivation_threshold.load(Ordering::Relaxed)
    }

    fn set_rotation_gesture_enabled(&self, enabled: bool) {
        self.rotation_gesture_enabled.store(enabled, Ordering::Relaxed);
        let phase = if enabled { PropagationPhase::Capture } else { PropagationPhase::None };
        if let Some(ref gesture) = *self.rotate_gesture.borrow() {
            gesture.set_propagation_phase(phase);
        }
    }

    fn rotation_gesture_enabled(&self) -> bool {
        self.rotation_gesture_enabled.load(Ordering::Relaxed)
    }

    fn set_markings_enabled(&self, enabled: bool) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.set_markings_enabled(enabled);
        }
    }

    fn markings_enabled(&self) -> bool {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.markings_enabled()
        } else {
            true
        }
    }
}
