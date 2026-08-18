use crate::color::RgbaColor;
use glib::subclass::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use gtk4::{gdk, graphene, gsk};
use std::cell::RefCell;

mod imp {
    use super::*;

    pub struct SquareItemWidgetImpl {
        pub icon_name: RefCell<String>,
        pub label: RefCell<String>,
        pub item_radius: RefCell<f32>,
        pub enabled: RefCell<bool>,
        pub selected: RefCell<bool>,
        pub bg_color: RefCell<gdk::RGBA>,
        pub label_color: RefCell<gdk::RGBA>,
    }

    impl Default for SquareItemWidgetImpl {
        fn default() -> Self {
            Self {
                icon_name: RefCell::new(String::new()),
                label: RefCell::new(String::new()),
                item_radius: RefCell::new(40.0),
                enabled: RefCell::new(true),
                selected: RefCell::new(false),
                bg_color: RefCell::new(gdk::RGBA::new(0.5, 0.5, 0.5, 1.0)),
                label_color: RefCell::new(gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SquareItemWidgetImpl {
        const NAME: &'static str = "SquareItemWidget";
        type Type = super::SquareItemWidget;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for SquareItemWidgetImpl {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_halign(gtk4::Align::Center);
            self.obj().set_valign(gtk4::Align::Center);
        }
    }

    impl WidgetImpl for SquareItemWidgetImpl {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let obj = self.obj();
            let width = obj.width() as f32;
            let height = obj.height() as f32;
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let item_radius = *self.item_radius.borrow();
            let corner_radius = 8.0_f32.min(item_radius * 0.2);

            let bg_color = *self.bg_color.borrow();
            let disabled_alpha = if *self.enabled.borrow() { 1.0 } else { 0.4 };
            let is_selected = *self.selected.borrow();
            let item_color = if is_selected {
                gdk::RGBA::new(bg_color.red() * 1.3, bg_color.green() * 1.3, bg_color.blue() * 1.3, disabled_alpha)
            } else {
                gdk::RGBA::new(bg_color.red(), bg_color.green(), bg_color.blue(), disabled_alpha)
            };

            // Draw shadow
            let shadow_color = gdk::RGBA::new(0.8, 0.8, 0.8, 0.1);
            let shadow_offset = 2.0;
            let shadow_size = item_radius * 2.0 + shadow_offset;
            let shadow_rect = graphene::Rect::new(center_x - shadow_size / 2.0, center_y - shadow_size / 2.0, shadow_size, shadow_size);
            let shadow_rounded = gsk::RoundedRect::from_rect(shadow_rect, corner_radius);
            snapshot.push_rounded_clip(&shadow_rounded);
            snapshot.append_color(&shadow_color, &shadow_rect);
            snapshot.pop();

            // Draw background square
            let size = item_radius * 2.0;
            let item_rect = graphene::Rect::new(center_x - item_radius, center_y - item_radius, size, size);
            let item_rounded = gsk::RoundedRect::from_rect(item_rect, corner_radius);
            snapshot.push_rounded_clip(&item_rounded);
            snapshot.append_color(&item_color, &item_rect);
            snapshot.pop();

            // Draw selection ring outline when keyboard-selected
            if is_selected {
                let selection_ring_radius = item_radius + 3.0;
                let selection_ring_rect = graphene::Rect::new(
                    center_x - selection_ring_radius,
                    center_y - selection_ring_radius,
                    selection_ring_radius * 2.0,
                    selection_ring_radius * 2.0,
                );
                let selection_ring_rounded = gsk::RoundedRect::from_rect(selection_ring_rect, corner_radius);
                let selection_ring_color = gdk::RGBA::new(1.0, 1.0, 1.0, 0.9);
                let stroke = gsk::Stroke::new(2.0);
                let builder = gsk::PathBuilder::new();
                builder.add_rounded_rect(&selection_ring_rounded);
                let path = builder.to_path();
                snapshot.append_stroke(&path, &stroke, &selection_ring_color);
            }

            // Draw icon
            let icon_name = self.icon_name.borrow();
            let icon_name = icon_name.as_str();
            if !icon_name.is_empty()
                && let Some(display) = gdk::Display::default()
            {
                let icon_theme = gtk4::IconTheme::for_display(&display);
                let icon_size = (item_radius * 0.6) as i32;
                let paintable = icon_theme.lookup_icon(icon_name, &[icon_name], icon_size, 1, gtk4::TextDirection::None, gtk4::IconLookupFlags::FORCE_SYMBOLIC);
                let icon_x = center_x - icon_size as f32 / 2.0;
                let icon_y = center_y - icon_size as f32 / 2.0;
                snapshot.translate(&graphene::Point::new(icon_x, icon_y));
                paintable.snapshot(snapshot, icon_size as f64, icon_size as f64);
                snapshot.translate(&graphene::Point::new(-icon_x, -icon_y));
            }

            // Draw label below icon
            let label_text = self.label.borrow();
            let label_text = label_text.as_str();
            if !label_text.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = gtk4::pango::Layout::new(&pango_context);
                pango_layout.set_text(label_text);
                let font_desc = gtk4::pango::FontDescription::from_string("Sans 7");
                pango_layout.set_font_description(Some(&font_desc));

                let label_color = *self.label_color.borrow();
                let label_color = gdk::RGBA::new(label_color.red(), label_color.green(), label_color.blue(), disabled_alpha);
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let label_width = logical_rect.width() as f32 / gtk4::pango::SCALE as f32;

                let label_x = center_x - label_width / 2.0;
                let label_y = center_y + item_radius;

                // Label shadow
                let shadow_offset = 1.0;
                let shadow_color = gdk::RGBA::new(0.0, 0.0, 0.0, 0.8);
                snapshot.translate(&graphene::Point::new(label_x + shadow_offset, label_y + shadow_offset));
                snapshot.append_layout(&pango_layout, &shadow_color);
                snapshot.translate(&graphene::Point::new(-(label_x + shadow_offset), -(label_y + shadow_offset)));

                // Label
                snapshot.translate(&graphene::Point::new(label_x, label_y));
                snapshot.append_layout(&pango_layout, &label_color);
                snapshot.translate(&graphene::Point::new(-label_x, -label_y));
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let item_radius = *self.item_radius.borrow();
            let size = (item_radius * 2.0 + 20.0) as i32;
            (size, size, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct SquareItemWidget(ObjectSubclass<imp::SquareItemWidgetImpl>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl SquareItemWidget {
    pub fn new(icon_name: &str, label: &str, bg_color: RgbaColor, label_color: RgbaColor, item_radius: f32, enabled: bool) -> Self {
        let widget: Self = glib::Object::builder().build();
        widget.imp().icon_name.replace(icon_name.to_string());
        widget.imp().label.replace(label.to_string());
        widget.imp().item_radius.replace(item_radius);
        widget.imp().enabled.replace(enabled);
        widget.imp().bg_color.replace(gdk::RGBA::from(bg_color));
        widget.imp().label_color.replace(gdk::RGBA::from(label_color));
        widget
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().selected.replace(selected);
        self.queue_draw();
    }
}
