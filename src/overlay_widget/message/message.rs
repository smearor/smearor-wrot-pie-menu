use serde::Deserialize;
use serde::Serialize;

/// Messages sent from the pie menu to the consumer application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieMenuMessage {
    /// Rotation delta in degrees (from rotation gesture)
    Rotate(f32),
    /// Custom event triggered by clicking a menu item.
    /// The string is the `event` field of the clicked `MenuItem`.
    Event(String),
}
