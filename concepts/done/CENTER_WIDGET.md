# Center Widget — Optional Rotating Center Content

---

## 1. Goal and Motivation

This document describes the concept for an optional center widget that renders inside the pie menu ring's transparent center area. The center widget rotates with the ring and is responsible for its own event handling, including close-menu and close-submenu interactions.

### Goal

Introduce a `set_center_widget(Option<&Widget>)` API on `PieMenuWidget` that allows consumers to place any GTK4 widget (logo, label, value display, gauge) in the center of the pie menu. When no center widget is set, behavior is unchanged — the existing center-click-to-close logic remains active. When a center widget is set, the consumer is responsible for implementing close-menu and close-submenu behavior via their own event controllers on the center widget.

### Motivation

The pie menu's center area is currently a transparent hole with hardcoded click-to-close behavior. Consumers cannot:

- Render a logo in the center (brand identity)
- Display a label or value in the center (contextual information)
- Render an interactive gauge in the center (e.g., a CPU gauge surrounded by metric gauges)
- Customize the center click behavior beyond the built-in close logic

By allowing an optional center widget with full event handling responsibility, the center becomes a first-class extension point — consistent with the widget system for menu items.

---

## 2. Current State

The `smearor-wrot-pie-menu` library currently provides:

- **Center rendering**: The pie menu ring is drawn as an annulus (outer circle + inner circle with EvenOdd fill rule) in `PieMenuWidgetImpl::snapshot`. The center is transparent — nothing is drawn inside `center_radius`.
- **Center click handling**: In `PieMenuOverlayWidgetImpl::connect_pressed` (`src/overlay_widget/imp/widget.rs:333-340`), clicks within `center_radius` trigger:
  - `close_submenu()` if `submenu_depth() > 0`
  - `hide_pie_menu()` if `submenu_depth() == 0`
- **Close callback**: `PieMenuWidget::set_close_callback` stores a `Box<dyn Fn()>` that is called when the center is clicked. This callback is set during `PieMenuOverlayWidget::new` to call `hide_pie_menu()`.
- **Rotation**: `PieMenuWidgetImpl::snapshot` applies `snapshot.save()` → `snapshot.translate(center)` → `snapshot.rotate(rotation)` → `snapshot.translate(-center)` before drawing ring content and rotating item widgets. The transform is restored with `snapshot.restore()` after rotating items are rendered.
- **Item widgets**: Menu item widgets are GTK4 child widgets, built by factories, registered via `set_parent`, positioned by `size_allocate`, and rendered via `snapshot_child`. They are cached in `item_widgets: RefCell<HashMap<String, Widget>>`.
- **`center_radius`**: Stored as `AtomicF32` in `PieMenuWidgetImpl`, controllable via `set_pie_menu_center_radius(f32)`.

### Files Affected

| File | Role |
|------|------|
| `src/menu_widget/imp/widget.rs` | `PieMenuWidgetImpl` — storage, `size_allocate`, `snapshot`, `measure` |
| `src/menu_widget/widget.rs` | `PieMenuWidget` — public API (`set_center_widget`) |
| `src/overlay_widget/imp/widget.rs` | `PieMenuOverlayWidgetImpl` — center click detection in `connect_pressed` |
| `src/overlay_widget/widget.rs` | `PieMenuOverlayWidget` — builder method `with_center_widget` |
| `src/overlay_widget/control/handler.rs` | `PieMenuControlHandler` trait — no changes needed (existing `close_submenu` / `hide_pie_menu` suffice) |

### What is Missing

| Feature | Status |
|---------|--------|
| `center_widget` storage in `PieMenuWidgetImpl` | Not implemented |
| `set_center_widget` / `center_widget` API on `PieMenuWidget` | Not implemented |
| `with_center_widget` builder on `PieMenuOverlayWidget` | Not implemented |
| Center widget allocation in `size_allocate` | Not implemented |
| Center widget rendering in `snapshot` (inside rotation transform) | Not implemented |
| Center widget measurement in `measure` | Not implemented |
| Conditional center-click bypass when center widget is set | Not implemented |
| Teardown (`unparent`) on center widget replacement/removal | Not implemented |

---

## 3. Data Model

### PieMenuWidgetImpl — New Field

```rust
/// Optional center widget rendered inside the ring's transparent center.
/// Rotates with the ring. When set, the consumer is responsible for
/// close-menu / close-submenu event handling.
pub(crate) center_widget: RefCell<Option<Widget>>,
```

### PieMenuWidget — Public API

```rust
impl PieMenuWidget {
    /// Sets or removes the center widget.
    ///
    /// When `Some(widget)`, the widget is registered as a child of
    /// `PieMenuWidget` via `set_parent`, made visible via `show()`,
    /// and a resize is triggered via `queue_resize()`.
    ///
    /// When `None`, any existing center widget is unparented and
    /// the default center-click-to-close behavior is restored.
    ///
    /// The center widget rotates with the ring. The consumer is
    /// responsible for attaching event controllers (e.g.,
    /// `GestureClick`) to handle close-menu / close-submenu
    /// interactions.
    ///
    /// # Reentrancy Safety
    ///
    /// `unparent()` and `set_parent()` emit GTK lifecycle signals
    /// (`unmap`, `hierarchy-changed`) that may trigger consumer
    /// callbacks. If those callbacks call `center_widget()` or
    /// `set_center_widget()`, a still-active `borrow_mut()` guard
    /// would cause a `BorrowMutError` panic.
    ///
    /// To prevent this, the old widget is taken out of the `RefCell`
    /// and the borrow guard is dropped **before** `unparent()` is
    /// called. Similarly, `set_parent()` is called before the new
    /// widget is stored in the `RefCell`.
    pub fn set_center_widget(&self, widget: Option<&Widget>) {
        let imp = self.imp();

        // 1. Take old widget out of RefCell — borrow guard drops immediately
        let old_widget = imp.center_widget.borrow_mut().take();

        // 2. Unparent OUTSIDE the borrow — safe even if GTK signals
        //    trigger reentrant access to center_widget
        if let Some(existing) = old_widget {
            existing.unparent();
        }

        // 3. Parent new widget BEFORE storing it in RefCell —
        //    set_parent emits hierarchy-changed which could trigger
        //    consumer callbacks; the RefCell must not be borrowed
        //    when those fire
        if let Some(new_widget) = widget {
            new_widget.set_parent(self);
            new_widget.show();
            *imp.center_widget.borrow_mut() = Some(new_widget.clone());
        }

        self.queue_resize();
        self.queue_draw();
    }

    /// Returns the current center widget, if any.
    pub fn center_widget(&self) -> Option<Widget> {
        self.imp().center_widget.borrow().clone()
    }
}
```

### PieMenuOverlayWidget — Builder Method

```rust
impl PieMenuOverlayWidget {
    /// Sets the center widget and returns self for chaining.
    /// The center widget rotates with the ring and is responsible
    /// for its own event handling (close-menu, close-submenu).
    pub fn with_center_widget(self, widget: &Widget) -> Self {
        self.set_center_widget(Some(widget));
        self
    }

    /// Delegates to `PieMenuWidget::set_center_widget`.
    pub fn set_center_widget(&self, widget: Option<&Widget>) {
        let pie_menu_widget_borrow = self.imp().pie_menu_widget.borrow();
        if let Some(pie_menu_widget) = pie_menu_widget_borrow.as_ref() {
            pie_menu_widget.set_center_widget(widget);
        }
    }
}
```

---

## 4. Runtime Lifecycle

### 4.1 Build Sequence

When `set_center_widget(Some(&widget))` is called:

1. **Unparent existing**: If `center_widget` is `Some`, call `existing.unparent()` to detach the old widget from the widget tree.
2. **Set parent**: Call `new_widget.set_parent(Some(&pie_menu_widget))` to attach the new widget as a child of `PieMenuWidget`.
3. **Show**: Call `new_widget.show()` to make the widget visible (GTK4 widgets are hidden by default after `set_parent`).
4. **Store**: Store `Some(widget.clone())` in `center_widget: RefCell<Option<Widget>>`.
5. **Queue resize**: Call `pie_menu_widget.queue_resize()` so the next layout pass allocates the center widget.
6. **Queue draw**: Call `pie_menu_widget.queue_draw()` so the next render pass snapshots the center widget.

### 4.2 Layout Sequence

In `WidgetImpl::size_allocate`, after positioning item widgets:

1. **Check**: If `center_widget` is `Some` and visible:
2. **Measure**: Call `child.measure(Orientation::Horizontal, -1)` and `child.measure(Orientation::Vertical, -1)` to get natural size.
3. **Clamp**: Clamp width and height to `2 * center_radius` (the center widget must not overflow the transparent center area).
4. **Allocate**: Create `Allocation::new(center_x - w/2, center_y - h/2, w, h)` — centered on the pie menu center, **without rotation** (the rotation transform is applied in `snapshot`, not in allocation).
5. **Call**: `child.size_allocate(&allocation, -1)`.

```rust
// Position center widget (inside rotation transform, unrotated position)
let center_radius = self.center_radius.load(Ordering::Relaxed);
if let Some(center) = &*self.center_widget.borrow() {
    if center.is_visible() {
        let (min_w, nat_w, _, _) = center.measure(gtk4::Orientation::Horizontal, -1);
        let (min_h, nat_h, _, _) = center.measure(gtk4::Orientation::Vertical, -1);
        let max_size = (center_radius * 2.0) as i32;
        let w = nat_w.max(min_w).min(max_size);
        let h = nat_h.max(min_h).min(max_size);
        let allocation = gtk4::Allocation::new(
            (center_x - w as f32 / 2.0) as i32,
            (center_y - h as f32 / 2.0) as i32,
            w,
            h,
        );
        center.size_allocate(&allocation, -1);
    }
}
```

### 4.3 Render Sequence

In `WidgetImpl::snapshot`, the center widget is rendered **inside** the `save/restore` rotation block, **after** ring drawing and **before** rotating item widgets:

```
snapshot.save()
snapshot.translate(center)
snapshot.rotate(rotation)
snapshot.translate(-center)

// 1. Draw ring background (annulus)
// 2. Draw ring markings (if enabled)

// 3. Snapshot center widget (rotates with ring)
if let Some(center) = &*self.center_widget.borrow() {
    if center.is_visible() {
        self.obj().snapshot_child(center, snapshot);
    }
}

// 4. Snapshot rotating item widgets (content_rotates == true)
// 5. Snapshot rotating submenu item widgets

snapshot.restore()

// 6. Snapshot non-rotating item widgets (content_rotates == false)
```

The center widget is rendered after the ring background (so the ring is behind it) and before item widgets (so items are on top if they overlap — which they shouldn't if `center_radius` is configured correctly).

### 4.4 Teardown Sequence

When `set_center_widget(None)` is called, or when a new widget replaces an existing one:

1. **Take**: `imp.center_widget.borrow_mut().take()` — extract the old widget, drop the borrow guard immediately.
2. **Unparent**: Call `existing.unparent()` outside the borrow to detach from the widget tree (reentrancy-safe, see §3).
3. **Queue resize**: `queue_resize()` — the center area is now empty.
4. **Queue draw**: `queue_draw()` — the center area now shows the transparent hole.

#### ObjectImpl::dispose

When `PieMenuWidget` itself is destroyed, GTK4 does **not** automatically unparent custom child widgets stored in Rust `RefCell<Option<Widget>>` fields. Without an explicit `dispose` implementation, orphaned GTK objects remain in memory, causing `GTK-Critical` warnings.

`PieMenuWidgetImpl` must override `ObjectImpl::dispose` to explicitly unparent the center widget:

```rust
impl ObjectImpl for PieMenuWidgetImpl {
    fn dispose(&self) {
        // Take center widget out of RefCell — borrow guard drops immediately
        if let Some(center) = self.center_widget.borrow_mut().take() {
            // Unparent outside the borrow — safe even if dispose
            // triggers GTK lifecycle signals
            center.unparent();
        }
        self.parent_dispose();
    }
}
```

The same reentrancy pattern as `set_center_widget` (§3) is used: `take()` extracts the widget and drops the borrow guard before `unparent()` is called, preventing `BorrowMutError` panics if `dispose` triggers GTK lifecycle signals that re-enter Rust code.

### 4.5 Visibility Lifecycle

The center widget is only visible when the pie menu is open. `PieMenuWidget` is set to `set_visible(false)` during construction and shown when `show_pie_menu()` is called. Since the center widget is a child of `PieMenuWidget`, it inherits visibility — when the parent is hidden, children are hidden too.

However, `show()` must still be called on the center widget after `set_parent()` because GTK4 widgets are hidden by default after being parented. The parent's visibility then gates the effective visibility.

---

## 5. Migration Inventory

This is an additive feature — no existing rendering code is removed. The only behavioral change is the conditional bypass of center-click handling.

| Existing Code | Location | Action | Replacement |
|---------------|----------|--------|-------------|
| Center click → `close_submenu` / `hide_pie_menu` | `src/overlay_widget/imp/widget.rs:333-340` | Conditional: skip when `center_widget` is `Some` | Consumer's own event controller on the center widget |
| `close_callback` (center click callback) | `src/menu_widget/imp/widget.rs:39`, `src/menu_widget/widget.rs:33` | Unchanged — remains as fallback when no center widget is set | N/A |
| Ring drawing (annulus) | `src/menu_widget/imp/widget.rs:348-384` | Unchanged | N/A |
| `measure` | `src/menu_widget/imp/widget.rs:216-228` | Extended: include center widget in measurement | N/A |
| `size_allocate` | `src/menu_widget/imp/widget.rs:236-315` | Extended: allocate center widget after item widgets | N/A |
| `snapshot` | `src/menu_widget/imp/widget.rs:320-593` | Extended: `snapshot_child` for center widget inside rotation block | N/A |

---

## 6. State Propagation

The center widget is a long-lived GTK4 child widget. Its state changes are managed by the consumer, not by `PieMenuWidget`:

| State | Owner | Mechanism |
|-------|-------|-----------|
| Widget content (label text, gauge value) | Consumer | Consumer calls methods on the widget directly (e.g., `label.set_label("...")`, `set_widget_config`) |
| Visibility | `PieMenuWidget` | Inherited from parent — `PieMenuWidget` visibility gates child visibility |
| Rotation | `PieMenuWidget` | Applied via `snapshot.rotate()` in the `snapshot` method — the child inherits the transform through `snapshot_child` |
| Allocation | `PieMenuWidget` | `size_allocate` positions the center widget at the unrotated center; the rotation transform in `snapshot` handles visual rotation |
| Close behavior | Consumer | Consumer attaches `GestureClick` or other event controllers to the center widget |

No GObject properties or signals are introduced for state propagation. The center widget is positioned and rendered by `PieMenuWidget`, but its content and interactions are entirely consumer-controlled.

---

## 7. Rotation Behavior

The center widget rotates with the ring. This is achieved by rendering it **inside** the `snapshot.save()` / `snapshot.rotate()` / `snapshot.restore()` block:

1. `snapshot.save()` — save current transform
2. `snapshot.translate(center_x, center_y)` — move origin to pie menu center
3. `snapshot.rotate(rotation)` — apply rotation
4. `snapshot.translate(-center_x, -center_y)` — move origin back
5. **Ring drawing** — drawn in rotated coordinate space
6. **Center widget** — `snapshot_child` renders the center widget in the rotated coordinate space
7. **Rotating item widgets** — `snapshot_child` for `content_rotates == true` items
8. `snapshot.restore()` — restore transform

The center widget's **allocation** uses the unrotated center position (`center_x, center_y`). The visual rotation is applied purely through the snapshot transform — the same mechanism used for `content_rotates == true` item widgets.

This means:
- A label in the center will rotate with the ring (text rotates)
- A gauge in the center will rotate with the ring (arc rotates)
- A logo in the center will rotate with the ring

If the consumer wants the center widget to stay upright despite rotation, they would need to counter-rotate the widget internally (e.g., by applying a `RotateTransform` in the widget's own `snapshot` override). This is out of scope for this concept — the center widget rotates with the ring, period.

---

## 8. Center Click Handling

### 8.1 Default Behavior (No Center Widget)

When `center_widget` is `None`, the existing center-click logic in `connect_pressed` remains active:

```rust
if distance <= center_radius {
    if widget.submenu_depth() > 0 {
        let _ = widget.close_submenu();
    } else {
        let _ = widget.hide_pie_menu();
    }
}
```

### 8.2 Custom Behavior (Center Widget Set)

When `center_widget` is `Some`, the center-click logic in `connect_pressed` is **bypassed**. The click event propagates through GTK4's normal event dispatch to the center widget, where the consumer's own event controllers handle it.

The consumer is responsible for implementing:

```rust
let center_label = Label::new(Some("Click to close"));
let click = gtk4::GestureClick::new();
let overlay_clone = overlay.clone();
click.connect_pressed(move |_, _, _, _| {
    if overlay_clone.submenu_depth() > 0 {
        let _ = overlay_clone.close_submenu();
    } else {
        let _ = overlay_clone.hide_pie_menu();
    }
});
center_label.add_controller(click);
```

### 8.3 Implementation in `connect_pressed`

```rust
if distance <= center_radius {
    // Check if a center widget is set — if so, let the event propagate
    let has_center_widget = widget
        .imp()
        .pie_menu_widget
        .borrow()
        .as_ref()
        .and_then(|menu_widget| menu_widget.imp().center_widget.borrow().as_ref())
        .is_some();

    if has_center_widget {
        // Event propagates to center widget naturally — do not claim
        debug!("Center click with center widget set — propagating to widget");
    } else {
        debug!("Center circle clicked, closing submenu or menu");
        gesture.set_state(EventSequenceState::Claimed);
        if widget.submenu_depth() > 0 {
            let _ = widget.close_submenu();
        } else {
            let _ = widget.hide_pie_menu();
        }
    }
}
```

---

## 9. Interaction with Existing Features

| Feature | Affected? | How |
|---------|-----------|-----|
| Keyboard navigation | No | Keyboard selection operates on menu items, not the center. The center widget is not part of the item iteration. |
| Rotation | Yes | The center widget rotates with the ring via the `snapshot.rotate()` transform. No additional rotation logic needed. |
| Submenus | Yes | When a center widget is set, the consumer must handle `close_submenu` on center click. The built-in tiered close logic is bypassed. |
| Disabled state | No | The center widget has no `enabled` flag — it is always visible when the pie menu is open. The consumer can disable it via GTK4's `set_sensitive()`. |
| Hover detection | No | Hover detection iterates menu items by angle and distance. The center area is excluded from item hover. |
| Click-to-select | No | Item click detection checks `distance > center_radius`. The center area is not checked for items. |
| Builder pattern | Yes | New `with_center_widget(&Widget)` builder method added. |
| Ring markings | No | Markings are drawn on the ring, not in the center. |
| `close_callback` | No | Remains as fallback when no center widget is set. When a center widget is set, `close_callback` is not called (the center-click logic is bypassed entirely). |
| `set_pie_menu_center_radius` | No | Controls the inner ring radius. The center widget is clamped to `2 * center_radius` in `size_allocate`. Changing `center_radius` triggers `queue_draw` but not `queue_resize` — the center widget allocation will update on the next layout pass if the widget is resized. |
| `set_pie_menu_radius` | Yes | Changing the ring radius triggers `queue_resize`, which causes `size_allocate` to re-allocate the center widget at the new center position. |
| Widget system (item widgets) | No | The center widget is independent of the item widget registry and factory system. |
| `measure` | Yes | `PieMenuWidgetImpl::measure` must account for the center widget's natural size when computing the widget's minimum/natural size. |

---

## 10. Phase Plan

### Phase 1 — Storage and API

**Scope**: Add `center_widget` field, `set_center_widget` / `center_widget` methods on `PieMenuWidget`, `with_center_widget` on `PieMenuOverlayWidget`.

**Files**:
- `src/menu_widget/imp/widget.rs` — add `center_widget: RefCell<Option<Widget>>` field, initialize in `new()`
- `src/menu_widget/widget.rs` — add `set_center_widget`, `center_widget` methods
- `src/overlay_widget/widget.rs` — add `with_center_widget`, `set_center_widget` methods

**Smoke test**: Set a `Label` as center widget, open the pie menu, verify the label appears in the center (may not be positioned correctly yet — that's Phase 2).

### Phase 2 — Layout and Rendering

**Scope**: Allocate and render the center widget inside the rotation transform.

**Files**:
- `src/menu_widget/imp/widget.rs` — extend `size_allocate` and `snapshot`

**Smoke test**: Set a `Label` as center widget, open the pie menu, verify the label is centered and rotates with the ring.

### Phase 3 — Click Handling Bypass

**Scope**: Bypass the built-in center-click logic when a center widget is set.

**Files**:
- `src/overlay_widget/imp/widget.rs` — modify `connect_pressed` center-click branch

**Smoke test**: Set a `Label` with a `GestureClick` that calls `hide_pie_menu` as center widget. Open the pie menu, click the center, verify the menu closes. Remove the center widget, click the center, verify the built-in close still works.

### Phase 4 — Measurement

**Scope**: Include center widget in `measure` to ensure the pie menu is large enough.

**Files**:
- `src/menu_widget/imp/widget.rs` — extend `measure`

**Smoke test**: Set a large widget (e.g., 200x200) as center widget with a small `center_radius`. Verify the widget is clamped to `2 * center_radius` and does not overflow the ring.

### Phase 5 — Example and Documentation

**Scope**: Update the `sysinfo_dashboard` example to use a center widget, update the book.

**Files**:
- `examples/sysinfo_dashboard.rs` — add a center label or gauge
- `book/src/widget.md` — document center widget
- `book/src/api.md` — add `set_center_widget`, `center_widget`, `with_center_widget`
- `book/src/builder_pattern.md` — add `with_center_widget`

**Smoke test**: Run `cargo run --example sysinfo_dashboard`, verify the center widget renders and rotates. Run `cargo test --lib`, verify all tests pass.

---

## 11. Tests

### Unit Tests (`src/menu_widget/imp/widget.rs`)

- `test_center_widget_default_none` — `center_widget` is `None` after construction
- `test_set_center_widget_some` — after `set_center_widget(Some(&label))`, `center_widget()` returns `Some`
- `test_set_center_widget_none_after_some` — after setting then clearing, `center_widget()` returns `None`
- `test_set_center_widget_replaces_existing` — setting a second widget replaces the first; the first is unparented
- `test_center_widget_clamped_to_center_radius` — a widget with natural size larger than `2 * center_radius` is clamped in `size_allocate`

### Integration Tests (`src/overlay_widget/imp/widget.rs`)

- `test_center_click_default_closes_menu` — without center widget, center click calls `hide_pie_menu`
- `test_center_click_default_closes_submenu` — without center widget, center click at `depth > 0` calls `close_submenu`
- `test_center_click_with_center_widget_propagates` — with center widget set, center click does not call `hide_pie_menu` or `close_submenu` (event propagates to the widget)

### Example Verification

- `cargo run --example sysinfo_dashboard` — center widget renders, rotates with ring, and close logic works via the consumer's event controller

---

## 12. Limitations

### Non-Goals

- **Counter-rotation**: The center widget always rotates with the ring. A counter-rotation mechanism (center stays upright while ring rotates) is not part of this concept.
- **Center widget factory/registry**: Unlike menu items, the center widget is not built by a factory. The consumer passes a pre-built widget directly.
- **Center widget animation**: Animated transitions when setting/removing the center widget are not part of this concept.
- **Multiple center widgets**: Only one center widget is supported. Replacing it requires calling `set_center_widget` again.

### Technical Limitations

- **`center_widget` is not thread-safe**: `RefCell<Option<Widget>>` is `!Sync`. All access occurs on the GTK main thread.
- **Center widget must fit within `2 * center_radius`**: If the widget's natural size exceeds this, it is clamped. The consumer should choose a widget and `center_radius` that are compatible.
- **Consumer must handle close logic**: When a center widget is set, the built-in center-click-to-close logic is disabled. The consumer is responsible for attaching event controllers to handle `close_submenu` and `hide_pie_menu`.
- **Rotation is visual only**: The center widget's allocation is unrotated. The rotation is applied via the `snapshot` transform. This means the widget's input region (for event handling) is at the unrotated position. For typical use cases (logo, label, gauge) this is fine because the widget is centered and symmetric. For asymmetric widgets, click areas may not visually align at extreme rotation angles.

### Backward Compatibility

`set_center_widget` is purely additive. When no center widget is set (the default), all existing behavior is preserved — center-click-to-close, `close_callback`, ring rendering, and item widget rendering are unchanged.

---

## 13. Summary

This concept paper outlines an optional center widget for `smearor-wrot-pie-menu`, organized into 5 implementation phases:

| Phase | Feature | Complexity |
|-------|---------|------------|
| 1 — Storage and API | `center_widget` field, `set_center_widget`, `with_center_widget` | Low |
| 2 — Layout and Rendering | `size_allocate`, `snapshot` inside rotation block | Medium |
| 3 — Click Handling Bypass | Conditional bypass in `connect_pressed` | Low |
| 4 — Measurement | `measure` includes center widget | Low |
| 5 — Example and Documentation | `sysinfo_dashboard` update, book updates | Low |

### Key Design Decisions

- **`RefCell<Option<Widget>>`** — simple storage, no factory or registry needed
- **`set_parent` / `unparent`** — standard GTK4 child management, same as item widgets
- **Rotation via `snapshot_child` inside rotation block** — center widget inherits the rotation transform, no additional rotation logic
- **Consumer-managed close logic** — when a center widget is set, the consumer is responsible for `close_submenu` / `hide_pie_menu` via their own event controllers
- **Default behavior preserved** — without a center widget, the existing center-click-to-close logic remains active
- **Clamped to `2 * center_radius`** — center widget cannot overflow the transparent center area

### Expected Outcome

After implementation, consumers can:

1. Set any GTK4 widget as the pie menu center via `set_center_widget` or `with_center_widget`
2. Render logos, labels, values, or gauges in the center
3. Have the center widget rotate with the ring automatically
4. Implement custom close-menu / close-submenu behavior via event controllers on the center widget
5. Remove the center widget to restore default center-click-to-close behavior
