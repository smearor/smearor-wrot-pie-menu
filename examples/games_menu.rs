//! Games menu demo showcasing hierarchical submenus (3 levels).
//!
//! Ring 1: Game genres (RPG, Action, Strategy, Shooter, Puzzle)
//! Ring 2: Publishers for the selected genre
//! Ring 3: Games by that publisher in that genre
//!
//! Selecting a game displays its title, genre, and publisher in the window.
//! Menu data is loaded from `games_menu.json` at compile time.

use smearor_wrot_pie_menu::MenuItem;
use smearor_wrot_pie_menu::PieMenuMessage;
use smearor_wrot_pie_menu::PieMenuOverlayWidget;
use smearor_wrot_pie_menu::SquareConfig;
use smearor_wrot_pie_menu::color::RgbaColor;
use smearor_wrot_pie_menu::menu_widget::menu_item::handler::PieMenuMenuItemHandler;
use smearor_wrot_pie_menu::overlay_widget::control::handler::PieMenuControlHandler;
use smearor_wrot_pie_menu::overlay_widget::message::handler::PieMenuMessageSender;
use smearor_wrot_pie_menu::{ButtonConfig, CircleConfig};

use dashmap::DashMap;
use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Frame;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Switch;
use gtk4::glib;
use gtk4::prelude::*;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use std::sync::mpsc::channel;

const APP_ID: &str = "io.smearor.pie_menu.games_menu";
const GAMES_JSON: &str = include_str!("games_menu.json");

/// Root JSON structure containing all genres.
#[derive(Deserialize)]
struct MenuData {
    genres: Vec<GenreEntry>,
}

/// A genre entry (ring 1) with its publishers.
#[derive(Deserialize)]
struct GenreEntry {
    id: String,
    label: String,
    #[allow(unused)]
    icon: String,
    color: RgbaColor,
    publisher_color: RgbaColor,
    angle: f32,
    publishers: Vec<PublisherEntry>,
}

/// A publisher entry (ring 2) with its games.
#[derive(Deserialize)]
struct PublisherEntry {
    id: String,
    label: String,
    games: Vec<GameEntry>,
    #[serde(skip)]
    full_id: String,
    #[serde(skip)]
    icon: String,
    #[serde(skip)]
    color: RgbaColor,
    #[serde(skip)]
    angle: f32,
}

impl From<PublisherEntry> for MenuItem {
    fn from(publisher: PublisherEntry) -> Self {
        let config = SquareConfig::from(&publisher);
        let submenu: Vec<MenuItem> = publisher.games.into_iter().map(Into::into).collect();
        MenuItem::builder()
            .id(&publisher.full_id)
            .angle(publisher.angle)
            .event(&publisher.full_id)
            .widget_type("square")
            .config(config)
            .submenu(submenu)
            .build()
    }
}

impl From<&PublisherEntry> for SquareConfig {
    fn from(publisher: &PublisherEntry) -> Self {
        SquareConfig::builder()
            .icon_name(&publisher.icon)
            .label(&publisher.label)
            .color(publisher.color)
            .build()
    }
}

/// A game entry (ring 3, leaf node).
#[derive(Deserialize)]
struct GameEntry {
    id: String,
    label: String,
    title: String,
    publisher: String,
    #[serde(skip)]
    full_id: String,
    #[serde(skip)]
    icon: String,
    #[serde(skip)]
    color: RgbaColor,
    #[serde(skip)]
    angle: f32,
}

impl From<GameEntry> for MenuItem {
    fn from(game: GameEntry) -> Self {
        MenuItem::builder()
            .id(&game.full_id)
            .angle(game.angle)
            .event(&game.full_id)
            .widget_type("circle")
            .config(CircleConfig::from(&game))
            .build()
    }
}

impl From<&GameEntry> for CircleConfig {
    fn from(game: &GameEntry) -> Self {
        CircleConfig::builder().icon_name(&game.icon).label(&game.label).color(game.color).build()
    }
}

/// Lookup info for displaying a selected game.
#[derive(Clone, TypedBuilder)]
struct GameInfo {
    title: String,
    genre: String,
    publisher: String,
}

/// Parsed menu data with a lookup map for game selection events.
struct MenuStore {
    data: MenuData,
    lookup: DashMap<String, GameInfo>,
}

impl MenuStore {
    fn new(data: MenuData) -> Self {
        Self { data, lookup: DashMap::new() }
    }
}

fn load_menu_data() -> MenuStore {
    let data: MenuData = serde_json::from_str(GAMES_JSON).expect("Failed to parse games_menu.json");
    let mut store = MenuStore::new(data);
    for genre in store.data.genres.iter_mut() {
        let publisher_count = genre.publishers.len() as f32;
        for (pub_index, publisher) in genre.publishers.iter_mut().enumerate() {
            publisher.full_id = format!("{}-{}", genre.id, publisher.id);
            publisher.icon = "system-users-symbolic".to_string();
            publisher.color = genre.publisher_color;
            publisher.angle = 360.0 * pub_index as f32 / publisher_count;

            let game_count = publisher.games.len() as f32;
            for (game_index, game) in publisher.games.iter_mut().enumerate() {
                game.full_id = format!("{}-{}", publisher.full_id, game.id);
                game.icon = "applications-games-symbolic".to_string();
                game.color = genre.color;
                game.angle = 360.0 * game_index as f32 / game_count;

                store.lookup.insert(
                    game.full_id.clone(),
                    GameInfo::builder()
                        .title(game.title.clone())
                        .genre(genre.label.clone())
                        .publisher(game.publisher.clone())
                        .build(),
                );
            }
        }
    }
    store
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Games Menu — 3-Level Submenu Demo")
        .default_width(1024)
        .default_height(1024)
        .build();

    window.connect_close_request({
        let app = app.clone();
        move |_| {
            app.quit();
            glib::Propagation::Proceed
        }
    });

    let main_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title_label = Label::builder()
        .label("Games Menu — Genre -> Publisher -> Game")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title_label);

    let hint_label = Label::builder()
        .label("Open the pie menu. Select a genre, then a publisher, then a game.")
        .halign(Align::Center)
        .build();
    main_box.append(&hint_label);

    // --- Info display ---
    let info_frame = Frame::builder().label("Selected Game").margin_top(12).margin_bottom(12).build();

    let info_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();

    let game_label = Label::builder()
        .label("— No game selected —")
        .css_classes(["title-2"])
        .halign(Align::Center)
        .build();

    let genre_label = Label::builder().label("Genre: —").halign(Align::Center).build();

    let publisher_label = Label::builder().label("Publisher: —").halign(Align::Center).build();

    info_box.append(&game_label);
    info_box.append(&genre_label);
    info_box.append(&publisher_label);
    info_frame.set_child(Some(&info_box));
    main_box.append(&info_frame);

    // --- Pie menu switch ---
    let controls_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let switch_label = Label::new(Some("Pie Menu"));
    let pie_menu_switch = Switch::builder().active(false).build();
    controls_box.append(&switch_label);
    controls_box.append(&pie_menu_switch);
    main_box.append(&controls_box);

    // --- Build pie menu: Genre -> Publisher -> Game ---
    let (sender, receiver) = channel::<PieMenuMessage>();

    let placeholder = Label::builder()
        .label("Open the pie menu to browse games by genre")
        .hexpand(true)
        .vexpand(true)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let pie_menu = PieMenuOverlayWidget::new(Some(placeholder.upcast_ref()));
    pie_menu.set_message_sender(sender);
    pie_menu.set_submenu_radius_step(90.0);

    let store = load_menu_data();

    for genre in store.data.genres {
        let submenu: Vec<MenuItem> = genre.publishers.into_iter().map(Into::into).collect();
        pie_menu
            .add_menu_item(
                MenuItem::builder()
                    .id(format!("genre-{id}", id = genre.id))
                    .angle(genre.angle)
                    .fixed_position(true)
                    .event(format!("genre-{id}", id = genre.id))
                    .widget_type("button")
                    .config(ButtonConfig::builder().label(genre.label).build())
                    .submenu(submenu)
                    .build(),
            )
            .expect("Failed to add genre");
    }

    let viewport_frame = Frame::builder()
        .label("Pie Menu Viewport")
        .hexpand(true)
        .vexpand(true)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    viewport_frame.set_child(Some(&pie_menu));
    main_box.append(&viewport_frame);

    // --- Switch connection ---
    {
        let pie_menu_clone = pie_menu.clone();
        pie_menu_switch.connect_state_set(move |_switch, is_active| {
            if is_active {
                let _ = pie_menu_clone.show_pie_menu();
            } else {
                let _ = pie_menu_clone.hide_pie_menu();
            }
            glib::Propagation::Proceed
        });
    }

    // --- Process pie menu messages ---
    let game_label_clone = game_label.clone();
    let genre_label_clone = genre_label.clone();
    let publisher_label_clone = publisher_label.clone();
    let switch_clone = pie_menu_switch.clone();

    glib::idle_add_local(move || {
        loop {
            match receiver.try_recv() {
                Ok(PieMenuMessage::Opened) => {
                    switch_clone.set_active(true);
                }
                Ok(PieMenuMessage::Closed) => {
                    switch_clone.set_active(false);
                }
                Ok(PieMenuMessage::Event(event)) => {
                    if let Some(info) = store.lookup.get(&event) {
                        game_label_clone.set_label(&info.title);
                        genre_label_clone.set_label(&format!("Genre: {}", info.genre));
                        publisher_label_clone.set_label(&format!("Publisher: {}", info.publisher));
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        glib::ControlFlow::Continue
    });

    window.set_child(Some(&main_box));
    window.present();
}
