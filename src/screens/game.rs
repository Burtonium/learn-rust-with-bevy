use crate::{AppState, despawn_screen, palette::LIGHT};
use bevy::prelude::*;

use crate::palette::DARKER;

const UI_TEXT_FONT_SIZE: f32 = 50.0;
const UI_PADDING: Val = Val::Percent(2.0);
const TEXT_COLOR: Color = LIGHT;

#[derive(Component)]
struct GameScreen;

#[derive(Clone, Default, Copy, Debug, Eq, Hash, PartialEq)]
enum Area {
    #[default]
    DeadbeatArea,
    Condo,
    LuxuryCondo,
    Mansion,
    BusinessDistrict,
    RestrictedArea, // Example of non-rentable area
}

impl Area {
    fn get_rent_cost(&self) -> Option<f32> {
        match self {
            Area::DeadbeatArea => Some(250.0),
            Area::Condo => Some(1000.0),
            Area::LuxuryCondo => Some(2500.0),
            Area::Mansion => Some(10000.0),
            Area::BusinessDistrict => Some(500.0),
            _ => None, // Non-rentable
        }
    }
}

impl Area {
    fn get_image(&self) -> &str {
        match self {
            Area::DeadbeatArea => "images/locations/deadbeat.png",
            Area::Condo => "images/locations/condo.png",
            Area::LuxuryCondo => "images/locations/luxury.png",
            Area::BusinessDistrict => "images/locations/business.png",
            Area::Mansion => "images/locations/mansion.png",
            _ => panic!("Non-rentable area"),
        }
    }
}

#[derive(Resource)]
struct HomeArea {
    location: Area,
    rent: f32,
}

impl Default for HomeArea {
    fn default() -> Self {
        HomeArea {
            rent: Area::DeadbeatArea.get_rent_cost().unwrap(),
            location: Area::DeadbeatArea,
        }
    }
}

#[derive(Resource)]
struct CurrentArea(Area);

impl Default for CurrentArea {
    fn default() -> Self {
        CurrentArea(Area::DeadbeatArea)
    }
}

#[derive(Resource, Default)]
struct WorkArea {
    location: Option<Area>,
    rent: u32,
}

#[derive(Resource, Default)]
struct Money {
    amount: u32,
}

#[derive(Resource, Default)]
struct Time {
    day: u32,
    hour: u32,
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct MoneyUi;

#[derive(Component)]
struct TimeUi;

#[derive(Component)]
struct RentUi;

pub fn game_plugin(app: &mut App) {
    app.init_resource::<HomeArea>()
        .init_resource::<CurrentArea>()
        .init_resource::<WorkArea>()
        .init_resource::<Money>()
        .init_resource::<Time>()
        .add_systems(OnEnter(AppState::Game), setup_game)
        .add_systems(OnExit(AppState::Game), despawn_screen::<GameScreen>)
        .add_systems(Update, update_ui.run_if(in_state(AppState::Game)));
}

fn setup_game(mut commands: Commands, area: Res<CurrentArea>, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/PressStart2P-Regular.ttf");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        GameScreen,
        Background,
        ImageNode::new(asset_server.load(area.0.get_image())),
        children![
            (
                Text::new("Money: "),
                TextFont {
                    font_size: UI_TEXT_FONT_SIZE,
                    font: font.clone(),
                    ..default()
                },
                TextShadow {
                    color: Color::BLACK,
                    offset: Vec2 { x: 3.0, y: 3.0 },
                },
                TextColor(TEXT_COLOR),
                MoneyUi,
                Node {
                    position_type: PositionType::Absolute,
                    top: UI_PADDING,
                    left: UI_PADDING,
                    ..default()
                },
                children![(
                    TextSpan::default(),
                    TextFont {
                        font_size: UI_TEXT_FONT_SIZE,
                        font: font.clone(),
                        ..default()
                    },
                    TextShadow {
                        color: Color::BLACK,
                        offset: Vec2 { x: 3.0, y: 3.0 },
                    },
                    TextColor(TEXT_COLOR),
                )],
            ),
            (
                Text::new("Rent: "),
                TextFont {
                    font_size: UI_TEXT_FONT_SIZE,
                    font: font.clone(),
                    ..default()
                },
                TextColor(TEXT_COLOR),
                RentUi,
                TextShadow {
                    color: Color::BLACK,
                    offset: Vec2 { x: 3.0, y: 3.0 },
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: UI_PADDING,
                    right: UI_PADDING,
                    ..default()
                },
                children![(
                    TextSpan::default(),
                    TextFont {
                        font_size: UI_TEXT_FONT_SIZE,
                        font: font.clone(),
                        ..default()
                    },
                    TextShadow {
                        color: Color::BLACK,
                        offset: Vec2 { x: 3.0, y: 3.0 },
                    },
                    TextColor(TEXT_COLOR),
                )],
            )
        ],
    ));
}

fn update_ui(
    money: Res<Money>,
    money_root: Single<Entity, With<MoneyUi>>,
    mut writer: TextUiWriter,
    home: Res<HomeArea>,
    rent_root: Single<Entity, With<RentUi>>,
) {
    *writer.text(*money_root, 1) = money.amount.to_string();
    *writer.text(*rent_root, 1) = home.rent.to_string();
}
