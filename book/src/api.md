# API Reference

## PieMenuMenuItemHandler Trait

The `PieMenuMenuItemHandler` trait provides methods for adding and removing menu items:

```rust
pub trait PieMenuMenuItemHandler {
    fn add_menu_item(&self, menu_item: MenuItem) -> Result<(), AddMenuItemError>;
    fn remove_menu_item(&self, id: &str) -> Result<(), RemoveMenuItemError>;
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

## PieMenuControlHandler Trait

The `PieMenuControlHandler` trait provides methods for showing and hiding the pie menu:

```rust
pub trait PieMenuControlHandler {
    fn show_pie_menu(&self) -> Result<(), ShowPieMenuError>;
    fn hide_pie_menu(&self) -> Result<(), HidePieMenuError>;
    fn is_pie_menu_open(&self) -> bool;
}
```

### `show_pie_menu()`

Shows the pie menu widget by setting it visible.

### `hide_pie_menu()`

Hides the pie menu widget by setting it invisible.

### `is_pie_menu_open() -> bool`

Returns whether the pie menu is currently visible.

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
    .build()
```

## PieMenuMessage

Messages sent from the pie menu to the consumer:

```rust
pub enum PieMenuMessage {
    Rotate(f32),      // rotation in degrees
    Event(String),    // menu item event name
}
```

## Method Reference Table

| Method | Trait | Parameters | Description |
|--------|-------|-----------|-------------|
| `add_menu_item` | `PieMenuMenuItemHandler` | `MenuItem` | Add a menu item |
| `remove_menu_item` | `PieMenuMenuItemHandler` | `&str` | Remove by id |
| `show_pie_menu` | `PieMenuControlHandler` | — | Show the menu |
| `hide_pie_menu` | `PieMenuControlHandler` | — | Hide the menu |
| `is_pie_menu_open` | `PieMenuControlHandler` | — | Check visibility |
| `set_message_sender` | `PieMenuMessageSender` | `Sender<PieMenuMessage>` | Set channel |
| `send_message` | `PieMenuMessageSender` | `PieMenuMessage` | Send message |
| `set_rotation` | `RotationHandler` | `f32` | Set rotation |
| `set_close_callback` | `PieMenuWidget` | `Fn() + 'static` | Center click callback |
