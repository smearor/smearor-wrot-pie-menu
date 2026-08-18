use serde::Deserialize;
use serde::Serialize;

/// Messages sent from the pie menu to the consumer application
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieMenuMessage {
    /// The pie menu was opened
    Opened,
    /// The pie menu was closed
    Closed,
    /// Rotation delta in degrees (from rotation gesture)
    Rotate(f32),
    /// Custom event triggered by clicking a menu item.
    /// The string is the `event` field of the clicked `MenuItem`.
    Event(String),
    /// A submenu was opened. Contains the parent item's id.
    SubmenuOpened(String),
    /// The submenu was closed, returning to the parent ring.
    /// Contains the parent item's id.
    SubmenuClosed(String),
}
