use crate::menu::item::DEFAULT_MENU_ITEM_RADIUS;
use crate::menu::item::MenuItem;
use crate::menu_widget::menu_item::error::AddMenuItemError;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::ops::Deref;

/// A collection of menu items indexed by their id
pub struct Menu(DashMap<String, MenuItem>);

impl Menu {
    /// Creates a new empty menu
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    /// Creates a builder for constructing a menu
    pub fn builder() -> MenuBuilder {
        MenuBuilder::default()
    }

    /// Updates the angle of the item with the given id.
    fn update_angle(&self, id: &str, angle: f32) {
        if let Some(mut entry) = self.get_mut(id) {
            entry.angle = angle;
        }
    }

    /// Checks whether the new item overlaps with any existing item on the ring.
    /// Returns `Ok(())` if `ring_radius` is zero (e.g. during initialization).
    pub fn validate_no_overlap(&self, new_item: &MenuItem, ring_radius: f32) -> Result<(), AddMenuItemError> {
        if ring_radius == 0.0 {
            return Ok(());
        }
        let new_angle_rad = new_item.angle.to_radians();
        let new_position = (new_angle_rad.cos(), new_angle_rad.sin());
        for entry in self.iter() {
            let existing_item = entry.value();
            if existing_item.id == new_item.id {
                continue;
            }
            let existing_angle_rad = existing_item.angle.to_radians();
            let existing_position = (existing_angle_rad.cos(), existing_angle_rad.sin());
            let distance = ((new_position.0 - existing_position.0).powi(2) + (new_position.1 - existing_position.1).powi(2)).sqrt();
            let min_distance = (new_item.radius.unwrap_or(DEFAULT_MENU_ITEM_RADIUS) + existing_item.radius.unwrap_or(DEFAULT_MENU_ITEM_RADIUS)) / ring_radius;
            if distance < min_distance {
                return Err(AddMenuItemError::ItemOverlap {
                    id: new_item.id.clone(),
                    overlapping_with: existing_item.id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Validates that no items in the menu overlap with each other.
    /// Used after `redistribute_angles` in `add_menu_item_auto` to check
    /// the full configuration, since all flexible items may have shifted.
    pub fn validate_all_no_overlap(&self, ring_radius: f32) -> Result<(), AddMenuItemError> {
        for item in self.iter() {
            self.validate_no_overlap(item.value(), ring_radius)?;
        }
        Ok(())
    }

    /// Redistributes non-fixed items proportionally in the gaps between fixed items.
    /// Wider angular segments receive proportionally more items.
    pub fn redistribute_angles(&self) {
        let items: Vec<MenuItem> = self.iter().map(|e| e.value().clone()).collect();
        let fixed: Vec<&MenuItem> = items.iter().filter(|item| item.fixed_position).collect();
        let flexible: Vec<&MenuItem> = items.iter().filter(|item| !item.fixed_position).collect();

        if fixed.is_empty() {
            let total = items.len() as f32;
            for (index, item) in items.iter().enumerate() {
                let angle = 360.0 * index as f32 / total;
                self.update_angle(&item.id, angle);
            }
            return;
        }

        let mut fixed_sorted: Vec<MenuItem> = fixed.into_iter().cloned().collect();
        for item in &mut fixed_sorted {
            item.angle = item.angle.rem_euclid(360.0);
        }
        fixed_sorted.sort_by(|a, b| a.angle.partial_cmp(&b.angle).unwrap_or(Ordering::Equal));

        let segment_count = fixed_sorted.len();
        let mut segments: Vec<(f32, f32)> = Vec::with_capacity(segment_count);
        for window in fixed_sorted.windows(2) {
            segments.push((window[0].angle, window[1].angle));
        }
        if let (Some(first), Some(last)) = (fixed_sorted.first(), fixed_sorted.last()) {
            segments.push((last.angle, first.angle + 360.0));
        }

        let total_width: f32 = segments.iter().map(|(start, end)| end - start).sum();

        if total_width == 0.0 {
            let flexible_count = flexible.len() as f32;
            if flexible_count > 0.0 {
                for (index, item) in flexible.iter().enumerate() {
                    let angle = 360.0 * index as f32 / flexible_count;
                    self.update_angle(&item.id, angle);
                }
            }
            return;
        }

        let flexible_count = flexible.len();

        let mut allocations: Vec<usize> = segments
            .iter()
            .map(|(start, end)| {
                let width = end - start;
                (flexible_count as f32 * width / total_width).floor() as usize
            })
            .collect();

        let allocated: usize = allocations.iter().sum();
        let remainder = flexible_count.saturating_sub(allocated);
        if remainder > 0 {
            let mut remainders: Vec<(usize, f32)> = segments
                .iter()
                .enumerate()
                .map(|(index, (start, end))| {
                    let width = end - start;
                    let fractional = flexible_count as f32 * width / total_width - (flexible_count as f32 * width / total_width).floor();
                    (index, fractional)
                })
                .collect();
            remainders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            for (index, _) in remainders.iter().take(remainder) {
                allocations[*index] += 1;
            }
        }

        let mut flexible_index = 0;
        for (segment_index, (start_angle, end_angle)) in segments.iter().enumerate() {
            let count = allocations[segment_index];
            if count == 0 {
                continue;
            }
            let segment_size = end_angle - start_angle;
            for offset in 0..count {
                let angle = start_angle + segment_size * (offset + 1) as f32 / (count + 1) as f32;
                let item = &flexible[flexible_index];
                self.update_angle(&item.id, angle.rem_euclid(360.0));
                flexible_index += 1;
            }
        }
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Menu {
    type Target = DashMap<String, MenuItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Menu {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for entry in self.0.iter() {
            map.serialize_entry(&entry.key(), &entry.value())?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Menu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::collections::HashMap;
        let items: HashMap<String, MenuItem> = HashMap::deserialize(deserializer)?;
        let dash_map = DashMap::new();
        for (id, item) in items {
            dash_map.insert(id, item);
        }
        Ok(Menu(dash_map))
    }
}

/// Builder for constructing a `Menu` by adding items one at a time
#[derive(Default)]
pub struct MenuBuilder {
    items: DashMap<String, MenuItem>,
}

impl MenuBuilder {
    /// Adds a menu item to the builder
    pub fn item(self, item: MenuItem) -> Self {
        self.items.insert(item.id.clone(), item);
        self
    }

    /// Builds the final `Menu`
    pub fn build(self) -> Menu {
        Menu(self.items)
    }
}
