use crate::PieMenuWidgetImpl;
use crate::menu::MenuItem;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::ObjectSubclassExt;
use std::sync::atomic::Ordering;

impl PieMenuWidgetImpl {
    /// Returns the current rotation in degrees.
    pub(crate) fn rotation(&self) -> f32 {
        self.rotation.load(Ordering::Relaxed)
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
    pub(crate) fn is_keyboard_selected(&self, id: &str) -> bool {
        self.keyboard_selection.borrow().as_ref().is_some_and(|selected| selected == id)
    }

    /// Sets the submenu navigation stack and triggers a redraw.
    /// An empty stack means the main ring is active.
    pub(crate) fn set_submenu_stack(&self, stack: Vec<String>) {
        *self.submenu_stack.borrow_mut() = stack;
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
        self.menu_items
            .find_item_recursive(parent_id)
            .and_then(|item| item.submenu)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, angle: f32) -> MenuItem {
        MenuItem::builder().id(id).label(id).icon_name("icon").angle(angle).event(id).build()
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
        // At 0°, both are 10° away — the result depends on iteration order (DashMap).
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
        MenuItem::builder()
            .id(id)
            .label(id)
            .icon_name("icon")
            .angle(angle)
            .event(id)
            .enabled(false)
            .build()
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
