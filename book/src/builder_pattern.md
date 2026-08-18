# Builder Pattern

`PieMenuOverlayWidget` supports a fluent builder API for ergonomic construction. Each `with_*` method takes ownership of `self`, applies the setting, and returns `Self` for chaining.

## Available Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `with_message_sender(sender)` | `Self` | Sets the message channel sender |
| `with_activation_threshold(f64)` | `Self` | Sets the pinch-to-zoom activation threshold |
| `with_deactivation_threshold(f64)` | `Self` | Sets the pinch-out deactivation threshold |
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
