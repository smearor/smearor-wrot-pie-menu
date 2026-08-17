# smearor-wrot-pie-menu

[![crates.io](https://img.shields.io/badge/crates.io-0.1.0-dc0073.svg)](https://crates.io/crates/smearor-wrot-pie-menu)
[![Rust Edition](https://img.shields.io/badge/rust-2024-f5b700.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![GTK4](https://img.shields.io/badge/GTK4-v4__20-f5b700.svg)](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
[![License](https://img.shields.io/badge/license-MIT-89fc00.svg)](LICENSE.md)
[![Book](https://img.shields.io/badge/book-main-00a1e4.svg)](https://smearor.github.io/smearor-wrot-pie-menu/book/)
[![Docs](https://img.shields.io/badge/docs-main-00a1e4.svg)](https://smearor.github.io/smearor-wrot-pie-menu/docs/)

A GTK4 pie menu widget with touch gesture activation for circular menu selection.

## Overview

`smearor-wrot-pie-menu` provides a circular pie menu widget that appears when a
pinch-to-zoom gesture is detected. Menu items are arranged in a ring layout for
easy touch access. The widget is fully configurable — consumers add menu items
via the [`MenuItem`] API and receive events via [`PieMenuMessage`].

## Features

- **Touch gesture activation**: Opens on pinch-to-zoom (scale > 3.5), closes on pinch-out (scale < 0.5)
- **Rotation gesture**: Rotate the menu ring with a two-finger rotation gesture
- **Configurable menu items**: Add/remove items programmatically with custom icons, colors, angles, and events
- **Hover detection**: Mouse hover highlights the nearest menu item
- **Click-to-select**: Click a menu item to trigger its event
- **Center close button**: Click the center circle to close the menu
- **GTK4 native**: Built as a proper GTK4 widget with `BinLayout` overlay

## Quick Start

```rust
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use std::sync::mpsc::channel;

let (sender, receiver) = channel::<PieMenuMessage>();

let overlay = PieMenuOverlayWidget::new(Some(&child_widget));
overlay.set_message_sender(sender);

// Add menu items
overlay.add_menu_item(
    MenuItem::builder()
        .id("rotate-cw")
        .label("Rotate CW")
        .icon_name("object-rotate-right-symbolic")
        .color("#00000077")
        .angle(0.0)
        .radius(30.0)
        .event("rotate-cw")
        .build(),
);

// Handle messages in your event loop
match receiver.try_recv() {
    Ok(PieMenuMessage::Rotate(degrees)) => { /* handle rotation */ }
    Ok(PieMenuMessage::Event(name)) => { /* handle event by name */ }
    Err(_) => {}
}
```

## Interactive Demo

![Interactive Demo](book/src/assets/interactive-demo.png)

Run the interactive demo that integrates `smearor-wrot-rotation`:

```sh
cargo run --example interactive_demo
```

## API

### `PieMenuOverlayWidget`

The main widget. Wrap any child widget with this overlay to add pie menu functionality.

### `MenuItem`

A single menu item with an id, label, icon, color, angle, radius, and event name.

### `PieMenuMessage`

Messages sent from the pie menu to the consumer:
- `Rotate(f32)` — rotation delta in degrees from the rotation gesture
- `Event(String)` — the event name of the clicked menu item

## License

MIT