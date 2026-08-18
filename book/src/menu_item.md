# MenuItem

The `MenuItem` struct represents a single item in the pie menu. It is constructed using the `TypedBuilder` pattern.

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `String` | Yes | — | Unique identifier |
| `angle` | `f32` | Yes | — | Position in degrees (0 = right, 90 = down) |
| `event` | `String` | Yes | — | Event name sent as `PieMenuMessage::Event` |
| `radius` | `Option<f32>` | No | `40.0` | Item circle radius in pixels |
| `enabled` | `bool` | No | `true` | Whether the item is clickable (disabled items render at reduced opacity) |
| `fixed_position` | `bool` | No | `false` | When `true`, the item's angle is treated as a fixed semantic position that resists auto-redistribution |
| `close_on_click` | `bool` | No | `true` | Whether the pie menu closes after this item is clicked |
| `submenu` | `Option<Vec<MenuItem>>` | No | `None` | Optional nested submenu items |
| `widget_type` | `Option<String>` | No | `None` (`"circle"`) | Widget type name for registry lookup |
| `widget_config` | `Option<serde_json::Value>` | No | `None` | Type-specific widget configuration |
| `content_size` | `Option<ItemSize>` | No | `None` | Non-square allocation size |
| `content_rotates` | `bool` | No | `true` | Whether the widget rotates with the ring |

Visual properties (icon, label, colors) are defined in widget-specific config structs, not on `MenuItem`. See [Widget System](widget_system.md) for details.

## Construction

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;

let item = MenuItem::builder()
    .id("rotate-cw")
    .angle(0.0)
    .event("rotate-cw")
    .widget_type("circle")
    .config(CircleConfig::builder()
        .icon_name("object-rotate-right-symbolic")
        .label("Rotate CW")
        .color("#00000077")
        .build())
    .build();
```

The `.config()` builder method accepts any `Serialize` type and serializes it to `serde_json::Value` internally.

## Widget Configuration

### widget_type

Resolves the factory from the registry. When `None`, defaults to `"circle"`. Standard types: `"circle"`, `"square"`, `"button"`. Custom types can be registered via `register_widget_factory()`.

### widget_config

Type-specific configuration stored as `serde_json::Value`. The factory's `Config` type defines the schema. When `None`, the factory's `Config::default()` is used.

### content_size

Optional non-square allocation for widgets that need dimensions other than `2 * radius`. See [Widget System](widget_system.md).

### content_rotates

When `true` (default), the widget rotates with the ring. When `false`, the widget stays upright.

## Equality

`MenuItem` implements `Hash`, `PartialEq`, and `Eq` based **only** on the `id` field. This allows items with the same id to be treated as equal regardless of other fields.

## Default Constants

- `DEFAULT_MENU_ITEM_RADIUS: f32 = 40.0`
- `DEFAULT_LABEL_COLOR: RgbaColor = white` — used by factories when `label_color` is `None`
- `DEFAULT_ICON_COLOR: RgbaColor = grey` — used by factories when `color` is `None`

## Submenus

A `MenuItem` can optionally contain a nested submenu via the `submenu` field. When a user clicks an item that has a submenu, the menu opens a new ring at a larger radius instead of sending an event.

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;

let child = MenuItem::builder()
    .id("child")
    .angle(0.0)
    .event("child-event")
    .config(CircleConfig::builder()
        .icon_name("icon")
        .label("Child")
        .build())
    .build();

let parent = MenuItem::builder()
    .id("parent")
    .angle(0.0)
    .event("parent-event")
    .config(CircleConfig::builder()
        .icon_name("icon")
        .label("Parent")
        .build())
    .submenu(vec![child])
    .build();
```

### Submenu IDs

IDs must be **globally unique** across the entire menu tree, including all submenu levels. Duplicate IDs at any level are undefined behavior.

### Submenu Navigation

- **Open**: Click an item with a submenu, or press `Enter`/`Space` on a keyboard-selected item with a submenu.
- **Close**: Press `Escape` or click the center circle. If a submenu is open, the first `Escape`/center-click closes the submenu and returns to the parent ring. A second `Escape`/center-click closes the entire menu.
- **Maximum depth**: `MAX_SUBMENU_DEPTH = 3`. Attempting to open a deeper submenu returns `SubmenuError::MaxDepthReached`.

### Submenu Messages

- `PieMenuMessage::SubmenuOpened(String)` — sent when a submenu is opened, containing the parent item's id.
- `PieMenuMessage::SubmenuClosed(String)` — sent when a submenu is closed, containing the parent item's id.

### Submenu Radii

Each submenu level renders at an increasing radius. The radius for level *n* is computed as:

```
radius = main_radius + n * submenu_radius_step
```

- `submenu_radius_step` defaults to `80.0` pixels and can be changed via `set_submenu_radius_step`.
- Individual level overrides can be set via `set_submenu_radius(level, radius)`.
