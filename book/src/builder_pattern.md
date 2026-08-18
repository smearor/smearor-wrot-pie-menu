# Builder Pattern

`PieMenuOverlayWidget` supports a fluent builder API for ergonomic construction. Each `with_*` method takes ownership of `self`, applies the setting, and returns `Self` for chaining.

## Available Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `with_message_sender(sender)` | `Self` | Sets the message channel sender |
| `with_activation_threshold(f64)` | `Self` | Sets the pinch-to-zoom activation threshold |
| `with_deactivation_threshold(f64)` | `Self` | Sets the pinch-out deactivation threshold |
| `with_rotation_gesture_enabled(bool)` | `Self` | Enables/disables the rotation gesture |
| `with_markings_enabled(bool)` | `Self` | Enables/disables ring markings |
| `with_scroll_rotation_step(f64)` | `Self` | Sets the scroll rotation sensitivity |
| `with_pie_menu_radius(f32)` | `Self` | Sets the main pie menu ring radius (default: 160.0) |
| `with_pie_menu_center_radius(f32)` | `Self` | Sets the inner ring radius / transparent center (default: 64.0) |
| `with_submenu_radius_step(f32)` | `Self` | Sets the step between consecutive submenu ring levels (default: 80.0) |
| `with_submenu_radius(u32, f32)` | `Self` | Sets the radius for a specific submenu level |
| `with_center_widget(&Widget)` | `Self` | Sets an optional center widget that rotates with the ring |
| `with_menu_item(MenuItem)` | `Result<Self, AddMenuItemError>` | Adds a menu item |

## Example

```rust
let overlay = PieMenuOverlayWidget::new(Some(&child))
    .with_message_sender(sender)
    .with_activation_threshold(2.5)
    .with_deactivation_threshold(0.3)
    .with_menu_item(
        MenuItem::builder()
            .id("rotate-cw")
            .widget_type("circle")
            .config(CircleConfig::builder()
                .icon_name("object-rotate-right-symbolic")
                .label("Rotate CW")
                .build())
            .angle(0.0)
            .fixed_position(true)
            .event("rotate-cw")
            .build(),
    )?;
```

## Error Handling

`with_menu_item()` returns `Result<Self, AddMenuItemError>`. If the item overlaps with an existing item, the error is propagated and the chain stops. Use `?` to short-circuit on failure.

## Ring Radius Configuration

The pie menu ring size can be controlled with `with_pie_menu_radius` and `with_pie_menu_center_radius`. Items are positioned at `0.7 * radius` from center. To center items in the ring, set `center_radius = 2 * (0.7 * radius) - radius`.

```rust
let overlay = PieMenuOverlayWidget::new(Some(&child))
    .with_pie_menu_radius(250.0)
    .with_pie_menu_center_radius(100.0)
    .with_menu_item(
        MenuItem::builder()
            .id("gauge")
            .angle(0.0)
            .event("gauge")
            .radius(70.0)
            .widget_type("gauge")
            .config(GaugeConfig::builder()
                .label("CPU")
                .value(42.0)
                .unit("%")
                .min(0.0)
                .warning(80.0)
                .critical(90.0)
                .max(100.0)
                .build())
            .build(),
    )?;
```

## Center Widget

The builder supports setting an optional center widget that rotates with the ring. The consumer is responsible for the center widget's event handling (e.g. click-to-close).

```rust
use smearor_wrot_pie_menu::GaugeItemWidget;
use smearor_wrot_pie_menu::GaugeItemWidgetParams;

let center_gauge = GaugeItemWidget::new(GaugeItemWidgetParams {
    label: "CPU".to_string(),
    value: 0.0,
    unit: "%".to_string(),
    min: 0.0,
    warning: 80.0,
    critical: 90.0,
    max: 100.0,
    item_radius: 90.0,
    enabled: true,
});

let overlay = PieMenuOverlayWidget::new(Some(&child))
    .with_pie_menu_center_radius(100.0)
    .with_center_widget(&center_gauge);
```
