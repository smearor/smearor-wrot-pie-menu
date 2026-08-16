# PieMenuMessage

The `PieMenuMessage` enum represents messages sent from the pie menu widget to the consumer application via an `mpsc` channel.

## Variants

```rust
pub enum PieMenuMessage {
    Rotate(f32),
    Event(String),
}
```

### `Rotate(f32)`

Sent when the user performs a rotation gesture on the open pie menu. The value is the absolute rotation in degrees (0–360), rounded to the nearest degree. Messages are only sent when the rotation changes by at least 1 degree from the last sent value.

### `Event(String)`

Sent when the user clicks a menu item. The string is the `event` field of the clicked `MenuItem`. The pie menu is automatically closed after a menu item click.

## Usage

```rust
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use std::sync::mpsc::channel;

let (sender, receiver) = channel::<PieMenuMessage>();
overlay.set_message_sender(sender);

// In your event loop:
match receiver.try_recv() {
    Ok(PieMenuMessage::Rotate(degrees)) => {
        println!("Rotated to {} degrees", degrees);
    }
    Ok(PieMenuMessage::Event(name)) => {
        println!("Menu item clicked: {}", name);
        match name.as_str() {
            "exit" => application.quit(),
            "settings" => open_settings(),
            _ => {}
        }
    }
    Err(_) => {}
}
```

## Channel Setup

The message channel uses `std::sync::mpsc::Sender<PieMenuMessage>`. The sender is set on the `PieMenuOverlayWidget` via the `PieMenuMessageSender` trait. Messages are sent from GTK signal handlers, so the receiver should poll with `try_recv()` in a `glib::timeout_add_local` callback or similar mechanism.
