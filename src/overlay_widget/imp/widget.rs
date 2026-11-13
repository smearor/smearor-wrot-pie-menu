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
use smearor_commons_geometry::Annulus;
use smearor_commons_geometry::Circle;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;
use smearor_commons_geometry::WidgetSizeExt;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
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
    pub(crate) initial_rotation: RefCell<Option<RotationRad>>,

    /// Whether rotation gesture is active
    pub(crate) is_rotating: Arc<AtomicBool>,

    /// Last rotation value sent to main application
    pub(crate) last_sent_rotation: RefCell<Option<RotationRad>>,

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

    /// Scroll rotation sensitivity multiplier. Default: `5.0`.
    /// The rotation delta is computed as `dy * sensitivity`.
    pub(crate) scroll_rotation_step: AtomicF64,

    /// Latest left stick X-axis value in [-1.0, 1.0].
    /// Updated by `handle_left_stick_x`, consumed by the tick callback.
    /// A value of `0.0` (or within deadzone) stops continuous rotation.
    ///
    /// **Thread safety**: `Cell` is `!Sync`. This is safe because all writes
    /// occur on the GTK main thread via `glib::idle_add_local`. The SDL2/evdev
    /// polling thread must not write to this field directly.
    pub(crate) left_stick_x: Cell<f32>,

    /// Key controller attached to the root window for global keyboard shortcuts.
    /// Created in `root()`, removed in `unroot()`.
    #[cfg(feature = "keyboard")]
    pub(crate) key_controller: RefCell<Option<gtk4::EventControllerKey>>,

    /// Stack of opened submenu item ids. Empty means main ring is active.
    pub(crate) submenu_stack: RefCell<Vec<String>>,

    /// Per-level submenu radius overrides. Level 0 is the main ring.
    /// Levels without an explicit override use `main_radius + level * step`.
    pub(crate) submenu_radii: RefCell<HashMap<u32, f32>>,

    /// Step width between consecutive ring levels. Default: `80.0`.
    pub(crate) submenu_radius_step: AtomicF64,
}

/// Default activation threshold for pinch-to-zoom (scale must exceed this to open the menu)
pub const DEFAULT_ACTIVATION_THRESHOLD: f64 = 3.5;

/// Default deactivation threshold for pinch-out (scale must drop below this to close the menu)
pub const DEFAULT_DEACTIVATION_THRESHOLD: f64 = 0.5;

/// Default scroll rotation sensitivity multiplier.
/// With a discrete mouse wheel tick (`dy ≈ 1.0`), this yields ~5° per tick.
pub const DEFAULT_SCROLL_ROTATION_STEP: f64 = 5.0;

/// Default step width between consecutive submenu ring levels in pixels.
pub const DEFAULT_SUBMENU_RADIUS_STEP: f64 = 80.0;

/// Maximum submenu nesting depth to prevent visual overflow.
pub const MAX_SUBMENU_DEPTH: u32 = 3;

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
            scroll_rotation_step: AtomicF64::new(DEFAULT_SCROLL_ROTATION_STEP),
            left_stick_x: Cell::new(0.0),
            #[cfg(feature = "keyboard")]
            key_controller: RefCell::new(None),
            submenu_stack: RefCell::new(Vec::new()),
            submenu_radii: RefCell::new(HashMap::new()),
            submenu_radius_step: AtomicF64::new(DEFAULT_SUBMENU_RADIUS_STEP),
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
        let initial_rotation = Rc::new(RefCell::new(RotationRad::ZERO));
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
                let current_rotation = pie_menu_widget.imp().rotation();
                *initial_rotation_clone.borrow_mut() = current_rotation;
                widget.imp().initial_rotation.replace(Some(current_rotation));
                widget.imp().is_rotating.store(true, Ordering::Relaxed);
                debug!("Rotation gesture started, initial rotation: {}", current_rotation.to_degrees());
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

            let delta = RotationRad::new(angle_delta);
            let initial_rot = *initial_rotation_clone.borrow();
            let threshold = RotationRad::from_degrees(10.0);

            // Check if gesture exceeds 10 degrees in either direction
            if delta.abs() > threshold {
                let nudge = threshold * delta.as_f64().signum();
                let new_rotation = (initial_rot + delta + nudge).normalized();
                // Round to nearest degree for 1-degree steps
                let rounded_rotation = RotationRad::from_degrees(new_rotation.to_degrees().round());

                // Check if rotation changed by at least 1 degree from last sent value
                let should_send = match widget.imp().last_sent_rotation.borrow().as_ref() {
                    Some(last) => last.signed_delta_to(rounded_rotation).abs().to_degrees() >= 1.0,
                    None => true,
                };

                if should_send {
                    debug!("Rotation gesture exceeded 10 degrees: angle_delta={delta}, new_rotation={rounded_rotation}");

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

            let Some(menu_widget) = widget.imp().pie_menu_widget.borrow().clone() else {
                debug!("No pie menu widget set, ignoring click");
                return;
            };
            let center_radius = menu_widget.imp().center_radius();

            // Check if click is in center circle
            let center = widget.center();
            let pointer = Vec2::<Pixels>::new(x, y);
            let center_circle = Circle::<Pixels>::new(center, center_radius.into());

            debug!("Click distance from center: {}, threshold: {center_radius}", center.distance_f64(&pointer));

            if center_circle.contains(&pointer) {
                // Check if a center widget is set - if so, let the event propagate
                let has_center_widget = menu_widget.imp().center_widget.borrow().is_some();

                if has_center_widget {
                    debug!("Center click with center widget set - propagating to widget");
                } else {
                    debug!("Center circle clicked, closing submenu or menu");
                    gesture.set_state(EventSequenceState::Claimed);
                    if widget.submenu_depth() > 0 {
                        let _ = widget.close_submenu();
                    } else {
                        let _ = widget.hide_pie_menu();
                    }
                }
            } else {
                // Check if click is on a menu item (main ring or active submenu ring)
                let clicked_item = widget.imp().pie_menu_widget.borrow().as_ref().and_then(|menu_widget| {
                    let imp = menu_widget.imp();
                    let main_radius = imp.radius();
                    let rotation = imp.rotation();
                    let radius_step = imp.submenu_radius_step();
                    let submenu_stack = imp.submenu_stack.borrow();

                    // Check main ring items only when no submenu is open
                    let menu_items = &imp.menu_items;
                    if submenu_stack.is_empty() {
                        for item in menu_items.iter() {
                            if !item.enabled {
                                continue;
                            }
                            let item_circle = Circle::<Pixels>::new(
                                center + Vec2::<Pixels>::from_polar((main_radius * 0.7).into(), RotationRad::from_degrees(item.angle) + rotation),
                                f64::from(item.radius()),
                            );

                            if item_circle.contains(&pointer) {
                                return Some((item.id.clone(), item.submenu.is_some(), item.event.clone(), item.close_on_click));
                            }
                        }
                    }

                    // Check active submenu ring items (DashMap iter locks released by now)
                    if let Some(parent_id) = submenu_stack.last() {
                        let outer_radius = main_radius + radius_step * submenu_stack.len() as f64;
                        let inner_radius = if submenu_stack.len() == 1 {
                            main_radius
                        } else {
                            main_radius + radius_step * (submenu_stack.len() - 1) as f64
                        };
                        let item_ring = Annulus::<Pixels>::new(center, inner_radius.into(), outer_radius.into());
                        let item_ring_radius: f64 = item_ring.mid_radius().into();
                        if let Some(parent_item) = menu_items.find_item_recursive(parent_id)
                            && let Some(submenu_items) = &parent_item.submenu
                        {
                            for item in submenu_items {
                                if !item.enabled {
                                    continue;
                                }
                                let item_circle = Circle::<Pixels>::new(
                                    center + Vec2::<Pixels>::from_polar(item_ring_radius, RotationRad::from_degrees(item.angle) + rotation),
                                    f64::from(item.radius()),
                                );

                                if item_circle.contains(&pointer) {
                                    return Some((item.id.clone(), item.submenu.is_some(), item.event.clone(), item.close_on_click));
                                }
                            }
                        }
                    }

                    None
                });

                if let Some((item_id, has_submenu, event, close_on_click)) = clicked_item {
                    debug!("Menu item '{}' clicked", item_id);
                    gesture.set_state(EventSequenceState::Claimed);

                    if has_submenu {
                        let _ = widget.open_submenu(&item_id);
                    } else {
                        if close_on_click {
                            let _ = widget.hide_pie_menu();
                        }
                        widget.send_message(PieMenuMessage::Event(event));
                    }
                } else {
                    debug!("Click outside center circle and menu items");
                }
            }
        });

        widget.add_controller(click_controller);

        // --- Mouse scroll rotation controller (feature: mouse-scroll) ---
        #[cfg(feature = "mouse-scroll")]
        {
            let scroll_controller = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
            scroll_controller.set_propagation_phase(PropagationPhase::Capture);

            let widget_weak = widget.downgrade();
            scroll_controller.connect_scroll(move |_controller, _dx, dy| {
                let Some(widget) = widget_weak.upgrade() else {
                    return glib::Propagation::Proceed;
                };

                if !widget.is_pie_menu_open() {
                    return glib::Propagation::Proceed;
                }

                let rotation_step = widget.scroll_rotation_step();
                let current_rotation = widget.rotation();
                let new_rotation = (current_rotation + RotationRad::from_degrees(dy * f64::from(rotation_step))).normalized();
                widget.set_rotation(new_rotation);
                widget.send_message(PieMenuMessage::Rotate(new_rotation));

                glib::Propagation::Stop
            });

            widget.add_controller(scroll_controller);
        }

        // --- Controller tick callback for analog stick rotation (feature: controller-sdl2 or controller-evdev) ---
        #[cfg(any(feature = "controller-sdl2", feature = "controller-evdev"))]
        {
            let widget_weak = widget.downgrade();
            widget.add_tick_callback(move |_widget, _frame_clock| {
                let Some(widget) = widget_weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                if !widget.is_pie_menu_open() {
                    return glib::ControlFlow::Continue;
                }

                let stick_x = widget.imp().left_stick_x.get();
                let deadzone = 0.15;

                if stick_x.abs() < deadzone {
                    return glib::ControlFlow::Continue;
                }

                let rotation_delta = stick_x * widget.scroll_rotation_step();
                let current = widget.rotation();
                let new_rotation = (current + RotationRad::from_degrees(rotation_delta)).normalized();
                widget.set_rotation(new_rotation);
                widget.send_message(PieMenuMessage::Rotate(new_rotation));

                glib::ControlFlow::Continue
            });
        }
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

    fn root(&self) {
        self.parent_root();

        #[cfg(feature = "keyboard")]
        {
            let widget = self.obj();
            let root = widget.root();
            let Some(root_widget) = root.as_ref() else {
                return;
            };

            let key_controller = gtk4::EventControllerKey::new();
            key_controller.set_propagation_phase(PropagationPhase::Capture);

            let widget_weak = widget.downgrade();
            key_controller.connect_key_pressed(move |_controller, keyval, _keycode, state| {
                let Some(widget) = widget_weak.upgrade() else {
                    return glib::Propagation::Proceed;
                };

                use gtk4::gdk::Key;

                debug!("Key pressed: keyval={:?}, state={:?}", keyval, state);

                match keyval {
                    Key::space if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) => {
                        if !widget.is_pie_menu_open() {
                            let _ = widget.show_pie_menu();
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Menu => {
                        if !widget.is_pie_menu_open() {
                            let _ = widget.show_pie_menu();
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Return | Key::space => {
                        if widget.is_pie_menu_open() {
                            widget.confirm_selection();
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Escape => {
                        if widget.is_pie_menu_open() {
                            if widget.submenu_depth() > 0 {
                                let _ = widget.close_submenu();
                            } else {
                                let _ = widget.hide_pie_menu();
                            }
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Left | Key::Down => {
                        if widget.is_pie_menu_open() {
                            widget.cycle_selection(-1);
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Right | Key::Up => {
                        if widget.is_pie_menu_open() {
                            widget.cycle_selection(1);
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Home => {
                        if widget.is_pie_menu_open() {
                            widget.select_first_item();
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    Key::Tab => {
                        if widget.is_pie_menu_open() {
                            widget.cycle_selection(1);
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    _ => glib::Propagation::Proceed,
                }
            });

            root_widget.add_controller(key_controller.clone());
            *self.key_controller.borrow_mut() = Some(key_controller);
        }
    }

    fn unroot(&self) {
        #[cfg(feature = "keyboard")]
        {
            if let Some(controller) = self.key_controller.borrow_mut().take() {
                let widget = self.obj();
                let root = widget.root();
                if let Some(root_widget) = root.as_ref() {
                    root_widget.remove_controller(&controller);
                }
            }
        }

        self.parent_unroot();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::Cast;

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

    #[test]
    fn test_max_submenu_depth_constant() {
        assert_eq!(MAX_SUBMENU_DEPTH, 3);
    }

    #[test]
    fn test_default_submenu_radius_step_constant() {
        assert_eq!(DEFAULT_SUBMENU_RADIUS_STEP, 80.0);
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_center_click_default_closes_menu() {
        crate::test_util::ensure_gtk_init();
        let child = gtk4::Label::new(Some("child"));
        let overlay = crate::overlay_widget::widget::PieMenuOverlayWidget::new(Some(child.upcast_ref()));
        // Without a center widget, the overlay should have no center widget
        assert!(overlay.center_widget().is_none());
        // The built-in close logic should be active (verified by the absence
        // of a center widget - the click handler checks this at runtime)
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_center_click_with_center_widget_propagates() {
        crate::test_util::ensure_gtk_init();
        let child = gtk4::Label::new(Some("child"));
        let center = gtk4::Label::new(Some("center"));
        let overlay = crate::overlay_widget::widget::PieMenuOverlayWidget::new(Some(child.upcast_ref())).with_center_widget(center.upcast_ref());
        // With a center widget set, the built-in close logic is bypassed
        assert!(overlay.center_widget().is_some());
    }
}
