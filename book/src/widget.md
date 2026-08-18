# The PieMenuOverlayWidget

The primary component of this library is the `PieMenuOverlayWidget` (internally subclassed in GObject as `PieMenuOverlayWidget`).

## Construction

Creating a widget is straightforward — pass an optional child widget to wrap:

```rust
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use gtk4::Label;

let label = Label::new(Some("Hello Pie Menu"));
let overlay = PieMenuOverlayWidget::new(Some(&label));
```

## Adding Menu Items

Menu items are added programmatically via the `PieMenuMenuItemHandler` trait:

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

overlay.add_menu_item(
    MenuItem::builder()
        .id("rotate-cw")
        .widget_type("circle")
        .config(CircleConfig::builder()
            .icon_name("object-rotate-right-symbolic")
            .label("Rotate CW")
            .color("#00000077")
            .build())
        .angle(0.0)
        .radius(30.0)
        .event("rotate-cw")
        .build(),
);
```

You can also use the [builder pattern](builder_pattern.md) for fluent construction, or `add_menu_item_auto()` for [automatic angle distribution](auto_distribution.md).

## Configurable Thresholds

The activation and deactivation thresholds are configurable at runtime via the `PieMenuControlHandler` trait:

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

overlay.set_activation_threshold(2.5);
overlay.set_deactivation_threshold(0.3);
```

See the [Thresholds](thresholds.md) page for details.

## Optional Rotation Gesture

The rotation gesture (two-finger rotate) can be enabled or disabled at runtime. When disabled, the gesture controller's propagation phase is set to `None`, effectively ignoring rotation input. Default: `true`.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

overlay.set_rotation_gesture_enabled(false);
```

## Optional Ring Markings

The inner and outer ring markings (5-degree tick marks) can be toggled at runtime. Default: `true`.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

overlay.set_markings_enabled(false);
```

## Optional Center Widget

An optional GTK4 widget can be placed in the center of the pie menu ring. The center widget rotates with the ring and is responsible for its own event handling, including close-menu and close-submenu interactions.

```rust
use gtk4::GestureClick;
use gtk4::Label;
use gtk4::prelude::*;
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

let center_label = Label::new(Some("Close"));

// The consumer is responsible for close behavior
let overlay_clone = overlay.clone();
let click = GestureClick::new();
click.connect_pressed(move |_, _, _, _| {
    if overlay_clone.submenu_depth() > 0 {
        let _ = overlay_clone.close_submenu();
    } else {
        let _ = overlay_clone.hide_pie_menu();
    }
});
center_label.add_controller(click);

overlay.set_center_widget(Some(&center_label));
```

When no center widget is set, the built-in center-click-to-close behavior remains active. When a center widget is set, the built-in close logic is bypassed — the click event propagates to the center widget's own event controllers.

The center widget is clamped to `2 * center_radius` pixels in size. Use `set_pie_menu_center_radius` to adjust the available space.

```rust
// Builder pattern
let overlay = PieMenuOverlayWidget::new(Some(&child))
    .with_center_widget(&center_label)
    .with_pie_menu_center_radius(100.0);
```

To remove the center widget and restore default behavior:

```rust
overlay.set_center_widget(None);
```

## Message Channel

Set up an `mpsc` channel to receive messages from the pie menu:

```rust
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use std::sync::mpsc::channel;

let (sender, receiver) = channel::<PieMenuMessage>();
overlay.set_message_sender(sender);
```

Messages received:
- `PieMenuMessage::Opened` — the pie menu was opened
- `PieMenuMessage::Closed` — the pie menu was closed
- `PieMenuMessage::Rotate(f32)` — rotation in degrees from the rotation gesture
- `PieMenuMessage::Event(String)` — the event name of the clicked menu item

## Gesture Handling

The `PieMenuOverlayWidget` responds to two touch gestures:

- **Pinch-to-zoom** (`GestureZoom`): Opens the menu when scale exceeds the activation threshold (default: 3.5), closes when scale drops below the deactivation threshold (default: 0.5)
- **Rotation** (`GestureRotate`): Rotates the menu ring and sends `Rotate` messages when the delta exceeds 10 degrees

Click detection handles:
- **Center circle click**: Closes the menu (unless a center widget is set — see [Optional Center Widget](#optional-center-widget))
- **Menu item click**: Sends `Event` message with the item's event name

### Additional Input Methods

Optional input methods are available via feature flags. See [Input Handling](input_handling.md) for details:

- **Keyboard** (`keyboard` feature, enabled by default): Navigate with arrow keys, confirm with `Enter`
- **Mouse scroll** (`mouse-scroll` feature, enabled by default): Rotate the ring with the mouse wheel
- **Controller** (`controller-sdl2` or `controller-evdev` feature): Navigate with analog sticks

## Rotation

Use the `RotationHandler` trait to set the menu rotation programmatically:

```rust
use smearor_wrot_pie_menu::RotationHandler;

overlay.set_rotation(45.0);
```

For the complete API including all traits and methods, see the [API Reference](api.md) page. For the widget system (custom widgets, registry, config types), see [Widget System](widget_system.md).
