//! Pie-Menu widget for touch gesture interface
//!
//! This crate provides a circular menu widget that appears when a pinch-to-zoom
//! gesture is detected. The menu items are arranged in a ring layout for easy
//! touch access. The widget is fully configurable — consumers add menu items
//! via the [`MenuItem`] API and receive events via [`PieMenuMessage`].

pub mod color;
pub mod menu;
pub mod menu_widget;
pub mod overlay_widget;

pub use menu::DEFAULT_MENU_ITEM_RADIUS;
pub use menu::MenuItem;
pub use menu_widget::PieMenuWidget;
pub use menu_widget::imp::PieMenuWidgetImpl;
pub use menu_widget::rotation::handler::RotationHandler;
pub use overlay_widget::PieMenuOverlayWidget;
pub use overlay_widget::imp::PieMenuOverlayWidgetImpl;
pub use overlay_widget::message::PieMenuMessage;
