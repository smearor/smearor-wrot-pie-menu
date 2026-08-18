# Widget System

All menu items are rendered as GTK4 child widgets, resolved by type name from a registry. This replaces the previous snapshot-based rendering with a unified widget-child pipeline.

## Overview

The widget system consists of:

- **`MenuItemWidgetFactory`** — typed factory trait that builds GTK4 widgets
- **`MenuItemWidgetFactoryErased`** — type-erased counterpart for registry storage
- **`MenuItemWidgetRegistry`** — maps type names to factories, pre-populated with standard implementations
- **`MenuItemContext`** — provides event trigger callback to factories
- **`ItemSize`** — optional non-square allocation for widgets

### Standard Implementations

The library ships four standard widget types:

| Type | Factory | Config | Description |
|------|---------|--------|-------------|
| `"circle"` | `CircleWidgetFactory` | `CircleConfig` | Circular item with icon + label |
| `"square"` | `SquareWidgetFactory` | `SquareConfig` | Square item with icon + label |
| `"button"` | `ButtonWidgetFactory` | `ButtonConfig` | Simple GTK4 Button (debug) |
| `"gauge"` | `GaugeWidgetFactory` | `GaugeConfig` | Tachometer-style gauge with color-coded zones |

When `widget_type` is `None`, `"circle"` is used as the default.

## Config Types

Each factory defines its own typed config struct with `TypedBuilder`. Visual properties (icon, label, colors) live in the config, not on `MenuItem`.

### CircleConfig / SquareConfig

```rust
use smearor_wrot_pie_menu::CircleConfig;

let config = CircleConfig::builder()
    .icon_name("media-playback-start-symbolic")
    .label("Play")
    .color("#00AA0077")        // optional, &str or RgbaColor
    .label_color("#FFFFFFFF")  // optional
    .icon_size(Some(48))       // optional
    .show_label(Some(true))    // optional, default: true
    .build();
```

Fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `icon_name` | `String` | Yes | — | GTK icon theme name |
| `label` | `String` | No | `""` | Display text |
| `color` | `Option<RgbaColor>` | No | `None` (factory default) | Background color |
| `label_color` | `Option<RgbaColor>` | No | `None` (factory default) | Label text color |
| `icon_size` | `Option<u32>` | No | `None` | Icon size in pixels |
| `show_label` | `Option<bool>` | No | `None` (default: `true`) | Whether to show the label |

When `color` or `label_color` is `None`, the factory uses `DEFAULT_ICON_COLOR` or `DEFAULT_LABEL_COLOR` respectively.

### ButtonConfig

```rust
use smearor_wrot_pie_menu::ButtonConfig;

let config = ButtonConfig::builder()
    .label("Click me")
    .build();
```

### GaugeConfig

The `GaugeConfig` defines a tachometer-style gauge with an 80% arc (288° sweep), color-coded zones (green / orange / red), and centered label + value text.

```rust
use smearor_wrot_pie_menu::GaugeConfig;

let config = GaugeConfig::builder()
    .label("CPU")
    .value(42.0)
    .unit("%")
    .min(0.0)
    .warning(80.0)
    .critical(90.0)
    .max(100.0)
    .build();
```

Fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `label` | `String` | No | `""` | Display label shown above the value |
| `value` | `f64` | No | `0.0` | Current value |
| `unit` | `String` | No | `""` | Unit suffix shown after the value |
| `min` | `f64` | No | `0.0` | Minimum value (arc start) |
| `warning` | `f64` | No | `0.0` | Threshold where the arc turns orange |
| `critical` | `f64` | No | `0.0` | Threshold where the arc turns red |
| `max` | `f64` | No | `100.0` | Maximum value (arc end) |

The arc is drawn from `min` to `max`. Between `warning` and `critical` the arc is orange; above `critical` it is red. Below `warning` it is green.

#### Dynamic Updates

To update a gauge value at runtime, use `set_widget_config`:

```rust
use smearor_wrot_pie_menu::GaugeConfig;

if let Some(item) = overlay.get_menu_item("cpu")
    && let Some(config_value) = &item.widget_config
    && let Ok(mut config) = serde_json::from_value::<GaugeConfig>(config_value.clone())
{
    config.value = 72.5;
    let _ = overlay.set_widget_config("cpu", serde_json::to_value(&config).unwrap_or(serde_json::Value::Null));
}
```

## Using Widgets in Menu Items

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;

let item = MenuItem::builder()
    .id("play")
    .angle(0.0)
    .event("play")
    .widget_type("circle")
    .config(CircleConfig::builder()
        .icon_name("media-playback-start-symbolic")
        .label("Play")
        .color("#00AA0077")
        .build())
    .build();
```

The `.config()` builder method accepts any type that implements `Serialize`. It serializes the config to `serde_json::Value` internally. This is the typed equivalent of `.widget_config(serde_json::to_value(&config).unwrap())`.

### Non-Square Allocation

For widgets that need non-square dimensions (e.g., sliders, gauges):

```rust
use smearor_wrot_pie_menu::ItemSize;

let item = MenuItem::builder()
    .id("volume")
    .angle(180.0)
    .event("volume")
    .widget_type("slider")
    .content_size(ItemSize::builder().width(40.0).height(100.0).build())
    .content_rotates(false)
    .build();
```

When `content_size` is `None`, the item's `radius` is used for a square allocation of `2 * radius` pixels.

### Rotation Behavior

- `content_rotates = true` (default): The widget rotates with the ring.
- `content_rotates = false`: The widget stays upright while its position on the ring rotates.

## Registering Custom Widgets

```rust
use smearor_wrot_pie_menu::menu::context::MenuItemContext;
use smearor_wrot_pie_menu::menu::widget_factory::MenuItemWidgetFactory;
use smearor_wrot_pie_menu::MenuItem;
use gtk4::LevelBar;
use gtk4::Widget;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CpuGaugeConfig {
    value: f64,
}

struct CpuGaugeFactory;

impl MenuItemWidgetFactory for CpuGaugeFactory {
    type Config = CpuGaugeConfig;

    fn type_name(&self) -> &str { "cpu-gauge" }

    fn build(&self, _item: &MenuItem, config: &CpuGaugeConfig, _context: &MenuItemContext) -> Widget {
        let bar = LevelBar::builder()
            .value(config.value)
            .min_value(0.0)
            .max_value(1.0)
            .width_request(80)
            .height_request(80)
            .build();
        bar.upcast::<Widget>()
    }
}

overlay.register_widget_factory(Box::new(CpuGaugeFactory));
```

## Dynamic Updates

### refresh_widgets

Clears the widget cache and rebuilds all item widgets on the next layout pass. Use this after registering new widget factories or changing `widget_type` on existing items.

```rust
overlay.refresh_widgets();
```

### set_widget_config

Replaces the widget configuration for a single item and forces a rebuild of that item's widget.

```rust
use serde_json::json;

overlay.set_widget_config("cpu", json!({ "value": 0.72 }))?;
```

## Widget Caching

Widgets are built once by the factory, registered as children of `PieMenuWidget` via `set_parent`, and cached in an `item_widgets` map. On subsequent layout passes, cached widgets are repositioned without calling the factory again.

All mutations of the widget cache (`refresh_widgets`, `set_widget_config`) are deferred to the next event loop iteration via `glib::idle_add_local` to prevent `RefCell` reentrancy panics during render or allocation passes.

## Serialization

`widget_type` (`Option<String>`) and `widget_config` (`Option<serde_json::Value>`) are serializable with `#[serde(default)]`. This allows storing widget configuration in JSON/TOML files.

The registry itself is not serializable — it is rebuilt at runtime with standard implementations and consumer-registered factories.
