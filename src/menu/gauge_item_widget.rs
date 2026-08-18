use glib::subclass::prelude::*;
use gtk4::gdk;
use gtk4::glib;
use gtk4::gsk;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use std::cell::RefCell;

/// Sweep angle of the gauge arc in degrees (80% of a full circle).
const GAUGE_SWEEP_DEGREES: f32 = 288.0;

/// Start angle of the gauge arc in degrees (screen coordinates, measured
/// clockwise from the positive x-axis). The gap is centered at the bottom
/// (90 degrees), spanning 72 degrees from 54 to 126.
const GAUGE_START_DEGREES: f32 = 126.0;

/// Number of line segments used to approximate each arc segment.
const ARC_SEGMENTS: i32 = 64;

mod imp {
    use super::*;

    pub struct GaugeItemWidgetImpl {
        pub label: RefCell<String>,
        pub value: RefCell<f64>,
        pub unit: RefCell<String>,
        pub min: RefCell<f64>,
        pub warning: RefCell<f64>,
        pub critical: RefCell<f64>,
        pub max: RefCell<f64>,
        pub item_radius: RefCell<f32>,
        pub enabled: RefCell<bool>,
        pub selected: RefCell<bool>,
    }

    impl Default for GaugeItemWidgetImpl {
        fn default() -> Self {
            Self {
                label: RefCell::new(String::new()),
                value: RefCell::new(0.0),
                unit: RefCell::new(String::new()),
                min: RefCell::new(0.0),
                warning: RefCell::new(0.0),
                critical: RefCell::new(0.0),
                max: RefCell::new(0.0),
                item_radius: RefCell::new(60.0),
                enabled: RefCell::new(true),
                selected: RefCell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GaugeItemWidgetImpl {
        const NAME: &'static str = "GaugeItemWidget";
        type Type = super::GaugeItemWidget;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for GaugeItemWidgetImpl {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_halign(gtk4::Align::Center);
            self.obj().set_valign(gtk4::Align::Center);
        }
    }

    impl WidgetImpl for GaugeItemWidgetImpl {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let obj = self.obj();
            let width = obj.width() as f32;
            let height = obj.height() as f32;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let item_radius = *self.item_radius.borrow();

            let label = self.label.borrow();
            let value = *self.value.borrow();
            let unit = self.unit.borrow();
            let min = *self.min.borrow();
            let warning = *self.warning.borrow();
            let critical = *self.critical.borrow();
            let max = *self.max.borrow();

            let disabled_alpha: f32 = if *self.enabled.borrow() { 1.0 } else { 0.4 };

            let outer_radius = item_radius;
            let track_radius = item_radius * 0.85;
            let inner_radius = item_radius * 0.70;

            let green_color = gdk::RGBA::new(0.2, 0.8, 0.2, disabled_alpha);
            let orange_color = gdk::RGBA::new(0.9, 0.6, 0.0, disabled_alpha);
            let red_color = gdk::RGBA::new(0.9, 0.2, 0.2, disabled_alpha);
            let track_color = gdk::RGBA::new(0.3, 0.3, 0.3, 0.3 * disabled_alpha);

            let value_clamped = value.clamp(min, max);
            let range = max - min;
            let warning_frac = (((warning - min) / range).clamp(0.0, 1.0)) as f32;
            let critical_frac = (((critical - min) / range).clamp(0.0, 1.0)) as f32;
            let value_frac = (((value_clamped - min) / range).clamp(0.0, 1.0)) as f32;

            let start_angle = GAUGE_START_DEGREES;
            let sweep = GAUGE_SWEEP_DEGREES;

            let warning_angle = start_angle + warning_frac * sweep;
            let critical_angle = start_angle + critical_frac * sweep;
            let value_angle = start_angle + value_frac * sweep;

            // Draw background track (full arc)
            draw_arc(snapshot, center_x, center_y, track_radius, start_angle, start_angle + sweep, track_color, item_radius * 0.12);

            // Draw colored zone segments
            // Green: min to warning
            draw_arc(snapshot, center_x, center_y, outer_radius, start_angle, warning_angle, green_color, item_radius * 0.14);
            // Orange: warning to critical
            draw_arc(snapshot, center_x, center_y, outer_radius, warning_angle, critical_angle, orange_color, item_radius * 0.14);
            // Red: critical to max
            draw_arc(snapshot, center_x, center_y, outer_radius, critical_angle, start_angle + sweep, red_color, item_radius * 0.14);

            // Draw value indicator (filled arc from start to value)
            let value_color = if value_frac < warning_frac {
                green_color
            } else if value_frac < critical_frac {
                orange_color
            } else {
                red_color
            };
            draw_arc(snapshot, center_x, center_y, inner_radius, start_angle, value_angle, value_color, item_radius * 0.10);

            // Draw needle at value position
            let needle_angle_rad = value_angle.to_radians();
            let needle_inner = inner_radius * 0.3;
            let needle_outer = outer_radius;
            let needle_start_x = center_x + needle_inner * needle_angle_rad.cos();
            let needle_start_y = center_y + needle_inner * needle_angle_rad.sin();
            let needle_end_x = center_x + needle_outer * needle_angle_rad.cos();
            let needle_end_y = center_y + needle_outer * needle_angle_rad.sin();

            let builder = gsk::PathBuilder::new();
            builder.move_to(needle_start_x, needle_start_y);
            builder.line_to(needle_end_x, needle_end_y);
            let needle_path = builder.to_path();
            let needle_stroke = gsk::Stroke::new(3.0);
            let needle_color = gdk::RGBA::new(1.0, 1.0, 1.0, disabled_alpha);
            snapshot.append_stroke(&needle_path, &needle_stroke, &needle_color);

            // Draw center hub
            let hub_radius = inner_radius * 0.15;
            let hub_rect = gtk4::graphene::Rect::new(center_x - hub_radius, center_y - hub_radius, hub_radius * 2.0, hub_radius * 2.0);
            let hub_rounded = gsk::RoundedRect::from_rect(hub_rect, hub_radius);
            snapshot.push_rounded_clip(&hub_rounded);
            snapshot.append_color(&gdk::RGBA::new(0.5, 0.5, 0.5, disabled_alpha), &hub_rect);
            snapshot.pop();

            // Draw label in center
            if !label.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = gtk4::pango::Layout::new(&pango_context);
                pango_layout.set_text(&label);
                let font_desc = gtk4::pango::FontDescription::from_string("Sans 10");
                pango_layout.set_font_description(Some(&font_desc));

                let label_color = gdk::RGBA::new(1.0, 1.0, 1.0, disabled_alpha);
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let label_width = logical_rect.width() as f32 / gtk4::pango::SCALE as f32;
                let label_x = center_x - label_width / 2.0;
                let label_y = center_y - item_radius * 0.15;

                snapshot.translate(&gtk4::graphene::Point::new(label_x, label_y));
                snapshot.append_layout(&pango_layout, &label_color);
                snapshot.translate(&gtk4::graphene::Point::new(-label_x, -label_y));
            }

            // Draw value + unit below label
            let value_text = format_value_text(value, &unit);
            if !value_text.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = gtk4::pango::Layout::new(&pango_context);
                pango_layout.set_text(&value_text);
                let font_desc = gtk4::pango::FontDescription::from_string("Sans 9");
                pango_layout.set_font_description(Some(&font_desc));

                let value_color = gdk::RGBA::new(1.0, 1.0, 1.0, disabled_alpha);
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let value_width = logical_rect.width() as f32 / gtk4::pango::SCALE as f32;
                let value_x = center_x - value_width / 2.0;
                let value_y = center_y + item_radius * 0.05;

                snapshot.translate(&gtk4::graphene::Point::new(value_x, value_y));
                snapshot.append_layout(&pango_layout, &value_color);
                snapshot.translate(&gtk4::graphene::Point::new(-value_x, -value_y));
            }

            // Draw selection ring outline when keyboard-selected
            if *self.selected.borrow() {
                let selection_ring_radius = item_radius + 3.0;
                let selection_ring_rect = gtk4::graphene::Rect::new(
                    center_x - selection_ring_radius,
                    center_y - selection_ring_radius,
                    selection_ring_radius * 2.0,
                    selection_ring_radius * 2.0,
                );
                let selection_ring_rounded = gsk::RoundedRect::from_rect(selection_ring_rect, selection_ring_radius);
                let selection_ring_color = gdk::RGBA::new(1.0, 1.0, 1.0, 0.9);
                let stroke = gsk::Stroke::new(2.0);
                let builder = gsk::PathBuilder::new();
                builder.add_rounded_rect(&selection_ring_rounded);
                let path = builder.to_path();
                snapshot.append_stroke(&path, &stroke, &selection_ring_color);
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let item_radius = *self.item_radius.borrow();
            let size = (item_radius * 2.0 + 20.0) as i32;
            (size, size, -1, -1)
        }
    }
}

/// Draws an arc segment approximated with line segments.
#[allow(clippy::too_many_arguments)]
fn draw_arc(snapshot: &gtk4::Snapshot, center_x: f32, center_y: f32, radius: f32, start_angle_deg: f32, end_angle_deg: f32, color: gdk::RGBA, line_width: f32) {
    let builder = gsk::PathBuilder::new();
    for i in 0..=ARC_SEGMENTS {
        let frac = i as f32 / ARC_SEGMENTS as f32;
        let angle_deg = start_angle_deg + frac * (end_angle_deg - start_angle_deg);
        let angle_rad = angle_deg.to_radians();
        let x = center_x + radius * angle_rad.cos();
        let y = center_y + radius * angle_rad.sin();
        if i == 0 {
            builder.move_to(x, y);
        } else {
            builder.line_to(x, y);
        }
    }
    let path = builder.to_path();
    let stroke = gsk::Stroke::new(line_width);
    snapshot.append_stroke(&path, &stroke, &color);
}

/// Parameters for constructing a [`GaugeItemWidget`].
pub struct GaugeItemWidgetParams {
    /// Label displayed in the center of the gauge.
    pub label: String,
    /// Current value to display on the gauge.
    pub value: f64,
    /// Unit string appended to the displayed value.
    pub unit: String,
    /// Minimum value on the gauge scale.
    pub min: f64,
    /// Warning threshold. Values between `min` and `warning` are green.
    pub warning: f64,
    /// Critical threshold. Values between `warning` and `critical` are orange.
    pub critical: f64,
    /// Maximum value on the gauge scale. Values between `critical` and `max` are red.
    pub max: f64,
    /// Radius of the gauge widget in pixels.
    pub item_radius: f32,
    /// Whether the widget is enabled (affects opacity).
    pub enabled: bool,
}

/// Formats the value with appropriate precision and appends the unit.
fn format_value_text(value: f64, unit: &str) -> String {
    let formatted = if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.1}", value)
    };
    if unit.is_empty() { formatted } else { format!("{}{}", formatted, unit) }
}

glib::wrapper! {
    pub struct GaugeItemWidget(ObjectSubclass<imp::GaugeItemWidgetImpl>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl GaugeItemWidget {
    pub fn new(params: GaugeItemWidgetParams) -> Self {
        let widget: Self = glib::Object::builder().build();
        widget.imp().label.replace(params.label);
        widget.imp().value.replace(params.value);
        widget.imp().unit.replace(params.unit);
        widget.imp().min.replace(params.min);
        widget.imp().warning.replace(params.warning);
        widget.imp().critical.replace(params.critical);
        widget.imp().max.replace(params.max);
        widget.imp().item_radius.replace(params.item_radius);
        widget.imp().enabled.replace(params.enabled);
        widget
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().selected.replace(selected);
        self.queue_draw();
    }

    /// Updates the gauge value and triggers a redraw.
    pub fn set_value(&self, value: f64) {
        self.imp().value.replace(value);
        self.queue_draw();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_text_integer() {
        assert_eq!(format_value_text(85.0, "%"), "85%");
    }

    #[test]
    fn test_format_value_text_decimal() {
        assert_eq!(format_value_text(64.4, "°C"), "64.4°C");
    }

    #[test]
    fn test_format_value_text_no_unit() {
        assert_eq!(format_value_text(42.0, ""), "42");
    }

    #[test]
    fn test_gauge_constants() {
        assert_eq!(GAUGE_SWEEP_DEGREES, 288.0);
        assert_eq!(GAUGE_START_DEGREES, 126.0);
    }
}
