use gtk4::gdk;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use smearor_commons_color::Color;
use smearor_commons_color::ParseColorError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;

/// Error encountered when parsing a hex color string.
///
/// Compatibility alias for [`ParseColorError`] (previously a dedicated
/// `ParseHexError` enum in this module).
pub type ParseHexError = ParseColorError;

/// An RGB color with f32 components in the range [0.0, 1.0].
///
/// Alias for [`Color<f32>`] (the alpha channel is always 1.0 for
/// colors created via [`Color::new`]).
pub type RgbColor = Color<f32>;

/// An RGBA color with f32 components in the range [0.0, 1.0].
///
/// Thin newtype around [`Color<f32>`]; all parsing, formatting and
/// conversion logic is provided by `smearor-commons-color`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct RgbaColor(pub Color<f32>);

impl RgbaColor {
    /// Creates a new RGBA color from an RGB color and alpha
    pub const fn new(color: RgbColor, alpha: f32) -> Self {
        Self(Color::<f32>::new_rgba(color.r, color.g, color.b, alpha))
    }

    /// Creates a new RGBA color from individual f32 components
    pub const fn with_rgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(Color::<f32>::new_rgba(red, green, blue, alpha))
    }

    /// Creates a fully transparent color
    pub const fn transparent() -> Self {
        Self(Color::<f32>::new_rgba(0.0, 0.0, 0.0, 0.0))
    }

    /// Clamps all components to the range [0.0, 1.0]
    pub fn clamp(&self) -> Self {
        Self(Color::<f32>::new_rgba(
            self.0.r.clamp(0.0, 1.0),
            self.0.g.clamp(0.0, 1.0),
            self.0.b.clamp(0.0, 1.0),
            self.0.a.clamp(0.0, 1.0),
        ))
    }

    /// Parses a hex color string with optional alpha (e.g. "#FF0000" or "#FF000077")
    pub fn parse_hex_with_optional_alpha(hex: &str) -> Result<Self, ParseHexError> {
        hex.parse()
    }

    /// Parses an 8-digit hex color string (e.g. "#FF000077" or "FF000077")
    pub fn parse_hex(hex: &str) -> Result<Self, ParseHexError> {
        hex.parse()
    }

    /// Serializes the color to a hex string (e.g. "#FF000077").
    pub fn to_hex_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for RgbaColor {
    fn default() -> Self {
        Self::transparent()
    }
}

impl Deref for RgbaColor {
    type Target = Color<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RgbaColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<RgbColor> for RgbaColor {
    fn from(color: RgbColor) -> Self {
        Self::new(color, 1.0)
    }
}

impl From<RgbaColor> for gdk::RGBA {
    fn from(color: RgbaColor) -> Self {
        gdk::RGBA::new(color.0.r, color.0.g, color.0.b, color.0.a)
    }
}

impl From<&RgbaColor> for gdk::RGBA {
    fn from(color: &RgbaColor) -> Self {
        gdk::RGBA::new(color.0.r, color.0.g, color.0.b, color.0.a)
    }
}

impl From<&str> for RgbaColor {
    fn from(hex: &str) -> Self {
        RgbaColor::from_str(hex).unwrap_or_else(|_| panic!("Invalid hex color string: '{hex}'"))
    }
}

impl From<String> for RgbaColor {
    fn from(hex: String) -> Self {
        RgbaColor::from(hex.as_str())
    }
}

impl FromStr for RgbaColor {
    type Err = ParseHexError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(Self(Color::from_str(hex)?))
    }
}

impl Serialize for RgbaColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex_string())
    }
}

impl<'de> Deserialize<'de> for RgbaColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        RgbaColor::from_str(&hex).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_color_from_rgb_color() {
        let rgb = RgbColor::new(0.5, 0.3, 0.7);
        let rgba: RgbaColor = rgb.into();
        assert_eq!(rgba.0.r, 0.5);
        assert_eq!(rgba.0.g, 0.3);
        assert_eq!(rgba.0.b, 0.7);
        assert_eq!(rgba.0.a, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_hex() {
        let rgba: RgbaColor = "#FF0000".into();
        assert_eq!(rgba.0.r, 1.0);
        assert_eq!(rgba.0.g, 0.0);
        assert_eq!(rgba.0.b, 0.0);
        assert_eq!(rgba.0.a, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_hex_with_alpha() {
        let rgba: RgbaColor = "#FF000077".into();
        assert_eq!(rgba.0.r, 1.0);
        assert_eq!(rgba.0.g, 0.0);
        assert_eq!(rgba.0.b, 0.0);
        assert!((rgba.0.a - 0.4667).abs() < 0.01);
    }

    #[test]
    fn test_rgba_color_from_string() {
        let rgba: RgbaColor = String::from("#00FF00").into();
        assert_eq!(rgba.0.r, 0.0);
        assert_eq!(rgba.0.g, 1.0);
        assert_eq!(rgba.0.b, 0.0);
    }

    #[test]
    fn test_rgba_color_from_str_trait() {
        let rgba = RgbaColor::from_str("#0000FF").unwrap();
        assert_eq!(rgba.0.r, 0.0);
        assert_eq!(rgba.0.g, 0.0);
        assert_eq!(rgba.0.b, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_invalid() {
        let result = RgbaColor::from_str("invalid");
        assert!(result.is_err());
    }
}
