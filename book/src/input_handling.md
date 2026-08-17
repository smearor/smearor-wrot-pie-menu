# Input Handling

The pie menu supports three additional input methods beyond the default touch gestures: keyboard navigation, mouse scroll rotation, and game controller input. Keyboard and mouse scroll are enabled by default; controller support is opt-in via feature flags.

## Feature Flags

| Feature | Description | Extra Dependencies |
|---------|-------------|--------------------|
| `keyboard` | Keyboard navigation | None |
| `mouse-scroll` | Mouse wheel ring rotation | None |
| `controller-sdl2` | Game controller via SDL2 | `sdl2` |
| `controller-evdev` | Game controller via evdev (Linux-only) | `evdev` |

```toml
[dependencies]
smearor-wrot-pie-menu = "0.1"
```

To opt out of keyboard or mouse scroll, use `default-features = false`:

```toml
[dependencies]
smearor-wrot-pie-menu = { version = "0.1", default-features = false }
```

To add controller support:

```toml
[dependencies]
smearor-wrot-pie-menu = { version = "0.1", features = ["controller-sdl2"] }
```

## Keyboard Navigation

When the `keyboard` feature is enabled, an `EventControllerKey` is registered on the root window (in `WidgetImpl::root()`) with `PropagationPhase::Capture`. This ensures key events are intercepted globally, regardless of which child widget currently holds keyboard focus. When the menu is closed, all keys except `Ctrl+Space` and `Menu` pass through to the focused child.

### Key Bindings

| Key | Action | Condition |
|-----|--------|-----------|
| `Ctrl+Space` | Open the pie menu | Menu closed |
| `Menu` | Open the pie menu | Menu closed |
| `Escape` | Close the pie menu | Menu open |
| `Enter` / `Space` | Confirm selection | Menu open |
| `Arrow Left` / `Arrow Down` | Cycle selection CCW | Menu open |
| `Arrow Right` / `Arrow Up` | Cycle selection CW | Menu open |
| `Tab` | Cycle selection CW | Menu open |
| `Home` | Select first item | Menu open |

### ID-based Selection

Keyboard selection uses item IDs (strings) rather than indices. This ensures selection remains stable across `DashMap` insertion/removal order changes. The selection state is stored as `RefCell<Option<String>>` in `PieMenuWidgetImpl`.

### Cycle Selection

The `cycle_selection(direction)` method filters out disabled items, sorts the remaining by angle using `total_cmp` for deterministic ordering, then advances the selection by `direction` (-1 for CCW, +1 for CW), wrapping around. Disabled items are skipped during navigation.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

// Cycle forward (CW)
overlay.cycle_selection(1);

// Cycle backward (CCW)
overlay.cycle_selection(-1);

// Select first item (smallest angle)
overlay.select_first_item();

// Confirm the current selection
overlay.confirm_selection();
```

## Mouse Scroll Rotation

When the `mouse-scroll` feature is enabled, an `EventControllerScroll` (vertical-only) is added with `PropagationPhase::Capture`. Scrolling rotates the menu ring proportionally to `dy`.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

// Set scroll sensitivity (default: 5.0)
overlay.set_scroll_rotation_step(10.0);

// Get current sensitivity
let step = overlay.scroll_rotation_step();
```

The rotation delta is computed as `dy * scroll_rotation_step`. With the default sensitivity of `5.0`, a single mouse wheel tick (`dy ≈ 1.0`) rotates the ring by ~5°.

## Controller Support

Controller input is available via two mutually exclusive backends:

- `controller-sdl2`: Uses the SDL2 game controller API (cross-platform)
- `controller-evdev`: Uses the Linux evdev API (Linux-only)

### Analog Stick Handling

The left stick X-axis drives continuous ring rotation via a `add_tick_callback`. The stick value is stored in a `Cell<f32>` on `PieMenuOverlayWidgetImpl` and applied per-frame.

A deadzone of `0.15` prevents drift when the stick is at rest.

```rust
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

// Update left stick X value (call from main thread)
overlay.handle_left_stick_x(0.8);
```

The right stick selects the nearest menu item based on stick direction. The stick angle is computed as `atan2(-y, x)` and converted to degrees. A deadzone of `0.3` prevents accidental selection.

```rust
// Update right stick (call from main thread)
overlay.handle_right_stick(0.7, -0.5);
```

### Thread Safety

`Cell<f32>` is `!Sync`. All writes to `left_stick_x` must occur on the GTK main thread. If polling controller events from a background thread, marshal updates via `glib::idle_add_local`:

```rust
let widget_clone = overlay.clone();
glib::idle_add_local(move || {
    widget_clone.handle_left_stick_x(0.5);
    glib::ControlFlow::Break
});
```

### Finding Nearest Item

The `find_nearest_item(target_angle)` method returns the ID of the enabled menu item whose angle is closest to `target_angle` (in degrees), accounting for wraparound at 360°. Disabled items are skipped.

```rust
if let Some(id) = overlay.find_nearest_item(45.0) {
    overlay.set_keyboard_selection(id);
}
```
