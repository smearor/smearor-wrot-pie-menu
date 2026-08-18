use thiserror::Error;

/// Error returned when setting widget configuration on a menu item fails.
#[derive(Debug, Clone, Error)]
pub enum SetWidgetConfigError {
    /// No menu item with the given id was found.
    #[error("Menu item not found: {id}")]
    NotFound { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_widget_config_error_not_found_display() {
        let error = SetWidgetConfigError::NotFound { id: "test".to_string() };
        assert_eq!(error.to_string(), "Menu item not found: test");
    }

    #[test]
    fn test_set_widget_config_error_clone() {
        let error = SetWidgetConfigError::NotFound { id: "test".to_string() };
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }
}
