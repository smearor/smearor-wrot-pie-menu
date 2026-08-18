use crate::MenuItem;
use crate::PieMenuOverlayWidget;
use crate::menu_widget::menu_item::submenu_error::SubmenuError;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::imp::widget::DEFAULT_ACTIVATION_THRESHOLD;
use crate::overlay_widget::imp::widget::DEFAULT_DEACTIVATION_THRESHOLD;
use crate::overlay_widget::imp::widget::MAX_SUBMENU_DEPTH;
use glib::subclass::prelude::ObjectSubclassIsExt;

pub trait PieMenuControlHandler {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError>;

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError>;

    fn is_pie_menu_open(&self) -> bool;

    /// Sets the pinch-to-zoom activation threshold.
    /// The pie menu opens when the zoom scale exceeds this value.
    /// Default: `3.5`.
    fn set_activation_threshold(&self, threshold: f64);

    /// Returns the current activation threshold.
    fn activation_threshold(&self) -> f64;

    /// Sets the pinch-out deactivation threshold.
    /// The pie menu closes when the zoom scale drops below this value.
    /// Default: `0.5`.
    fn set_deactivation_threshold(&self, threshold: f64);

    /// Returns the current deactivation threshold.
    fn deactivation_threshold(&self) -> f64;

    /// Enables or disables the rotation gesture when the pie menu is open.
    /// When disabled, the rotation gesture controller is set to `PropagationPhase::None`.
    /// Default: `true`.
    fn set_rotation_gesture_enabled(&self, enabled: bool);

    /// Returns whether the rotation gesture is currently enabled.
    fn rotation_gesture_enabled(&self) -> bool;

    /// Enables or disables drawing of inner and outer ring markings.
    /// Default: `true`.
    fn set_markings_enabled(&self, enabled: bool);

    /// Returns whether ring markings are currently enabled.
    fn markings_enabled(&self) -> bool;

    /// Sets the scroll rotation sensitivity multiplier.
    /// The rotation delta is computed as `dy * sensitivity`, so higher
    /// values produce faster rotation. Default: `5.0`.
    fn set_scroll_rotation_step(&self, sensitivity: f64);

    /// Returns the current scroll rotation sensitivity multiplier.
    fn scroll_rotation_step(&self) -> f32;

    /// Cycles the keyboard selection by `direction` (-1 for CCW, +1 for CW).
    /// Items are sorted by angle to ensure deterministic navigation order.
    fn cycle_selection(&self, direction: i32);

    /// Selects the first item (smallest angle, typically 0°).
    fn select_first_item(&self);

    /// Confirms the current keyboard selection by sending `PieMenuMessage::Event`
    /// for the selected item. Does nothing if no item is selected or if the
    /// selected item is disabled.
    fn confirm_selection(&self);

    /// Updates the stored left stick X-axis value for continuous rotation.
    /// The actual rotation is applied per-frame via the tick callback.
    /// Must be called on the GTK main thread.
    /// `x` is in the range [-1.0, 1.0].
    fn handle_left_stick_x(&self, x: f32);

    /// Selects the nearest menu item based on the right stick direction.
    /// `x` and `y` are in the range [-1.0, 1.0].
    /// Must be called on the GTK main thread.
    fn handle_right_stick(&self, x: f32, y: f32);

    /// Finds the menu item whose angle is closest to `target_angle` (in degrees).
    /// Returns the item ID, or `None` if the menu is empty.
    fn find_nearest_item(&self, target_angle: f32) -> Option<String>;

    /// Sets the keyboard selection to the given item ID and triggers a redraw.
    fn set_keyboard_selection(&self, id: String);

    /// Opens the submenu of the item with the given id.
    /// Submenu item angles are redistributed (fixed items keep their
    /// angles, flexible items are auto-distributed) before rendering.
    fn open_submenu(&self, parent_id: &str) -> Result<(), SubmenuError>;

    /// Closes the current submenu and returns to the parent ring.
    fn close_submenu(&self) -> Result<(), SubmenuError>;

    /// Returns the current submenu depth (0 = main ring).
    fn submenu_depth(&self) -> u32;

    /// Returns the submenu items of the item with the given parent id.
    fn get_submenu_items(&self, parent_id: &str) -> Vec<MenuItem>;

    /// Redistributes submenu item angles for the submenu of the
    /// item with the given parent id. Fixed items keep their angles;
    /// flexible items are re-spaced proportionally in the gaps.
    fn redistribute_submenu(&self, parent_id: &str);

    /// Updates the submenu items of the item with the given parent id.
    /// Replaces the entire submenu item list. Triggers redistribution
    /// and overlap validation.
    fn set_submenu_items(&self, parent_id: &str, items: Vec<MenuItem>) -> Result<(), SubmenuError>;

    /// Sets the radius for a specific submenu level.
    /// Level 0 is the main ring, level 1 is the first submenu, etc.
    fn set_submenu_radius(&self, level: u32, radius: f32);

    /// Sets the global step width between consecutive ring levels.
    /// Each submenu level's radius is computed as:
    /// `main_radius + level * step`. Default: `80.0`.
    fn set_submenu_radius_step(&self, step: f32);

    /// Returns the maximum submenu nesting depth.
    fn max_submenu_depth(&self) -> u32;
}

impl PieMenuControlHandler for PieMenuOverlayWidget {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError> {
        self.imp().show_pie_menu()
    }

    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError> {
        self.imp().hide_pie_menu()
    }

    fn is_pie_menu_open(&self) -> bool {
        self.imp().is_pie_menu_open()
    }

    fn set_activation_threshold(&self, threshold: f64) {
        self.imp().set_activation_threshold(threshold);
    }

    fn activation_threshold(&self) -> f64 {
        self.imp().activation_threshold()
    }

    fn set_deactivation_threshold(&self, threshold: f64) {
        self.imp().set_deactivation_threshold(threshold);
    }

    fn deactivation_threshold(&self) -> f64 {
        self.imp().deactivation_threshold()
    }

    fn set_rotation_gesture_enabled(&self, enabled: bool) {
        self.imp().set_rotation_gesture_enabled(enabled);
    }

    fn rotation_gesture_enabled(&self) -> bool {
        self.imp().rotation_gesture_enabled()
    }

    fn set_markings_enabled(&self, enabled: bool) {
        self.imp().set_markings_enabled(enabled);
    }

    fn markings_enabled(&self) -> bool {
        self.imp().markings_enabled()
    }

    fn set_scroll_rotation_step(&self, sensitivity: f64) {
        self.imp().set_scroll_rotation_step(sensitivity);
    }

    fn scroll_rotation_step(&self) -> f32 {
        self.imp().scroll_rotation_step()
    }

    fn cycle_selection(&self, direction: i32) {
        self.imp().cycle_selection(direction);
    }

    fn select_first_item(&self) {
        self.imp().select_first_item();
    }

    fn confirm_selection(&self) {
        self.imp().confirm_selection();
    }

    fn handle_left_stick_x(&self, x: f32) {
        self.imp().handle_left_stick_x(x);
    }

    fn handle_right_stick(&self, x: f32, y: f32) {
        self.imp().handle_right_stick(x, y);
    }

    fn find_nearest_item(&self, target_angle: f32) -> Option<String> {
        self.imp().find_nearest_item(target_angle)
    }

    fn set_keyboard_selection(&self, id: String) {
        self.imp().set_keyboard_selection(id);
    }

    fn open_submenu(&self, parent_id: &str) -> Result<(), SubmenuError> {
        self.imp().open_submenu(parent_id)
    }

    fn close_submenu(&self) -> Result<(), SubmenuError> {
        self.imp().close_submenu()
    }

    fn submenu_depth(&self) -> u32 {
        self.imp().submenu_depth()
    }

    fn get_submenu_items(&self, parent_id: &str) -> Vec<MenuItem> {
        self.imp().get_submenu_items(parent_id)
    }

    fn redistribute_submenu(&self, parent_id: &str) {
        self.imp().redistribute_submenu(parent_id);
    }

    fn set_submenu_items(&self, parent_id: &str, items: Vec<MenuItem>) -> Result<(), SubmenuError> {
        self.imp().set_submenu_items(parent_id, items)
    }

    fn set_submenu_radius(&self, level: u32, radius: f32) {
        self.imp().set_submenu_radius(level, radius);
    }

    fn set_submenu_radius_step(&self, step: f32) {
        self.imp().set_submenu_radius_step(step);
    }

    fn max_submenu_depth(&self) -> u32 {
        MAX_SUBMENU_DEPTH
    }
}

/// Returns the default activation threshold.
pub fn default_activation_threshold() -> f64 {
    DEFAULT_ACTIVATION_THRESHOLD
}

/// Returns the default deactivation threshold.
pub fn default_deactivation_threshold() -> f64 {
    DEFAULT_DEACTIVATION_THRESHOLD
}
