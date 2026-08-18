# How to Write Concepts — A Concept Concept

---

## 1. Goal

This document provides a template and checklist for writing technical concept documents in this project. It distills lessons from the `CUSTOM_WIDGET` retrospective into actionable guidelines, so that future concepts avoid the same class of runtime problems.

---

## 2. Core Principle

**A concept must describe the runtime behavior, not just the data model.**

Architecture (traits, structs, modules) is necessary but insufficient. A concept that fully specifies the API but leaves the runtime lifecycle to implementation will produce severe bugs in GUI systems where object lifecycles, visibility, layout passes, and state propagation are critical.

---

## 3. Concept Template

Every concept document should contain the following sections. Sections marked **[required]** must be present. Sections marked **[conditional]** are required when the concept touches that area.

### 3.1 Goal and Motivation [required]

What problem does this solve and why. Keep it short.

### 3.2 Current State [required]

What exists today. Be specific: list the files, functions, and code paths that will be affected. This is the baseline for the migration inventory.

### 3.3 Data Model [required]

Types, traits, structs, fields, signatures. This is what most concepts already do well.

### 3.4 Runtime Lifecycle [conditional: required when touching rendering, widgets, or GTK4]

Specify the exact sequence of runtime calls and their order. For GTK4 widgets:

- **Build sequence**: `factory.build() → widget.set_parent() → widget.show() → parent.queue_resize()`
- **Layout sequence**: `WidgetImpl::size_allocate() → child.size_allocate()`
- **Render sequence**: `WidgetImpl::snapshot() → child snapshot via GTK4 pipeline`
- **Teardown sequence**: `child.unparent() → remove from cache`

If the concept introduces a new rendering path, specify every GTK4 call in order. Do not assume the reader knows that `show()` or `queue_resize()` is needed.

### 3.5 Migration Inventory [conditional: required when replacing existing code]

An explicit table listing what is removed and what replaces it:

| Existing Code | Location | Action | Replacement |
|---------------|----------|--------|-------------|
| Icon drawing in `snapshot` | `imp/widget.rs:120-145` | Remove | `CircleWidgetFactory::build` |
| Label drawing in `snapshot` | `imp/widget.rs:147-160` | Remove | `CircleWidgetFactory::build` |
| Selection ring in `snapshot` | `imp/widget.rs:165-180` | Remove | Widget-internal state |

**Every** code path that is affected must be listed. If a path is not listed, it will remain active and conflict with the new path.

### 3.6 State Propagation [conditional: required when caching long-lived objects]

If objects are built once and cached, specify how they learn about state changes:

- What state can change? (selection, hover, enabled, rotation, submenu level)
- How does the state reach the cached object? (method call, GObject property, rebuild, signal)
- When is the state pushed? (on every state change, on next render, on demand)

Example:

> When keyboard selection changes, `PieMenuWidgetImpl` calls `widget.set_property("is-selected", true)` on the affected child widget. The widget's `notify::is-selected` handler triggers a redraw.

### 3.7 Multi-Level / Hierarchical Concerns [conditional: required for submenu or layered UI]

If the feature interacts with submenus or layered rendering:

- How does the cache distinguish between ring levels?
- What happens to cached widgets when a submenu opens/closes? (unparent, hide, opacity?)
- How does `size_allocate` know which level is active?
- How are widgets rebuilt when returning to a parent level?

### 3.8 Interaction with Existing Features [required]

For each existing feature, state whether it is affected and how:

| Feature | Affected? | How |
|---------|-----------|-----|
| Keyboard navigation | Yes | Selection state must propagate to cached widgets |
| Rotation | Yes | Rotation angle must reach child widget via property, not allocation |
| Submenus | Yes | Cache must handle ring-level switching |
| Disabled state | No | Unchanged |

### 3.9 Phase Plan [required]

Break the implementation into phases. Each phase must be independently testable. Include a "smoke test" criterion for each phase — the minimum visible behavior that proves the phase works.

### 3.10 Tests [required]

List all tests. For rendering changes, include at least one test that verifies the new path is taken and the old path is not.

---

## 4. Checklist

Before finalizing a concept, verify:

- [ ] **Runtime lifecycle**: Are all GTK4 calls specified in order?
- [ ] **Migration inventory**: Is every removed code path listed?
- [ ] **State propagation**: For cached objects, is the update mechanism specified?
- [ ] **Multi-level**: For submenu/layered features, is cache invalidation designed?
- [ ] **Feature interaction**: Is every existing feature checked for impact?
- [ ] **Positioning vs rendering**: Are `size_allocate` (positioning) and `snapshot` (rendering/transform) correctly separated?
- [ ] **Visibility**: Is `show()` / `set_visible()` specified for new widgets?
- [ ] **Resize triggering**: Is `queue_resize()` specified after widget tree changes?
- [ ] **Teardown**: Is `unparent()` specified for removed/replaced widgets?
- [ ] **Smoke test**: Does each phase have a visible verification criterion?

---

## 5. Anti-Patterns to Avoid

### 5.1 "Additive-Only" Concepts

Concepts that only describe what to add, never what to remove. In rendering migrations, the removal is the hard part.

**Bad**: "Add `MenuItemWidgetFactory` trait and registry."
**Good**: "Add `MenuItemWidgetFactory` trait. Remove icon drawing from `snapshot` (lines 120-145). Remove label drawing from `snapshot` (lines 147-160). The factory's `build` method now produces the widget that renders these."

### 5.2 "Implementation Detail" Hand-Waving

Concepts that say "the widget is rendered by GTK4" without specifying the lifecycle calls.

**Bad**: "Widgets are registered as children and positioned by `size_allocate`."
**Good**: "Widgets are built by the factory, registered via `set_parent(Some(&pie_menu_widget))`, made visible via `widget.show()`, and a resize is triggered via `pie_menu_widget.queue_resize()`. The `WidgetImpl::size_allocate` override then positions each child."

### 5.3 Conflating Allocation and Rendering

`size_allocate` sets position and size. `snapshot` draws content including transforms. Do not specify rotation in `size_allocate`.

**Bad**: "A rotation transform is applied during the allocation pass."
**Good**: "The rotation angle is stored as a GObject property on the child widget. The widget's `snapshot` override reads this property and applies a `gtk4::Transform::rotate()` before drawing."

### 5.4 "Same Logic Applies" Without Design

Saying "the same logic applies for submenus" without designing the cache invalidation, visibility, and ring-level awareness.

**Bad**: "Widgets in submenu rings are also supported — the same caching and allocation logic applies regardless of ring level."
**Good**: "When a submenu opens, parent-ring widgets are hidden via `set_visible(false)`. The `item_widgets` cache is keyed by ring level. `size_allocate` only iterates the active ring level's widgets. When the submenu closes, parent-ring widgets are shown again via `set_visible(true)`."

---

## 6. Review Process

A concept is ready for implementation when:

1. A developer can read only the concept and list every file that will change
2. A developer can read only the concept and write the exact GTK4 lifecycle calls
3. The concept explicitly lists what code is removed
4. Every existing feature has been checked for interaction
5. The checklist in section 4 is fully ticked

If any of these are not met, the concept needs another revision pass before implementation begins.
