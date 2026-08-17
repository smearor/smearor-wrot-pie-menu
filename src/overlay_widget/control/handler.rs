use crate::PieMenuOverlayWidget;
use crate::overlay_widget::control::error::HidePieMenuError;
use crate::overlay_widget::control::error::ShowPieMenuError;
use crate::overlay_widget::imp::widget::DEFAULT_ACTIVATION_THRESHOLD;
use crate::overlay_widget::imp::widget::DEFAULT_DEACTIVATION_THRESHOLD;
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
}

/// Returns the default activation threshold.
pub fn default_activation_threshold() -> f64 {
    DEFAULT_ACTIVATION_THRESHOLD
}

/// Returns the default deactivation threshold.
pub fn default_deactivation_threshold() -> f64 {
    DEFAULT_DEACTIVATION_THRESHOLD
}
