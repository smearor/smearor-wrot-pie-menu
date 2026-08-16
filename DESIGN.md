# Design: smearor-wrot-pie-menu

## Architecture

The crate is organized into three main layers:

### 1. Data Model (`menu/`)

- **`MenuItem`**: A single pie menu item with id, label, icon, color, angle, radius, and event name. Uses `TypedBuilder` for ergonomic construction. `Hash`/`Eq` by id.
- **`Menu`**: A `DashMap`-backed collection of `MenuItem` indexed by id. Supports a builder pattern for declarative construction.
- **`color`**: Self-contained `RgbaColor` and `RgbColor` types with hex parsing and `gdk::RGBA` conversion.

### 2. Pie Menu Widget (`menu_widget/`)

- **`PieMenuWidget`**: The inner GTK4 widget that renders the circular menu. Handles:
  - Ring-shaped background rendering with shadow
  - 5-degree markings with highlights for current rotation and zero position
  - Menu item rendering (icon, label, background circle with hover highlight)
  - Mouse hover detection via `EventControllerMotion`
  - Rotation state (stored as `AtomicF32`)

### 3. Overlay Widget (`overlay_widget/`)

- **`PieMenuOverlayWidget`**: The outer GTK4 widget that wraps a child widget and overlays the pie menu on top. Handles:
  - Pinch-to-zoom gesture detection (open/close menu)
  - Rotation gesture detection (rotate menu + send `Rotate` messages)
  - Click detection (center circle to close, menu items to send `Event` messages)
  - Message channel via `mpsc::Sender<PieMenuMessage>`

## Message Flow

```
User Gesture → PieMenuOverlayWidget → mpsc::Sender<PieMenuMessage> → Consumer App
```

- **Rotation gesture**: Sends `PieMenuMessage::Rotate(f32)` with the absolute rotation in degrees
- **Menu item click**: Sends `PieMenuMessage::Event(String)` with the clicked item's event name
- **Center click**: Closes the menu (no message sent)

## Trait Hierarchy

- `RotationHandler` — `set_rotation(f32)` — implemented for `PieMenuWidget`, `PieMenuWidgetImpl`, `PieMenuOverlayWidget`, `PieMenuOverlayWidgetImpl`
- `PieMenuMenuItemHandler` — `add_menu_item`/`remove_menu_item` — implemented for `PieMenuWidget`, `PieMenuWidgetImpl`, `PieMenuOverlayWidget`, `PieMenuOverlayWidgetImpl`
- `PieMenuControlHandler` — `show_pie_menu`/`hide_pie_menu`/`is_pie_menu_open` — implemented for `PieMenuOverlayWidget`, `PieMenuOverlayWidgetImpl`
- `PieMenuMessageSender` — `set_message_sender`/`send_message` — implemented for `PieMenuOverlayWidget`, `PieMenuOverlayWidgetImpl`

## Dependencies

The crate is fully self-contained with no dependencies on other `smearor-wrot` crates.

| Dependency | Purpose |
|------------|---------|
| `gtk4` | GTK4 widget framework |
| `glib` | GObject type system |
| `dashmap` | Thread-safe menu item storage |
| `typed-builder` | Ergonomic struct construction |
| `atomic_float` | Atomic rotation state |
| `tracing` | Logging |
| `thiserror` | Error types |
| `miette` | User-facing errors |
| `serde` | Serialization (future config support) |
| `gtk4-layer-shell` | Optional layer shell support |
