//! Media player demo demonstrating all pie menu features:
//!
//! - Fixed-position items (Prev at 180°, Next at 0°)
//! - Flexible items (Play/Pause, Stop, Exit) via `add_menu_item_auto`
//! - Auto distribution with proportional segment sizing
//! - Overlap validation with rollback
//! - Builder pattern (`with_message_sender`, `with_menu_item`, etc.)
//! - Disabled state (Prev disabled at first song, Next disabled at last song)
//! - Progress bar, title, artist, album rendering

use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Box;
use gtk4::Button;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::ProgressBar;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::time::Duration;

const APP_ID: &str = "io.smearor.pie_menu.media_player_demo";

/// A fictional track in the playlist.
struct Track {
    title: &'static str,
    artist: &'static str,
    album: &'static str,
    duration_secs: f64,
}

const PLAYLIST: &[Track] = &[
    Track {
        title: "Neon Horizon",
        artist: "Synthwave Collective",
        album: "Midnight Drive",
        duration_secs: 214.0,
    },
    Track {
        title: "Echoes of Tomorrow",
        artist: "Aurora Falls",
        album: "Parallel Worlds",
        duration_secs: 187.0,
    },
    Track {
        title: "Gravity Pulse",
        artist: "Quantum Beats",
        album: "Event Horizon",
        duration_secs: 243.0,
    },
    Track {
        title: "Velvet Storm",
        artist: "The Crimson Hours",
        album: "Afterglow",
        duration_secs: 198.0,
    },
    Track {
        title: "Solar Drift",
        artist: "Lumina Path",
        album: "Lightyears",
        duration_secs: 256.0,
    },
];

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Pie Menu Media Player Demo")
        .default_width(500)
        .default_height(600)
        .build();

    window.connect_close_request({
        let app = app.clone();
        move |_| {
            app.quit();
            glib::Propagation::Proceed
        }
    });

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title_label = Label::builder()
        .label("Media Player Demo")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title_label);

    let hint_label = Label::builder()
        .label("Pinch to open the pie menu. Use Prev/Next to navigate songs.")
        .halign(Align::Center)
        .build();
    main_box.append(&hint_label);

    // --- Now playing info ---
    let track_title_label = Label::builder().label(PLAYLIST[0].title).css_classes(["title-2"]).halign(Align::Center).build();
    main_box.append(&track_title_label);

    let artist_label = Label::builder().label(PLAYLIST[0].artist).halign(Align::Center).build();
    main_box.append(&artist_label);

    let album_label = Label::builder()
        .label(PLAYLIST[0].album)
        .halign(Align::Center)
        .css_classes(["dim-label"])
        .build();
    main_box.append(&album_label);

    // --- Progress bar ---
    let progress_bar = ProgressBar::builder().hexpand(true).margin_top(12).margin_start(24).margin_end(24).build();
    main_box.append(&progress_bar);

    let progress_label = Label::builder().label("0:00 / 0:00").halign(Align::Center).build();
    main_box.append(&progress_label);

    // --- Dynamic menu item pool ---
    const EXTRA_ITEMS: &[(&str, &str, &str, &str)] = &[
        ("shuffle", "Shuffle", "media-playlist-shuffle-symbolic", "#6600AA77"),
        ("repeat", "Repeat", "media-playlist-repeat-symbolic", "#0066AA77"),
        ("like", "Like", "emblem-favorite-symbolic", "#AA006677"),
        ("mute", "Mute", "audio-volume-muted-symbolic", "#AA660077"),
    ];
    let extra_item_index = Rc::new(RefCell::new(0usize));

    // --- Track index state ---
    let track_index = Rc::new(RefCell::new(0usize));
    let is_playing = Rc::new(RefCell::new(false));
    let progress = Rc::new(RefCell::new(0.0f64));

    // --- Pie menu with builder pattern ---
    let (sender, receiver) = channel::<PieMenuMessage>();

    let overlay = PieMenuOverlayWidget::new(Some(&main_box.clone().upcast_ref()))
        .with_message_sender(sender)
        .with_activation_threshold(2.0)
        .with_deactivation_threshold(0.4)
        .with_rotation_gesture_enabled(false)
        .with_markings_enabled(false)
        .with_menu_item(
            MenuItem::builder()
                .id("next")
                .label("Next")
                .icon_name("media-skip-forward-symbolic")
                .color("#0044AA77")
                .angle(0.0)
                .fixed_position(true)
                .close_on_click(false)
                .event("next")
                .build(),
        )
        .expect("Failed to add 'next' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("prev")
                .label("Prev")
                .icon_name("media-skip-backward-symbolic")
                .color("#0044AA77")
                .angle(180.0)
                .fixed_position(true)
                .close_on_click(false)
                .event("prev")
                .build(),
        )
        .expect("Failed to add 'prev' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("play-pause")
                .label("Play/Pause")
                .icon_name("media-playback-start-symbolic")
                .color("#00AA0077")
                .angle(90.0)
                .event("play-pause")
                .build(),
        )
        .expect("Failed to add 'play-pause' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("stop")
                .label("Stop")
                .icon_name("media-playback-stop-symbolic")
                .color("#AA000077")
                .angle(270.0)
                .event("stop")
                .build(),
        )
        .expect("Failed to add 'stop' menu item")
        .with_menu_item(
            MenuItem::builder()
                .id("exit")
                .label("Exit")
                .icon_name("window-close-symbolic")
                .color("#44444477")
                .angle(45.0)
                .event("exit")
                .build(),
        )
        .expect("Failed to add 'exit' menu item");

    // --- Disable Prev at first track and Next at last track ---
    update_disabled_state(&overlay, 0);

    // --- Dynamic item controls ---
    let dynamic_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .margin_top(8)
        .build();

    let add_button = Button::with_label("Add Item");
    let remove_button = Button::with_label("Remove Item");
    let item_count_label = Label::new(Some("5 items"));
    dynamic_box.append(&add_button);
    dynamic_box.append(&remove_button);
    dynamic_box.append(&item_count_label);
    main_box.append(&dynamic_box);

    // --- Add/Remove button connections ---
    {
        let overlay_clone = overlay.clone();
        let extra_index_clone = extra_item_index.clone();
        let count_label_clone = item_count_label.clone();

        add_button.connect_clicked(move |_| {
            let mut idx = extra_index_clone.borrow_mut();
            if *idx >= EXTRA_ITEMS.len() {
                return;
            }
            let (id, label, icon, color) = EXTRA_ITEMS[*idx];
            *idx += 1;
            let current_count = *idx;
            drop(idx);
            let _ = overlay_clone.add_menu_item_auto(
                MenuItem::builder()
                    .id(id)
                    .label(label)
                    .icon_name(icon)
                    .color(color)
                    .angle(0.0)
                    .event(id)
                    .build(),
            );
            count_label_clone.set_label(&format!("{} items", 5 + current_count));
        });
    }

    {
        let overlay_clone = overlay.clone();
        let extra_index_clone = extra_item_index.clone();
        let count_label_clone = item_count_label.clone();

        remove_button.connect_clicked(move |_| {
            let mut idx = extra_index_clone.borrow_mut();
            if *idx == 0 {
                return;
            }
            *idx -= 1;
            let (id, _, _, _) = EXTRA_ITEMS[*idx];
            let current_count = *idx;
            drop(idx);
            let _ = overlay_clone.remove_menu_item(id);
            overlay_clone.redistribute();
            count_label_clone.set_label(&format!("{} items", 5 + current_count));
        });
    }

    // --- Update now-playing labels ---
    fn update_now_playing(track_title_label: &Label, artist_label: &Label, album_label: &Label, index: usize) {
        let track = &PLAYLIST[index];
        track_title_label.set_label(track.title);
        artist_label.set_label(track.artist);
        album_label.set_label(track.album);
    }

    fn update_disabled_state(overlay: &PieMenuOverlayWidget, index: usize) {
        let _ = overlay.set_menu_item_enabled("prev", index > 0);
        let _ = overlay.set_menu_item_enabled("next", index < PLAYLIST.len() - 1);
    }

    fn format_time(secs: f64) -> String {
        let total = secs as u64;
        let minutes = total / 60;
        let seconds = total % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    // --- Progress timer ---
    let progress_clone = progress.clone();
    let is_playing_clone = is_playing.clone();
    let progress_bar_clone = progress_bar.clone();
    let progress_label_clone = progress_label.clone();
    let track_index_clone = track_index.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || {
        if *is_playing_clone.borrow() {
            let mut prog = progress_clone.borrow_mut();
            *prog += 0.1;
            let track = &PLAYLIST[*track_index_clone.borrow()];
            if *prog >= track.duration_secs {
                *prog = 0.0;
            }
            progress_bar_clone.set_fraction(*prog / track.duration_secs);
            progress_label_clone.set_label(&format!("{} / {}", format_time(*prog), format_time(track.duration_secs)));
        }
        glib::ControlFlow::Continue
    });

    // --- Process pie menu messages ---
    let overlay_for_messages = overlay.clone();
    let track_title_label_clone = track_title_label.clone();
    let artist_label_clone = artist_label.clone();
    let album_label_clone = album_label.clone();
    let track_index_for_messages = track_index.clone();
    let is_playing_for_messages = is_playing.clone();
    let progress_for_messages = progress.clone();
    let progress_bar_for_messages = progress_bar.clone();
    let progress_label_for_messages = progress_label.clone();
    let window_clone = window.clone();

    glib::idle_add_local(move || {
        loop {
            match receiver.try_recv() {
                Ok(PieMenuMessage::Opened) | Ok(PieMenuMessage::Closed) => {}
                Ok(PieMenuMessage::Rotate(_)) => {}
                Ok(PieMenuMessage::Event(event)) => match event.as_str() {
                    "play-pause" => {
                        let mut playing = is_playing_for_messages.borrow_mut();
                        *playing = !*playing;
                        if *playing {
                            let mut prog = progress_for_messages.borrow_mut();
                            let track = &PLAYLIST[*track_index_for_messages.borrow()];
                            if *prog >= track.duration_secs {
                                *prog = 0.0;
                            }
                        }
                    }
                    "stop" => {
                        *is_playing_for_messages.borrow_mut() = false;
                        *progress_for_messages.borrow_mut() = 0.0;
                        progress_bar_for_messages.set_fraction(0.0);
                        let track = &PLAYLIST[*track_index_for_messages.borrow()];
                        progress_label_for_messages.set_label(&format!("0:00 / {}", format_time(track.duration_secs)));
                    }
                    "next" => {
                        let mut idx = track_index_for_messages.borrow_mut();
                        if *idx < PLAYLIST.len() - 1 {
                            *idx += 1;
                            let new_index = *idx;
                            drop(idx);
                            *progress_for_messages.borrow_mut() = 0.0;
                            update_now_playing(&track_title_label_clone, &artist_label_clone, &album_label_clone, new_index);
                            update_disabled_state(&overlay_for_messages, new_index);
                            let track = &PLAYLIST[new_index];
                            progress_bar_for_messages.set_fraction(0.0);
                            progress_label_for_messages.set_label(&format!("0:00 / {}", format_time(track.duration_secs)));
                        }
                    }
                    "prev" => {
                        let mut idx = track_index_for_messages.borrow_mut();
                        if *idx > 0 {
                            *idx -= 1;
                            let new_index = *idx;
                            drop(idx);
                            *progress_for_messages.borrow_mut() = 0.0;
                            update_now_playing(&track_title_label_clone, &artist_label_clone, &album_label_clone, new_index);
                            update_disabled_state(&overlay_for_messages, new_index);
                            let track = &PLAYLIST[new_index];
                            progress_bar_for_messages.set_fraction(0.0);
                            progress_label_for_messages.set_label(&format!("0:00 / {}", format_time(track.duration_secs)));
                        }
                    }
                    "exit" => {
                        window_clone.close();
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&overlay));
    window.present();
}
