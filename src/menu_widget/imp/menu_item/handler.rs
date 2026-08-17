use crate::PieMenuWidgetImpl;
use crate::menu::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use crate::menu_widget::menu_item::error::RemoveMenuItemError;
use crate::menu_widget::menu_item::error::SetMenuItemEnabledError;
use crate::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::ObjectSubclassExt;
use std::sync::atomic::Ordering;

impl PieMenuMenuItemHandler for PieMenuWidgetImpl {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        self.menu_items.insert(menu_item.id.clone(), menu_item.clone());
        let ring_radius = self.radius.load(Ordering::Relaxed);
        if let Err(error) = self.menu_items.validate_no_overlap(&menu_item, ring_radius) {
            self.menu_items.remove(&menu_item.id);
            return Err(error);
        }
        self.obj().queue_draw();
        Ok(())
    }

    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError> {
        self.menu_items.remove(id);
        self.obj().queue_draw();
        Ok(())
    }

    fn remove_all_menu_items(&self) {
        self.menu_items.clear();
        self.obj().queue_draw();
    }

    fn menu_item_count(&self) -> usize {
        self.menu_items.len()
    }

    fn set_menu_item_enabled(&self, id: &str, enabled: bool) -> Result<(), SetMenuItemEnabledError> {
        let mut item = self
            .menu_items
            .get_mut(id)
            .ok_or(SetMenuItemEnabledError::NotFound { id: id.to_string() })?;
        item.enabled = enabled;
        drop(item);
        self.obj().queue_draw();
        Ok(())
    }

    fn add_menu_item_auto(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError> {
        let previous_item = self.menu_items.get(&menu_item.id).map(|entry| entry.value().clone());
        let angle_snapshot: Vec<(String, f32)> = self
            .menu_items
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().angle))
            .collect();

        self.menu_items.insert(menu_item.id.clone(), menu_item.clone());
        self.menu_items.redistribute_angles();

        let ring_radius = self.radius.load(Ordering::Relaxed);
        if let Err(error) = self.menu_items.validate_all_no_overlap(ring_radius) {
            if let Some(previous) = previous_item {
                self.menu_items.insert(menu_item.id.clone(), previous);
            } else {
                self.menu_items.remove(&menu_item.id);
            }
            for (id, angle) in &angle_snapshot {
                if let Some(mut entry) = self.menu_items.get_mut(id) {
                    entry.angle = *angle;
                }
            }
            return Err(error);
        }

        self.obj().queue_draw();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::MenuItem;
    use crate::menu::Menu;
    use crate::menu_widget::menu_item::error::AddMenuItemError;

    fn make_item(id: &str, angle: f32) -> MenuItem {
        MenuItem::builder()
            .id(id)
            .label(id)
            .icon_name("icon")
            .angle(angle)
            .event(id)
            .build()
    }

    #[test]
    fn test_remove_all_menu_items() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        menu.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(menu.len(), 2);
        menu.clear();
        assert_eq!(menu.len(), 0);
    }

    #[test]
    fn test_remove_all_menu_items_empty() {
        let menu = Menu::new();
        menu.clear();
        assert_eq!(menu.len(), 0);
    }

    #[test]
    fn test_menu_item_count() {
        let menu = Menu::new();
        assert_eq!(menu.len(), 0);
        menu.insert("a".to_string(), make_item("a", 0.0));
        assert_eq!(menu.len(), 1);
        menu.insert("b".to_string(), make_item("b", 90.0));
        assert_eq!(menu.len(), 2);
        menu.remove("a");
        assert_eq!(menu.len(), 1);
    }

    #[test]
    fn test_set_menu_item_enabled() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        assert!(menu.get("a").unwrap().enabled);
        {
            let mut item = menu.get_mut("a").unwrap();
            item.enabled = false;
        }
        assert!(!menu.get("a").unwrap().enabled);
    }

    #[test]
    fn test_set_menu_item_enabled_not_found() {
        let menu = Menu::new();
        assert!(menu.get_mut("nonexistent").is_none());
    }

    #[test]
    fn test_disabled_item_skipped_in_iteration() {
        let menu = Menu::new();
        let mut enabled_count = 0;
        menu.insert("a".to_string(), make_item("a", 0.0));
        {
            let mut item = menu.get_mut("a").unwrap();
            item.enabled = false;
        }
        for item in menu.iter() {
            if item.enabled {
                enabled_count += 1;
            }
        }
        assert_eq!(enabled_count, 0);
    }

    #[test]
    fn test_validate_no_overlap_no_items() {
        let menu = Menu::new();
        let item = make_item("a", 0.0);
        assert!(menu.validate_no_overlap(&item, 160.0).is_ok());
    }

    #[test]
    fn test_validate_no_overlap_zero_ring_radius() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        let item = make_item("b", 0.0);
        assert!(menu.validate_no_overlap(&item, 0.0).is_ok());
    }

    #[test]
    fn test_validate_no_overlap_self() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        let item = make_item("a", 0.0);
        assert!(menu.validate_no_overlap(&item, 160.0).is_ok());
    }

    #[test]
    fn test_validate_no_overlap_distant_items() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        let item = make_item("b", 180.0);
        assert!(menu.validate_no_overlap(&item, 160.0).is_ok());
    }

    #[test]
    fn test_validate_no_overlap_overlapping_items() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        let item = make_item("b", 0.0);
        let result = menu.validate_no_overlap(&item, 160.0);
        assert!(result.is_err());
        match result {
            Err(AddMenuItemError::ItemOverlap { id, overlapping_with }) => {
                assert_eq!(id, "b");
                assert_eq!(overlapping_with, "a");
            }
            _ => panic!("Expected ItemOverlap error"),
        }
    }

    #[test]
    fn test_validate_all_no_overlap() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        menu.insert("b".to_string(), make_item("b", 120.0));
        menu.insert("c".to_string(), make_item("c", 240.0));
        assert!(menu.validate_all_no_overlap(160.0).is_ok());
    }

    #[test]
    fn test_validate_all_no_overlap_with_overlap() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        menu.insert("b".to_string(), make_item("b", 0.0));
        assert!(menu.validate_all_no_overlap(160.0).is_err());
    }

    #[test]
    fn test_add_menu_item_rollback_on_overlap() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 0.0));
        let overlapping_item = make_item("b", 0.0);
        let result = menu.validate_no_overlap(&overlapping_item, 160.0);
        assert!(result.is_err());
        // Simulate rollback: insert then remove on validation failure
        menu.insert("b".to_string(), overlapping_item);
        if menu.validate_no_overlap(&menu.get("b").unwrap().clone(), 160.0).is_err() {
            menu.remove("b");
        }
        assert_eq!(menu.len(), 1);
        assert!(menu.get("b").is_none());
    }

    #[test]
    fn test_redistribute_angles_no_fixed() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 10.0));
        menu.insert("b".to_string(), make_item("b", 20.0));
        menu.insert("c".to_string(), make_item("c", 30.0));
        menu.redistribute_angles();
        let angles: Vec<f32> = ["a", "b", "c"].iter().map(|id| menu.get(*id).unwrap().angle).collect();
        let mut sorted = angles.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // Angles should be 0, 120, 240 (in some order due to DashMap iteration)
        assert!((sorted[0] - 0.0).abs() < 0.01);
        assert!((sorted[1] - 120.0).abs() < 0.01);
        assert!((sorted[2] - 240.0).abs() < 0.01);
    }

    #[test]
    fn test_redistribute_angles_single_item() {
        let menu = Menu::new();
        menu.insert("a".to_string(), make_item("a", 42.0));
        menu.redistribute_angles();
        assert!((menu.get("a").unwrap().angle - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_redistribute_angles_with_fixed() {
        let menu = Menu::new();
        let fixed_a = MenuItem::builder().id("fixed_a").label("F").icon_name("icon").angle(0.0).event("e").fixed_position(true).build();
        let fixed_b = MenuItem::builder().id("fixed_b").label("F").icon_name("icon").angle(180.0).event("e").fixed_position(true).build();
        let flex_a = make_item("flex_a", 999.0);
        let flex_b = make_item("flex_b", 999.0);
        menu.insert("fixed_a".to_string(), fixed_a);
        menu.insert("fixed_b".to_string(), fixed_b);
        menu.insert("flex_a".to_string(), flex_a);
        menu.insert("flex_b".to_string(), flex_b);
        menu.redistribute_angles();
        assert!((menu.get("fixed_a").unwrap().angle - 0.0).abs() < 0.01);
        assert!((menu.get("fixed_b").unwrap().angle - 180.0).abs() < 0.01);
        let flex_a_angle = menu.get("flex_a").unwrap().angle;
        let flex_b_angle = menu.get("flex_b").unwrap().angle;
        // One flexible item in each segment (0-180 and 180-360)
        assert!(flex_a_angle > 0.0 && flex_a_angle < 180.0);
        assert!(flex_b_angle > 180.0 && flex_b_angle < 360.0);
    }

    #[test]
    fn test_redistribute_angles_all_fixed_same_angle() {
        let menu = Menu::new();
        let fixed_a = MenuItem::builder().id("fixed_a").label("F").icon_name("icon").angle(0.0).event("e").fixed_position(true).build();
        let fixed_b = MenuItem::builder().id("fixed_b").label("F").icon_name("icon").angle(0.0).event("e").fixed_position(true).build();
        let flex = make_item("flex", 999.0);
        menu.insert("fixed_a".to_string(), fixed_a);
        menu.insert("fixed_b".to_string(), fixed_b);
        menu.insert("flex".to_string(), flex);
        menu.redistribute_angles();
        // Two fixed at 0° create segments (0,0) and (0,360).
        // The 360° segment gets the flexible item at its midpoint: 180°.
        let flex_angle = menu.get("flex").unwrap().angle;
        assert!((flex_angle - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_add_menu_item_auto_even_distribution() {
        let menu = Menu::new();
        let item_a = make_item("a", 999.0);
        menu.insert("a".to_string(), item_a);
        menu.redistribute_angles();
        // Single item always at 0°
        assert!((menu.get("a").unwrap().angle - 0.0).abs() < 0.01);

        let item_b = make_item("b", 999.0);
        menu.insert("b".to_string(), item_b);
        menu.redistribute_angles();
        // Two items at 0° and 180° (in some order)
        let angle_a = menu.get("a").unwrap().angle;
        let angle_b = menu.get("b").unwrap().angle;
        let diff = (angle_a - angle_b).abs().min(360.0 - (angle_a - angle_b).abs());
        assert!((diff - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_fixed_position_field_default() {
        let item = MenuItem::builder().id("test").label("Test").icon_name("icon").angle(0.0).event("event").build();
        assert!(!item.fixed_position);
    }

    #[test]
    fn test_fixed_position_field_set() {
        let item = MenuItem::builder()
            .id("test")
            .label("Test")
            .icon_name("icon")
            .angle(0.0)
            .event("event")
            .fixed_position(true)
            .build();
        assert!(item.fixed_position);
    }
}
