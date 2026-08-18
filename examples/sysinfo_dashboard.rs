//! Sysinfo dashboard demo displaying real-time system metrics using
//! custom gauge widgets.
//!
//! Metrics displayed:
//! - CPU usage percentage (global)
//! - CPU temperature (first thermal sensor)
//! - Memory usage percentage
//! - Disk usage percentage (root partition)
//!
//! Data is refreshed every second via `glib::timeout_add_local`.

use smearor_wrot_pie_menu::GaugeConfig;
use smearor_wrot_pie_menu::GaugeItemWidget;
use smearor_wrot_pie_menu::GaugeItemWidgetParams;
use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Box;
use gtk4::GestureClick;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Picture;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use sysinfo::Components;
use sysinfo::Disks;
use sysinfo::System;

const APP_ID: &str = "io.smearor.pie_menu.sysinfo_dashboard";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Sysinfo Dashboard")
        .default_width(600)
        .default_height(700)
        .build();

    window.connect_close_request({
        let app = app.clone();
        move |_| {
            app.quit();
            glib::Propagation::Proceed
        }
    });

    let info_label = Label::builder()
        .label("Pinch to open the pie menu. Gauges update every second.")
        .halign(Align::Center)
        .margin_top(12)
        .margin_bottom(8)
        .build();

    let logo = Picture::for_filename("assets/smearor.png");
    logo.set_hexpand(true);
    logo.set_vexpand(true);
    logo.set_halign(Align::Center);
    logo.set_valign(Align::Center);

    // Center widget: a gauge showing overall CPU load, with click-to-close
    let center_gauge = GaugeItemWidget::new(GaugeItemWidgetParams {
        label: "CPU".to_string(),
        value: 0.0,
        unit: "%".to_string(),
        min: 0.0,
        warning: 80.0,
        critical: 90.0,
        max: 100.0,
        item_radius: 90.0,
        enabled: true,
    });

    let overlay = PieMenuOverlayWidget::new(Some(logo.upcast_ref()))
        .with_pie_menu_radius(250.0)
        .with_pie_menu_center_radius(100.0)
        .with_activation_threshold(2.0)
        .with_deactivation_threshold(0.4)
        .with_center_widget(center_gauge.upcast_ref())
        // .with_rotation_gesture_enabled(false)
        // .with_markings_enabled(false)
        .with_menu_item(
            MenuItem::builder()
                .id("temp")
                .angle(45.0)
                .event("temp")
                .radius(70.0)
                .widget_type("gauge")
                .config(
                    GaugeConfig::builder()
                        .label("Temp")
                        .value(0.0)
                        .unit("°C")
                        .min(0.0)
                        .warning(80.0)
                        .critical(90.0)
                        .max(110.0)
                        .build(),
                )
                .build(),
        )
        .expect("Failed to add 'temp' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("mem")
                .angle(135.0)
                .event("mem")
                .radius(70.0)
                .widget_type("gauge")
                .config(
                    GaugeConfig::builder()
                        .label("Mem")
                        .value(0.0)
                        .unit("%")
                        .min(0.0)
                        .warning(70.0)
                        .critical(90.0)
                        .max(100.0)
                        .build(),
                )
                .build(),
        )
        .expect("Failed to add 'mem' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("disk")
                .angle(270.0)
                .event("disk")
                .radius(70.0)
                .widget_type("gauge")
                .config(
                    GaugeConfig::builder()
                        .label("Disk")
                        .value(0.0)
                        .unit("%")
                        .min(0.0)
                        .warning(70.0)
                        .critical(90.0)
                        .max(100.0)
                        .build(),
                )
                .build(),
        )
        .expect("Failed to add 'disk' menu item");

    // Center widget click handler: close submenu or menu
    let overlay_for_center = overlay.clone();
    let center_click = GestureClick::new();
    center_click.connect_pressed(move |_, _, _, _| {
        if overlay_for_center.submenu_depth() > 0 {
            let _ = overlay_for_center.close_submenu();
        } else {
            let _ = overlay_for_center.hide_pie_menu();
        }
    });
    center_gauge.add_controller(center_click);

    let system = Rc::new(RefCell::new(System::new()));
    let components = Rc::new(RefCell::new(Components::new_with_refreshed_list()));
    let disks = Rc::new(RefCell::new(Disks::new_with_refreshed_list()));

    // Initial refresh so CPU usage is available on next refresh
    system.borrow_mut().refresh_cpu_usage();
    system.borrow_mut().refresh_memory();

    let overlay_clone = overlay.clone();
    let center_gauge_clone = center_gauge.clone();
    let system_clone = system.clone();
    let components_clone = components.clone();
    let disks_clone = disks.clone();

    glib::timeout_add_local(Duration::from_secs(1), move || {
        // Refresh system data
        {
            let mut sys = system_clone.borrow_mut();
            sys.refresh_cpu_usage();
            sys.refresh_memory();
        }
        components_clone.borrow_mut().refresh(false);
        disks_clone.borrow_mut().refresh(false);

        // CPU usage
        let cpu_usage = system_clone.borrow().global_cpu_usage() as f64;
        update_gauge(&overlay_clone, "cpu", cpu_usage);
        center_gauge_clone.set_value(cpu_usage);

        // CPU temperature — find first component with a valid temperature
        let temp = components_clone
            .borrow()
            .iter()
            .filter_map(|component| component.temperature())
            .find(|temp| temp.is_finite())
            .map(|temp| temp as f64)
            .unwrap_or(0.0);
        update_gauge(&overlay_clone, "temp", temp);

        // Memory usage percentage
        let mem_sys = system_clone.borrow();
        let total_mem = mem_sys.total_memory() as f64;
        let used_mem = mem_sys.used_memory() as f64;
        let mem_percent = if total_mem > 0.0 { (used_mem / total_mem) * 100.0 } else { 0.0 };
        drop(mem_sys);
        update_gauge(&overlay_clone, "mem", mem_percent);

        // Disk usage percentage (use first disk / root partition)
        let disk_percent = {
            let disks = disks_clone.borrow();
            disks
                .list()
                .iter()
                .find(|disk| disk.mount_point() == std::path::Path::new("/"))
                .or_else(|| disks.list().first())
                .map(|disk| {
                    let total = disk.total_space() as f64;
                    let available = disk.available_space() as f64;
                    if total > 0.0 { ((total - available) / total) * 100.0 } else { 0.0 }
                })
                .unwrap_or(0.0)
        };
        update_gauge(&overlay_clone, "disk", disk_percent);

        glib::ControlFlow::Continue
    });

    let container = Box::new(Orientation::Vertical, 0);
    container.append(&info_label);
    container.append(&overlay);
    overlay.set_vexpand(true);
    overlay.set_hexpand(true);

    window.set_child(Some(&container));
    window.present();
}

fn update_gauge(overlay: &PieMenuOverlayWidget, id: &str, value: f64) {
    if let Some(item) = overlay.get_menu_item(id)
        && let Some(config_value) = &item.widget_config
        && let Ok(mut config) = serde_json::from_value::<GaugeConfig>(config_value.clone())
    {
        config.value = value;
        let _ = overlay.set_widget_config(id, serde_json::to_value(&config).unwrap_or(serde_json::Value::Null));
    }
}
