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
}
```

### `add_menu_item(MenuItem)`

Adds a menu item to the pie menu. The item is inserted into the internal `DashMap` keyed by its `id`.

```rust
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

overlay.add_menu_item(
    MenuItem::builder()
        .id("exit")
        .label("Exit")
        .icon_name("window-close-symbolic")
        .color("#55222277")
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
overlay.add_menu_item_auto(
    MenuItem::builder()
        .id("save")
        .label("Save")
        .icon_name("document-save-symbolic")
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
}
```

### `set_rotation(f32)`

Sets the menu rotation in degrees. The menu ring is redrawn at the new angle.

```rust
use smearor_wrot_pie_menu::RotationHandler;

overlay.set_rotation(45.0);
```

## MenuItem

A single menu item with `TypedBuilder` construction:

```rust
MenuItem::builder()
    .id("unique-id")          // required
    .label("Display Label")    // required
    .icon_name("icon-name")    // required, GTK icon theme name
    .angle(45.0)               // required, degrees
    .event("event-name")       // required, sent as PieMenuMessage::Event
    .color("#RRGGBBAA")        // optional, default: grey
    .label_color("#RRGGBBAA")  // optional, default: white
    .radius(30.0)              // optional, default: 40.0
    .enabled(true)             // optional, default: true
    .fixed_position(false)     // optional, default: false
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
| `set_message_sender` | `PieMenuMessageSender` | `Sender<PieMenuMessage>` | Set channel |
| `send_message` | `PieMenuMessageSender` | `PieMenuMessage` | Send message |
| `set_rotation` | `RotationHandler` | `f32` | Set rotation |
| `set_close_callback` | `PieMenuWidget` | `Fn() + 'static` | Center click callback |
| `set_markings_enabled` | `PieMenuWidget` | `bool` | Enable/disable ring markings |
| `markings_enabled` | `PieMenuWidget` | — | Get markings state |
| `with_message_sender` | `PieMenuOverlayWidget` | `Sender<PieMenuMessage>` | Builder: set sender |
| `with_activation_threshold` | `PieMenuOverlayWidget` | `f64` | Builder: set activation threshold |
| `with_deactivation_threshold` | `PieMenuOverlayWidget` | `f64` | Builder: set deactivation threshold |
| `with_rotation_gesture_enabled` | `PieMenuOverlayWidget` | `bool` | Builder: enable/disable rotation gesture |
| `with_markings_enabled` | `PieMenuOverlayWidget` | `bool` | Builder: enable/disable markings |
| `with_menu_item` | `PieMenuOverlayWidget` | `MenuItem` | Builder: add item |
