use crate::MenuItem;
use crate::PieMenuOverlayWidgetImpl;
use crate::menu::Menu;
use crate::menu_widget::menu_item::submenu_error::SubmenuError;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::imp::widget::MAX_SUBMENU_DEPTH;
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
        self.submenu_stack.borrow_mut().clear();
        *pie_menu_widget.imp().keyboard_selection.borrow_mut() = None;
        pie_menu_widget.imp().set_submenu_stack(Vec::new());
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
        self.submenu_stack.borrow_mut().clear();
        *pie_menu_widget.imp().keyboard_selection.borrow_mut() = None;
        pie_menu_widget.imp().set_submenu_stack(Vec::new());
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
            let (has_submenu, event) = {
                let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
                let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() else {
                    return;
                };
                let Some(item) = pie_menu_widget.imp().menu_items.find_item_recursive(&id) else {
                    return;
                };
                if !item.enabled {
                    return;
                }
                (item.submenu.is_some(), item.event.clone())
            };
            if has_submenu {
                let _ = self.open_submenu(&id);
            } else {
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

    fn open_submenu(&self, parent_id: &str) -> Result<(), SubmenuError> {
        if self.submenu_depth() >= MAX_SUBMENU_DEPTH {
            return Err(SubmenuError::MaxDepthReached { max_depth: MAX_SUBMENU_DEPTH });
        }

        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() else {
            return Err(SubmenuError::NotFound { id: parent_id.to_string() });
        };

        let parent_item = pie_menu_widget
            .imp()
            .menu_items
            .find_item_recursive(parent_id)
            .ok_or(SubmenuError::NotFound { id: parent_id.to_string() })?;

        let submenu_items = parent_item.submenu.clone().ok_or(SubmenuError::NoSubmenu { id: parent_id.to_string() })?;
        if submenu_items.is_empty() {
            return Err(SubmenuError::NoSubmenu { id: parent_id.to_string() });
        }

        let temp_menu = Menu::from_items(submenu_items);
        temp_menu.redistribute_angles();

        let ring_radius = self.submenu_radius_for_level(self.submenu_depth() + 1);
        if temp_menu.validate_all_no_overlap(ring_radius).is_err() {
            return Err(SubmenuError::ItemOverlap {
                parent_id: parent_id.to_string(),
            });
        }

        let redistributed_items = temp_menu.to_items();
        pie_menu_widget.imp().menu_items.replace_submenu_recursive(parent_id, redistributed_items);

        self.submenu_stack.borrow_mut().push(parent_id.to_string());
        pie_menu_widget.imp().set_submenu_stack(self.submenu_stack.borrow().clone());

        // Do not auto-select first submenu item - selection should only
        // be set when keyboard navigation is actively used.
        *pie_menu_widget.imp().keyboard_selection.borrow_mut() = None;

        pie_menu_widget.queue_draw();
        self.send_message(PieMenuMessage::SubmenuOpened(parent_id.to_string()));
        Ok(())
    }

    fn close_submenu(&self) -> Result<(), SubmenuError> {
        let parent_id = self.submenu_stack.borrow_mut().pop().ok_or(SubmenuError::NoSubmenuOpen)?;

        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().set_submenu_stack(self.submenu_stack.borrow().clone());
            // Do not auto-select parent item - selection should only
            // be set when keyboard navigation is actively used.
            *pie_menu_widget.imp().keyboard_selection.borrow_mut() = None;
            pie_menu_widget.queue_draw();
        }

        self.send_message(PieMenuMessage::SubmenuClosed(parent_id));
        Ok(())
    }

    fn submenu_depth(&self) -> u32 {
        self.submenu_stack.borrow().len() as u32
    }

    fn get_submenu_items(&self, parent_id: &str) -> Vec<MenuItem> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        pie_menu_widget_borrow
            .as_ref()
            .and_then(|widget| widget.imp().menu_items.find_item_recursive(parent_id))
            .and_then(|item| item.submenu)
            .unwrap_or_default()
    }

    fn redistribute_submenu(&self, parent_id: &str) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref()
            && let Some(parent_item) = pie_menu_widget.imp().menu_items.find_item_recursive(parent_id)
            && let Some(submenu_items) = parent_item.submenu
        {
            let temp_menu = Menu::from_items(submenu_items);
            temp_menu.redistribute_angles();
            let redistributed = temp_menu.to_items();
            pie_menu_widget.imp().menu_items.replace_submenu_recursive(parent_id, redistributed);
            pie_menu_widget.queue_draw();
        }
    }

    fn set_submenu_items(&self, parent_id: &str, items: Vec<MenuItem>) -> Result<(), SubmenuError> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() else {
            return Err(SubmenuError::NotFound { id: parent_id.to_string() });
        };

        if pie_menu_widget.imp().menu_items.find_item_recursive(parent_id).is_none() {
            return Err(SubmenuError::NotFound { id: parent_id.to_string() });
        }

        let temp_menu = Menu::from_items(items);
        temp_menu.redistribute_angles();

        let ring_radius = self.submenu_radius_for_level(self.submenu_depth() + 1);
        if temp_menu.validate_all_no_overlap(ring_radius).is_err() {
            return Err(SubmenuError::ItemOverlap {
                parent_id: parent_id.to_string(),
            });
        }

        let redistributed = temp_menu.to_items();
        pie_menu_widget.imp().menu_items.replace_submenu_recursive(parent_id, redistributed);
        pie_menu_widget.queue_draw();
        Ok(())
    }

    fn set_pie_menu_radius(&self, radius: f32) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().radius.store(radius, Ordering::Relaxed);
            pie_menu_widget.queue_resize();
            pie_menu_widget.queue_draw();
        }
    }

    fn pie_menu_radius(&self) -> f32 {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        pie_menu_widget_borrow
            .as_ref()
            .map(|widget| widget.imp().radius.load(Ordering::Relaxed))
            .unwrap_or(160.0)
    }

    fn set_pie_menu_center_radius(&self, radius: f32) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().center_radius.store(radius, Ordering::Relaxed);
            pie_menu_widget.queue_draw();
        }
    }

    fn pie_menu_center_radius(&self) -> f32 {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        pie_menu_widget_borrow
            .as_ref()
            .map(|widget| widget.imp().center_radius.load(Ordering::Relaxed))
            .unwrap_or(64.0)
    }

    fn set_submenu_radius(&self, level: u32, radius: f32) {
        self.submenu_radii.borrow_mut().insert(level, radius);
    }

    fn set_submenu_radius_step(&self, step: f32) {
        self.submenu_radius_step.store(step as f64, Ordering::Relaxed);
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().submenu_radius_step.store(step, Ordering::Relaxed);
            pie_menu_widget.queue_draw();
        }
    }

    fn max_submenu_depth(&self) -> u32 {
        MAX_SUBMENU_DEPTH
    }
}
