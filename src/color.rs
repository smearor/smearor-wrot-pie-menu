//! Hex-encoded RGBA colors used by menu item configs.

use smearor_commons_color::Color;
pub use smearor_commons_color::ParseColorError as ParseHexError;

/// An RGB color with f32 components in the range [0.0, 1.0].
///
/// Alias for [`Color<f32>`] (the alpha channel is always 1.0 for
/// colors created via [`Color::new`]).
pub type RgbColor = Color<f32>;

/// An RGBA color with f32 components in the range [0.0, 1.0].
///
/// Alias for [`Color<f32>`]; all parsing, formatting, serde and
/// `gdk::RGBA` conversion logic is provided by `smearor-commons-color`.
///
/// `#[builder(setter(into))]` config fields take `Option<RgbaColor>`;
/// pass a parsed value (e.g. `"#FF000077".parse().unwrap_or_default()`)
/// since `From<&str>` cannot be implemented on a foreign type.
pub type RgbaColor = Color<f32>;
