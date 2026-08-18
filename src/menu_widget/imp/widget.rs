use crate::menu::menu::Menu;
use crate::menu::widget_registry::MenuItemWidgetRegistry;
use crate::menu_widget::widget::PieMenuWidget;
use atomic_float::AtomicF32;
use glib::subclass::prelude::*;
use gtk4::EventControllerMotion;
use gtk4::Widget;
use gtk4::gdk::RGBA;
use gtk4::glib;
use gtk4::graphene::Point;
use gtk4::graphene::Rect;
use gtk4::gsk::RoundedRect;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use gtk4::subclass::widget::WidgetImplExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tracing::debug;

pub const DEFAULT_PIE_MENU_RADIUS: f32 = 160.0;
pub const DEFAULT_PIE_MENU_CENTER_RADIUS: f32 = 64.0;

pub struct PieMenuWidgetImpl {
    /// The menu items to be displayed in the pie menu.
    pub(crate) menu_items: Arc<Menu>,

    /// The radius of the pie menu.
    pub(crate) rotation: AtomicF32,

    /// The radius of the pie menu.
    pub(crate) radius: AtomicF32,

    /// The radius of the pie menu.
    pub(crate) center_radius: AtomicF32,

    /// Callback to invoke when the center circle is clicked to close the menu.
    pub(crate) close_callback: RefCell<Option<Box<dyn Fn() + 'static>>>,

    /// Index of the currently hovered menu item (-1 if none).
    pub(crate) hovered_item_index: RefCell<i32>,

    /// Whether inner and outer ring markings are drawn. Default: `true`.
    pub(crate) markings_enabled: AtomicBool,

    /// ID of the currently keyboard-selected item, if any.
    /// Stored as a string ID (not an index) to remain stable across
    /// DashMap insertion/removal order changes.
    pub(crate) keyboard_selection: RefCell<Option<String>>,

    /// Stack of opened submenu item ids. Empty means main ring is active.
    /// Synced from `PieMenuOverlayWidgetImpl` via `set_submenu_stack`.
    pub(crate) submenu_stack: RefCell<Vec<String>>,

    /// Step width between consecutive ring levels in pixels.
    pub(crate) submenu_radius_step: AtomicF32,

    /// Widget registry resolving type names to factories.
    /// Pre-populated with standard implementations (`"circle"`, `"square"`).
    pub(crate) widget_registry: RefCell<MenuItemWidgetRegistry>,

    /// Cached item widgets keyed by item ID.
    /// Widgets are built once, registered as children of `PieMenuWidget`
    /// via `set_parent`, and positioned during the GTK4 layout phase
    /// in `WidgetImpl::size_allocate`.
    pub(crate) item_widgets: RefCell<HashMap<String, gtk4::Widget>>,

    /// Optional center widget rendered inside the ring's transparent center.
    /// Rotates with the ring. When set, the consumer is responsible for
    /// close-menu / close-submenu event handling.
    pub(crate) center_widget: RefCell<Option<Widget>>,
}

impl Default for PieMenuWidgetImpl {
    fn default() -> Self {
        Self {
            menu_items: Arc::new(Menu::new()),
            rotation: AtomicF32::new(0.0),
            radius: AtomicF32::new(DEFAULT_PIE_MENU_RADIUS),
            center_radius: AtomicF32::new(DEFAULT_PIE_MENU_CENTER_RADIUS),
            close_callback: RefCell::new(None),
            hovered_item_index: RefCell::new(-1),
            markings_enabled: AtomicBool::new(true),
            keyboard_selection: RefCell::new(None),
            submenu_stack: RefCell::new(Vec::new()),
            submenu_radius_step: AtomicF32::new(80.0),
            widget_registry: RefCell::new(MenuItemWidgetRegistry::new()),
            item_widgets: RefCell::new(HashMap::new()),
            center_widget: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for PieMenuWidgetImpl {
    const NAME: &'static str = "PieMenuWidget";
    type Type = PieMenuWidget;
    type ParentType = gtk4::Widget;
}
impl ObjectImpl for PieMenuWidgetImpl {
    fn constructed(&self) {
        self.parent_constructed();
        let widget = self.obj();

        // No layout manager — child widgets are positioned manually in
        // `size_allocate` based on their ring position and rotation.

        // Add motion controller for mouse hover detection
        let motion_controller = EventControllerMotion::new();
        motion_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let widget_weak = widget.downgrade();
        let widget_weak_for_leave = widget_weak.clone();
        motion_controller.connect_motion(move |_controller, x, y| {
            let Some(widget) = widget_weak.upgrade() else {
                return;
            };
            let imp = widget.imp();
            let radius = imp.radius.load(Ordering::Relaxed) as f64;
            let center_radius = imp.center_radius.load(Ordering::Relaxed) as f64;
            let rotation = imp.rotation.load(Ordering::Relaxed) as f64;
            let radius_step = imp.submenu_radius_step.load(Ordering::Relaxed) as f64;

            // Calculate distance from center (center is middle of widget)
            let widget_obj = imp.obj();
            let width = widget_obj.width() as f64;
            let height = widget_obj.height() as f64;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let dx = x - center_x;
            let dy = y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Determine the active ring bounds
            let submenu_stack = imp.submenu_stack.borrow();
            let submenu_depth = submenu_stack.len();
            let (ring_inner, ring_outer) = if submenu_depth == 0 {
                (center_radius, radius)
            } else {
                let outer = radius + submenu_depth as f64 * radius_step;
                let inner = if submenu_depth == 1 {
                    radius
                } else {
                    radius + (submenu_depth - 1) as f64 * radius_step
                };
                (inner, outer)
            };
            drop(submenu_stack);

            // Check if mouse is in the active ring area
            if distance < ring_inner || distance > ring_outer {
                // Outside the active ring, no item hovered
                let mut hovered_index = imp.hovered_item_index.borrow_mut();
                if *hovered_index != -1 {
                    *hovered_index = -1;
                    widget_obj.queue_draw();
                }
                return;
            }

            // Calculate angle of mouse position
            let angle_rad = dy.atan2(dx);
            let angle_deg = angle_rad.to_degrees();
            let normalized_angle = (angle_deg - rotation).rem_euclid(360.0);

            // Get the active ring's items
            let menu_items = imp.active_ring_items();
            let num_items = menu_items.len();
            if num_items == 0 {
                return;
            }

            // Find the item with the closest angle to the mouse position
            let mut closest_index = -1i32;
            let mut closest_distance = f64::MAX;
            for (index, item) in menu_items.iter().enumerate() {
                if !item.enabled {
                    continue;
                }
                let item_angle = item.angle as f64;
                let angle_diff = (normalized_angle - item_angle).abs();
                let angle_diff = angle_diff.min(360.0 - angle_diff); // Handle wrap-around
                if angle_diff < closest_distance {
                    closest_distance = angle_diff;
                    closest_index = index as i32;
                }
            }
            let item_index = closest_index;

            let mut hovered_index = imp.hovered_item_index.borrow_mut();
            if *hovered_index != item_index {
                *hovered_index = item_index;
                widget_obj.queue_draw();
            }
        });

        motion_controller.connect_leave(move |_controller| {
            let Some(widget) = widget_weak_for_leave.upgrade() else {
                return;
            };
            let imp = widget.imp();
            let mut hovered_index = imp.hovered_item_index.borrow_mut();
            if *hovered_index != -1 {
                *hovered_index = -1;
                widget.queue_draw();
            }
        });

        widget.add_controller(motion_controller);
    }

    fn dispose(&self) {
        // Clear center_widget RefCell to drop our strong reference,
        // then unparent all children (including the center widget)
        let _ = self.center_widget.borrow_mut().take();
        let widget = self.obj();
        while let Some(child) = widget.first_child() {
            child.unparent();
        }
    }
}

impl WidgetImpl for PieMenuWidgetImpl {
    fn measure(&self, orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let radius = self.radius.load(Ordering::Relaxed);
        let diameter = (radius * 2.0) as i32;

        match orientation {
            gtk4::Orientation::Horizontal => (diameter, diameter, -1, -1),
            gtk4::Orientation::Vertical => (diameter, diameter, -1, -1),
            _ => (diameter, diameter, -1, -1),
        }
    }

    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);

        // Build any missing child widgets before positioning so they
        // exist in the same allocation pass.  Calling this here (rather
        // than only in `snapshot`) avoids a one-frame delay where
        // widgets are created in `snapshot` but not allocated until the
        // next layout pass.
        self.ensure_item_widgets();

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = self.radius.load(Ordering::Relaxed);
        let rotation = self.rotation.load(Ordering::Relaxed);
        let radius_step = self.submenu_radius_step.load(Ordering::Relaxed);

        let item_widgets = self.item_widgets.borrow();
        let menu_items = self.menu_items.clone();

        // Position top-level item widgets on the main ring.
        // When content_rotates is true, widgets are positioned at their
        // un-rotated angle — the rotation transform in `snapshot` handles
        // the visual rotation.  When false, the position includes rotation.
        for item in menu_items.iter() {
            if let Some(child_widget) = item_widgets.get(&item.id) {
                let effective_angle = if item.content_rotates { item.angle } else { item.angle + rotation };
                let angle_rad = effective_angle.to_radians();
                let item_x = center_x + (radius * 0.7) * angle_rad.cos();
                let item_y = center_y + (radius * 0.7) * angle_rad.sin();

                let (w, h) = match &item.content_size {
                    Some(size) => (size.width, size.height),
                    None => {
                        let (min_w, nat_w, _, _) = child_widget.measure(gtk4::Orientation::Horizontal, -1);
                        let (min_h, nat_h, _, _) = child_widget.measure(gtk4::Orientation::Vertical, -1);
                        let w = nat_w.max(min_w).max(item.radius() as i32 * 2);
                        let h = nat_h.max(min_h).max(item.radius() as i32 * 2);
                        (w as f32, h as f32)
                    }
                };

                let allocation = gtk4::Allocation::new((item_x - w / 2.0) as i32, (item_y - h / 2.0) as i32, w as i32, h as i32);
                debug!(
                    "Allocating item '{}' at ({}, {}) size {}x{}",
                    item.id,
                    allocation.x(),
                    allocation.y(),
                    allocation.width(),
                    allocation.height()
                );
                child_widget.size_allocate(&allocation, -1);
            }
        }

        // Position submenu item widgets on their respective rings
        let submenu_stack = self.submenu_stack.borrow();
        for (level, parent_id) in submenu_stack.iter().enumerate() {
            let submenu_radius = radius + (level + 1) as f32 * radius_step;
            let inner_radius = if level == 0 { radius } else { radius + level as f32 * radius_step };
            let item_ring_radius = (inner_radius + submenu_radius) / 2.0;

            if let Some(parent_item) = menu_items.find_item_recursive(parent_id)
                && let Some(submenu_items) = &parent_item.submenu
            {
                for item in submenu_items {
                    if let Some(child_widget) = item_widgets.get(&item.id) {
                        let effective_angle = if item.content_rotates { item.angle } else { item.angle + rotation };
                        let angle_rad = effective_angle.to_radians();
                        let item_x = center_x + item_ring_radius * angle_rad.cos();
                        let item_y = center_y + item_ring_radius * angle_rad.sin();

                        let (w, h) = match &item.content_size {
                            Some(size) => (size.width, size.height),
                            None => {
                                let (min_w, nat_w, _, _) = child_widget.measure(gtk4::Orientation::Horizontal, -1);
                                let (min_h, nat_h, _, _) = child_widget.measure(gtk4::Orientation::Vertical, -1);
                                let w = nat_w.max(min_w).max(item.radius() as i32 * 2);
                                let h = nat_h.max(min_h).max(item.radius() as i32 * 2);
                                (w as f32, h as f32)
                            }
                        };

                        let allocation = gtk4::Allocation::new((item_x - w / 2.0) as i32, (item_y - h / 2.0) as i32, w as i32, h as i32);
                        child_widget.size_allocate(&allocation, -1);
                    }
                }
            }
        }

        // Position center widget (unrotated — rotation is applied in snapshot)
        let center_radius = self.center_radius.load(Ordering::Relaxed);
        if let Some(center) = self.center_widget.borrow().as_ref()
            && center.is_visible()
        {
            let (min_w, nat_w, _, _) = center.measure(gtk4::Orientation::Horizontal, -1);
            let (min_h, nat_h, _, _) = center.measure(gtk4::Orientation::Vertical, -1);
            let max_size = (center_radius * 2.0) as i32;
            let w = nat_w.max(min_w).min(max_size);
            let h = nat_h.max(min_h).min(max_size);
            let allocation = gtk4::Allocation::new((center_x - w as f32 / 2.0) as i32, (center_y - h as f32 / 2.0) as i32, w, h);
            debug!(
                "Allocating center widget at ({}, {}) size {}x{}",
                allocation.x(),
                allocation.y(),
                allocation.width(),
                allocation.height()
            );
            center.size_allocate(&allocation, -1);
        }
    }

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        self.ensure_item_widgets();

        let obj = self.obj();
        let width = obj.width() as f32;
        let height = obj.height() as f32;

        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let radius = self.radius.load(Ordering::Relaxed);
        let rotation = self.rotation.load(Ordering::Relaxed);
        let rotation_rad = rotation.to_radians();
        debug!("Rotation: {} degrees, {} radians", rotation, rotation_rad);

        // Apply rotation to the entire menu
        snapshot.save();
        snapshot.translate(&Point::new(center_x, center_y));
        snapshot.rotate(rotation); // rotation_rad
        snapshot.translate(&Point::new(-center_x, -center_y));

        // Draw shadow for the ring
        let shadow_color = RGBA::new(0.0, 0.0, 0.0, 0.3);
        let shadow_offset = 8.0;
        let shadow_radius = radius + shadow_offset;
        let shadow_rect = Rect::new(center_x - shadow_radius, center_y - shadow_radius, shadow_radius * 2.0, shadow_radius * 2.0);
        let shadow_rounded = RoundedRect::from_rect(shadow_rect, shadow_radius);
        snapshot.push_rounded_clip(&shadow_rounded);
        snapshot.append_color(&shadow_color, &shadow_rect);
        snapshot.pop();
        debug!("Drew shadow at ({}, {}) with radius {}", center_x, center_y, shadow_radius);

        // Draw background circle with transparent center (ring shape)
        let center_radius = self.center_radius.load(Ordering::Relaxed);
        let bg_color = RGBA::new(0.2, 0.2, 0.2, 0.5);

        // Create path with outer circle (clockwise) and inner circle (counter-clockwise) for even-odd fill rule
        let builder = gtk4::gsk::PathBuilder::new();

        // Outer circle - clockwise
        for i in 0..=360 {
            let angle = (i as f32).to_radians();
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            if i == 0 {
                builder.move_to(x, y);
            } else {
                builder.line_to(x, y);
            }
        }
        builder.close();

        // Inner circle - counter-clockwise (creates hole with even-odd fill)
        for i in (0..=360).rev() {
            let angle = (i as f32).to_radians();
            let x = center_x + center_radius * angle.cos();
            let y = center_y + center_radius * angle.sin();
            if i == 360 {
                builder.move_to(x, y);
            } else {
                builder.line_to(x, y);
            }
        }
        builder.close();

        let path = builder.to_path();

        // Draw the ring shape directly using append_fill with EvenOdd rule
        snapshot.append_fill(&path, gtk4::gsk::FillRule::EvenOdd, &bg_color);
        debug!(
            "Drew background ring at ({}, {}) with outer radius {} and inner radius {}",
            center_x, center_y, radius, center_radius
        );

        // Draw markings every 5 degrees on both edges of the ring
        if self.markings_enabled.load(Ordering::Relaxed) {
            let marking_offset = -90.0;
            let marking_color_outer_current_angle = RGBA::new(0.8, 0.8, 0.8, 0.5);
            let marking_color_inner_zero_angle = RGBA::new(0.8, 0.8, 0.8, 0.5);
            let marking_color_highlight_outer_zero_angle = RGBA::new(1.0, 0.6, 0.6, 1.0);
            let marking_color_highlight_inner_current_angle = RGBA::new(0.6, 1.0, 0.6, 1.0);
            let marking_length_outer_current_angle = 5.0;
            let marking_length_inner_zero_angle = 5.0;
            let marking_line_width = 2.0;
            let marking_line_width_outer_current_angle = 6.0;
            let marking_line_width_inner_zero_angle = 4.0;
            let rotation = self.rotation.load(Ordering::Relaxed);
            let nearest_angle = ((rotation / 5.0).round() * 5.0) - marking_offset;
            let nearest_angle = nearest_angle.rem_euclid(360.0) as i32;

            for angle in (0i32..360).step_by(5) {
                let shifted_angle = angle.rem_euclid(360);
                let angle_rad = (angle as f32).to_radians();
                let is_zero_degree = shifted_angle == 90;
                let is_current_angle = shifted_angle == nearest_angle;
                let (outer_color, marking_line_width_outer) = if is_current_angle {
                    (marking_color_highlight_outer_zero_angle, marking_line_width_outer_current_angle)
                } else {
                    (marking_color_outer_current_angle, marking_line_width)
                };
                let (inner_color, marking_line_width_inner) = if is_zero_degree {
                    (marking_color_highlight_inner_current_angle, marking_line_width_inner_zero_angle)
                } else {
                    (marking_color_inner_zero_angle, marking_line_width)
                };

                // Draw outer edge marking
                let outer_inner_radius = radius - marking_length_outer_current_angle;
                let outer_outer_radius = radius;

                let outer_start_x = center_x + outer_inner_radius * angle_rad.cos();
                let outer_start_y = center_y + outer_inner_radius * angle_rad.sin();
                let outer_end_x = center_x + outer_outer_radius * angle_rad.cos();
                let outer_end_y = center_y + outer_outer_radius * angle_rad.sin();

                let builder = gtk4::gsk::PathBuilder::new();
                builder.move_to(outer_start_x, outer_start_y);
                builder.line_to(outer_end_x, outer_end_y);
                let path = builder.to_path();

                let stroke = gtk4::gsk::Stroke::new(marking_line_width_outer);
                snapshot.append_stroke(&path, &stroke, &outer_color);

                // Draw inner edge marking
                let inner_inner_radius = center_radius;
                let inner_outer_radius = center_radius + marking_length_inner_zero_angle;

                let inner_start_x = center_x + inner_inner_radius * angle_rad.cos();
                let inner_start_y = center_y + inner_inner_radius * angle_rad.sin();
                let inner_end_x = center_x + inner_outer_radius * angle_rad.cos();
                let inner_end_y = center_y + inner_outer_radius * angle_rad.sin();

                let builder = gtk4::gsk::PathBuilder::new();
                builder.move_to(inner_start_x, inner_start_y);
                builder.line_to(inner_end_x, inner_end_y);
                let path = builder.to_path();

                let stroke = gtk4::gsk::Stroke::new(marking_line_width_inner);
                snapshot.append_stroke(&path, &stroke, &inner_color);
            }
            debug!("Drew 5-degree markings on both edges of the ring with highlights at 0° and {}°", nearest_angle);
        }

        // All items are rendered as child widgets by the widget
        // registry — no manual drawing needed.
        let item_widgets = self.item_widgets.borrow();

        // Draw submenu rings for each open submenu level
        let submenu_stack = self.submenu_stack.borrow().clone();
        let main_radius = self.radius.load(Ordering::Relaxed);
        let radius_step = self.submenu_radius_step.load(Ordering::Relaxed);
        let stack_depth = self.submenu_depth() as usize;

        for (level, parent_id) in submenu_stack.iter().enumerate() {
            let submenu_radius = main_radius + (level + 1) as f32 * radius_step;
            let is_active_level = level + 1 == stack_depth;
            let ring_opacity = if is_active_level { 0.5 } else { 0.25 };

            // Draw submenu ring outline
            let builder = gtk4::gsk::PathBuilder::new();
            for i in 0..=360 {
                let angle = (i as f32).to_radians();
                let x = center_x + submenu_radius * angle.cos();
                let y = center_y + submenu_radius * angle.sin();
                if i == 0 {
                    builder.move_to(x, y);
                } else {
                    builder.line_to(x, y);
                }
            }
            builder.close();
            let path = builder.to_path();
            let stroke = gtk4::gsk::Stroke::new(2.0);
            let ring_stroke_color = RGBA::new(0.5, 0.5, 0.5, ring_opacity);
            snapshot.append_stroke(&path, &stroke, &ring_stroke_color);

            // Draw parent item indicator on the parent ring
            if let Some(parent_item) = self.menu_items.find_item_recursive(parent_id) {
                let parent_angle_rad = parent_item.angle.to_radians();
                let parent_ring_radius = if level == 0 { main_radius } else { main_radius + level as f32 * radius_step };
                let indicator_x = center_x + parent_ring_radius * 0.7 * parent_angle_rad.cos();
                let indicator_y = center_y + parent_ring_radius * 0.7 * parent_angle_rad.sin();

                let indicator_color = RGBA::new(1.0, 0.8, 0.2, 0.9);
                let indicator_radius = 4.0;
                let indicator_rect = Rect::new(indicator_x - indicator_radius, indicator_y - indicator_radius, indicator_radius * 2.0, indicator_radius * 2.0);
                let indicator_rounded = RoundedRect::from_rect(indicator_rect, indicator_radius);
                snapshot.push_rounded_clip(&indicator_rounded);
                snapshot.append_color(&indicator_color, &indicator_rect);
                snapshot.pop();
            }

            // Draw submenu items on the submenu ring
            // All submenu items are rendered as child widgets by the widget
            // registry — no manual drawing needed.
            if let Some(parent_item) = self.menu_items.find_item_recursive(parent_id)
                && let Some(_submenu_items) = &parent_item.submenu
            {
                // Items rendered by widget registry
            }

            // Draw breadcrumb dots between rings
            let breadcrumb_color = RGBA::new(0.6, 0.6, 0.6, 0.6);
            let breadcrumb_radius = 2.0;
            for i in 0..=level as i32 + 1 {
                let breadcrumb_y = center_y - main_radius - (i as f32 + 0.5) * radius_step;
                let breadcrumb_rect = Rect::new(
                    center_x - breadcrumb_radius,
                    breadcrumb_y - breadcrumb_radius,
                    breadcrumb_radius * 2.0,
                    breadcrumb_radius * 2.0,
                );
                let breadcrumb_rounded = RoundedRect::from_rect(breadcrumb_rect, breadcrumb_radius);
                snapshot.push_rounded_clip(&breadcrumb_rounded);
                snapshot.append_color(&breadcrumb_color, &breadcrumb_rect);
                snapshot.pop();
            }
        }

        // Snapshot center widget (rotates with ring)
        // Rendered after ring drawing, before item widgets
        if let Some(center) = self.center_widget.borrow().as_ref()
            && center.is_visible()
        {
            self.obj().snapshot_child(center, snapshot);
        }

        // Snapshot child widgets whose content rotates with the ring.
        // These are rendered INSIDE the rotation transform so they rotate.
        // Their allocation positions use item.angle (without rotation).
        let menu_items = self.menu_items.clone();
        for item in menu_items.iter() {
            if item.content_rotates
                && let Some(child) = item_widgets.get(&item.id)
                && child.is_visible()
            {
                self.obj().snapshot_child(child, snapshot);
            }
        }
        // Also snapshot submenu item widgets with content_rotates
        for parent_id in submenu_stack.iter() {
            if let Some(parent_item) = menu_items.find_item_recursive(parent_id)
                && let Some(submenu_items) = &parent_item.submenu
            {
                for item in submenu_items {
                    if item.content_rotates
                        && let Some(child) = item_widgets.get(&item.id)
                        && child.is_visible()
                    {
                        self.obj().snapshot_child(child, snapshot);
                    }
                }
            }
        }

        // Restore transformation state
        snapshot.restore();

        // Snapshot child widgets whose content does NOT rotate.
        // These are rendered AFTER the rotation transform is restored.
        // Their allocation positions already include rotation.
        for item in menu_items.iter() {
            if !item.content_rotates
                && let Some(child) = item_widgets.get(&item.id)
                && child.is_visible()
            {
                self.obj().snapshot_child(child, snapshot);
            }
        }
        for parent_id in submenu_stack.iter() {
            if let Some(parent_item) = menu_items.find_item_recursive(parent_id)
                && let Some(submenu_items) = &parent_item.submenu
            {
                for item in submenu_items {
                    if !item.content_rotates
                        && let Some(child) = item_widgets.get(&item.id)
                        && child.is_visible()
                    {
                        self.obj().snapshot_child(child, snapshot);
                    }
                }
            }
        }
    }
}
