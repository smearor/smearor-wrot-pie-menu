# Overlap Validation

When adding menu items, the widget validates that the new item does not visually overlap with existing items on the ring.

## How Overlap Is Detected

Two items overlap when the distance between their positions on the ring is less than the sum of their radii, normalized by the ring radius:

```text
distance < (item_a.radius + item_b.radius) / ring_radius
```

Positions are computed from the item angle and ring radius using trigonometry.

## Error Type

```rust
pub enum AddMenuItemError {
    MenuWidgetNotAvailable,
    ItemOverlap { id: String, overlapping_with: String },
}
```

## Transactional Rollback

When `add_menu_item()` detects an overlap after inserting the item into the `DashMap`:

1. The item is **removed** from the map
2. `AddMenuItemError::ItemOverlap` is returned

For `add_menu_item_auto()`, the rollback is more comprehensive:

1. The new item is removed (or the previous item is restored on overwrite)
2. All angle changes from `redistribute_angles()` are reverted using an angle snapshot
3. `AddMenuItemError::ItemOverlap` is returned

## Validation Methods

- `validate_no_overlap(new_item, ring_radius)` — checks a single item against all existing items
- `validate_all_no_overlap(ring_radius)` — checks all items against each other (used after redistribution)

## Guards

- **Zero ring radius**: Returns `Ok(())` (e.g. during initialization when the widget has no size yet)
- **Self-overlap**: An item is never compared against itself (matched by id)
