# Quick Start

## Installation

Add `smearor-wrot-pie-menu` to your `Cargo.toml`:

```toml
[dependencies]
smearor-wrot-pie-menu = "0.1"
gtk4 = { version = "0.11", features = ["v4_20"] }
```

## Minimal Example

The following example creates a GTK4 application window containing a `PieMenuOverlayWidget` with a label as child and two menu items:

```rust
use smearor_wrot_pie_menu::CircleConfig;
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};
use std::sync::mpsc::channel;

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("io.smearor.pie_menu.example")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Pie Menu Example")
            .default_width(400)
            .default_height(400)
            .build();

        let label = Label::new(Some("Pinch to open pie menu"));

        let (sender, receiver) = channel::<PieMenuMessage>();

        let overlay = PieMenuOverlayWidget::new(Some(&label))
            .with_message_sender(sender)
            .with_activation_threshold(2.5)
            .with_menu_item(
                MenuItem::builder()
                    .id("red")
                    .widget_type("circle")
                    .config(CircleConfig::builder()
                        .icon_name("media-playback-stop-symbolic")
                        .label("Red")
                        .color("#FF000077")
                        .build())
                    .angle(0.0)
                    .fixed_position(true)
                    .event("red")
                    .build(),
            )
            .unwrap()
            .with_menu_item(
                MenuItem::builder()
                    .id("green")
                    .widget_type("circle")
                    .config(CircleConfig::builder()
                        .icon_name("media-playback-stop-symbolic")
                        .label("Green")
                        .color("#00FF0077")
                        .build())
                    .angle(180.0)
                    .fixed_position(true)
                    .event("green")
                    .build(),
            )
            .unwrap();

        window.set_child(Some(&overlay));
        window.present();
    });

    application.run()
}
```

## Running

Build and run the example:

```sh
cargo run
```

## Next Steps

- See the [Examples](examples.md) page for interactive demos.
- Read about the full [API Reference](api.md) for all available methods.
- Learn about the [Architecture](architecture.md) for internal details.
