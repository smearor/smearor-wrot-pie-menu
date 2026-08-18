use thiserror::Error;

/// Error returned when a submenu operation fails.
#[derive(Debug, Clone, Error)]
pub enum SubmenuError {
    /// No menu item with the given id was found.
    #[error("Menu item not found: {id}")]
    NotFound { id: String },
    /// The menu item does not have a submenu.
    #[error("Menu item '{id}' has no submenu")]
    NoSubmenu { id: String },
    /// The maximum submenu depth has been reached.
    #[error("Maximum submenu depth reached: {max_depth}")]
    MaxDepthReached { max_depth: u32 },
    /// No submenu is currently open.
    #[error("No submenu is currently open")]
    NoSubmenuOpen,
    /// Submenu items overlap after redistribution.
    #[error("Submenu items overlap after redistribution for parent '{parent_id}'")]
    ItemOverlap { parent_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_display() {
        let error = SubmenuError::NotFound { id: "test".to_string() };
        assert_eq!(error.to_string(), "Menu item not found: test");
    }

    #[test]
    fn test_no_submenu_display() {
        let error = SubmenuError::NoSubmenu { id: "test".to_string() };
        assert_eq!(error.to_string(), "Menu item 'test' has no submenu");
    }

    #[test]
    fn test_max_depth_reached_display() {
        let error = SubmenuError::MaxDepthReached { max_depth: 3 };
        assert_eq!(error.to_string(), "Maximum submenu depth reached: 3");
    }

    #[test]
    fn test_no_submenu_open_display() {
        let error = SubmenuError::NoSubmenuOpen;
        assert_eq!(error.to_string(), "No submenu is currently open");
    }

    #[test]
    fn test_item_overlap_display() {
        let error = SubmenuError::ItemOverlap {
            parent_id: "parent".to_string(),
        };
        assert_eq!(error.to_string(), "Submenu items overlap after redistribution for parent 'parent'");
    }

    #[test]
    fn test_error_clone() {
        let error = SubmenuError::NotFound { id: "test".to_string() };
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }
}
