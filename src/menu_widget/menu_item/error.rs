use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AddMenuItemError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
    /// The new item overlaps with an existing item on the ring.
    #[error("Menu item '{id}' overlaps with existing item '{overlapping_with}'")]
    ItemOverlap { id: String, overlapping_with: String },
}

#[derive(Debug, Clone, Error)]
pub enum RemoveMenuItemError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
}

/// Error returned when setting the enabled state of a menu item that does not exist.
#[derive(Debug, Clone, Error)]
pub enum SetMenuItemEnabledError {
    /// No menu item with the given id was found.
    #[error("Menu item not found: {id}")]
    NotFound { id: String },
}

/// Error returned when updating a menu item that does not exist.
#[derive(Debug, Clone, Error)]
pub enum UpdateMenuItemError {
    /// No menu item with the given id was found.
    #[error("Menu item not found: {id}")]
    NotFound { id: String },
    /// The updated item overlaps with an existing item on the ring.
    #[error("Menu item '{id}' overlaps with existing item '{overlapping_with}'")]
    ItemOverlap { id: String, overlapping_with: String },
}
