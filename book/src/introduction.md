# Introduction

Welcome to the documentation for `smearor-wrot-pie-menu`.

This library provides a GTK4 pie menu widget—`PieMenuOverlayWidget`—that overlays a circular touch-activated menu on top of any child widget.

## Why smearor-wrot-pie-menu?

Touch interfaces for desktop applications often lack quick-access menus. The `PieMenuOverlayWidget` solves this by providing:

1. **Touch Gesture Activation**: Opens on pinch-to-zoom (scale > 3.5), closes on pinch-out (scale < 0.5).
2. **Configurable Menu Items**: Add/remove items programmatically with custom icons, colors, angles, and events.
3. **Rotation Gesture**: Rotate the menu ring with a two-finger rotation gesture.
4. **Event Channel**: Communicates with the consumer application via `mpsc::Sender<PieMenuMessage>`.

## Getting Started

Head over to the [Quick Start](quickstart.md) page to get up and running in minutes. For a visual overview of how the widget works internally, see the [Architecture](architecture.md) page.
