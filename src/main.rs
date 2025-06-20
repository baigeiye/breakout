use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const PADDLE_WIDTH: f32 = 120.0;
const PADDLE_HEIGHT: f32 = 20.0;
const BALL_SIZE: f32 = 15.0;
const BRICK_WIDTH: f32 = 60.0;
const BRICK_HEIGHT: f32 = 30.0;

const BALL_SPEED: f32 = 300.0;
const PADDLE_SPEED: f32 = 500.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                title: "Breakout in Rust (Bevy 0.13)".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_paddle, move_ball, ball_collision))
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Component)]
struct Brick;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Paddle
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

    // Ball
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

    // Bricks
    for row in 0..2 {
        for col in 0..10 {
            let x = -WINDOW_WIDTH / 2.0 + BRICK_WIDTH / 2.0 + col as f32 * (BRICK_WIDTH + 5.0);
            let y = WINDOW_HEIGHT / 2.0 - BRICK_HEIGHT - row as f32 * (BRICK_HEIGHT + 5.0);
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        scale: Vec3::new(BRICK_WIDTH, BRICK_HEIGHT, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.2, 0.2),
                        ..default()
                    },
                    ..default()
                },
                Brick,
            ));
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Paddle>>,
) {
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

fn move_ball(time: Res<Time>, mut query: Query<(&mut Transform, &Ball)>) {
    let (mut transform, ball) = query.single_mut();
    let delta = ball.velocity * time.delta_seconds();
    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}

fn ball_collision(
    mut commands: Commands,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    paddle_query: Query<&Transform, With<Paddle>>,
    brick_query: Query<(Entity, &Transform), With<Brick>>,
) {
    let (mut ball, ball_transform) = ball_query.single_mut();
    let ball_pos = ball_transform.translation;

    // Bounce on walls
    if ball_pos.x - BALL_SIZE / 2.0 <= -WINDOW_WIDTH / 2.0
        || ball_pos.x + BALL_SIZE / 2.0 >= WINDOW_WIDTH / 2.0
    {
        ball.velocity.x = -ball.velocity.x;
    }

    if ball_pos.y + BALL_SIZE / 2.0 >= WINDOW_HEIGHT / 2.0 {
        ball.velocity.y = -ball.velocity.y;
    }

    // Bottom (game over)
    if ball_pos.y - BALL_SIZE / 2.0 <= -WINDOW_HEIGHT / 2.0 {
        println!("Game Over!");
        ball.velocity = Vec2::ZERO;
    }

    // Paddle collision
    let paddle_transform = paddle_query.single();
    if collide(ball_pos, BALL_SIZE, paddle_transform.translation, PADDLE_WIDTH, PADDLE_HEIGHT) {
        ball.velocity.y = ball.velocity.y.abs();
    }

    // Brick collision
    for (entity, brick_transform) in brick_query.iter() {
        if collide(ball_pos, BALL_SIZE, brick_transform.translation, BRICK_WIDTH, BRICK_HEIGHT) {
            ball.velocity.y = -ball.velocity.y;
            commands.entity(entity).despawn();
            break;
        }
    }
}

fn collide(ball_pos: Vec3, ball_size: f32, other_pos: Vec3, other_w: f32, other_h: f32) -> bool {
    let dx = (ball_pos.x - other_pos.x).abs();
    let dy = (ball_pos.y - other_pos.y).abs();
    dx < (ball_size / 2.0 + other_w / 2.0) && dy < (ball_size / 2.0 + other_h / 2.0)
}
