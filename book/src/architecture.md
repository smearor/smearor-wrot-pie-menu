# Architecture

`PieMenuOverlayWidget` achieves its pie menu functionality through three core components working together within the GTK4 layout and rendering pipeline.

## Overview

The pie menu is composed of two widgets:

- **`PieMenuOverlayWidget`** — The outer overlay that wraps a child widget and handles gestures
- **`PieMenuWidget`** — The inner widget that renders the circular menu ring

## Core Components

### 1. Overlay Widget (`PieMenuOverlayWidget`)

The overlay widget is a `gtk4::Widget` subclass that contains:

- A `gtk4::Overlay` as its layout container
- A `PieMenuWidget` added as an overlay on top of the child widget
- Gesture controllers for zoom, rotation, and click detection

**Gesture handling:**

- **`GestureZoom`**: Detects pinch-to-zoom. When scale > 3.5, the pie menu is shown. When scale < 0.5, it is hidden.
- **`GestureRotate`**: Detects two-finger rotation. When the angle delta exceeds 10 degrees, the menu rotation is updated and a `PieMenuMessage::Rotate` message is sent.
- **`GestureClick`**: Detects clicks on the center circle (close menu) and on menu items (send `PieMenuMessage::Event`).

### 2. Pie Menu Widget (`PieMenuWidget`)

The pie menu widget is a `gtk4::Widget` subclass that renders:

- A ring-shaped background (outer circle minus inner circle, using even-odd fill rule)
- 5-degree markings on both inner and outer edges, with highlights at the zero position and current rotation
- Menu items positioned at their configured angles, each consisting of:
  - A colored background circle with shadow
  - A GTK icon from the icon theme
  - A text label below the icon
- Mouse hover highlighting via `EventControllerMotion`

**State:**

- `rotation: AtomicF32` — current rotation in degrees
- `radius: AtomicF32` — outer ring radius (default: 160px)
- `center_radius: AtomicF32` — inner ring radius (default: 64px)
- `menu_items: Arc<Menu>` — thread-safe collection of `MenuItem`
- `hovered_item_index: RefCell<i32>` — currently hovered item (-1 = none)

### 3. Data Model (`MenuItem`, `Menu`)

- **`MenuItem`**: Built with `TypedBuilder`. Fields: `id`, `label`, `label_color`, `icon_name`, `color`, `angle`, `radius`, `event`. `Hash`/`Eq` by `id`.
- **`Menu`**: `DashMap<String, MenuItem>` wrapper with a builder pattern.
- **`RgbaColor`/`RgbColor`**: Self-contained color types with hex parsing and `gdk::RGBA` conversion.

## Message Flow

```mermaid
graph LR
    classDef default fill: #1e1e1e, stroke: #333333, stroke-width: 1px, color: #ffffff
    classDef gesture fill: #00a1e4, stroke: #ffffff, stroke-width: 2px, color: #ffffff
    classDef widget fill: #89fc00, stroke: #333333, stroke-width: 2px, color: #000000
    classDef message fill: #f5b700, stroke: #333333, stroke-width: 1px, color: #000000
    classDef consumer fill: #04e762, stroke: #333333, stroke-width: 1px, color: #000

    A["Pinch-to-zoom"] --> B["PieMenuOverlayWidget"]
    C["Rotation gesture"] --> B
    D["Click menu item"] --> B
    B -->|"PieMenuMessage::Rotate(f32)"| E["Consumer App"]
    B -->|"PieMenuMessage::Event(String)"| E
    B -->|"hide_pie_menu()"| F["Menu closed"]

    class A gesture
    class C gesture
    class D gesture
    class B widget
    class E consumer
    class F widget
```

## Trait Hierarchy

```mermaid
graph TD
    classDef default fill: #1e1e1e, stroke: #333333, stroke-width: 1px, color: #ffffff
    classDef trait fill: #00a1e4, stroke: #ffffff, stroke-width: 2px, color: #ffffff
    classDef impl fill: #89fc00, stroke: #333333, stroke-width: 1px, color: #000

    RH["RotationHandler"] --> PW["PieMenuWidget"]
    RH --> POW["PieMenuOverlayWidget"]
    RH --> PWI["PieMenuWidgetImpl"]
    RH --> POWI["PieMenuOverlayWidgetImpl"]

    MIH["PieMenuMenuItemHandler"] --> PW
    MIH --> POW
    MIH --> PWI
    MIH --> POWI

    CH["PieMenuControlHandler"] --> POW
    CH --> POWI

    MS["PieMenuMessageSender"] --> POW
    MS --> POWI

    class RH trait
    class MIH trait
    class CH trait
    class MS trait
    class PW impl
    class POW impl
    class PWI impl
    class POWI impl
```
