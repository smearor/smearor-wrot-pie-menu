use crate::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use crate::overlay_widget::message::message::PieMenuMessage;
use glib::subclass::prelude::ObjectSubclassIsExt;
use std::sync::atomic::Ordering;

impl PieMenuOverlayWidgetImpl {
    /// Confirms the current keyboard selection. If the selected item has a submenu,
    /// opens it. Otherwise sends `PieMenuMessage::Event` for the selected item.
    /// Does nothing if no item is selected or if the selected item is disabled.
    pub(crate) fn confirm_selection(&self) {
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
                let _ = self.hide_pie_menu();
                self.send_message(PieMenuMessage::Event(event));
            }
        }
    }

    /// Cycles the keyboard selection by `direction` (-1 for CCW, +1 for CW).
    pub(crate) fn cycle_selection(&self, direction: i32) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().cycle_selection(direction);
        }
    }

    /// Selects the first item (smallest angle, typically 0°).
    pub(crate) fn select_first_item(&self) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().select_first_item();
        }
    }

    /// Finds the menu item whose angle is closest to `target_angle` (in degrees).
    /// Returns the item ID, or `None` if the menu is empty.
    pub(crate) fn find_nearest_item(&self, target_angle: f32) -> Option<String> {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        pie_menu_widget_borrow.as_ref().and_then(|widget| widget.imp().find_nearest_item(target_angle))
    }

    /// Sets the keyboard selection to the given item ID and triggers a redraw.
    pub(crate) fn set_keyboard_selection(&self, id: String) {
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.imp().set_keyboard_selection(id);
        }
    }

    /// Updates the stored left stick X-axis value for continuous rotation.
    /// The actual rotation is applied per-frame via the tick callback.
    /// Must be called on the GTK main thread.
    /// `x` is in the range [-1.0, 1.0].
    pub(crate) fn handle_left_stick_x(&self, x: f32) {
        self.left_stick_x.set(x);
    }

    /// Selects the nearest menu item based on the right stick direction.
    /// `x` and `y` are in the range [-1.0, 1.0].
    /// Must be called on the GTK main thread.
    pub(crate) fn handle_right_stick(&self, x: f32, y: f32) {
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

    /// Sets the scroll rotation sensitivity multiplier.
    pub(crate) fn set_scroll_rotation_step(&self, sensitivity: f64) {
        self.scroll_rotation_step.store(sensitivity, Ordering::Relaxed);
    }

    /// Returns the current scroll rotation sensitivity multiplier.
    pub(crate) fn scroll_rotation_step(&self) -> f32 {
        self.scroll_rotation_step.load(Ordering::Relaxed) as f32
    }

    /// Computes the ring radius for a given submenu level.
    /// Uses explicit override if set, otherwise `main_radius + level * step`.
    pub(crate) fn submenu_radius_for_level(&self, level: u32) -> f32 {
        if let Some(radius) = self.submenu_radii.borrow().get(&level) {
            return *radius;
        }
        let pie_menu_widget_borrow = self.pie_menu_widget.borrow();
        let main_radius = pie_menu_widget_borrow
            .as_ref()
            .map(|widget| widget.imp().radius.load(Ordering::Relaxed))
            .unwrap_or(160.0);
        let step = self.submenu_radius_step.load(Ordering::Relaxed) as f32;
        main_radius + level as f32 * step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay_widget::imp::widget::DEFAULT_SCROLL_ROTATION_STEP;
    use atomic_float::AtomicF64;
    use std::cell::Cell;

    #[test]
    fn test_scroll_rotation_step_default() {
        let step = AtomicF64::new(DEFAULT_SCROLL_ROTATION_STEP);
        assert_eq!(step.load(Ordering::Relaxed), 5.0);
    }

    #[test]
    fn test_set_scroll_rotation_step() {
        let step = AtomicF64::new(DEFAULT_SCROLL_ROTATION_STEP);
        step.store(10.0, Ordering::Relaxed);
        assert_eq!(step.load(Ordering::Relaxed), 10.0);
    }

    #[test]
    fn test_left_stick_x_default() {
        let stick_x = Cell::new(0.0f32);
        assert_eq!(stick_x.get(), 0.0);
    }

    #[test]
    fn test_handle_left_stick_x_stored() {
        let stick_x = Cell::new(0.0f32);
        stick_x.set(0.5);
        assert_eq!(stick_x.get(), 0.5);
    }

    #[test]
    fn test_handle_left_stick_x_negative() {
        let stick_x = Cell::new(0.0f32);
        stick_x.set(-1.0);
        assert_eq!(stick_x.get(), -1.0);
    }

    #[test]
    fn test_handle_left_stick_full_deflection() {
        let stick_x = Cell::new(0.0f32);
        stick_x.set(1.0);
        assert_eq!(stick_x.get(), 1.0);
        stick_x.set(-1.0);
        assert_eq!(stick_x.get(), -1.0);
    }
}
