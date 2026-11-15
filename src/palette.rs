//! Semantic color palette of the pie menu.
//!
//! Every color drawn by the menu ring and the item widgets is assigned
//! to a [`PieMenuSlot`]. The compiled-in [`DEFAULT_PALETTE`] provides a
//! color for each slot; [`PieMenuSlot::color`] is the single lookup
//! point used by all render code.

use smearor_commons_color::Alpha;
use smearor_commons_color::Color;
use smearor_commons_color::StaticPalette;
use smearor_commons_color::palette_definition;
use smearor_commons_color::static_palette;

palette_definition! {
    /// Semantic color slots of the pie menu theme.
    ///
    /// Each variant names one visual element whose color is resolved
    /// through the palette instead of being hardcoded at the draw site.
    /// The `name()` of each variant is the snake_case theme key.
    pub enum PieMenuSlot {
        /// Fill color of the ring-shaped menu background.
        Background => "background",
        /// Drop shadow behind the menu ring.
        Shadow => "shadow",
        /// Angle markings on the inner and outer ring edges.
        Marking => "marking",
        /// Outer-edge marking at the current rotation angle.
        MarkingCurrentAngle => "marking_current_angle",
        /// Inner-edge marking at the zero-degree reference.
        MarkingZeroAngle => "marking_zero_angle",
        /// Outline stroke of a submenu ring. The slot alpha modulates
        /// the per-level ring opacity applied at draw time.
        SubmenuRing => "submenu_ring",
        /// Dot marking the parent item position on its ring.
        SubmenuIndicator => "submenu_indicator",
        /// Breadcrumb dots between open submenu rings.
        Breadcrumb => "breadcrumb",
        /// Default background of a menu item widget.
        ItemBackground => "item_background",
        /// Drop shadow behind a menu item widget.
        ItemShadow => "item_shadow",
        /// Outline drawn around the keyboard-selected item.
        SelectionRing => "selection_ring",
        /// Label text color.
        Label => "label",
        /// Drop shadow below label text.
        LabelShadow => "label_shadow",
        /// Gauge arc segment below the warning threshold.
        GaugeZoneGreen => "gauge_zone_green",
        /// Gauge arc segment between warning and critical thresholds.
        GaugeZoneOrange => "gauge_zone_orange",
        /// Gauge arc segment above the critical threshold.
        GaugeZoneRed => "gauge_zone_red",
        /// Inactive gauge arc track.
        GaugeTrack => "gauge_track",
        /// Gauge needle.
        GaugeNeedle => "gauge_needle",
        /// Gauge center hub.
        GaugeHub => "gauge_hub"
    }
}

impl PieMenuSlot {
    /// The color assigned to this slot.
    ///
    /// Resolved against [`DEFAULT_PALETTE`] - the single lookup point
    /// where a runtime-overridable palette can be consulted first once
    /// theme loading is added. Returns transparent black when the slot
    /// has no color registered.
    pub fn color(&self) -> Color<f32> {
        DEFAULT_PALETTE.get_or(self, Color::<f32>::default())
    }
}

/// Opacity applied to disabled menu items - dims all item colors to 40%.
pub const DISABLED_ALPHA: Alpha = Alpha::new(0.4);

/// The compiled-in colors of the pie menu theme.
pub static DEFAULT_PALETTE: StaticPalette<PieMenuSlot, f32> = static_palette! {
    PieMenuSlot::Background => Color::<f32>::new_rgba(0.2, 0.2, 0.2, 0.5),
    PieMenuSlot::Shadow => Color::<f32>::new_rgba(0.0, 0.0, 0.0, 0.3),
    PieMenuSlot::Marking => Color::<f32>::new_rgba(0.8, 0.8, 0.8, 0.5),
    PieMenuSlot::MarkingCurrentAngle => Color::<f32>::new_rgba(1.0, 0.6, 0.6, 1.0),
    PieMenuSlot::MarkingZeroAngle => Color::<f32>::new_rgba(0.6, 1.0, 0.6, 1.0),
    PieMenuSlot::SubmenuRing => Color::<f32>::new_rgba(0.5, 0.5, 0.5, 1.0),
    PieMenuSlot::SubmenuIndicator => Color::<f32>::new_rgba(1.0, 0.8, 0.2, 0.9),
    PieMenuSlot::Breadcrumb => Color::<f32>::new_rgba(0.6, 0.6, 0.6, 0.6),
    PieMenuSlot::ItemBackground => Color::<f32>::new_rgba(0.5, 0.5, 0.5, 1.0),
    PieMenuSlot::ItemShadow => Color::<f32>::new_rgba(0.8, 0.8, 0.8, 0.1),
    PieMenuSlot::SelectionRing => Color::<f32>::new_rgba(1.0, 1.0, 1.0, 0.9),
    PieMenuSlot::Label => Color::<f32>::WHITE,
    PieMenuSlot::LabelShadow => Color::<f32>::new_rgba(0.0, 0.0, 0.0, 0.8),
    PieMenuSlot::GaugeZoneGreen => Color::<f32>::new_rgba(0.2, 0.8, 0.2, 1.0),
    PieMenuSlot::GaugeZoneOrange => Color::<f32>::new_rgba(0.9, 0.6, 0.0, 1.0),
    PieMenuSlot::GaugeZoneRed => Color::<f32>::new_rgba(0.9, 0.2, 0.2, 1.0),
    PieMenuSlot::GaugeTrack => Color::<f32>::new_rgba(0.3, 0.3, 0.3, 0.3),
    PieMenuSlot::GaugeNeedle => Color::<f32>::WHITE,
    PieMenuSlot::GaugeHub => Color::<f32>::new_rgba(0.5, 0.5, 0.5, 1.0),
};

#[cfg(test)]
mod tests {
    use super::*;
    use smearor_commons_color::PaletteDefinition;

    #[test]
    fn default_palette_fills_every_slot() {
        assert!(DEFAULT_PALETTE.is_complete());
        assert_eq!(DEFAULT_PALETTE.missing(), Vec::<PieMenuSlot>::new());
        assert_eq!(DEFAULT_PALETTE.len(), PieMenuSlot::slots().len());
    }

    #[test]
    fn slot_color_resolves_to_default() {
        assert_eq!(PieMenuSlot::Label.color(), Color::<f32>::WHITE);
        assert_eq!(PieMenuSlot::Background.color(), Color::<f32>::new_rgba(0.2, 0.2, 0.2, 0.5));
    }

    #[test]
    fn slot_names_use_snake_case() {
        assert_eq!(PieMenuSlot::MarkingCurrentAngle.name(), "marking_current_angle");
    }
}
