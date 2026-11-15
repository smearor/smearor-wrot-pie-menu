use crate::palette::DISABLED_ALPHA;
use crate::palette::PieMenuSlot;
use glib::subclass::prelude::*;
use gtk4::gdk;
use gtk4::glib;
use gtk4::graphene;
use gtk4::gsk;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use smearor_commons_color::Alpha;
use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;
use smearor_commons_geometry::WidgetSizeExt;
use std::cell::RefCell;

/// Sweep angle of the gauge arc in degrees (80% of a full circle).
const GAUGE_SWEEP_DEGREES: f32 = 288.0;

/// Start angle of the gauge arc in degrees (screen coordinates, measured
/// clockwise from the positive x-axis). The gap is centered at the bottom
/// (90 degrees), spanning 72 degrees from 54 to 126.
const GAUGE_START_DEGREES: f32 = 126.0;

/// Number of line segments used to approximate each arc segment.
const ARC_SEGMENTS: i32 = 64;

/// Radius of the background track as a fraction of the item radius.
const TRACK_RADIUS_FRACTION: f64 = 0.85;

/// Radius of the inner value arc as a fraction of the item radius.
const INNER_RADIUS_FRACTION: f64 = 0.70;

/// Stroke width of the background track as a fraction of the item radius.
const TRACK_WIDTH_FRACTION: f64 = 0.12;

/// Stroke width of the colored zone segments as a fraction of the item radius.
const ZONE_WIDTH_FRACTION: f64 = 0.14;

/// Stroke width of the filled value arc as a fraction of the item radius.
const VALUE_ARC_WIDTH_FRACTION: f64 = 0.10;

/// Needle start radius as a fraction of the inner radius.
const NEEDLE_INNER_FRACTION: f64 = 0.3;

/// Stroke width of the value needle in pixels.
const NEEDLE_STROKE_WIDTH: Pixels<f64> = Pixels::new(3.0);

/// Radius of the center hub as a fraction of the inner radius.
const HUB_RADIUS_FRACTION: f64 = 0.15;

/// Vertical label offset as a fraction of the item radius (above center).
const LABEL_OFFSET_FRACTION: f64 = 0.15;

/// Vertical value text offset as a fraction of the item radius (below center).
const VALUE_OFFSET_FRACTION: f64 = 0.05;

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
        pub item_radius: RefCell<Pixels<f32>>,
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
                item_radius: RefCell::new(Pixels::new(60.0)),
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
            let center = obj.center();
            let item_radius = Pixels::from(f64::from(self.item_radius.borrow().as_raw()));

            // Bounding square of a centered circle of the given radius -
            // converts to graphene coordinates at the draw boundary.
            let bounds_at = |radius: Pixels<f64>| graphene::Rect::from(Rect::<Pixels>::centered(center, Vec2::<Pixels>::splat(radius * 2.0)).to_f32());

            let label = self.label.borrow();
            let value = *self.value.borrow();
            let unit = self.unit.borrow();
            let min = *self.min.borrow();
            let warning = *self.warning.borrow();
            let critical = *self.critical.borrow();
            let max = *self.max.borrow();

            let disabled_alpha = if *self.enabled.borrow() { Alpha::OPAQUE } else { DISABLED_ALPHA };
            let apply_alpha = |color: Color<f32>| gdk::RGBA::from(color.fade(disabled_alpha));

            let outer_radius = item_radius;
            let track_radius = item_radius * TRACK_RADIUS_FRACTION;
            let inner_radius = item_radius * INNER_RADIUS_FRACTION;

            let green_color = apply_alpha(PieMenuSlot::GaugeZoneGreen.color());
            let orange_color = apply_alpha(PieMenuSlot::GaugeZoneOrange.color());
            let red_color = apply_alpha(PieMenuSlot::GaugeZoneRed.color());
            let track_color = apply_alpha(PieMenuSlot::GaugeTrack.color());

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
            draw_arc(
                snapshot,
                center,
                track_radius,
                start_angle,
                start_angle + sweep,
                track_color,
                item_radius * TRACK_WIDTH_FRACTION,
            );

            // Draw colored zone segments
            // Green: min to warning
            draw_arc(snapshot, center, outer_radius, start_angle, warning_angle, green_color, item_radius * ZONE_WIDTH_FRACTION);
            // Orange: warning to critical
            draw_arc(snapshot, center, outer_radius, warning_angle, critical_angle, orange_color, item_radius * ZONE_WIDTH_FRACTION);
            // Red: critical to max
            draw_arc(
                snapshot,
                center,
                outer_radius,
                critical_angle,
                start_angle + sweep,
                red_color,
                item_radius * ZONE_WIDTH_FRACTION,
            );

            // Draw value indicator (filled arc from start to value)
            let value_color = if value_frac < warning_frac {
                green_color
            } else if value_frac < critical_frac {
                orange_color
            } else {
                red_color
            };
            draw_arc(snapshot, center, inner_radius, start_angle, value_angle, value_color, item_radius * VALUE_ARC_WIDTH_FRACTION);

            // Draw needle at value position
            let needle_angle = RotationRad::from_degrees(f64::from(value_angle));
            let needle_start = (center + Vec2::<Pixels>::from_polar((inner_radius * NEEDLE_INNER_FRACTION).as_f64(), needle_angle)).to_f32();
            let needle_end = (center + Vec2::<Pixels>::from_polar(outer_radius.as_f64(), needle_angle)).to_f32();

            let builder = gsk::PathBuilder::new();
            builder.move_to(needle_start.x, needle_start.y);
            builder.line_to(needle_end.x, needle_end.y);
            let needle_path = builder.to_path();
            let needle_stroke = gsk::Stroke::new(NEEDLE_STROKE_WIDTH.as_f64() as f32);
            let needle_color = apply_alpha(PieMenuSlot::GaugeNeedle.color());
            snapshot.append_stroke(&needle_path, &needle_stroke, &needle_color);

            // Draw center hub
            let hub_radius = inner_radius * HUB_RADIUS_FRACTION;
            let hub_rect = bounds_at(hub_radius);
            let hub_rounded = gsk::RoundedRect::from_rect(hub_rect, hub_radius.as_f64() as f32);
            snapshot.push_rounded_clip(&hub_rounded);
            snapshot.append_color(&apply_alpha(PieMenuSlot::GaugeHub.color()), &hub_rect);
            snapshot.pop();

            // Draw label in center
            if !label.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = gtk4::pango::Layout::new(&pango_context);
                pango_layout.set_text(&label);
                let font_desc = gtk4::pango::FontDescription::from_string("Sans 10");
                pango_layout.set_font_description(Some(&font_desc));

                let label_color = apply_alpha(PieMenuSlot::Label.color());
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let label_width = logical_rect.width() as f64 / gtk4::pango::SCALE as f64;
                let label_pos = Vec2::<Pixels>::new(center.x - label_width / 2.0, center.y - item_radius.as_f64() * LABEL_OFFSET_FRACTION);

                snapshot.translate(&graphene::Point::from(label_pos.to_f32()));
                snapshot.append_layout(&pango_layout, &label_color);
                snapshot.translate(&graphene::Point::from((-label_pos).to_f32()));
            }

            // Draw value + unit below label
            let value_text = format_value_text(value, &unit);
            if !value_text.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = gtk4::pango::Layout::new(&pango_context);
                pango_layout.set_text(&value_text);
                let font_desc = gtk4::pango::FontDescription::from_string("Sans 9");
                pango_layout.set_font_description(Some(&font_desc));

                let value_color = apply_alpha(PieMenuSlot::Label.color());
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let value_width = logical_rect.width() as f64 / gtk4::pango::SCALE as f64;
                let value_pos = Vec2::<Pixels>::new(center.x - value_width / 2.0, center.y + item_radius.as_f64() * VALUE_OFFSET_FRACTION);

                snapshot.translate(&graphene::Point::from(value_pos.to_f32()));
                snapshot.append_layout(&pango_layout, &value_color);
                snapshot.translate(&graphene::Point::from((-value_pos).to_f32()));
            }

            // Draw selection ring outline when keyboard-selected
            if *self.selected.borrow() {
                let selection_ring_radius = item_radius + Pixels::new(3.0);
                let selection_ring_rect = bounds_at(selection_ring_radius);
                let selection_ring_rounded = gsk::RoundedRect::from_rect(selection_ring_rect, selection_ring_radius.as_f64() as f32);
                let selection_ring_color = gdk::RGBA::from(PieMenuSlot::SelectionRing.color());
                let stroke = gsk::Stroke::new(2.0);
                let builder = gsk::PathBuilder::new();
                builder.add_rounded_rect(&selection_ring_rounded);
                let path = builder.to_path();
                snapshot.append_stroke(&path, &stroke, &selection_ring_color);
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let item_radius = self.item_radius.borrow().as_raw();
            let size = (item_radius * 2.0 + 20.0) as i32;
            (size, size, -1, -1)
        }
    }
}

/// Draws an arc segment approximated with line segments.
#[allow(clippy::too_many_arguments)]
fn draw_arc(
    snapshot: &gtk4::Snapshot,
    center: Vec2<Pixels>,
    radius: Pixels<f64>,
    start_angle_deg: f32,
    end_angle_deg: f32,
    color: gdk::RGBA,
    line_width: Pixels<f64>,
) {
    let builder = gsk::PathBuilder::new();
    for i in 0..=ARC_SEGMENTS {
        let frac = i as f32 / ARC_SEGMENTS as f32;
        let angle_deg = start_angle_deg + frac * (end_angle_deg - start_angle_deg);
        let point = (center + Vec2::<Pixels>::from_polar(radius.as_f64(), RotationRad::from_degrees(f64::from(angle_deg)))).to_f32();
        if i == 0 {
            builder.move_to(point.x, point.y);
        } else {
            builder.line_to(point.x, point.y);
        }
    }
    let path = builder.to_path();
    let stroke = gsk::Stroke::new(line_width.as_f64() as f32);
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
    pub item_radius: Pixels<f32>,
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
