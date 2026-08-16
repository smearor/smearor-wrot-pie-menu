use crate::menu::item::MenuItem;
use dashmap::DashMap;
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

impl Deref for Menu {
    type Target = DashMap<String, MenuItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
