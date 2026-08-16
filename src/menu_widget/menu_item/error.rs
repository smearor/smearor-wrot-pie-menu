use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AddMenuItemError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
}

#[derive(Debug, Clone, Error)]
pub enum RemoveMenuItemError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
}
