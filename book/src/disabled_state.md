# Disabled State

Menu items can be individually disabled. A disabled item is rendered at reduced opacity, does not respond to hover, and click events are suppressed.

## The `enabled` Field

`MenuItem` has an `enabled` field that defaults to `true`:

```rust
let item = MenuItem::builder()
    .id("save")
    .angle(0.0)
    .event("save")
    .config(CircleConfig::builder()
        .icon_name("document-save-symbolic")
        .label("Save")
        .build())
    .enabled(false) // disabled by default
    .build();
```

## Toggling at Runtime

Use `set_menu_item_enabled()` to change the state at runtime. This triggers a redraw:

```rust
// Disable an item
overlay.set_menu_item_enabled("save", false)?;

// Re-enable an item
overlay.set_menu_item_enabled("save", true)?;
```

Returns `Err(SetMenuItemEnabledError::NotFound)` if the item id does not exist.

## Rendering

Disabled items are rendered with **40% opacity** on both the item circle and label. Hover detection skips disabled items, so no highlight is applied. Click events on disabled items are ignored entirely.
