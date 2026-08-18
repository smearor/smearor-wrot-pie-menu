use crate::PieMenuWidgetImpl;
use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use gtk4::prelude::Cast;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::ObjectSubclassExt;
use std::sync::atomic::Ordering;
use tracing::debug;

impl PieMenuWidgetImpl {
    /// Returns the current rotation in degrees.
    pub(crate) fn rotation(&self) -> f32 {
        self.rotation.load(Ordering::Relaxed)
    }

    /// Builds widgets for all menu items that don't have a cached widget yet.
    ///
    /// This is called during the snapshot pass. Widgets are built once,
    /// registered as children of `PieMenuWidget` via `set_parent`,
    /// and stored in `item_widgets`. Subsequent renders reuse the cache.
    ///
    /// Widgets for items on the active ring are made visible; all others
    /// are hidden.
    pub(crate) fn ensure_item_widgets(&self) {
        let pie_widget = self.obj();
        let registry = self.widget_registry.borrow();
        let mut item_widgets = self.item_widgets.borrow_mut();

        let submenu_stack = self.submenu_stack.borrow().clone();

        // Collect all items that should have widgets: top-level + submenu items
        // of every open submenu level.
        let mut all_items: Vec<MenuItem> = self.menu_items.iter().map(|e| e.value().clone()).collect();

        for parent_id in &submenu_stack {
            if let Some(parent_item) = self.menu_items.find_item_recursive(parent_id)
                && let Some(submenu) = &parent_item.submenu
            {
                all_items.extend(submenu.iter().cloned());
            }
        }

        // Build missing widgets
        let mut created_any = false;
        for item in &all_items {
            if item_widgets.contains_key(&item.id) {
                continue;
            }

            let type_name = item.widget_type.as_deref().unwrap_or("circle");
            let Some(factory) = registry.get(type_name) else {
                debug!("No factory registered for widget type '{}', skipping item '{}'", type_name, item.id);
                continue;
            };

            let context = MenuItemContext {
                id: item.id.clone(),
                event: item.event.clone(),
                trigger_event: Box::new(|| {}),
            };

            let widget = factory.build(item, &context);
            let parent_widget = pie_widget.upcast_ref::<gtk4::Widget>();
            widget.set_parent(parent_widget);
            widget.set_visible(false);
            debug!("Created widget of type '{}' for item '{}'", type_name, item.id);
            item_widgets.insert(item.id.clone(), widget);
            created_any = true;
        }

        // Update visibility: items on ALL active rings (top-level + every
        // open submenu level) should be visible, not just the innermost.
        let visible_ids: std::collections::HashSet<&str> = all_items.iter().map(|i| i.id.as_str()).collect();

        let keyboard_selection = self.keyboard_selection.borrow().clone();

        for (id, widget) in item_widgets.iter() {
            let should_be_visible = visible_ids.contains(id.as_str());
            widget.set_visible(should_be_visible);

            // Update keyboard selection highlight on custom widgets
            let is_selected = keyboard_selection.as_ref().is_some_and(|selected| selected == id);
            if let Some(circle) = widget.downcast_ref::<crate::menu::circle_item_widget::CircleItemWidget>() {
                circle.set_selected(is_selected);
            } else if let Some(square) = widget.downcast_ref::<crate::menu::square_item_widget::SquareItemWidget>() {
                square.set_selected(is_selected);
            } else {
                // Fallback for standard GTK widgets (e.g. Button): use CSS class
                if is_selected {
                    widget.add_css_class("selected");
                } else {
                    widget.remove_css_class("selected");
                }
                widget.queue_draw();
            }
        }

        // Drop borrows before triggering layout to avoid reentrancy panics.
        drop(item_widgets);
        drop(registry);

        // If new widgets were created, trigger a new allocate pass so
        // `size_allocate` positions them. `queue_resize` is insufficient
        // here because the widget's own size request does not change -
        // `queue_allocate` forces a reallocation regardless.
        if created_any {
            pie_widget.queue_allocate();
        }
    }

    /// Unparents and removes all cached item widgets.
    ///
    /// Called by `refresh_widgets` to force a full rebuild on the
    /// next layout pass. Mutations are deferred to the next event
    /// loop iteration via `glib::idle_add_local_once` by the caller
    /// to prevent `RefCell` reentrancy panics.
    pub(crate) fn clear_item_widgets(&self) {
        let mut item_widgets = self.item_widgets.borrow_mut();
        for (_, widget) in item_widgets.drain() {
            widget.unparent();
        }
    }

    /// Unparents and removes the cached widget for a single item.
    ///
    /// Called by `set_widget_config` to force a rebuild of a single
    /// item's widget on the next layout pass.
    pub(crate) fn remove_item_widget(&self, id: &str) {
        let mut item_widgets = self.item_widgets.borrow_mut();
        if let Some(widget) = item_widgets.remove(id) {
            widget.unparent();
        }
    }

    /// Computes the next selection ID when cycling by `direction` (-1 for CCW, +1 for CW).
    /// Skips disabled items. Returns `None` if the menu has no enabled items.
    /// Does not mutate state or trigger redraw.
    fn compute_next_selection(&self, direction: i32) -> Option<String> {
        let mut items = self.active_ring_items();
        items.retain(|item| item.enabled);
        if items.is_empty() {
            return None;
        }

        items.sort_by(|a, b| a.angle.total_cmp(&b.angle));

        let current_id = self.keyboard_selection.borrow().clone();
        let current_index = current_id.and_then(|id| items.iter().position(|item| item.id == id));

        let next_index = match current_index {
            Some(index) => (index as i32 + direction).rem_euclid(items.len() as i32) as usize,
            None => 0,
        };

        Some(items[next_index].id.clone())
    }

    /// Computes the first enabled item ID (smallest angle). Returns `None` if the menu
    /// has no enabled items. Does not mutate state or trigger redraw.
    fn compute_first_selection(&self) -> Option<String> {
        let mut items = self.active_ring_items();
        items.retain(|item| item.enabled);
        if items.is_empty() {
            return None;
        }

        items.sort_by(|a, b| a.angle.total_cmp(&b.angle));
        Some(items[0].id.clone())
    }

    /// Cycles the keyboard selection by `direction` (-1 for CCW, +1 for CW).
    /// Items are sorted by angle to ensure deterministic navigation order.
    pub(crate) fn cycle_selection(&self, direction: i32) {
        if let Some(id) = self.compute_next_selection(direction) {
            *self.keyboard_selection.borrow_mut() = Some(id);
            self.obj().queue_draw();
        }
    }

    /// Selects the first item (smallest angle, typically 0°).
    pub(crate) fn select_first_item(&self) {
        if let Some(id) = self.compute_first_selection() {
            *self.keyboard_selection.borrow_mut() = Some(id);
            self.obj().queue_draw();
        }
    }

    /// Finds the enabled menu item whose angle is closest to `target_angle` (in degrees).
    /// Returns the item ID, or `None` if the menu has no enabled items.
    pub(crate) fn find_nearest_item(&self, target_angle: f32) -> Option<String> {
        let mut items = self.active_ring_items();
        items.retain(|item| item.enabled);
        if items.is_empty() {
            return None;
        }

        let nearest = items.iter().min_by(|a, b| {
            let angle_a = a.angle.rem_euclid(360.0);
            let dist_a = (angle_a - target_angle).abs().min(360.0 - (angle_a - target_angle).abs());
            let angle_b = b.angle.rem_euclid(360.0);
            let dist_b = (angle_b - target_angle).abs().min(360.0 - (angle_b - target_angle).abs());
            dist_a.total_cmp(&dist_b)
        })?;

        Some(nearest.id.clone())
    }

    /// Sets the keyboard selection to the given item ID and triggers a redraw.
    pub(crate) fn set_keyboard_selection(&self, id: String) {
        *self.keyboard_selection.borrow_mut() = Some(id);
        self.obj().queue_draw();
    }

    /// Returns the currently keyboard-selected item ID, if any.
    pub(crate) fn keyboard_selection(&self) -> Option<String> {
        self.keyboard_selection.borrow().clone()
    }

    /// Returns whether the item with the given ID is the current keyboard selection.
    #[allow(unused)]
    pub(crate) fn is_keyboard_selected(&self, id: &str) -> bool {
        self.keyboard_selection.borrow().as_ref().is_some_and(|selected| selected == id)
    }

    /// Sets the submenu navigation stack and triggers a redraw.
    /// An empty stack means the main ring is active.
    pub(crate) fn set_submenu_stack(&self, stack: Vec<String>) {
        *self.submenu_stack.borrow_mut() = stack;
        self.obj().queue_allocate();
        self.obj().queue_draw();
    }

    /// Returns the current submenu depth (0 = main ring).
    pub(crate) fn submenu_depth(&self) -> u32 {
        self.submenu_stack.borrow().len() as u32
    }

    /// Returns the items of the currently active ring.
    /// When no submenu is open, returns top-level items.
    /// When a submenu is open, returns the items of the innermost open submenu.
    pub(crate) fn active_ring_items(&self) -> Vec<MenuItem> {
        let submenu_stack = self.submenu_stack.borrow();
        let Some(parent_id) = submenu_stack.last() else {
            return self.menu_items.iter().map(|entry| entry.value().clone()).collect();
        };
        self.menu_items.find_item_recursive(parent_id).and_then(|item| item.submenu).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, angle: f32) -> MenuItem {
        MenuItem::builder().id(id).angle(angle).event(id).build()
    }

    #[test]
    fn test_keyboard_selection_default_none() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.keyboard_selection().is_none());
    }

    #[test]
    fn test_cycle_selection_empty_menu() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.compute_next_selection(1).is_none());
    }

    #[test]
    fn test_cycle_selection_forward() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("b".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("b".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("c".to_string()));
    }

    #[test]
    fn test_cycle_selection_backward() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        *imp.keyboard_selection.borrow_mut() = Some("b".to_string());
        assert_eq!(imp.compute_next_selection(-1), Some("a".to_string()));
    }

    #[test]
    fn test_cycle_selection_wraps_around() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("b".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("b".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert_eq!(imp.compute_next_selection(-1), Some("b".to_string()));
    }

    #[test]
    fn test_cycle_selection_deterministic_order() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("b".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("b".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("c".to_string()));
    }

    #[test]
    fn test_select_first_item() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(imp.compute_first_selection(), Some("a".to_string()));
    }

    #[test]
    fn test_select_first_item_empty() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.compute_first_selection().is_none());
    }

    #[test]
    fn test_find_nearest_item() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        assert_eq!(imp.find_nearest_item(5.0), Some("a".to_string()));
        assert_eq!(imp.find_nearest_item(85.0), Some("b".to_string()));
        assert_eq!(imp.find_nearest_item(175.0), Some("c".to_string()));
    }

    #[test]
    fn test_find_nearest_item_empty() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.find_nearest_item(0.0).is_none());
    }

    #[test]
    fn test_find_nearest_item_wraparound() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 10.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 350.0));
        // At 0°, both are 10° away - the result depends on iteration order (DashMap).
        // Test with non-ambiguous angles instead:
        assert_eq!(imp.find_nearest_item(5.0), Some("a".to_string()));
        assert_eq!(imp.find_nearest_item(355.0), Some("b".to_string()));
    }

    #[test]
    fn test_is_keyboard_selected_none() {
        let imp = PieMenuWidgetImpl::default();
        assert!(!imp.is_keyboard_selected("a"));
    }

    #[test]
    fn test_is_keyboard_selected_with_id() {
        let imp = PieMenuWidgetImpl::default();
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert!(imp.is_keyboard_selected("a"));
        assert!(!imp.is_keyboard_selected("b"));
    }

    fn make_disabled_item(id: &str, angle: f32) -> MenuItem {
        MenuItem::builder().id(id).angle(angle).event(id).enabled(false).build()
    }

    #[test]
    fn test_cycle_selection_skips_disabled() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_disabled_item("b", 90.0));
        imp.menu_items.insert("c".to_string(), make_item("c", 180.0));
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("a".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("c".to_string()));
        *imp.keyboard_selection.borrow_mut() = Some("c".to_string());
        assert_eq!(imp.compute_next_selection(1), Some("a".to_string()));
    }

    #[test]
    fn test_select_first_item_skips_disabled() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_disabled_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(imp.compute_first_selection(), Some("b".to_string()));
    }

    #[test]
    fn test_find_nearest_item_skips_disabled() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_disabled_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(imp.find_nearest_item(5.0), Some("b".to_string()));
    }

    #[test]
    fn test_all_disabled_returns_none() {
        let imp = PieMenuWidgetImpl::default();
        imp.menu_items.insert("a".to_string(), make_disabled_item("a", 0.0));
        imp.menu_items.insert("b".to_string(), make_disabled_item("b", 90.0));
        assert!(imp.compute_next_selection(1).is_none());
        assert!(imp.compute_first_selection().is_none());
        assert!(imp.find_nearest_item(0.0).is_none());
    }

    #[test]
    fn test_submenu_stack_default_empty() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.submenu_stack.borrow().is_empty());
        assert_eq!(imp.submenu_depth(), 0);
    }

    #[test]
    fn test_submenu_stack_push_pop() {
        let imp = PieMenuWidgetImpl::default();
        imp.submenu_stack.borrow_mut().push("parent".to_string());
        assert_eq!(imp.submenu_depth(), 1);
        imp.submenu_stack.borrow_mut().clear();
        assert_eq!(imp.submenu_depth(), 0);
    }
}
