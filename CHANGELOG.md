# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Optional Center Widget
- `set_center_widget(Option<&Widget>)` / `center_widget()` - set, remove, or get an optional GTK4 widget rendered in the center of the pie menu ring
- `with_center_widget(&Widget)` - builder method for fluent center widget setup
- Center widget rotates with the ring via the `snapshot.rotate()` transform
- Center widget is clamped to `2 * center_radius` pixels in `size_allocate`
- When a center widget is set, the built-in center-click-to-close logic is bypassed - the consumer handles events (e.g. close-menu, close-submenu) via the widget's own event controllers
- When no center widget is set (default), the built-in center-click-to-close behavior remains active
- Reentrancy-safe `set_center_widget` implementation: old widget is taken out of `RefCell` before `unparent()` to avoid `BorrowMutError` from GTK signal callbacks
- `ObjectImpl::dispose` override to explicitly unparent the center widget and clear the `RefCell`, preventing GTK memory leaks
- `GaugeItemWidget::set_value(f64)` - dynamic value updates for gauge widgets
- `GaugeItemWidget` and `GaugeItemWidgetParams` exported as public API
- Unit tests: `test_center_widget_default_none`, `test_set_center_widget_some`, `test_set_center_widget_none_after_some`, `test_set_center_widget_replaces_existing`
- Integration tests: `test_center_click_default_closes_menu`, `test_center_click_with_center_widget_propagates`
- Book documentation: center widget section in `widget.md`, API entries in `api.md`, builder method in `builder_pattern.md`
- `sysinfo_dashboard` example updated with a `GaugeItemWidget` center widget showing live CPU usage

---

## [0.2.0] - 2026-08-18

### Added

#### Configurable Thresholds
- `set_activation_threshold()` / `activation_threshold()` — configurable pinch-to-zoom activation threshold (default: `3.5`)
- `set_deactivation_threshold()` / `deactivation_threshold()` — configurable pinch-out deactivation threshold (default: `0.5`)
- Thresholds stored as `AtomicF64` for lock-free access

#### Disabled State
- `enabled` field on `MenuItem` (default: `true`) — disabled items render at 40% opacity, no hover, no click
- `set_menu_item_enabled()` — runtime toggle with `queue_draw()` invalidation
- `SetMenuItemEnabledError` error type

#### Convenience Methods
- `remove_all_menu_items()` — clears all items at once
- `menu_item_count()` — returns current item count
- `get_menu_item()` — returns a clone of an item by id
- `update_menu_item()` — replaces an item (all fields except `id`) with overlap validation and rollback
- `redistribute()` — manual redistribution of flexible item angles

#### Builder Pattern
- `with_message_sender()` — fluent sender setup
- `with_activation_threshold()` — fluent threshold setup
- `with_deactivation_threshold()` — fluent threshold setup
- `with_rotation_gesture_enabled()` — fluent gesture toggle
- `with_markings_enabled()` — fluent markings toggle
- `with_scroll_rotation_step()` — fluent scroll sensitivity
- `with_menu_item()` — fluent item addition returning `Result<Self, AddMenuItemError>`

#### Overlap Validation
- `AddMenuItemError::ItemOverlap` — prevents visually overlapping items
- Bounding-circle distance check based on item radius, angle, and ring radius
- Transactional rollback in `add_menu_item()` on validation failure
- Full validation via `validate_all_no_overlap()` after redistribution

#### Automatic Angle Distribution
- `add_menu_item_auto()` — auto-calculated angle distribution
- `fixed_position` field on `MenuItem` — semantic anchor points that resist redistribution
- Proportional segment sizing — wider gaps receive proportionally more flexible items
- Largest remainder method for even allocation
- Angle normalization to `[0, 360)` for fixed items
- Zero-width guard for co-located fixed items
- Rollback with angle snapshot on validation failure

#### Keyboard Navigation (feature: `keyboard`)
- `Ctrl+Space` / `Menu` key to open the pie menu
- `Arrow Left/Right/Up/Down` / `Tab` to cycle selection
- `Home` to select first enabled item
- `Enter` / `Space` to confirm selection
- `Escape` to close
- `cycle_selection()`, `select_first_item()`, `confirm_selection()`, `set_keyboard_selection()`
- Disabled items skipped during keyboard navigation

#### Mouse Scroll Rotation (feature: `mouse-scroll`)
- `EventControllerScroll` for smooth ring rotation via mouse wheel
- `set_scroll_rotation_step()` / `scroll_rotation_step()` — configurable sensitivity (default: `5.0`)
- Rotation delta computed as `dy * sensitivity`

#### Controller Support (features: `controller-sdl2` / `controller-evdev`)
- Analog stick rotation — `handle_left_stick_x()` for continuous rotation
- Analog stick selection — `handle_right_stick()` for nearest-item selection
- `find_nearest_item()` — finds enabled item closest to a target angle
- SDL2 backend (`controller-sdl2`) and evdev backend (`controller-evdev`)

#### Ring Markings
- `set_markings_enabled()` / `markings_enabled()` — inner and outer ring markings (default: `true`)

#### Submenus
- `submenu: Option<Vec<MenuItem>>` field — nested rings at increasing radii
- `SubmenuOpened(String)` / `SubmenuClosed(String)` messages
- `open_submenu()`, `close_submenu()`, `submenu_depth()` — navigation API
- `get_submenu_items()`, `set_submenu_items()`, `redistribute_submenu()` — submenu management
- `set_submenu_radius()`, `set_submenu_radius_step()` — per-level radius configuration
- Tiered Escape behavior — first Escape closes submenu, second closes menu
- Keyboard selection reset on level change
- Submenu angle distribution with fixed-position support
- `MAX_SUBMENU_DEPTH = 3` with `SubmenuError::MaxDepthReached`
- `close_on_click` field on `MenuItem` (default: `true`)
- Parent ring rendered at reduced opacity with yellow indicator dot
- Breadcrumb dots between rings

#### Registry-Based Widget System
- `MenuItemWidgetFactory` trait — typed factory with associated `Config` type
- `MenuItemWidgetFactoryErased` — type-erased trait for registry storage
- `MenuItemWidgetRegistry` — maps type names to factories, pre-populated with standard implementations
- `MenuItemContext` — provides event trigger callback to factories
- `ItemSize` — optional non-square allocation for widgets
- `widget_type` field on `MenuItem` (default: `"circle"`) — registry lookup key
- `widget_config` field on `MenuItem` — type-specific config as `serde_json::Value`
- `content_size` field on `MenuItem` — non-square widget dimensions
- `content_rotates` field on `MenuItem` (default: `true`) — widget rotation with ring
- Standard implementations: `"circle"` (`CircleWidgetFactory`), `"square"` (`SquareWidgetFactory`), `"button"` (`ButtonWidgetFactory`)
- `CircleConfig`, `SquareConfig`, `ButtonConfig` with `TypedBuilder` and `setter(into)`
- `RgbaColor` for color fields with custom serde (hex string parsing)
- `Option<RgbaColor>` for truly optional color fields (no sentinel values)
- `register_widget_factory()` — register custom GTK4 widget factories
- `refresh_widgets()` — clears widget cache and rebuilds on next layout pass
- `set_widget_config()` — replaces widget config for a single item with cache invalidation
- `SetWidgetConfigError` error type
- Widget caching with `glib::idle_add_local` for reentrancy safety
- Migration from snapshot-based rendering to widget-child rendering pipeline
- Serializable `widget_type` and `widget_config` for JSON/TOML configuration files

### Changed

- `MenuItem` visual properties (`label`, `icon_name`, `color`, `label_color`) moved to widget-specific config structs (`CircleConfig`, `SquareConfig`, `ButtonConfig`)
- `MenuItem` uses `TypedBuilder` with `setter(into)` for ergonomic construction
- `.config()` builder method on `MenuItemBuilder` accepts any `Serialize` type
- Rendering pipeline migrated from `gtk4::Snapshot` drawing to GTK4 child widgets via `set_parent` / `size_allocate`
- `PieMenuMessage` extended with `SubmenuOpened(String)` and `SubmenuClosed(String)` variants
- Book documentation updated with widget system chapter and all examples migrated to builder pattern

---

## [0.1.0] - 2026-08-15

### Changed

- Extracted `smearor-wrot-pie-menu` as a standalone crate from `smearor-wrot`
- Self-contained `RgbaColor` and `RgbColor` types with hex parsing
- `MenuItem` with `TypedBuilder` construction
- `PieMenuMessage` generalized to `Rotate(f32)` and `Event(String)` only
- Configurable menu items via `PieMenuMenuItemHandler` trait (no hardcoded `DefaultMenuProvider`)
- Book documentation
- Infrastructure files (.github, .run, book, AGENTS.md, etc.)

