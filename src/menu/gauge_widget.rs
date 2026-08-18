use crate::menu::MenuItem;
use crate::menu::context::MenuItemContext;
use crate::menu::gauge_item_widget::GaugeItemWidget;
use crate::menu::gauge_item_widget::GaugeItemWidgetParams;
use crate::menu::widget_factory::MenuItemWidgetFactory;
use gtk4::Widget;
use gtk4::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

/// Typed configuration for the `"gauge"` widget type.
///
/// A gauge/tachometer widget that renders an 80% arc with color-coded
/// zones (green, orange, red) and a needle indicator showing the
/// current value. The label is displayed in the center, with the
/// value and unit below it.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct GaugeConfig {
    /// Label displayed in the center of the gauge (e.g. "CPU").
    #[builder(setter(into))]
    pub label: String,
    /// Current value to display on the gauge.
    pub value: f64,
    /// Unit string appended to the displayed value (e.g. "%", "°C").
    #[builder(setter(into))]
    pub unit: String,
    /// Minimum value on the gauge scale.
    pub min: f64,
    /// Warning threshold. Values between `min` and `warning` are green.
    pub warning: f64,
    /// Critical threshold. Values between `warning` and `critical` are orange.
    pub critical: f64,
    /// Maximum value on the gauge scale. Values between `critical` and `max` are red.
    pub max: f64,
}

impl Default for GaugeConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            value: 0.0,
            unit: String::new(),
            min: 0.0,
            warning: 0.0,
            critical: 0.0,
            max: 0.0,
        }
    }
}

/// Factory for creating gauge menu item widgets.
///
/// Produces a `GaugeItemWidget` - a custom GTK4 widget subclass
/// that draws a tachometer-style gauge with color-coded zones and
/// a needle indicator. Registered under the `"gauge"` type name.
pub struct GaugeWidgetFactory;

impl MenuItemWidgetFactory for GaugeWidgetFactory {
    type Config = GaugeConfig;

    fn type_name(&self) -> &str {
        "gauge"
    }

    fn build(&self, item: &MenuItem, config: &GaugeConfig, _context: &MenuItemContext) -> Widget {
        let widget = GaugeItemWidget::new(GaugeItemWidgetParams {
            label: config.label.clone(),
            value: config.value,
            unit: config.unit.clone(),
            min: config.min,
            warning: config.warning,
            critical: config.critical,
            max: config.max,
            item_radius: item.radius(),
            enabled: item.enabled,
        });

        widget.upcast::<Widget>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_config_default() {
        let config = GaugeConfig::default();
        assert!(config.label.is_empty());
        assert_eq!(config.value, 0.0);
        assert!(config.unit.is_empty());
        assert_eq!(config.min, 0.0);
        assert_eq!(config.warning, 0.0);
        assert_eq!(config.critical, 0.0);
        assert_eq!(config.max, 0.0);
    }

    #[test]
    fn test_gauge_config_builder() {
        let config = GaugeConfig::builder()
            .label("CPU")
            .value(85.0)
            .unit("%")
            .min(0.0)
            .warning(80.0)
            .critical(90.0)
            .max(100.0)
            .build();
        assert_eq!(config.label, "CPU");
        assert_eq!(config.value, 85.0);
        assert_eq!(config.unit, "%");
        assert_eq!(config.min, 0.0);
        assert_eq!(config.warning, 80.0);
        assert_eq!(config.critical, 90.0);
        assert_eq!(config.max, 100.0);
    }

    #[test]
    fn test_gauge_config_serialize() {
        let config = GaugeConfig::builder()
            .label("CPU")
            .value(85.0)
            .unit("%")
            .min(0.0)
            .warning(80.0)
            .critical(90.0)
            .max(100.0)
            .build();
        let json = serde_json::to_string(&config).expect("serialize should succeed");
        assert!(json.contains("CPU"));
        assert!(json.contains("85"));
    }

    #[test]
    fn test_gauge_config_deserialize() {
        let json = "{\"label\":\"CPU\",\"value\":85.0,\"unit\":\"%\",\"min\":0.0,\"warning\":80.0,\"critical\":90.0,\"max\":100.0}";
        let config: GaugeConfig = serde_json::from_str(json).expect("deserialize should succeed");
        assert_eq!(config.label, "CPU");
        assert_eq!(config.value, 85.0);
        assert_eq!(config.unit, "%");
        assert_eq!(config.warning, 80.0);
        assert_eq!(config.critical, 90.0);
        assert_eq!(config.max, 100.0);
    }

    #[test]
    fn test_gauge_factory_type_name() {
        let factory = GaugeWidgetFactory;
        assert_eq!(factory.type_name(), "gauge");
    }
}
