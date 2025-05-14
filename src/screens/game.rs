use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use crate::stepping;
use crate::{AppState, TEXT_COLOR, despawn_screen};

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Disabled,
    Starting,
    Playing,
}

pub fn game_plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Lives(INITIAL_LIVES))
        .add_event::<GameEvent>()
        .add_systems(OnEnter(GameState::Starting), countdown_setup)
        .add_systems(OnEnter(AppState::Game), game_setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_sounds,
            )
                // `chain`ing systems together runs them in order
                .chain()
                .run_if(in_state(AppState::Game))
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (update_scoreboard, update_lives).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>)
        .add_systems(
            Update,
            update_countdown.run_if(|state: Res<State<GameState>>| *state != GameState::Disabled),
        );
}

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// These values are exact
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const UI_TEXT_FONT_SIZE: f32 = 33.0;
const UI_PADDING: Val = Val::Px(5.0);

const INITIAL_LIVES: usize = 3;

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const LIVES_COLOR: Color = Color::srgb(0.6, 0.8, 0.5);

const STARTING_GAME_DURATION_SECS: f32 = 3.0;
const GO_DISPLAY_DURATION_SECS: f32 = 1.0;

// Tag components
#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct OnStartingScreen;

#[derive(Resource, Deref, DerefMut)]
struct CountdownTimer(Timer);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Deadly;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

// This resource tracks the game's score
#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Resource, Deref, DerefMut)]
struct Lives(usize);

#[derive(Component)]
struct LivesUi;

#[derive(Component)]
struct ScoreboardUi;
#[derive(Event, Default)]
// Menu
#[derive(Component)]
struct MenuTag;

// Events
#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Event, Default)]
struct GameOverEvent;

#[derive(Event, Default)]
struct LostLifeEvent;

#[derive(Event)]
enum GameEvent {
    Collision(CollisionEvent),
    LostLife(LostLifeEvent),
    GameOver(GameOverEvent),
}

#[derive(Component)]
struct Brick;

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// Default must be implemented to define this as a required component for the Wall component below
#[derive(Component, Default)]
struct Collider;

// This is a collection of the components that define a "Wall" in our game
#[derive(Component)]
#[require(Sprite, Transform, Collider)]
struct Wall;

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    /// Location of the *center* of the wall, used in `transform.translation()`
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    /// (x, y) dimensions of the wall, used in `transform.scale()`
    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl Wall {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    // Notice the use of Sprite and Transform alongside Wall, overwriting the default values defined for the required components
    fn new(location: WallLocation) -> (Wall, Sprite, Transform) {
        (
            Wall,
            Sprite::from_color(WALL_COLOR, Vec2::ONE),
            Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: location.position().extend(0.0),
                // The z-scale of 2D objects must always be 1.0,
                // or their ordering will be affected in surprising ways.
                // See https://github.com/bevyengine/bevy/issues/4149
                scale: location.size().extend(1.0),
                ..default()
            },
        )
    }
}

fn game_setup(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    **lives = INITIAL_LIVES;
    **score = 0;

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
        Transform {
            translation: Vec3::new(0.0, paddle_y, 0.0),
            scale: PADDLE_SIZE.extend(1.0),
            ..default()
        },
        Paddle,
        Collider,
        OnGameScreen,
    ));

    // Ball
    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
        OnGameScreen,
    ));

    // Scoreboard
    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: UI_TEXT_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
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
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
        OnGameScreen,
    ));

    // Lives
    commands.spawn((
        Text::new("Lives: "),
        TextFont {
            font_size: UI_TEXT_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        LivesUi,
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
                ..default()
            },
            TextColor(LIVES_COLOR),
        )],
        OnGameScreen,
    ));

    // Walls
    commands.spawn((Wall::new(WallLocation::Left), OnGameScreen));
    commands.spawn((Wall::new(WallLocation::Right), OnGameScreen));
    commands.spawn((Wall::new(WallLocation::Bottom), Deadly, OnGameScreen));
    commands.spawn((Wall::new(WallLocation::Top), OnGameScreen));

    // Bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
      // Space taken up by the bricks
      - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
      // Space taken up by the gaps
      - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn((
                Sprite {
                    color: BRICK_COLOR,
                    ..default()
                },
                Transform {
                    translation: brick_position.extend(0.0),
                    scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                    ..default()
                },
                Brick,
                Collider,
                OnGameScreen,
            ));
        }
    }

    commands.set_state(GameState::Starting);
}

#[derive(Component)]
struct CountdownUI;

fn countdown_setup(mut commands: Commands) {
    commands.insert_resource(CountdownTimer(Timer::from_seconds(
        STARTING_GAME_DURATION_SECS + GO_DISPLAY_DURATION_SECS,
        TimerMode::Once,
    )));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        OnStartingScreen,
        OnGameScreen,
        children![(
            Text::new(STARTING_GAME_DURATION_SECS.to_string()),
            TextFont {
                font_size: 120.0,
                ..default()
            },
            CountdownUI,
            OnStartingScreen,
            TextColor(TEXT_COLOR)
        )],
    ));
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn play_sounds(
    mut commands: Commands,
    mut events: EventReader<GameEvent>,
    sound: Res<CollisionSound>,
) {
    if !events.is_empty() {
        // Play sounds for each relevant event
        for event in events.read() {
            match event {
                GameEvent::Collision(_) => {
                    commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
                }
                GameEvent::LostLife(_) => {
                    info!("Lost Life");
                }
                GameEvent::GameOver(_) => {
                    info!("Game Over");
                }
            }
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_transform: Single<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut events: EventWriter<GameEvent>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    ball_query: Single<(&mut Velocity, &mut Transform), With<Ball>>,
    collider_query: Query<
        (Entity, &Transform, Option<&Brick>, Option<&Deadly>),
        (With<Collider>, Without<Ball>),
    >,
) {
    let (mut ball_velocity, mut ball_transform) = ball_query.into_inner();

    for (collider_entity, collider_transform, maybe_brick, maybe_deadly) in &collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Writes a collision event so that other systems can react to the collision
            events.write(GameEvent::Collision(CollisionEvent));

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn();
                **score += 1;

                let remaining_bricks = collider_query
                    .iter()
                    .filter(|(_, _, maybe_brick, _)| maybe_brick.is_some())
                    .count();

                if remaining_bricks == 0 {
                    commands.set_state(AppState::Win);
                    return;
                }
            }

            if maybe_deadly.is_some() {
                if **lives == 0 {
                    events.write(GameEvent::GameOver(GameOverEvent));
                    commands.set_state(AppState::GameOver);
                    return;
                } else {
                    ball_transform.translation = BALL_STARTING_POSITION;
                    ball_velocity.0 = INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED;

                    **lives -= 1;
                    events.write(GameEvent::LostLife(LostLifeEvent));
                }
            }

            // Reflect the ball's velocity when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // Reflect only if the velocity is in the opposite direction of the collision
            // This prevents the ball from getting stuck inside the bar
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // Reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // Reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn update_lives(
    lives: Res<Lives>,
    lives_root: Single<Entity, (With<Text>, With<LivesUi>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*lives_root, 1) = lives.to_string();
}

fn update_countdown(
    time: Res<Time>,
    countdown: Option<ResMut<CountdownTimer>>,
    mut commands: Commands,
    countdown_root: Single<Entity, With<CountdownUI>>,
    mut writer: TextUiWriter,
    starting_entities: Query<Entity, With<OnStartingScreen>>,
) {
    if let Some(mut countdown) = countdown {
        if countdown.just_finished() {
            despawn_screen::<OnStartingScreen>(starting_entities, commands);
            return;
        }

        if countdown.tick(time.delta()).elapsed_secs() >= STARTING_GAME_DURATION_SECS {
            commands.set_state(GameState::Playing);
        }

        let secs = (countdown.remaining_secs() - GO_DISPLAY_DURATION_SECS).ceil();
        *writer.text(*countdown_root, 0) = if secs == 0. {
            "GO!".to_string()
        } else {
            secs.to_string()
        };

        // Interpolate background color based on countdown remaining time
        let progress = 1.0
            - (countdown.remaining_secs() - STARTING_GAME_DURATION_SECS) / GO_DISPLAY_DURATION_SECS;

        // do dimming here somehow
    }
}
