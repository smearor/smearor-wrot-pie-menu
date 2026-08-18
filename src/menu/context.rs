/// Context provided to widget factories, allowing them to trigger
/// pie menu events and access item metadata.
///
/// This struct is passed to the `MenuItemWidgetFactory::build` method
/// alongside the [`MenuItem`](crate::menu::MenuItem) reference. It
/// provides a callback channel for custom widgets to interact with
/// the pie menu's event system without needing direct access to the
/// widget implementation.
///
/// `MenuItemContext` is `!Clone`, `!Send`, and `!Sync` because
/// `trigger_event` contains a `Box<dyn Fn()>`. This is unproblematic
/// since `MenuItemContext` is not stored in `MenuItem` - it is
/// constructed fresh at build time and passed by reference to the
/// factory's `build` method.
pub struct MenuItemContext {
    /// The unique identifier of the menu item.
    pub id: String,
    /// The event name associated with the menu item.
    pub event: String,
    /// Callback to trigger the item's event via the pie menu message system.
    pub trigger_event: Box<dyn Fn()>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_context_fields() {
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "click".to_string(),
            trigger_event: Box::new(|| {}),
        };
        assert_eq!(context.id, "test");
        assert_eq!(context.event, "click");
    }

    #[test]
    fn test_trigger_event_invokes_callback() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        let context = MenuItemContext {
            id: "test".to_string(),
            event: "click".to_string(),
            trigger_event: Box::new(move || {
                called_clone.store(true, Ordering::Relaxed);
            }),
        };
        (context.trigger_event)();
        assert!(called.load(Ordering::Relaxed));
    }
}
