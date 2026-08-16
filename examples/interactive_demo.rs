//! Interactive demo combining smearor-wrot-pie-menu with smearor-wrot-rotation.
//!
//! Renders the smearor logo inside a RotationWidget, wrapped by a PieMenuOverlayWidget.
//! Snap buttons and a manual angle slider control the rotation. Pie menu items
//! trigger clockwise / counter-clockwise rotation snaps.

use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::RotationHandler;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use smearor_wrot_rotation::RotationControlHandler;
use smearor_wrot_rotation::RotationWidget;
use smearor_wrot_rotation::SmearorRotation;

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Button;
use gtk4::Frame;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Picture;
use gtk4::Scale;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;

const APP_ID: &str = "io.smearor.pie_menu.interactive_demo";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Pie Menu + Rotation Interactive Demo")
        .default_width(600)
        .default_height(700)
        .build();

    let main_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title_label = Label::builder()
        .label("Pie Menu + Rotation Interactive Demo")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title_label);

    let hint_label = Label::builder()
        .label("Pinch to open the pie menu. Use snap buttons or the slider to rotate.")
        .halign(Align::Center)
        .build();
    main_box.append(&hint_label);

    // --- Snap buttons ---
    let button_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .build();

    let button_0 = Button::with_label("Snap to 0°");
    let button_90 = Button::with_label("Snap to 90°");
    let button_180 = Button::with_label("Snap to 180°");
    let button_270 = Button::with_label("Snap to 270°");

    button_box.append(&button_0);
    button_box.append(&button_90);
    button_box.append(&button_180);
    button_box.append(&button_270);
    main_box.append(&button_box);

    // --- RotationWidget with smearor.png ---
    let picture = Picture::for_filename("assets/smearor.png");
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_halign(Align::Center);
    picture.set_valign(Align::Center);

    let rotation_widget = RotationWidget::new(SmearorRotation::Deg0)
        .with_gesture_rotation_enabled(true)
        .with_animations_enabled(true);
    rotation_widget.set_animation_speed(500);
    rotation_widget.set_animation_overshoot(1.7);
    rotation_widget.set_hexpand(true);
    rotation_widget.set_vexpand(true);
    rotation_widget.set_halign(Align::Center);
    rotation_widget.set_valign(Align::Center);
    rotation_widget.set_child(Some(&picture));

    // --- PieMenuOverlayWidget wrapping the rotation widget ---
    let (sender, receiver) = channel::<PieMenuMessage>();

    let pie_menu = PieMenuOverlayWidget::new(Some(rotation_widget.upcast_ref()));
    pie_menu.set_message_sender(sender);

    pie_menu
        .add_menu_item(
            MenuItem::builder()
                .id("rotate-cw")
                .label("Rotate CW")
                .icon_name("object-rotate-right-symbolic")
                .color("#00000077")
                .angle(0.0)
                .radius(30.0)
                .event("rotate-cw")
                .build(),
        )
        .expect("Failed to add menu item");

    pie_menu
        .add_menu_item(
            MenuItem::builder()
                .id("rotate-ccw")
                .label("Rotate CCW")
                .icon_name("object-rotate-left-symbolic")
                .color("#00000077")
                .angle(180.0)
                .radius(30.0)
                .event("rotate-ccw")
                .build(),
        )
        .expect("Failed to add menu item");

    let viewport_frame = Frame::builder()
        .label("Pie Menu + Rotation Viewport")
        .hexpand(true)
        .vexpand(true)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    viewport_frame.set_child(Some(&pie_menu));
    main_box.append(&viewport_frame);

    // --- Angle label + manual slider ---
    let current_angle_label = Label::builder().label("Current Angle: 0.00°").margin_bottom(6).build();
    main_box.append(&current_angle_label);

    let manual_label = Label::new(Some("Manual Angle:"));
    let manual_scale = Scale::with_range(Orientation::Horizontal, 0.0, 360.0, 1.0);
    manual_scale.set_value(0.0);
    manual_scale.set_hexpand(true);
    main_box.append(&manual_label);
    main_box.append(&manual_scale);

    // --- Snap button connections ---
    button_0.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        #[weak]
        pie_menu,
        move |_| {
            rotation_widget.set_rotation_with_animation(0.0);
            manual_scale.set_value(0.0);
            pie_menu.set_rotation(0.0);
            current_angle_label.set_label("Current Angle: 0.00°");
        }
    ));

    button_90.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        #[weak]
        pie_menu,
        move |_| {
            rotation_widget.set_rotation_with_animation(90.0);
            manual_scale.set_value(90.0);
            pie_menu.set_rotation(90.0);
            current_angle_label.set_label("Current Angle: 90.00°");
        }
    ));

    button_180.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        #[weak]
        pie_menu,
        move |_| {
            rotation_widget.set_rotation_with_animation(180.0);
            manual_scale.set_value(180.0);
            pie_menu.set_rotation(180.0);
            current_angle_label.set_label("Current Angle: 180.00°");
        }
    ));

    button_270.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        #[weak]
        pie_menu,
        move |_| {
            rotation_widget.set_rotation_with_animation(270.0);
            manual_scale.set_value(270.0);
            pie_menu.set_rotation(270.0);
            current_angle_label.set_label("Current Angle: 270.00°");
        }
    ));

    // --- Manual slider connection ---
    manual_scale.connect_value_changed(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        current_angle_label,
        #[weak]
        pie_menu,
        move |scale| {
            let angle = scale.value();
            rotation_widget.set_rotation(SmearorRotation::Deg(angle as f32));
            pie_menu.set_rotation(angle as f32);
            current_angle_label.set_label(&format!("Current Angle: {:.2}°", angle));
        }
    ));

    // --- Sync rotation from RotationWidget to PieMenuOverlayWidget ---
    let last_rotation = Rc::new(RefCell::new(0.0f32));
    let rotation_widget_clone = rotation_widget.clone();
    let pie_menu_clone = pie_menu.clone();
    let manual_scale_clone = manual_scale.clone();
    let angle_label_clone = current_angle_label.clone();

    glib::timeout_add_local(Duration::from_millis(16), move || {
        let current = rotation_widget_clone.rotation();
        let mut last = last_rotation.borrow_mut();
        if (current - *last).abs() > 0.1 {
            *last = current;
            pie_menu_clone.set_rotation(current);
            manual_scale_clone.set_value(current as f64);
            angle_label_clone.set_label(&format!("Current Angle: {:.2}°", current));
        }
        glib::ControlFlow::Continue
    });

    // --- Process pie menu messages ---
    let rotation_widget_for_messages = rotation_widget.clone();

    glib::idle_add_local(move || {
        let mut last_rotation_msg: Option<f32> = None;

        loop {
            match receiver.try_recv() {
                Ok(PieMenuMessage::Rotate(degrees)) => {
                    last_rotation_msg = Some(degrees);
                }
                Ok(PieMenuMessage::Event(event)) => match event.as_str() {
                    "rotate-cw" => {
                        let current = rotation_widget_for_messages.rotation();
                        let new_rotation = (current + 90.0) % 360.0;
                        rotation_widget_for_messages.set_rotation_with_animation(new_rotation as f64);
                    }
                    "rotate-ccw" => {
                        let current = rotation_widget_for_messages.rotation();
                        let new_rotation = (current - 90.0 + 360.0) % 360.0;
                        rotation_widget_for_messages.set_rotation_with_animation(new_rotation as f64);
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }

        if let Some(rotation) = last_rotation_msg {
            rotation_widget_for_messages.set_rotation(SmearorRotation::Deg(rotation));
        }

        glib::ControlFlow::Continue
    });

    window.set_child(Some(&main_box));
    window.present();
}
