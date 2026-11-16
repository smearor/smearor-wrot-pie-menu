use crate::menu::item::DEFAULT_MENU_ITEM_RADIUS;
use crate::palette::DISABLED_ALPHA;
use crate::palette::PieMenuSlot;
use glib::subclass::prelude::*;
use gtk4::glib;
use gtk4::graphene;
use gtk4::prelude::*;
use gtk4::subclass::prelude::WidgetImpl;
use smearor_commons_color::Alpha;
use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::Vec2;
use smearor_commons_geometry::WidgetSizeExt;
use std::cell::RefCell;

mod imp {
    use super::*;
    use gtk4::gdk::{Display, RGBA};
    use gtk4::graphene::Point;
    use gtk4::gsk::{PathBuilder, RoundedRect, Stroke};
    use gtk4::pango::{FontDescription, Layout};
    use gtk4::{IconLookupFlags, IconTheme, Orientation, TextDirection};

    pub struct CircleItemWidgetImpl {
        pub icon_name: RefCell<String>,
        pub label: RefCell<String>,
        pub item_radius: RefCell<Pixels<f32>>,
        pub icon_size: RefCell<Option<Pixels<f32>>>,
        pub enabled: RefCell<bool>,
        pub selected: RefCell<bool>,
        pub bg_color: RefCell<Color<f32>>,
        pub label_color: RefCell<Color<f32>>,
    }

    impl Default for CircleItemWidgetImpl {
        fn default() -> Self {
            Self {
                icon_name: RefCell::new(String::new()),
                label: RefCell::new(String::new()),
                item_radius: RefCell::new(DEFAULT_MENU_ITEM_RADIUS),
                icon_size: RefCell::new(None),
                enabled: RefCell::new(true),
                selected: RefCell::new(false),
                bg_color: RefCell::new(PieMenuSlot::ItemBackground.color()),
                label_color: RefCell::new(PieMenuSlot::Label.color()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CircleItemWidgetImpl {
        const NAME: &'static str = "CircleItemWidget";
        type Type = super::CircleItemWidget;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for CircleItemWidgetImpl {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_halign(gtk4::Align::Center);
            self.obj().set_valign(gtk4::Align::Center);
        }
    }

    impl WidgetImpl for CircleItemWidgetImpl {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let obj = self.obj();
            let center = obj.center();
            let item_radius = Pixels::new(f64::from(self.item_radius.borrow().as_raw()));

            // Bounding square of a centered circle of the given radius -
            // converts to graphene coordinates at the draw boundary.
            let bounds_at = |radius: Pixels<f64>| graphene::Rect::from(Rect::<Pixels>::centered(center, Vec2::<Pixels>::splat(radius * 2.0)).to_f32());

            let bg_color = *self.bg_color.borrow();
            let disabled_alpha = if *self.enabled.borrow() { Alpha::OPAQUE } else { DISABLED_ALPHA };
            let is_selected = *self.selected.borrow();
            let item_color = if is_selected { bg_color.lighten(0.3) } else { bg_color };
            let item_color = RGBA::from(item_color.with_alpha(disabled_alpha));

            // Draw shadow
            let shadow_color = RGBA::from(PieMenuSlot::ItemShadow.color());
            let shadow_radius = item_radius + Pixels::new(2.0);
            let shadow_rect = bounds_at(shadow_radius);
            let shadow_rounded = RoundedRect::from_rect(shadow_rect, shadow_radius.as_f64() as f32);
            snapshot.push_rounded_clip(&shadow_rounded);
            snapshot.append_color(&shadow_color, &shadow_rect);
            snapshot.pop();

            // Draw background circle
            let item_rect = bounds_at(item_radius);
            let item_rounded = RoundedRect::from_rect(item_rect, item_radius.as_f64() as f32);
            snapshot.push_rounded_clip(&item_rounded);
            snapshot.append_color(&item_color, &item_rect);
            snapshot.pop();

            // Draw selection ring outline when keyboard-selected
            if is_selected {
                let selection_ring_radius = item_radius + Pixels::new(3.0);
                let selection_ring_rect = bounds_at(selection_ring_radius);
                let selection_ring_rounded = RoundedRect::from_rect(selection_ring_rect, selection_ring_radius.as_f64() as f32);
                let selection_ring_color = RGBA::from(PieMenuSlot::SelectionRing.color());
                let stroke = Stroke::new(2.0);
                let builder = PathBuilder::new();
                builder.add_rounded_rect(&selection_ring_rounded);
                let path = builder.to_path();
                snapshot.append_stroke(&path, &stroke, &selection_ring_color);
            }

            // Draw icon
            let icon_name = self.icon_name.borrow();
            let icon_name = icon_name.as_str();
            if !icon_name.is_empty()
                && let Some(display) = Display::default()
            {
                let icon_theme = IconTheme::for_display(&display);
                let icon_size = self
                    .icon_size
                    .borrow()
                    .map(|size| Pixels::new(f64::from(size.as_raw())))
                    .unwrap_or(item_radius * 0.6);
                let paintable =
                    icon_theme.lookup_icon(icon_name, &[icon_name], icon_size.as_f64() as i32, 1, TextDirection::None, IconLookupFlags::FORCE_SYMBOLIC);
                let icon_pos = center - Vec2::<Pixels>::splat(icon_size / 2.0);
                snapshot.translate(&Point::from(icon_pos.to_f32()));
                paintable.snapshot(snapshot, icon_size.as_f64(), icon_size.as_f64());
                snapshot.translate(&Point::from((-icon_pos).to_f32()));
            }

            // Draw label below icon
            let label_text = self.label.borrow();
            let label_text = label_text.as_str();
            if !label_text.is_empty() {
                let pango_context = obj.pango_context();
                let pango_layout = Layout::new(&pango_context);
                pango_layout.set_text(label_text);
                let font_desc = FontDescription::from_string("Sans 7");
                pango_layout.set_font_description(Some(&font_desc));

                let label_color = *self.label_color.borrow();
                let label_color = RGBA::from(label_color.with_alpha(disabled_alpha));
                let (_ink_rect, logical_rect) = pango_layout.extents();
                let label_width = logical_rect.width() as f64 / gtk4::pango::SCALE as f64;
                let label_pos = Vec2::<Pixels>::new(center.x - label_width / 2.0, center.y + item_radius.as_f64());

                // Label shadow
                let shadow_offset = Vec2::<Pixels>::splat(Pixels::new(1.0));
                let shadow_color = RGBA::from(PieMenuSlot::LabelShadow.color());
                snapshot.translate(&Point::from((label_pos + shadow_offset).to_f32()));
                snapshot.append_layout(&pango_layout, &shadow_color);
                snapshot.translate(&Point::from((-(label_pos + shadow_offset)).to_f32()));

                // Label
                snapshot.translate(&Point::from(label_pos.to_f32()));
                snapshot.append_layout(&pango_layout, &label_color);
                snapshot.translate(&Point::from((-label_pos).to_f32()));
            }
        }

        fn measure(&self, _orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let item_radius = self.item_radius.borrow().as_raw();
            let size = (item_radius * 2.0 + 20.0) as i32;
            (size, size, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct CircleItemWidget(ObjectSubclass<imp::CircleItemWidgetImpl>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl CircleItemWidget {
    pub fn new(
        icon_name: &str,
        label: &str,
        bg_color: Color<f32>,
        label_color: Color<f32>,
        item_radius: Pixels<f32>,
        icon_size: Option<Pixels<f32>>,
        enabled: bool,
    ) -> Self {
        let widget: Self = glib::Object::builder().build();
        widget.imp().icon_name.replace(icon_name.to_string());
        widget.imp().label.replace(label.to_string());
        widget.imp().item_radius.replace(item_radius);
        widget.imp().icon_size.replace(icon_size);
        widget.imp().enabled.replace(enabled);
        widget.imp().bg_color.replace(bg_color);
        widget.imp().label_color.replace(label_color);
        widget
    }

    pub fn set_selected(&self, selected: bool) {
        self.imp().selected.replace(selected);
        self.queue_draw();
    }
}
