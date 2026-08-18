# API Reference

## PieMenuMenuItemHandler Trait

The `PieMenuMenuItemHandler` trait provides methods for adding and removing menu items:

```rust
pub trait PieMenuMenuItemHandler {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;
    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError>;
    fn remove_all_menu_items(&self);
    fn menu_item_count(&self) -> usize;
    fn set_menu_item_enabled(&self, id: &str, enabled: bool) -> Result<(), SetMenuItemEnabledError>;
    fn add_menu_item_auto(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;
    fn redistribute(&self);
    fn get_menu_item(&self, id: &str) -> Option<MenuItem>;
    fn update_menu_item(&self, menu_item: MenuItem) -> Result<(), UpdateMenuItemError>;
    fn refresh_widgets(&self);
    fn set_widget_config(&self, id: &str, config: serde_json::Value) -> Result<(), SetWidgetConfigError>;
}
```

### `add_menu_item(MenuItem)`

Adds a menu item to the pie menu. The item is inserted into the internal `DashMap` keyed by its `id`.

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

overlay.add_menu_item(
    MenuItem::builder()
        .id("exit")
        .widget_type("circle")
        .config(CircleConfig::builder()
            .icon_name("window-close-symbolic")
            .label("Exit")
            .color("#55222277")
            .build())
        .angle(135.0)
        .event("exit")
        .build(),
);
```

### `remove_menu_item(&str)`

Removes the menu item with the given id.

```rust
overlay.remove_menu_item("exit");
```

### `remove_all_menu_items()`

Removes all menu items from the pie menu.

```rust
overlay.remove_all_menu_items();
```

### `menu_item_count() -> usize`

Returns the number of menu items currently in the pie menu.

```rust
let count = overlay.menu_item_count();
```

### `set_menu_item_enabled(&str, bool)`

Sets the enabled state of a menu item. Disabled items render at reduced opacity and do not respond to hover or click.

```rust
overlay.set_menu_item_enabled("exit", false)?;
```

### `add_menu_item_auto(MenuItem)`

Adds a menu item with an automatically calculated angle. See [Auto Distribution](auto_distribution.md).

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

overlay.add_menu_item_auto(
    MenuItem::builder()
        .id("save")
        .widget_type("circle")
        .config(CircleConfig::builder()
            .icon_name("document-save-symbolic")
            .label("Save")
            .build())
        .angle(0.0)
        .event("save")
        .build(),
)?;
```

### `redistribute()`

Redistributes all non-fixed items proportionally in the gaps between fixed items. Triggers a redraw. Useful after `remove_menu_item()` to re-space remaining items.

```rust
overlay.remove_menu_item("shuffle")?;
overlay.redistribute();
```

### `get_menu_item(&str) -> Option<MenuItem>`

Returns a clone of the menu item with the given id, or `None` if not found.

```rust
if let Some(mut item) = overlay.get_menu_item("play-pause") {
    item.widget_config = Some(serde_json::to_value(&CircleConfig::builder()
        .icon_name("media-playback-pause-symbolic")
        .label("Pause")
        .build()).unwrap());
    overlay.update_menu_item(item)?;
}
```

### `update_menu_item(MenuItem) -> Result<(), UpdateMenuItemError>`

Replaces an existing menu item (matched by `id`) with the given item. All fields except `id` can be changed. If `angle` or `radius` changed, overlap validation is performed and the update is rolled back on failure. Triggers a redraw on success.

```rust
let mut item = overlay.get_menu_item("play-pause").unwrap();
item.widget_config = Some(serde_json::to_value(&CircleConfig::builder()
    .icon_name("media-playback-pause-symbolic")
    .label("Pause")
    .build()).unwrap());
overlay.update_menu_item(item)?;
```

### `refresh_widgets()`

Clears the widget cache and rebuilds all item widgets on the next layout pass. Use this after registering new widget factories or changing `widget_type` on existing items.

```rust
overlay.refresh_widgets();
```

### `set_widget_config(&str, serde_json::Value) -> Result<(), SetWidgetConfigError>`

Replaces the `widget_config` for a single menu item and clears its cached widget, forcing a rebuild on the next layout pass. Returns `SetWidgetConfigError::NotFound` if no item with the given id exists.

```rust
use serde_json::json;

overlay.set_widget_config("cpu", json!({ "value": 0.72 }))?;
```

## PieMenuControlHandler Trait

The `PieMenuControlHandler` trait provides methods for showing and hiding the pie menu:

```rust
pub trait PieMenuControlHandler {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError>;
    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError>;
    fn is_pie_menu_open(&self) -> bool;
    fn set_activation_threshold(&self, threshold: f64);
    fn activation_threshold(&self) -> f64;
    fn set_deactivation_threshold(&self, threshold: f64);
    fn deactivation_threshold(&self) -> f64;
    fn set_rotation_gesture_enabled(&self, enabled: bool);
    fn rotation_gesture_enabled(&self) -> bool;
    fn set_markings_enabled(&self, enabled: bool);
    fn markings_enabled(&self) -> bool;
    fn set_scroll_rotation_step(&self, sensitivity: f64);
    fn scroll_rotation_step(&self) -> f32;
    fn set_pie_menu_radius(&self, radius: f32);
    fn pie_menu_radius(&self) -> f32;
    fn set_pie_menu_center_radius(&self, radius: f32);
    fn pie_menu_center_radius(&self) -> f32;
    fn set_submenu_radius(&self, level: u32, radius: f32);
    fn set_submenu_radius_step(&self, step: f32);
    fn cycle_selection(&self, direction: i32);
    fn select_first_item(&self);
    fn confirm_selection(&self);
    fn handle_left_stick_x(&self, x: f32);
    fn handle_right_stick(&self, x: f32, y: f32);
    fn find_nearest_item(&self, target_angle: f32) -> Option<String>;
    fn set_keyboard_selection(&self, id: String);
}
```

### `show_pie_menu()`

Shows the pie menu widget by setting it visible.

### `hide_pie_menu()`

Hides the pie menu widget by setting it invisible.

### `is_pie_menu_open() -> bool`

Returns whether the pie menu is currently visible.

### `set_activation_threshold(f64)` / `activation_threshold() -> f64`

Sets/gets the pinch-to-zoom activation threshold. Default: `3.5`.

### `set_deactivation_threshold(f64)` / `deactivation_threshold() -> f64`

Sets/gets the pinch-out deactivation threshold. Default: `0.5`.

### `set_rotation_gesture_enabled(bool)` / `rotation_gesture_enabled() -> bool`

Enables or disables the rotation gesture when the pie menu is open. When disabled, the gesture controller's propagation phase is set to `None`, effectively ignoring rotation input. Default: `true`.

### `set_markings_enabled(bool)` / `markings_enabled() -> bool`

Enables or disables drawing of inner and outer ring markings. Default: `true`.

### `set_scroll_rotation_step(f64)` / `scroll_rotation_step() -> f32`

Sets/gets the scroll rotation sensitivity multiplier. The rotation delta is computed as `dy * sensitivity`. Default: `5.0`. See [Input Handling](input_handling.md).

### `set_pie_menu_radius(f32)` / `pie_menu_radius() -> f32`

Sets/gets the radius of the main pie menu ring in pixels. Default: `160.0`. Increasing this enlarges the ring to accommodate larger item widgets.

### `set_pie_menu_center_radius(f32)` / `pie_menu_center_radius() -> f32`

Sets/gets the inner radius of the pie menu ring (the transparent center) in pixels. Default: `64.0`. Items are positioned at `0.7 * radius` from center; to center items in the ring, set `center_radius = 2 * (0.7 * radius) - radius`.

### `set_submenu_radius(u32, f32)`

Sets the radius for a specific submenu level. Level 0 is the main ring, level 1 is the first submenu, etc. This overrides the computed `main_radius + level * step` value for that level.

### `set_submenu_radius_step(f32)`

Sets the global step width between consecutive ring levels. Each submenu level's radius is computed as `main_radius + level * step`. Default: `80.0`.

### `set_center_widget(Option<&Widget>)` / `center_widget() -> Option<Widget>`

Sets or removes an optional center widget rendered inside the pie menu ring's transparent center. The center widget rotates with the ring. When set, the consumer is responsible for implementing close-menu / close-submenu behavior via event controllers on the center widget. When `None`, the built-in center-click-to-close behavior is restored.

### `with_center_widget(&Widget) -> Self`

Builder method that sets the center widget and returns self for chaining. See [Builder Pattern](builder_pattern.md).

### `cycle_selection(i32)`

Cycles the keyboard selection by `direction` (-1 for CCW, +1 for CW). Items are sorted by angle for deterministic navigation order. Disabled items are skipped.

### `select_first_item()`

Selects the first enabled item (smallest angle, typically 0°). Disabled items are skipped.

### `confirm_selection()`

Confirms the current keyboard selection by sending `PieMenuMessage::Event` for the selected item. Does nothing if no item is selected or if the selected item is disabled.

### `handle_left_stick_x(f32)`

Updates the stored left stick X-axis value for continuous rotation. Must be called on the GTK main thread. See [Input Handling](input_handling.md).

### `handle_right_stick(f32, f32)`

Selects the nearest menu item based on the right stick direction. Must be called on the GTK main thread. See [Input Handling](input_handling.md).

### `find_nearest_item(f32) -> Option<String>`

Finds the enabled menu item whose angle is closest to `target_angle` (in degrees). Returns the item ID, or `None` if the menu has no enabled items. Disabled items are skipped.

### `set_keyboard_selection(String)`

Sets the keyboard selection to the given item ID and triggers a redraw.

## PieMenuMessageSender Trait

The `PieMenuMessageSender` trait provides the message channel interface:

```rust
pub trait PieMenuMessageSender {
    fn set_message_sender(&self, sender: Sender<PieMenuMessage>);
    fn send_message(&self, message: PieMenuMessage);
}
```

### `set_message_sender(Sender<PieMenuMessage>)`

Sets the `mpsc` sender for communicating with the consumer application.

### `send_message(PieMenuMessage)`

Sends a message through the channel. Called internally by gesture handlers.

## RotationHandler Trait

The `RotationHandler` trait provides rotation control:

```rust
pub trait RotationHandler {
    fn set_rotation(&self, rotation: f32);
    fn rotation(&self) -> f32;
}
```

### `set_rotation(f32)`

Sets the menu rotation in degrees. The menu ring is redrawn at the new angle.

### `rotation() -> f32`

Returns the current rotation in degrees.

```rust
use smearor_wrot_pie_menu::RotationHandler;

overlay.set_rotation(45.0);
```

## MenuItem

A single menu item with `TypedBuilder` construction. Visual properties (icon, label, colors) are defined in widget-specific config structs (`CircleConfig`, `SquareConfig`, `ButtonConfig`). See [Widget System](widget_system.md) for details.

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;

MenuItem::builder()
    .id("unique-id")              // required
    .angle(45.0)                   // required, degrees
    .event("event-name")          // required, sent as PieMenuMessage::Event
    .widget_type("circle")        // optional, default: "circle"
    .config(CircleConfig::builder() // optional, typed widget config
        .icon_name("icon-name")
        .label("Display Label")
        .color("#RRGGBBAA")
        .build())
    .radius(30.0)                  // optional, default: 40.0
    .enabled(true)                 // optional, default: true
    .fixed_position(false)         // optional, default: false
    .content_rotates(true)         // optional, default: true
    .build()
```

## PieMenuMessage

Messages sent from the pie menu to the consumer:

```rust
pub enum PieMenuMessage {
    Opened,           // pie menu was opened
    Closed,           // pie menu was closed
    Rotate(f32),      // rotation in degrees
    Event(String),    // menu item event name
}
```

## Method Reference Table

| Method | Trait | Parameters | Description |
|--------|-------|-----------|-------------|
| `add_menu_item` | `PieMenuMenuItemHandler` | `MenuItem` | Add a menu item |
| `add_menu_item_auto` | `PieMenuMenuItemHandler` | `MenuItem` | Add with auto-calculated angle |
| `redistribute` | `PieMenuMenuItemHandler` | — | Redistribute flexible item angles |
| `remove_menu_item` | `PieMenuMenuItemHandler` | `&str` | Remove by id |
| `remove_all_menu_items` | `PieMenuMenuItemHandler` | — | Remove all items |
| `menu_item_count` | `PieMenuMenuItemHandler` | — | Get item count |
| `set_menu_item_enabled` | `PieMenuMenuItemHandler` | `&str, bool` | Enable/disable an item |
| `get_menu_item` | `PieMenuMenuItemHandler` | `&str` | Get a clone of an item |
| `update_menu_item` | `PieMenuMenuItemHandler` | `MenuItem` | Replace an item (all fields except `id`) |
| `refresh_widgets` | `PieMenuMenuItemHandler` | — | Rebuild all item widgets |
| `set_widget_config` | `PieMenuMenuItemHandler` | `&str, serde_json::Value` | Replace widget config for one item |
| `show_pie_menu` | `PieMenuControlHandler` | — | Show the menu |
| `hide_pie_menu` | `PieMenuControlHandler` | — | Hide the menu |
| `is_pie_menu_open` | `PieMenuControlHandler` | — | Check visibility |
| `set_activation_threshold` | `PieMenuControlHandler` | `f64` | Set activation threshold |
| `activation_threshold` | `PieMenuControlHandler` | — | Get activation threshold |
| `set_deactivation_threshold` | `PieMenuControlHandler` | `f64` | Set deactivation threshold |
| `deactivation_threshold` | `PieMenuControlHandler` | — | Get deactivation threshold |
| `set_rotation_gesture_enabled` | `PieMenuControlHandler` | `bool` | Enable/disable rotation gesture |
| `rotation_gesture_enabled` | `PieMenuControlHandler` | — | Get rotation gesture state |
| `set_markings_enabled` | `PieMenuControlHandler` | `bool` | Enable/disable ring markings |
| `markings_enabled` | `PieMenuControlHandler` | — | Get markings state |
| `set_scroll_rotation_step` | `PieMenuControlHandler` | `f64` | Set scroll rotation sensitivity |
| `scroll_rotation_step` | `PieMenuControlHandler` | — | Get scroll rotation sensitivity |
| `set_pie_menu_radius` | `PieMenuControlHandler` | `f32` | Set main ring radius |
| `pie_menu_radius` | `PieMenuControlHandler` | — | Get main ring radius |
| `set_pie_menu_center_radius` | `PieMenuControlHandler` | `f32` | Set inner ring radius (transparent center) |
| `pie_menu_center_radius` | `PieMenuControlHandler` | — | Get inner ring radius |
| `set_submenu_radius` | `PieMenuControlHandler` | `u32, f32` | Set radius for a specific submenu level |
| `set_submenu_radius_step` | `PieMenuControlHandler` | `f32` | Set step between consecutive ring levels |
| `cycle_selection` | `PieMenuControlHandler` | `i32` | Cycle keyboard selection |
| `select_first_item` | `PieMenuControlHandler` | — | Select first item |
| `confirm_selection` | `PieMenuControlHandler` | — | Confirm keyboard selection |
| `handle_left_stick_x` | `PieMenuControlHandler` | `f32` | Update left stick X value |
| `handle_right_stick` | `PieMenuControlHandler` | `f32, f32` | Select nearest item by stick direction |
| `find_nearest_item` | `PieMenuControlHandler` | `f32` | Find nearest item by angle |
| `set_keyboard_selection` | `PieMenuControlHandler` | `String` | Set keyboard selection by ID |
| `set_message_sender` | `PieMenuMessageSender` | `Sender<PieMenuMessage>` | Set channel |
| `send_message` | `PieMenuMessageSender` | `PieMenuMessage` | Send message |
| `set_rotation` | `RotationHandler` | `f32` | Set rotation |
| `rotation` | `RotationHandler` | — | Get rotation |
| `set_close_callback` | `PieMenuWidget` | `Fn() + 'static` | Center click callback |
| `set_center_widget` | `PieMenuOverlayWidget` | `Option<&Widget>` | Set/remove center widget |
| `center_widget` | `PieMenuOverlayWidget` | — | Get current center widget |
| `set_markings_enabled` | `PieMenuWidget` | `bool` | Enable/disable ring markings |
| `markings_enabled` | `PieMenuWidget` | — | Get markings state |
| `with_message_sender` | `PieMenuOverlayWidget` | `Sender<PieMenuMessage>` | Builder: set sender |
| `with_activation_threshold` | `PieMenuOverlayWidget` | `f64` | Builder: set activation threshold |
| `with_deactivation_threshold` | `PieMenuOverlayWidget` | `f64` | Builder: set deactivation threshold |
| `with_rotation_gesture_enabled` | `PieMenuOverlayWidget` | `bool` | Builder: enable/disable rotation gesture |
| `with_markings_enabled` | `PieMenuOverlayWidget` | `bool` | Builder: enable/disable markings |
| `with_scroll_rotation_step` | `PieMenuOverlayWidget` | `f64` | Builder: set scroll rotation sensitivity |
| `with_pie_menu_radius` | `PieMenuOverlayWidget` | `f32` | Builder: set main ring radius |
| `with_pie_menu_center_radius` | `PieMenuOverlayWidget` | `f32` | Builder: set inner ring radius |
| `with_submenu_radius_step` | `PieMenuOverlayWidget` | `f32` | Builder: set step between ring levels |
| `with_submenu_radius` | `PieMenuOverlayWidget` | `u32, f32` | Builder: set radius for a specific submenu level |
| `with_center_widget` | `PieMenuOverlayWidget` | `&Widget` | Builder: set center widget |
| `with_menu_item` | `PieMenuOverlayWidget` | `MenuItem` | Builder: add item |
| `register_widget_factory` | `PieMenuOverlayWidget` | `Box<dyn MenuItemWidgetFactoryErased>` | Register a custom widget factory |
