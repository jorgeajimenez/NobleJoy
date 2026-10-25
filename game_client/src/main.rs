mod capture;
mod components;
mod level_builder;
mod network;
mod scene_parser;
mod systems;

use bevy::prelude::*;
use components::GameSessionState;
use network::StudioNetworkChannel;
use systems::{
    handle_studio_hotkeys, player_movement_and_collision, poll_studio_network_events,
    setup_base_environment, update_hud_overlay,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Argon Game Studio — Jorge Rust Engine (Bevy 0.14)".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<StudioNetworkChannel>()
        .init_resource::<GameSessionState>()
        .add_systems(Startup, setup_base_environment)
        .add_systems(
            Update,
            (
                handle_studio_hotkeys,
                poll_studio_network_events,
                player_movement_and_collision,
                update_hud_overlay,
            ),
        )
        .run();
}
