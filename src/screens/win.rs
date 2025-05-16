use crate::{AppState, TEXT_COLOR, despawn_screen};
use bevy::prelude::*;

#[derive(Component)]
struct OnWinScreen;

// Plugin definition
pub fn win_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::GameOver), setup_gameover_screen)
        .add_systems(OnExit(AppState::GameOver), despawn_screen::<OnWinScreen>)
        .add_systems(
            Update,
            process_commands.run_if(in_state(AppState::GameOver)),
        );
}

fn setup_gameover_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnWinScreen,
        BackgroundColor(Color::BLACK),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Text::new("You win!"),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    TextShadow::default()
                ),
                (
                    Text::new("Press any key to restart or esc for the menu."),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    TextShadow::default()
                )
            ],
        )],
    ));
}

fn process_commands(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.set_state(AppState::Menu);
        return;
    }

    if keyboard_input.get_just_pressed().len() > 0 {
        commands.set_state(AppState::Game);
        return;
    }
}
