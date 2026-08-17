use crate::PieMenuMessage;
use crate::PieMenuOverlayWidget;
use crate::PieMenuWidget;
use crate::RotationHandler;
use crate::overlay_widget::control::handler::PieMenuControlHandler;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use atomic_float::AtomicF64;
use glib::Propagation;
use glib::subclass::prelude::*;
use gtk4::BinLayout;
use gtk4::EventController;
use gtk4::EventControllerLegacy;
use gtk4::EventSequenceState;
use gtk4::GestureRotate;
use gtk4::GestureZoom;
use gtk4::Overlay;
use gtk4::PropagationPhase;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use gtk4::subclass::widget::WidgetImplExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use tracing::debug;
use tracing::info;

pub struct PieMenuOverlayWidgetImpl {
    /// Whether the pie menu is visible.
    /// This is used to determine to draw the pie menu or not.
    /// If this is true, the pie menu will be drawn.
    /// If this is false, the child widget will be drawn.
    pub(crate) visible: Arc<AtomicBool>,

    pub(crate) overlay: Overlay,

    pub(crate) pie_menu_widget: RefCell<Option<PieMenuWidget>>,

    pub(crate) is_zooming: Arc<AtomicBool>,

    /// Message sender for communicating with main application
    pub(crate) message_sender: RefCell<Option<Sender<PieMenuMessage>>>,

    /// Initial rotation when rotation gesture starts
    pub(crate) initial_rotation: RefCell<Option<f32>>,

    /// Whether rotation gesture is active
    pub(crate) is_rotating: Arc<AtomicBool>,

    /// Last rotation value sent to main application
    pub(crate) last_sent_rotation: RefCell<Option<f32>>,

    /// Pinch-to-zoom scale threshold above which the pie menu opens.
    /// Configurable via `set_activation_threshold()`. Default: `3.5`.
    pub(crate) activation_threshold: AtomicF64,

    /// Pinch-out scale threshold below which the pie menu closes.
    /// Configurable via `set_deactivation_threshold()`. Default: `0.5`.
    pub(crate) deactivation_threshold: AtomicF64,

    /// Whether the rotation gesture is active when the pie menu is open. Default: `true`.
    pub(crate) rotation_gesture_enabled: AtomicBool,

    /// Reference to the rotate gesture controller for propagation phase toggling.
    pub(crate) rotate_gesture: RefCell<Option<GestureRotate>>,
}

/// Default activation threshold for pinch-to-zoom (scale must exceed this to open the menu)
pub const DEFAULT_ACTIVATION_THRESHOLD: f64 = 3.5;

/// Default deactivation threshold for pinch-out (scale must drop below this to close the menu)
pub const DEFAULT_DEACTIVATION_THRESHOLD: f64 = 0.5;

impl Default for PieMenuOverlayWidgetImpl {
    fn default() -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(false)),
            overlay: Overlay::new(),
            pie_menu_widget: RefCell::new(None),
            is_zooming: Arc::new(AtomicBool::new(false)),
            message_sender: RefCell::new(None),
            initial_rotation: RefCell::new(None),
            is_rotating: Arc::new(AtomicBool::new(false)),
            last_sent_rotation: RefCell::new(None),
            activation_threshold: AtomicF64::new(DEFAULT_ACTIVATION_THRESHOLD),
            deactivation_threshold: AtomicF64::new(DEFAULT_DEACTIVATION_THRESHOLD),
            rotation_gesture_enabled: AtomicBool::new(true),
            rotate_gesture: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for PieMenuOverlayWidgetImpl {
    const NAME: &'static str = "PieMenuOverlayWidget";
    type Type = PieMenuOverlayWidget;
    type ParentType = gtk4::Widget;
}

impl ObjectImpl for PieMenuOverlayWidgetImpl {
    fn constructed(&self) {
        self.parent_constructed();

        let widget = self.obj();

        widget.set_layout_manager(Some(BinLayout::new()));

        let menu_widget = PieMenuWidget::new();
        menu_widget.set_visible(false);

        // Set close callback to hide pie menu when center circle is clicked
        let widget_weak = widget.downgrade();
        menu_widget.set_close_callback(move || {
            if let Some(widget) = widget_weak.upgrade() {
                let _ = widget.hide_pie_menu();
            }
        });

        let mut menu_widget_borrow = self.pie_menu_widget.borrow_mut();
        menu_widget_borrow.replace(menu_widget.clone());

        self.overlay.add_overlay(&menu_widget);
        self.overlay.set_parent(&*self.obj());

        let event_controller = EventControllerLegacy::new();
        event_controller.set_propagation_phase(PropagationPhase::Capture);

        let widget_weak = widget.downgrade();
        event_controller.connect_event(move |_event_controller, _event| {
            let _ = widget_weak.upgrade();
            Propagation::Proceed
        });

        widget.add_controller(event_controller);

        let zoom_gesture = GestureZoom::new();
        zoom_gesture.set_propagation_phase(PropagationPhase::Capture);

        let widget_weak = widget.downgrade();
        zoom_gesture.connect_scale_changed(move |gesture, scale| {
            debug!("Zoom gesture detected (scale: {:.2})", scale);
            let Some(widget) = widget_weak.upgrade() else {
                return;
            };
            widget.imp().is_zooming.store(true, Ordering::Relaxed);
            let is_open = widget.is_pie_menu_open();
            let activation_threshold = widget.imp().activation_threshold.load(Ordering::Relaxed);
            let deactivation_threshold = widget.imp().deactivation_threshold.load(Ordering::Relaxed);
            if scale > activation_threshold && !is_open {
                info!("Zoom gesture detected (scale: {:.2}), opening pie menu", scale);
                let _ = widget.show_pie_menu();
                gesture.set_state(EventSequenceState::Claimed);
            } else if scale < deactivation_threshold && is_open {
                info!("Zoom gesture detected (scale: {:.2}), closing pie menu", scale);
                let _ = widget.hide_pie_menu();
                gesture.set_state(EventSequenceState::Claimed);
            }
            widget.imp().is_zooming.store(false, Ordering::Relaxed);
        });

        widget.add_controller(zoom_gesture.clone().upcast::<EventController>());

        // Add rotation gesture controller
        let rotate_gesture = GestureRotate::new();
        rotate_gesture.set_propagation_phase(PropagationPhase::Capture);

        let widget_weak = widget.downgrade();
        let initial_rotation = Rc::new(RefCell::new(0.0f32));
        let initial_rotation_clone = initial_rotation.clone();

        rotate_gesture.connect_begin(move |_gesture, _sequence| {
            let Some(widget) = widget_weak.upgrade() else {
                return;
            };
            if !widget.is_pie_menu_open() {
                return;
            }

            // Get current rotation from pie menu widget
            if let Some(pie_menu_widget) = widget.imp().pie_menu_widget.borrow().as_ref() {
                let current_rotation = pie_menu_widget.imp().rotation.load(Ordering::Relaxed);
                *initial_rotation_clone.borrow_mut() = current_rotation;
                widget.imp().initial_rotation.replace(Some(current_rotation));
                widget.imp().is_rotating.store(true, Ordering::Relaxed);
                debug!("Rotation gesture started, initial rotation: {}", current_rotation);
            }
        });

        let widget_weak = widget.downgrade();
        let initial_rotation_clone = initial_rotation.clone();
        rotate_gesture.connect_angle_changed(move |_gesture, _sequence, angle_delta| {
            let Some(widget) = widget_weak.upgrade() else {
                return;
            };
            if !widget.is_pie_menu_open() {
                return;
            }

            let angle_degrees = angle_delta.to_degrees() as f32;
            let initial_rot = *initial_rotation_clone.borrow();

            // Check if gesture exceeds 10 degrees in either direction
            if angle_degrees.abs() > 10.0 {
                let direction = if angle_degrees > 0.0 { 1.0 } else { -1.0 };
                let new_rotation = initial_rot + angle_degrees + (10.0 * direction);
                let new_rotation = new_rotation.rem_euclid(360.0);

                // Round to nearest degree for 1-degree steps
                let rounded_rotation = new_rotation.round();

                // Check if rotation changed by at least 1 degree from last sent value
                let should_send = match widget.imp().last_sent_rotation.borrow().as_ref() {
                    Some(last) => (rounded_rotation - *last).abs() >= 1.0,
                    None => true,
                };

                if should_send {
                    debug!("Rotation gesture exceeded 10 degrees: angle_delta={}, new_rotation={}", angle_degrees, rounded_rotation);

                    // Update pie menu rotation
                    widget.set_rotation(rounded_rotation);
                    widget.queue_draw();

                    // Send rotation message
                    widget.send_message(PieMenuMessage::Rotate(rounded_rotation));
                    widget.imp().last_sent_rotation.replace(Some(rounded_rotation));
                }
            }
        });

        let widget_weak = widget.downgrade();
        rotate_gesture.connect_end(move |_gesture, _sequence| {
            let Some(widget) = widget_weak.upgrade() else {
                return;
            };
            widget.imp().is_rotating.store(false, Ordering::Relaxed);
            widget.imp().initial_rotation.replace(None);
            widget.imp().last_sent_rotation.replace(None);
            debug!("Rotation gesture ended");
        });

        widget.add_controller(rotate_gesture.clone().upcast::<EventController>());
        *self.rotate_gesture.borrow_mut() = Some(rotate_gesture);

        // Add click controller for center circle to close menu
        let click_controller = gtk4::GestureClick::new();
        click_controller.set_button(0);
        click_controller.set_propagation_phase(PropagationPhase::Bubble);

        let widget_weak = widget.downgrade();
        click_controller.connect_pressed(move |gesture, _n_press, x, y| {
            debug!("Overlay click pressed at ({}, {})", x, y);
            let Some(widget) = widget_weak.upgrade() else {
                debug!("Widget upgrade failed");
                return;
            };
            if !widget.is_pie_menu_open() {
                debug!("Pie menu not open, ignoring click");
                return;
            }

            let width = widget.width() as f32;
            let height = widget.height() as f32;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let center_radius = widget
                .imp()
                .pie_menu_widget
                .borrow()
                .as_ref()
                .unwrap()
                .imp()
                .center_radius
                .load(Ordering::Relaxed);

            // Check if click is in center circle
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            debug!("Click distance from center: {distance}, threshold: {center_radius}");

            if distance <= center_radius {
                debug!("Center circle clicked, closing menu");
                gesture.set_state(EventSequenceState::Claimed);
                let _ = widget.hide_pie_menu();
            } else {
                // Check if click is on a menu item
                if let Some(menu_widget) = widget.imp().pie_menu_widget.borrow().as_ref() {
                    let radius = menu_widget.imp().radius.load(Ordering::Relaxed);
                    let rotation = menu_widget.imp().rotation.load(Ordering::Relaxed);
                    let menu_items = &menu_widget.imp().menu_items;

                    for item in menu_items.iter() {
                        if !item.enabled {
                            continue;
                        }
                        let angle_rad = (item.angle + rotation).to_radians();
                        let item_x = center_x + (radius * 0.7) * angle_rad.cos();
                        let item_y = center_y + (radius * 0.7) * angle_rad.sin();

                        let item_dx = x as f32 - item_x;
                        let item_dy = y as f32 - item_y;
                        let item_distance = (item_dx * item_dx + item_dy * item_dy).sqrt();
                        let item_radius = item.radius();

                        if item_distance <= item_radius {
                            debug!("Menu item '{}' clicked, sending event: {}", item.id, item.event);
                            gesture.set_state(EventSequenceState::Claimed);
                            let _ = widget.hide_pie_menu();

                            // Send event message
                            widget.send_message(PieMenuMessage::Event(item.event.clone()));
                            break;
                        }
                    }
                }

                debug!("Click outside center circle and menu items");
            }
        });

        widget.add_controller(click_controller);
    }

    fn dispose(&self) {
        self.overlay.unparent();
    }
}

impl WidgetImpl for PieMenuOverlayWidgetImpl {
    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        self.parent_snapshot(snapshot);
    }

    fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
        self.parent_measure(orientation, for_size)
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);
        self.overlay.allocate(width, height, baseline, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_activation_threshold() {
        let threshold = AtomicF64::new(DEFAULT_ACTIVATION_THRESHOLD);
        assert_eq!(threshold.load(Ordering::Relaxed), 3.5);
    }

    #[test]
    fn test_default_deactivation_threshold() {
        let threshold = AtomicF64::new(DEFAULT_DEACTIVATION_THRESHOLD);
        assert_eq!(threshold.load(Ordering::Relaxed), 0.5);
    }

    #[test]
    fn test_set_activation_threshold() {
        let threshold = AtomicF64::new(DEFAULT_ACTIVATION_THRESHOLD);
        threshold.store(2.5, Ordering::Relaxed);
        assert_eq!(threshold.load(Ordering::Relaxed), 2.5);
    }

    #[test]
    fn test_set_deactivation_threshold() {
        let threshold = AtomicF64::new(DEFAULT_DEACTIVATION_THRESHOLD);
        threshold.store(0.3, Ordering::Relaxed);
        assert_eq!(threshold.load(Ordering::Relaxed), 0.3);
    }

    #[test]
    fn test_default_rotation_gesture_enabled() {
        let enabled = AtomicBool::new(true);
        assert!(enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_set_rotation_gesture_enabled() {
        let enabled = AtomicBool::new(true);
        enabled.store(false, Ordering::Relaxed);
        assert!(!enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_default_markings_enabled() {
        let enabled = AtomicBool::new(true);
        assert!(enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_set_markings_enabled() {
        let enabled = AtomicBool::new(true);
        enabled.store(false, Ordering::Relaxed);
        assert!(!enabled.load(Ordering::Relaxed));
    }
}
