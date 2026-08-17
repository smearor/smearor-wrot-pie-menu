use crate::menu::item::MenuItem;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;
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
