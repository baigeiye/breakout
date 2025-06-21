use bevy::prelude::*;
use bevy::ui::{AlignItems, JustifyContent, PositionType};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const PADDLE_WIDTH: f32 = 120.0;
const PADDLE_HEIGHT: f32 = 20.0;
const BALL_SIZE: f32 = 15.0;
const BRICK_WIDTH: f32 = 60.0;
const BRICK_HEIGHT: f32 = 30.0;
const BRICK_SPACING: f32 = 5.0;

const BALL_SPEED: f32 = 300.0;
const PADDLE_SPEED: f32 = 500.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .insert_resource(GameState::Running)
        .insert_resource(Score(0))
        .insert_resource(LastScore(None))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                title: "Breakout in Rust (Bevy 0.13)".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_paddle, move_ball, ball_collision, check_game_end, restart_game))
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Resource, PartialEq, Eq)]
enum GameState {
    Running,
    Won,
    Lost,
}

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct LastScore(Option<u32>);

#[derive(Component)]
struct GameMessage;

#[derive(Component)]
struct Brick {
    health: u32,
    is_special: bool,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -250.0, 0.0),
                scale: Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            ..default()
        },
        Paddle,
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -200.0, 0.0),
                scale: Vec3::splat(BALL_SIZE),
                ..default()
            },
            sprite: Sprite {
                color: Color::YELLOW,
                ..default()
            },
            ..default()
        },
        Ball {
            velocity: Vec2::new(0.3, 0.5).normalize() * BALL_SPEED,
        },
    ));

    let bricks_per_row = ((WINDOW_WIDTH + BRICK_SPACING) / (BRICK_WIDTH + BRICK_SPACING)).floor() as usize;
    for row in 0..2 {
        for col in 0..bricks_per_row {
            let x = -WINDOW_WIDTH / 2.0 + BRICK_WIDTH / 2.0 + col as f32 * (BRICK_WIDTH + BRICK_SPACING);
            let y = WINDOW_HEIGHT / 2.0 - BRICK_HEIGHT - row as f32 * (BRICK_HEIGHT + BRICK_SPACING);

            let is_special = (row + col) % 4 == 0;

            let color = if is_special {
                Color::rgb(0.2, 0.4, 0.9)
            } else {
                Color::rgb(0.8, 0.2, 0.2)
            };

            let health = if is_special { 2 } else { 1 };

            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        scale: Vec3::new(BRICK_WIDTH, BRICK_HEIGHT, 1.0),
                        ..default()
                    },
                    sprite: Sprite { color, ..default() },
                    ..default()
                },
                Brick { health, is_special },
            ));
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Paddle>>,
    game_state: Res<GameState>,
) {
    if *game_state != GameState::Running {
        return;
    }

    let mut transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    transform.translation.x += direction * PADDLE_SPEED * time.delta_seconds();
    transform.translation.x = transform
        .translation
        .x
        .clamp(-WINDOW_WIDTH / 2.0 + PADDLE_WIDTH / 2.0, WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0);
}

fn move_ball(time: Res<Time>, mut query: Query<(&mut Transform, &Ball)>, game_state: Res<GameState>) {
    if *game_state != GameState::Running {
        return;
    }

    let (mut transform, ball) = query.single_mut();
    let delta = ball.velocity * time.delta_seconds();
    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}

fn ball_collision(
    mut commands: Commands,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    paddle_query: Query<&Transform, With<Paddle>>,
    mut brick_query: Query<(Entity, &Transform, &mut Brick, &mut Sprite)>,
    mut game_state: ResMut<GameState>,
    mut score: ResMut<Score>,
) {
    if *game_state != GameState::Running {
        return;
    }

    let (mut ball, ball_transform) = ball_query.single_mut();
    let ball_pos = ball_transform.translation;

    if ball_pos.x - BALL_SIZE / 2.0 <= -WINDOW_WIDTH / 2.0
        || ball_pos.x + BALL_SIZE / 2.0 >= WINDOW_WIDTH / 2.0
    {
        ball.velocity.x = -ball.velocity.x;
    }

    if ball_pos.y + BALL_SIZE / 2.0 >= WINDOW_HEIGHT / 2.0 {
        ball.velocity.y = -ball.velocity.y;
    }

    if ball_pos.y - BALL_SIZE / 2.0 <= -WINDOW_HEIGHT / 2.0 {
        ball.velocity = Vec2::ZERO;
        *game_state = GameState::Lost;
        return;
    }

    let paddle_transform = paddle_query.single();
    if collide(ball_pos, BALL_SIZE, paddle_transform.translation, PADDLE_WIDTH, PADDLE_HEIGHT) {
        ball.velocity.y = ball.velocity.y.abs();
    }

    let mut hit_brick = None;
    for (entity, brick_transform, mut brick, mut sprite) in brick_query.iter_mut() {
        if collide(ball_pos, BALL_SIZE, brick_transform.translation, BRICK_WIDTH, BRICK_HEIGHT) {
            hit_brick = Some((entity, brick, sprite));
            ball.velocity.y = -ball.velocity.y;
            break;
        }
    }

    if let Some((entity, mut brick, mut sprite)) = hit_brick {
        brick.health -= 1;
        if brick.health == 0 {
            score.0 += if brick.is_special { 2 } else { 1 };
            commands.entity(entity).despawn();
        } else {
            sprite.color.set_a(0.5);
        }
    }

    if brick_query.iter().count() == 0 {
        ball.velocity = Vec2::ZERO;
        *game_state = GameState::Won;
    }
}

fn check_game_end(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<GameMessage>>,
    score: Res<Score>,
    last_score: Res<LastScore>,
) {
    if !game_state.is_changed() || existing.iter().next().is_some() {
        return;
    }

    let main_message = match *game_state {
        GameState::Won => format!("Congratulations! Final Score: {}", score.0),
        GameState::Lost => format!("Game Over! Final Score: {}", score.0),
        _ => return,
    };

    let hint_message = "Press R to restart";

    let compare_message = match last_score.0 {
        None => "",
        Some(last) => {
            if score.0 > last {
                "Better than just now"
            } else if score.0 < last {
                "Keep up the good work"
            } else {
                "Keep it up"
            }
        }
    };

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        },
        GameMessage,
    ))
    .with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                main_message,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                align_self: AlignSelf::Center,
                ..default()
            }),
        );

        if !compare_message.is_empty() {
            parent.spawn(
                TextBundle::from_section(
                    compare_message,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::ORANGE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                }),
            );
        }

        parent.spawn(
            TextBundle::from_section(
                hint_message,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 24.0,
                    color: Color::GRAY,
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(5.0)),
                align_self: AlignSelf::Center,
                ..default()
            }),
        );
    });
}

fn restart_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut score: ResMut<Score>,
    mut last_score: ResMut<LastScore>,
    bricks: Query<Entity, With<Brick>>,
    paddle: Query<Entity, With<Paddle>>,
    ball: Query<Entity, With<Ball>>,
    messages: Query<Entity, With<GameMessage>>,
) {
    if *game_state == GameState::Running || !keyboard_input.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in bricks.iter().chain(paddle.iter()).chain(ball.iter()) {
        commands.entity(entity).despawn();
    }

    for entity in messages.iter() {
        commands.entity(entity).despawn_recursive();
    }

    last_score.0 = Some(score.0);
    score.0 = 0;
    *game_state = GameState::Running;

    setup(commands);
}

fn collide(ball_pos: Vec3, ball_size: f32, other_pos: Vec3, other_w: f32, other_h: f32) -> bool {
    let dx = (ball_pos.x - other_pos.x).abs();
    let dy = (ball_pos.y - other_pos.y).abs();
    dx < (ball_size / 2.0 + other_w / 2.0) && dy < (ball_size / 2.0 + other_h / 2.0)
}
