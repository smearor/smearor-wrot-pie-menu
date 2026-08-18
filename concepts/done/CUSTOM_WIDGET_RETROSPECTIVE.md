# Custom Widget — Retrospective

---

## 1. Overview

This document is a retrospective on the implementation of the custom widget system described in `concepts/planned/CUSTOM_WIDGET.md`. It catalogs the severe runtime problems that occurred after the initial implementation, identifies the root causes in the concept, and records lessons learned.

---

## 2. Problems After Initial Implementation

After the concept was implemented as written, the following severe problems were present:

1. **Widgets were not rendered** — Built and parented widgets were invisible
2. **Redraws were not triggered** — State changes did not cause visual updates
3. **Size calculations were not performed** — Widgets received no allocation
4. **Old snapshot rendering path was still active** — The legacy `snapshot` code drew items directly, and the new widget-child pipeline did not take effect
5. **Submenu support was insufficient** — The concept did not account for ring-level switching, widget visibility across submenu levels, or cache invalidation on submenu open/close
6. **Keyboard navigation was non-functional** — Selection state changes did not propagate to cached widgets
7. **Rotation was non-functional** — Widget rotation did not work because the concept conflated `size_allocate` (positioning) with `snapshot` (transform)

---

## 3. Root Causes in the Concept

### 3.1 No Migration Strategy: Snapshot → Widget-Child

The concept described the target state ("snapshot draws only ring-level elements") but not the transition. There was no inventory of the existing `snapshot` code — which lines draw what, and where each responsibility migrates. This led to the old snapshot path remaining active while the new path was never engaged.

**What was missing**: An explicit list of what to remove from `snapshot` and what to replace it with in the widget pipeline.

### 3.2 GTK4 Widget Lifecycle Not Specified

The concept mentioned `set_parent` and `size_allocate`, but not the critical steps between and after:

- `widget.show()` — A parented but not shown widget is invisible
- `queue_resize()` — A built widget receives no allocation without a resize request
- **Sequence**: `build → set_parent → show → queue_resize → size_allocate`

Without this sequence, widgets were built and parented but never visible or allocated. This explains "widgets were not rendered" and "size calculations were not performed".

### 3.3 No Trigger Strategy for Redraws

The concept mentioned `queue_draw()` in `refresh_widgets`/`set_widget_config`, but not that the initial widget build must also trigger a resize + draw. `set_parent` alone does not trigger a layout pass.

**What was missing**: "After `set_parent`, call `queue_resize()` on the parent to trigger the allocation phase."

### 3.4 Submenu Rendering Not Designed

The concept stated "widgets in submenu rings are also supported — the same caching and allocation logic applies regardless of ring level." But it did not design:

- How `item_widgets` cache works when the visible ring level changes (submenu open/close)
- That parent-ring widgets must be **unparented/hidden** when a submenu opens
- That `size_allocate` must know **which ring level** is currently active
- Whether `item_widgets` is per-ring-level or flat with a visibility flag

The `size_allocate` code in the concept iterated only `menu_items` — a flat list with no concept of "current ring level" or "submenu_stack".

### 3.5 State Propagation to Cached Widgets Not Designed

The concept stated "the keyboard selection highlight is drawn by the standard implementations." But the widget is **built once and cached** — when keyboard selection state changes, how does the widget learn about it? There was no mechanism such as:

- An `update_state(&self, widget: &Widget, state: &WidgetState)` method on the factory
- A property-based update (GObject property `is-selected` on the custom widget)
- A rebuild on state change

This led to keyboard navigation changing state internally, but cached widgets never reflecting the change.

### 3.6 Rotation Interaction with Widget Pipeline Not Designed

The old rotation was implemented as a transform in the `snapshot` callback. The concept said "rotation transform is applied to the widget via `gtk4::Transform` during the allocation pass" — but `size_allocate` works with `Allocation` (position/size), not transforms. Transforms are applied in a widget's `snapshot`, not in `size_allocate`. The concept conflated two GTK4 concepts:

- **Positioning** (`size_allocate` → where the widget sits on the parent)
- **Rotation** (`snapshot` → how the widget renders its content)

Rotating a child widget requires either a `Transform` property on the widget or a custom `snapshot` override that reads the rotation angle from the parent. The concept did not specify how the rotation angle travels from `PieMenuWidgetImpl` to the child widget.

### 3.7 No "What Must Go" Section

The concept was framed **additively** — "add trait, add registry, add fields." But the core task was **subtractive**: the entire item-rendering code in `snapshot` had to be removed. There was no explicit listing such as:

- Remove icon drawing from `snapshot`
- Remove label drawing from `snapshot`
- Remove circle drawing from `snapshot`
- Remove selection-ring drawing from `snapshot`
- Remove hover-highlight drawing from `snapshot`

Without this list, the old code remained active, and the new widget path ran in parallel or not at all.

---

## 4. Summary

The core weakness: **The concept described the architecture (traits, structs, registry) in detail, but the GTK4 rendering lifecycle only superficially.** All severe problems stemmed from the transition from "GTK4 draws in snapshot" to "GTK4 manages child widgets" not being designed with concrete GTK4 mechanisms (`show`, `queue_resize`, `unparent`, transform properties, ring-level switching, state propagation).

### Problem → Root Cause Mapping

| Problem | Root Cause in Concept |
|---------|----------------------|
| Widgets not rendered | No widget lifecycle sequence (show, queue_resize) |
| Redraws not triggered | No trigger strategy after set_parent |
| Size calculations not done | No queue_resize after build |
| Old snapshot still active | No "what must go" section, no migration inventory |
| Submenu not working | No ring-level awareness in cache/allocate |
| Keyboard navigation broken | No state propagation to cached widgets |
| Rotation broken | Conflated size_allocate with snapshot transform |

---

## 5. Lessons Learned

1. **A concept must cover the "how" of the runtime, not just the "what" of the API.**
2. **For rendering migrations, explicitly list what code is removed and what replaces it.**
3. **Specify the exact GTK4 lifecycle calls and their order.**
4. **Design state propagation for cached/long-lived objects.**
5. **For multi-level UI (submenus), design cache invalidation and visibility per level.**
6. **Distinguish between positioning (allocation) and rendering (snapshot/transform).**
