use crate::menu::menu::Menu;
use crate::menu_widget::widget::PieMenuWidget;
use atomic_float::AtomicF32;
use glib::subclass::prelude::*;
use gtk4::EventControllerMotion;
use gtk4::IconLookupFlags;
use gtk4::IconTheme;
use gtk4::TextDirection;
use gtk4::gdk::Display;
use gtk4::gdk::RGBA;
use gtk4::glib;
use gtk4::graphene::Point;
use gtk4::graphene::Rect;
use gtk4::gsk::RoundedRect;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use std::cell::RefCell;
use std::sync::Arc;
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

        widget.set_layout_manager(Some(gtk4::BinLayout::new()));

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

            // Calculate distance from center (center is middle of widget)
            let widget = imp.obj();
            let width = widget.width() as f64;
            let height = widget.height() as f64;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let dx = x - center_x;
            let dy = y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Check if mouse is in the ring area (between center_radius and radius)
            if distance < center_radius || distance > radius {
                // Outside the ring, no item hovered
                let mut hovered_index = imp.hovered_item_index.borrow_mut();
                if *hovered_index != -1 {
                    *hovered_index = -1;
                    widget.queue_draw();
                }
                return;
            }

            // Calculate angle of mouse position
            let angle_rad = dy.atan2(dx);
            let angle_deg = angle_rad.to_degrees();
            let normalized_angle = (angle_deg - rotation).rem_euclid(360.0);

            // Find which menu item corresponds to this angle
            let menu_items = imp.menu_items.clone();
            let num_items = menu_items.len();
            if num_items == 0 {
                return;
            }

            // Find the item with the closest angle to the mouse position
            let mut closest_index = -1i32;
            let mut closest_distance = f64::MAX;
            for (index, item) in menu_items.iter().enumerate() {
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
                widget.queue_draw();
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

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
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

        // Draw menu items in ring layout
        let Some(display) = Display::default() else {
            return;
        };
        let icon_theme = IconTheme::for_display(&display);
        let icon_size = (radius - center_radius) / 3.0;
        debug!("Icon size: {icon_size}",);
        let hovered_index = *self.hovered_item_index.borrow();
        for (index, item) in self.menu_items.iter().enumerate() {
            let angle_rad = item.angle.to_radians();
            let item_x = center_x + (radius * 0.7) * angle_rad.cos();
            let item_y = center_y + (radius * 0.7) * angle_rad.sin();

            // Draw item background circle
            let item_color: RGBA = item.color.into();
            let item_radius = item.radius();

            // Highlight if hovered
            let is_hovered = index as i32 == hovered_index;
            let item_color = if is_hovered {
                RGBA::new(item_color.red() * 1.3, item_color.green() * 1.3, item_color.blue() * 1.3, 1.0)
            } else {
                item_color
            };

            // Draw shadow for item circle
            let item_shadow_color = RGBA::new(0.8, 0.8, 0.8, 0.1);
            let item_shadow_offset = 2.0;
            let item_shadow_radius = item_radius + item_shadow_offset;
            let item_shadow_rect = Rect::new(item_x - item_shadow_radius, item_y - item_shadow_radius, item_shadow_radius * 2.0, item_shadow_radius * 2.0);
            let item_shadow_rounded = RoundedRect::from_rect(item_shadow_rect, item_shadow_radius);
            snapshot.push_rounded_clip(&item_shadow_rounded);
            snapshot.append_color(&item_shadow_color, &item_shadow_rect);
            snapshot.pop();

            let item_rect = Rect::new(item_x - item_radius, item_y - item_radius, item_radius * 2.0, item_radius * 2.0);
            let item_rounded = RoundedRect::from_rect(item_rect, item_radius);
            snapshot.push_rounded_clip(&item_rounded);
            snapshot.append_color(&item_color, &item_rect);
            snapshot.pop();
            debug!("Drew menu item at ({}, {})", item_x, item_y);

            // Draw icon from icon_name
            let paintable =
                icon_theme.lookup_icon(&item.icon_name, &[&item.icon_name], icon_size as i32, 1, TextDirection::None, IconLookupFlags::FORCE_SYMBOLIC);
            snapshot.translate(&Point::new(item_x - icon_size / 2.0, item_y - icon_size / 2.0));
            paintable.snapshot(snapshot, icon_size as f64, icon_size as f64);
            snapshot.translate(&Point::new(-item_x + icon_size / 2.0, -item_y + icon_size / 2.0));
            debug!("Drew icon '{}' at ({}, {})", item.icon_name, item_x, item_y);

            // Draw label below icon
            let widget = self.obj();
            let pango_context = widget.pango_context();
            let pango_layout = gtk4::pango::Layout::new(&pango_context);
            pango_layout.set_text(&item.label);
            let font_desc = gtk4::pango::FontDescription::from_string("Sans 7");
            pango_layout.set_font_description(Some(&font_desc));

            let label_color: RGBA = item.label_color.into();
            let (_ink_rect, logical_rect) = pango_layout.extents();
            let label_width = logical_rect.width() as f32 / gtk4::pango::SCALE as f32;

            let label_x = item_x - label_width / 2.0;
            let label_y = item_y + item_radius;

            // Draw label shadow
            let shadow_offset = 1.0;
            let shadow_color = RGBA::new(0.0, 0.0, 0.0, 0.8);
            snapshot.translate(&Point::new(label_x + shadow_offset, label_y + shadow_offset));
            snapshot.append_layout(&pango_layout, &shadow_color);
            snapshot.translate(&Point::new(-(label_x + shadow_offset), -(label_y + shadow_offset)));

            // Draw label
            snapshot.translate(&Point::new(label_x, label_y));
            snapshot.append_layout(&pango_layout, &label_color);
            snapshot.translate(&Point::new(-label_x, -label_y));

            debug!("Drew label '{}' at ({}, {})", item.label, label_x, label_y);
        }

        // Restore transformation state
        snapshot.restore();
    }
}
