use bevy::{
    app::AppExit,
    ecs::spawn::{SpawnIter, SpawnWith},
    prelude::*,
};

use crate::palette::{BLUE, CORAL, DARK, DARKER};

use crate::{AppState, Volume, despawn_screen};
// This plugin manages the menu, with 5 different screens:
// - a main menu with "New Game", "Settings", "Quit"
// - a settings menu with two submenus and a back button
// - two settings screen with a setting that can be set and a back button
pub fn menu_plugin(app: &mut App) {
    app
        // At start, the menu is not enabled. This will be changed in `menu_setup` when
        // entering the `GameState::Menu` state.
        // Current screen in the menu is handled by an independent state from `GameState`
        .init_state::<MenuState>()
        .add_systems(OnEnter(AppState::Menu), menu_setup)
        // Systems to handle the main menu screen
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        // Systems to handle the settings menu screen
        .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
        .add_systems(
            OnExit(MenuState::Settings),
            despawn_screen::<OnSettingsMenuScreen>,
        )
        // Systems to handle the display settings screen
        // Systems to handle the sound settings screen
        .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
        .add_systems(
            Update,
            setting_button::<Volume>.run_if(in_state(MenuState::SettingsSound)),
        )
        .add_systems(
            OnExit(MenuState::SettingsSound),
            despawn_screen::<OnSoundSettingsMenuScreen>,
        )
        // Common systems to all screens that handles buttons behavior
        .add_systems(
            Update,
            (menu_action, button_system).run_if(in_state(AppState::Menu)),
        );
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    SettingsSound,
    #[default]
    Disabled,
}

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
struct OnSettingsMenuScreen;

// Tag component used to tag entities added on the display settings menu screen
#[derive(Component)]
struct OnDisplaySettingsMenuScreen;

// Tag component used to tag entities added on the sound settings menu screen
#[derive(Component)]
struct OnSoundSettingsMenuScreen;

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    SettingsSound,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &Children, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut TextColor, With<Text>>,
) {
    for (interaction, children, selected) in &mut interaction_query {
        for child in children.iter() {
            let maybe_child = text_query.get_mut(child); // Removed dereference operator (*)
            if let Ok(mut color) = maybe_child {
                *color = match (*interaction, selected) {
                    (Interaction::Pressed, _) | (Interaction::None, Some(_)) => BLUE.into(),
                    (Interaction::Hovered, Some(_)) => BLUE.into(),
                    (Interaction::Hovered, None) => BLUE.into(),
                    (Interaction::None, None) => DARK.into(),
                }
            }
        }
    }
}

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected
fn setting_button<T: Resource + Component + PartialEq + Copy>(
    mut interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    selected_query: Single<Entity, With<SelectedOption>>,
    mut bg_colors: Query<&mut BackgroundColor, With<Button>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    let previously_selected_button = selected_query.into_inner();
    for (interaction, button_setting, current_interacted_button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed if *setting != *button_setting => {
                bg_colors.get_mut(previously_selected_button).unwrap().0 = DARKER;
                bg_colors.get_mut(current_interacted_button).unwrap().0 = CORAL;
                commands
                    .entity(previously_selected_button)
                    .remove::<SelectedOption>();
                commands
                    .entity(current_interacted_button)
                    .insert(SelectedOption);
                *setting = *button_setting;
            }
            Interaction::Hovered if *setting != *button_setting => {
                bg_colors.get_mut(current_interacted_button).unwrap().0 = BLUE;
            }
            Interaction::None if *setting != *button_setting => {
                bg_colors.get_mut(current_interacted_button).unwrap().0 = DARKER;
            }
            _ => {}
        }
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(0.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        font: assets.load("fonts/PressStart2P-Regular.ttf"),
        ..default()
    };

    let bg = assets.load("images/title.png");

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        ImageNode::new(bg),
        OnMainMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect {
                    left: Val::Percent(0.0),
                    right: Val::Percent(19.5),
                    top: Val::Percent(0.0),
                    bottom: Val::Percent(10.0),
                },
                ..default()
            },
            children![
                (
                    Button,
                    button_node.clone(),
                    MenuButtonAction::Play,
                    children![(
                        Text::new("New Game"),
                        button_text_font.clone(),
                        TextColor(DARKER),
                    )]
                ),
                (
                    Button,
                    button_node.clone(),
                    MenuButtonAction::Settings,
                    children![(
                        Text::new("Settings"),
                        button_text_font.clone(),
                        TextColor(DARKER),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    MenuButtonAction::Quit,
                    children![(Text::new("Quit"), button_text_font, TextColor(DARKER),),]
                ),
            ]
        )],
    ));
}

fn settings_menu_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            font: assets.load("fonts/PressStart2P-Regular.ttf"),
            ..default()
        },
        TextColor(DARKER),
    );

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            Children::spawn(SpawnIter(
                [
                    (MenuButtonAction::SettingsSound, "Sound"),
                    (MenuButtonAction::BackToMainMenu, "Back"),
                ]
                .into_iter()
                .map(move |(action, text)| {
                    (
                        Button,
                        button_node.clone(),
                        action,
                        children![(Text::new(text), button_text_style.clone())],
                    )
                })
            ))
        )],
    ));
}

fn sound_settings_menu_setup(
    mut commands: Commands,
    volume: Res<Volume>,
    assets: Res<AssetServer>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: 33.0,
            font: assets.load("fonts/PressStart2P-Regular.ttf"),
            ..default()
        },
        TextColor(DARKER),
    );

    let volume = *volume;
    let button_node_clone = button_node.clone();
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSoundSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Children::spawn((
                        Spawn((Text::new("Volume"), button_text_style.clone())),
                        SpawnWith(move |parent: &mut ChildSpawner| {
                            for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                                let mut entity = parent.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(65.0),
                                        ..button_node_clone.clone()
                                    },
                                    if volume == Volume(volume_setting) {
                                        BackgroundColor(CORAL)
                                    } else {
                                        BackgroundColor(DARKER)
                                    },
                                    Volume(volume_setting),
                                ));

                                if volume == Volume(volume_setting) {
                                    entity.insert(SelectedOption);
                                }
                            }
                        })
                    ))
                ),
                (
                    Button,
                    button_node,
                    MenuButtonAction::BackToSettings,
                    children![(Text::new("Back"), button_text_style)]
                )
            ]
        )],
    ));
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(AppState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}
