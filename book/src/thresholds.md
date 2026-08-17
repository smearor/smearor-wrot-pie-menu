# Thresholds

The pie menu opens and closes based on pinch-to-zoom gesture scale values. These thresholds are configurable at runtime.

## Default Values

| Threshold | Default | Constant |
|-----------|---------|----------|
| Activation | `3.5` | `DEFAULT_ACTIVATION_THRESHOLD` |
| Deactivation | `0.5` | `DEFAULT_DEACTIVATION_THRESHOLD` |

## Configuration

Thresholds are stored as `AtomicF64` values, allowing thread-safe updates without locking.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

// Set custom thresholds
overlay.set_activation_threshold(2.5);
overlay.set_deactivation_threshold(0.3);

// Read current thresholds
let active = overlay.activation_threshold();
let inactive = overlay.deactivation_threshold();
```

## How It Works

When a pinch-to-zoom gesture is detected:

1. If `scale > activation_threshold` and the menu is closed → **open** the menu
2. If `scale < deactivation_threshold` and the menu is open → **close** the menu

The gesture is claimed to prevent the child widget from receiving the event.

## Builder Pattern

You can also set thresholds using the builder pattern:

```rust
let overlay = PieMenuOverlayWidget::new(Some(&child))
    .with_activation_threshold(2.5)
    .with_deactivation_threshold(0.3);
```
