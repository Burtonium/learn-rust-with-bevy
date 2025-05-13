use bevy::prelude::*;
use crate::{despawn_screen, GameState, TEXT_COLOR};

// Tag component to mark entities added on the game over screen
#[derive(Component)]
struct OnGameOverScreen;

// Plugin definition
pub fn gameover_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::GameOver), setup_gameover_screen)
        .add_systems(OnExit(GameState::GameOver), despawn_screen::<OnGameOverScreen>)
        .add_systems(Update, process_commands.run_if(in_state(GameState::GameOver)));
}

// Marker struct to help identify the color-changing Text component
#[derive(Component)]
struct AnimatedText;

fn setup_gameover_screen(mut commands: Commands) {
  commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    },
    OnGameOverScreen,
    BackgroundColor(Color::BLACK),
    children![(
      Node {
          flex_direction: FlexDirection::Column,
          align_items: AlignItems::Center,
          ..default()
      },
      children![(
          Text::new("Game Over!"),
          TextFont {
              font_size: 67.0,
              ..default()
          },
          TextColor(TEXT_COLOR),
          TextShadow::default()
      ), (
          Text::new("Press any key to restart or esc for the menu."),
          TextFont {
              font_size: 33.0,
              ..default()
          },
          TextColor(TEXT_COLOR),
          TextShadow::default()
      )],
    )]
  ));
}


fn process_commands(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut commands: Commands,
) {
  if keyboard_input.pressed(KeyCode::Escape) {
      commands.set_state(GameState::Menu);
      return;
  }

  if (keyboard_input.get_pressed().len() > 0) {
      commands.set_state(GameState::Game);
      return;
  }
}