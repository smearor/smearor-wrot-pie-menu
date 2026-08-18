use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

/// The dimensions of a widget slot on the ring.
///
/// Used when a widget requires a non-square allocation
/// (e.g., a wide slider or a tall gauge). When `None`,
/// the item's `radius` is used for a square allocation
/// of `2 * radius` pixels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypedBuilder, PartialEq)]
pub struct ItemSize {
    /// The width of the slot in pixels.
    pub width: f32,
    /// The height of the slot in pixels.
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_size_builder() {
        let size = ItemSize::builder().width(80.0).height(40.0).build();
        assert_eq!(size.width, 80.0);
        assert_eq!(size.height, 40.0);
    }

    #[test]
    fn test_item_size_copy() {
        let size = ItemSize::builder().width(100.0).height(50.0).build();
        let copied = size;
        assert_eq!(size, copied);
    }
}
