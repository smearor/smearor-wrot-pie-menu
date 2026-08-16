use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ShowPieMenuError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
}

#[derive(Debug, Clone, Error)]
pub enum HidePieMenuError {
    #[error("Failed to borrow menu widget")]
    MenuWidgetNotAvailable,
}
