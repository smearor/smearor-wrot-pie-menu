use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// A submenu nesting level - `0` is the main ring, `1` the first
/// submenu, and so on.
///
/// The index marks a depth in the `submenu_stack`, not a count or a
/// length: a `SubmenuLevel` cannot be passed where a pixel radius or
/// an angle is expected. Serializes as a bare number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubmenuLevel(pub u32);

impl SubmenuLevel {
    /// The main ring - the root level of the menu.
    pub const MAIN_RING: Self = Self(0);

    /// Creates a level from its depth index.
    pub const fn new(level: u32) -> Self {
        Self(level)
    }

    /// The level index (`0` = main ring).
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// The next deeper level - the submenu opened below this one.
    pub const fn deeper(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u32> for SubmenuLevel {
    fn from(level: u32) -> Self {
        Self(level)
    }
}

impl From<SubmenuLevel> for u32 {
    fn from(level: SubmenuLevel) -> Self {
        level.0
    }
}

impl Display for SubmenuLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SubmenuLevel;

    /// `deeper` walks one level down the stack.
    #[test]
    fn deeper_steps_down_one_level() {
        assert_eq!(SubmenuLevel::MAIN_RING.deeper(), SubmenuLevel::new(1));
        assert_eq!(SubmenuLevel::new(2).deeper().as_u32(), 3);
    }
}
