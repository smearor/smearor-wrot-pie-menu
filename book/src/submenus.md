# Submenus

The pie menu supports hierarchical submenus — nested rings that appear at increasing radii when a parent item is activated.

## Concept

Each `MenuItem` can optionally carry a `submenu: Option<Vec<MenuItem>>` field. When a user clicks (or confirms via keyboard) an item that has a submenu, a new ring is rendered at a larger radius instead of sending an event. The user can navigate back by pressing `Escape` or clicking the center circle.

## Building Submenu Items

```rust
use smearor_wrot_pie_menu::MenuItem;

let sub_item_a = MenuItem::builder()
    .id("sub-a")
    .label("Sub A")
    .icon_name("icon")
    .angle(0.0)
    .event("sub-a-event")
    .build();

let sub_item_b = MenuItem::builder()
    .id("sub-b")
    .label("Sub B")
    .icon_name("icon")
    .angle(180.0)
    .event("sub-b-event")
    .build();

let parent = MenuItem::builder()
    .id("parent")
    .label("Parent")
    .icon_name("icon")
    .angle(90.0)
    .event("parent-event")
    .submenu(vec![sub_item_a, sub_item_b])
    .build();
```

## Navigation API

The `PieMenuControlHandler` trait provides the following submenu methods:

| Method | Returns | Description |
|--------|---------|-------------|
| `open_submenu(parent_id)` | `Result<(), SubmenuError>` | Opens the submenu of the item with the given id |
| `close_submenu()` | `Result<(), SubmenuError>` | Closes the current submenu and returns to the parent ring |
| `submenu_depth()` | `u32` | Returns the current submenu depth (0 = main ring) |
| `get_submenu_items(parent_id)` | `Vec<MenuItem>` | Returns the submenu items of the given parent |
| `redistribute_submenu(parent_id)` | `()` | Redistributes submenu item angles |
| `set_submenu_items(parent_id, items)` | `Result<(), SubmenuError>` | Replaces submenu items and redistributes angles |
| `set_submenu_radius(level, radius)` | `()` | Sets a per-level radius override |
| `set_submenu_radius_step(step)` | `()` | Sets the global step width between ring levels |
| `max_submenu_depth()` | `u32` | Returns the maximum allowed nesting depth |

## Rendering

- The main ring renders at `radius` (default: 160px).
- Each submenu level *n* renders at `main_radius + n * submenu_radius_step`.
- `submenu_radius_step` defaults to `80.0` pixels.
- Inactive (parent) rings are rendered at reduced opacity.
- A yellow indicator dot is drawn on the parent item to show which submenu is open.
- Breadcrumb dots are drawn between rings to indicate depth.

## Input Behavior

| Input | Submenu open | No submenu open |
|-------|-------------|-----------------|
| Click item with submenu | Opens submenu | Opens submenu |
| Click item without submenu | Sends event | Sends event |
| Click center circle | Closes submenu | Closes menu |
| `Escape` | Closes submenu | Closes menu |
| `Enter` / `Space` on item with submenu | Opens submenu | Opens submenu |
| `Enter` / `Space` on item without submenu | Sends event | Sends event |

## Error Handling

`SubmenuError` covers the following cases:

- `NotFound` — no menu item with the given id exists
- `NoSubmenu` — the item exists but has no submenu
- `MaxDepthReached` — the maximum nesting depth (`3`) has been reached
- `NoSubmenuOpen` — `close_submenu` was called but no submenu is open
- `ItemOverlap` — submenu items overlap after angle redistribution

## Constants

- `MAX_SUBMENU_DEPTH: u32 = 3`
- `DEFAULT_SUBMENU_RADIUS_STEP: f64 = 80.0`
