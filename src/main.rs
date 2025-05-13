//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

mod stepping;
mod screens;

use bevy::{
    prelude::*,
};

use screens::{game, menu};

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Low,
    Medium,
    High,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Game,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins(
            (menu::menu_plugin, game::game_plugin)
        )
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