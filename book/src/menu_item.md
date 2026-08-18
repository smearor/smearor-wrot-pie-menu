# MenuItem

The `MenuItem` struct represents a single item in the pie menu. It is constructed using the `TypedBuilder` pattern.

## Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `String` | Yes | — | Unique identifier |
| `label` | `String` | Yes | — | Display text below the icon |
| `label_color` | `RgbaColor` | No | White (`#FFFFFFFF`) | Color of the label text |
| `icon_name` | `String` | Yes | — | GTK icon theme name |
| `color` | `RgbaColor` | No | Grey (`#77777777`) | Background circle color |
| `angle` | `f32` | Yes | — | Position in degrees (0 = right, 90 = down) |
| `radius` | `Option<f32>` | No | `40.0` | Item circle radius in pixels |
| `event` | `String` | Yes | — | Event name sent as `PieMenuMessage::Event` |
| `enabled` | `bool` | No | `true` | Whether the item is clickable (disabled items render at reduced opacity) |
| `fixed_position` | `bool` | No | `false` | When `true`, the item's angle is treated as a fixed semantic position that resists auto-redistribution |
| `close_on_click` | `bool` | No | `true` | Whether the pie menu closes after this item is clicked |
| `submenu` | `Option<Vec<MenuItem>>` | No | `None` | Optional nested submenu items. When present, clicking the item opens the submenu ring instead of sending an event. |

## Construction

```rust
use smearor_wrot_pie_menu::MenuItem;

let item = MenuItem::builder()
    .id("rotate-cw")
    .label("Rotate CW")
    .icon_name("object-rotate-right-symbolic")
    .color("#00000077")
    .label_color("#FFFFFFFF")
    .angle(0.0)
    .radius(30.0)
    .event("rotate-cw")
    .enabled(true)
    .fixed_position(false)
    .close_on_click(true)
    .build();
```

## Colors

Colors accept any type that implements `Into<RgbaColor>`:
- `&str` / `String` — hex string like `"#RRGGBBAA"` or `"#RRGGBB"`
- `RgbColor` — RGB only (alpha defaults to 1.0)
- `RgbaColor` — full RGBA

## Equality

`MenuItem` implements `Hash`, `PartialEq`, and `Eq` based **only** on the `id` field. This allows items with the same id to be treated as equal regardless of other fields.

## Default Constants

- `DEFAULT_MENU_ITEM_RADIUS: f32 = 40.0`
- `DEFAULT_LABEL_COLOR: RgbaColor = white`
- `DEFAULT_ICON_COLOR: RgbaColor = grey`

## Submenus

A `MenuItem` can optionally contain a nested submenu via the `submenu` field. When a user clicks an item that has a submenu, the menu opens a new ring at a larger radius instead of sending an event.

```rust
use smearor_wrot_pie_menu::MenuItem;

let child = MenuItem::builder()
    .id("child")
    .label("Child")
    .icon_name("icon")
    .angle(0.0)
    .event("child-event")
    .build();

let parent = MenuItem::builder()
    .id("parent")
    .label("Parent")
    .icon_name("icon")
    .angle(0.0)
    .event("parent-event")
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
