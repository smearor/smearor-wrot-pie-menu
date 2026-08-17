use crate::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::message::PieMenuMessage;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use glib::subclass::prelude::ObjectSubclassIsExt;
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

    fn set_scroll_rotation_step(&self, sensitivity: f64) {
        self.scroll_rotation_step.store(sensitivity, Ordering::Relaxed);
    }

    fn scroll_rotation_step(&self) -> f32 {
        self.scroll_rotation_step.load(Ordering::Relaxed) as f32
    }

    fn cycle_selection(&self, direction: i32) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().cycle_selection(direction);
        }
    }

    fn select_first_item(&self) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().select_first_item();
        }
    }

    fn confirm_selection(&self) {
        let selected_id = self.pie_menu_widget.borrow().as_ref().and_then(|widget| widget.imp().keyboard_selection());
        if let Some(id) = selected_id {
            let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
            if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref()
                && let Some(item) = pie_menu_widget.imp().menu_items.get(&id)
            {
                if !item.enabled {
                    return;
                }
                let event = item.event.clone();
                self.send_message(PieMenuMessage::Event(event));
            }
        }
    }

    fn handle_left_stick_x(&self, x: f32) {
        self.left_stick_x.set(x);
    }

    fn handle_right_stick(&self, x: f32, y: f32) {
        if !self.is_pie_menu_open() {
            return;
        }
        let magnitude = (x * x + y * y).sqrt();
        if magnitude < 0.3 {
            return;
        }
        let stick_angle = (-y).atan2(x).to_degrees().rem_euclid(360.0);
        if let Some(nearest) = self.find_nearest_item(stick_angle) {
            self.set_keyboard_selection(nearest);
        }
    }

    fn find_nearest_item(&self, target_angle: f32) -> Option<String> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        pie_menu_widget_borrow.as_ref().and_then(|widget| widget.imp().find_nearest_item(target_angle))
    }

    fn set_keyboard_selection(&self, id: String) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().set_keyboard_selection(id);
        }
    }
}
