//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

mod palette;
mod screens;
mod stepping;

use bevy::prelude::*;

use screens::{game, gameover, menu, win};

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Game,
    GameOver,
    Win,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Volume(7))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_plugins((
            menu::menu_plugin,
            game::game_plugin,
            gameover::gameover_plugin,
            win::win_plugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
