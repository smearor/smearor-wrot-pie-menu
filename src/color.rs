use gtk4::gdk;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;
use typed_builder::TypedBuilder;

/// Error encountered when parsing a hex color string
#[derive(Debug, Clone, Error)]
pub enum ParseHexError {
    #[error("Invalid size")]
    InvalidSize,
    #[error("Invalid red component")]
    InvalidRed,
    #[error("Invalid green component")]
    InvalidGreen,
    #[error("Invalid blue component")]
    InvalidBlue,
    #[error("Invalid alpha component")]
    InvalidAlpha,
}

/// An RGB color with f32 components in the range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder)]
pub struct RgbColor {
    /// The red component
    pub red: f32,
    /// The green component
    pub green: f32,
    /// The blue component
    pub blue: f32,
}

impl RgbColor {
    /// Creates a new RGB color from f32 components
    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    /// Creates a new RGB color from u8 components
    pub fn new_from_u8(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: (red as f32 / 255.0).clamp(0.0, 1.0),
            green: (green as f32 / 255.0).clamp(0.0, 1.0),
            blue: (blue as f32 / 255.0).clamp(0.0, 1.0),
        }
    }

    /// Clamps all components to the range [0.0, 1.0]
    pub fn clamp(&self) -> Self {
        Self {
            red: self.red.clamp(0.0, 1.0),
            green: self.green.clamp(0.0, 1.0),
            blue: self.blue.clamp(0.0, 1.0),
        }
    }

    /// Parses a 6-digit hex color string (e.g. "#FF0000" or "FF0000")
    pub fn parse_hex(hex: &str) -> Result<Self, ParseHexError> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(ParseHexError::InvalidSize);
        }
        Ok(RgbColor::new_from_u8(
            u8::from_str_radix(&hex[0..2], 16).map_err(|_| ParseHexError::InvalidRed)?,
            u8::from_str_radix(&hex[2..4], 16).map_err(|_| ParseHexError::InvalidGreen)?,
            u8::from_str_radix(&hex[4..6], 16).map_err(|_| ParseHexError::InvalidBlue)?,
        ))
    }
}

impl Default for RgbColor {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Display for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RgbColor(r={}, g={}, b={})", self.red, self.green, self.blue)
    }
}

impl From<RgbColor> for RgbaColor {
    fn from(color: RgbColor) -> Self {
        RgbaColor::new(color, 1.0)
    }
}

/// An RGBA color with f32 components in the range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder)]
pub struct RgbaColor {
    /// The RGB color components
    pub color: RgbColor,
    /// The alpha (opacity) component
    pub alpha: f32,
}

impl RgbaColor {
    /// Creates a new RGBA color from an RGB color and alpha
    pub const fn new(color: RgbColor, alpha: f32) -> Self {
        Self { color, alpha }
    }

    /// Creates a new RGBA color from individual f32 components
    pub const fn with_rgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            color: RgbColor::new(red, green, blue),
            alpha,
        }
    }

    /// Creates a fully transparent color
    pub fn transparent() -> Self {
        Self::new(RgbColor::default(), 0.0)
    }

    /// Clamps all components to the range [0.0, 1.0]
    pub fn clamp(&self) -> Self {
        Self {
            color: self.color.clamp(),
            alpha: self.alpha.clamp(0.0, 1.0),
        }
    }

    /// Parses a hex color string with optional alpha (e.g. "#FF0000" or "#FF000077")
    pub fn parse_hex_with_optional_alpha(hex: &str) -> Result<Self, ParseHexError> {
        Self::parse_hex(hex).or_else(|_| RgbColor::parse_hex(hex).map(|rgb| RgbaColor::new(rgb, 1.0)))
    }

    /// Parses an 8-digit hex color string (e.g. "#FF000077" or "FF000077")
    pub fn parse_hex(hex: &str) -> Result<Self, ParseHexError> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 8 {
            return Err(ParseHexError::InvalidSize);
        }
        let alpha = u8::from_str_radix(&hex[6..8], 16).map_err(|_| ParseHexError::InvalidAlpha)?;
        Ok(RgbaColor::new(
            RgbColor::new_from_u8(
                u8::from_str_radix(&hex[0..2], 16).map_err(|_| ParseHexError::InvalidRed)?,
                u8::from_str_radix(&hex[2..4], 16).map_err(|_| ParseHexError::InvalidGreen)?,
                u8::from_str_radix(&hex[4..6], 16).map_err(|_| ParseHexError::InvalidBlue)?,
            ),
            alpha as f32 / 255.0,
        ))
    }
}

impl Display for RgbaColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RgbaColor(r={}, g={}, b={} a={})",
            self.color.red, self.color.green, self.color.blue, self.alpha
        )
    }
}

impl From<RgbaColor> for gdk::RGBA {
    fn from(color: RgbaColor) -> Self {
        gdk::RGBA::new(color.color.red, color.color.green, color.color.blue, color.alpha)
    }
}

impl From<&RgbaColor> for gdk::RGBA {
    fn from(color: &RgbaColor) -> Self {
        gdk::RGBA::new(color.color.red, color.color.green, color.color.blue, color.alpha)
    }
}

impl From<&str> for RgbaColor {
    fn from(hex: &str) -> Self {
        RgbaColor::parse_hex_with_optional_alpha(hex)
            .unwrap_or_else(|_| panic!("Invalid hex color string: '{hex}'"))
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
        RgbaColor::parse_hex_with_optional_alpha(hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_color_from_rgb_color() {
        let rgb = RgbColor::new(0.5, 0.3, 0.7);
        let rgba: RgbaColor = rgb.into();
        assert_eq!(rgba.color.red, 0.5);
        assert_eq!(rgba.color.green, 0.3);
        assert_eq!(rgba.color.blue, 0.7);
        assert_eq!(rgba.alpha, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_hex() {
        let rgba: RgbaColor = "#FF0000".into();
        assert_eq!(rgba.color.red, 1.0);
        assert_eq!(rgba.color.green, 0.0);
        assert_eq!(rgba.color.blue, 0.0);
        assert_eq!(rgba.alpha, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_hex_with_alpha() {
        let rgba: RgbaColor = "#FF000077".into();
        assert_eq!(rgba.color.red, 1.0);
        assert_eq!(rgba.color.green, 0.0);
        assert_eq!(rgba.color.blue, 0.0);
        assert!((rgba.alpha - 0.4667).abs() < 0.01);
    }

    #[test]
    fn test_rgba_color_from_string() {
        let rgba: RgbaColor = String::from("#00FF00").into();
        assert_eq!(rgba.color.red, 0.0);
        assert_eq!(rgba.color.green, 1.0);
        assert_eq!(rgba.color.blue, 0.0);
    }

    #[test]
    fn test_rgba_color_from_str_trait() {
        let rgba = RgbaColor::from_str("#0000FF").unwrap();
        assert_eq!(rgba.color.red, 0.0);
        assert_eq!(rgba.color.green, 0.0);
        assert_eq!(rgba.color.blue, 1.0);
    }

    #[test]
    fn test_rgba_color_from_str_invalid() {
        let result = RgbaColor::from_str("invalid");
        assert!(result.is_err());
    }
}
