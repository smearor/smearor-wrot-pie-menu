# Examples

The library ships with an interactive demo application demonstrating the `PieMenuOverlayWidget` in action.

## Interactive Demo

A `PieMenuOverlayWidget` with a label child and several configurable menu items.

### Launch

```sh
cargo run --example interactive_demo
```

### Description

The interactive demo provides:

- **Pinch-to-zoom**: Opens the pie menu when scale > 3.5, closes when scale < 0.5
- **Rotation gesture**: Rotates the menu ring with a two-finger twist
- **Menu item clicks**: Sends `PieMenuMessage::Event` with the item's event name
- **Center click**: Closes the pie menu
- **Mouse hover**: Highlights the nearest menu item

The demo shows how to set up the message channel, add menu items with custom icons/colors/angles, and handle incoming messages in the application event loop.
