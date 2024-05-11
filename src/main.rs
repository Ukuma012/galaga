use bevy::{ecs::query, prelude::*};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_game)
        .add_systems(FixedUpdate, (move_player).chain())
        .run();
}

fn setup_game(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Player,
        Collider,
    ));
}

const TIME_STEP: f32 = 1.0 / 60.0;
const PLAYER_SPEED: f32 = 100.0;

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        println!("[KEYBOARD] Pressed left");
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        println!("[KEYBOARD] Pressed right");
        direction += 1.0;
    }

    let new_paddle_position = paddle_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

    paddle_transform.translation.x = new_paddle_position;
}
