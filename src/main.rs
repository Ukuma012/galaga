use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Projectile;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Resource)]
struct ProjectileTimer(Timer);

const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

const PROJECTILE_SIZE: Vec3 = Vec3::splat(3.0);
const PROJECTILE_TIME_LIMIT: f32 = 0.1;
const PROJECTILE_COLOR: Color = Color::rgb(1.0, 1.0, 0.0);
const PROJECTILE_STARTING_POSITION: Vec3 = Vec3::new(0.0, 50.0, 1.0);
const PLAYER_PROJECTILE_DIRECTION: Vec2 = Vec2::new(0.5, 0.5);
const PROJECTILE_SPEED: f32 = 400.0;

#[derive(Debug, PartialEq, Eq)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
    Inside,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ProjectileTimer(Timer::from_seconds(
            PROJECTILE_TIME_LIMIT,
            TimerMode::Once,
        )))
        .add_systems(Startup, setup_game)
        .add_systems(
            FixedUpdate,
            (
                move_player,
                shoot_projectile,
                move_projectiles,
                destroy_projectiles,
                check_for_collisions,
            )
                .chain(),
        )
        .run();
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -250.0, 0.0),
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

    // Spawn enemies
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(ColorMaterial::from(PROJECTILE_COLOR)),
            transform: Transform::from_translation(PROJECTILE_STARTING_POSITION)
                .with_scale(PROJECTILE_SIZE * Vec3::new(4.0, 4.0, 4.0)),
            ..default()
        },
        Enemy,
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
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    let new_paddle_position = paddle_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

    paddle_transform.translation.x = new_paddle_position;
}

fn shoot_projectile(
    time: Res<Time>,
    mut projectile_timer: ResMut<ProjectileTimer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let player_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Space) {
        if projectile_timer.0.tick(time.delta()).finished() {
            // Reset the timer
            projectile_timer.0.reset();

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::default()).into(),
                    material: materials.add(ColorMaterial::from(PROJECTILE_COLOR)),
                    transform: Transform::from_translation(player_transform.translation)
                        .with_scale(PROJECTILE_SIZE),
                    ..default()
                },
                Projectile,
                Velocity(PLAYER_PROJECTILE_DIRECTION.normalize() * PROJECTILE_SPEED),
            ));
        }
    }
}

fn move_projectiles(mut query: Query<&mut Transform, With<Projectile>>) {
    for mut collider_transform in &mut query {
        let new_projectile_position = collider_transform.translation.y + 250.0 * TIME_STEP;
        collider_transform.translation.y = new_projectile_position;
    }
}

fn destroy_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Projectile>>,
) {
    for (collider_entity, collider_transform) in &query {
        if collider_transform.translation.y > 350.0 {
            commands.entity(collider_entity).despawn();
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    projectiles_query: Query<(Entity, &Transform), With<Projectile>>,
    collider_query: Query<(Entity, &Transform, Option<&Enemy>), With<Collider>>,
) {
    // Loop through all the projectiles on screen
    for (projectile_entity, projectile_transform) in &projectiles_query {
        // Loop through all collidable elements on the screen
        // TODO: Figure out how to flatten this - 2 for loops no bueno
        for (collider_entity, collider_transform, enemy_check) in &collider_query {
            let collision = projectiles_collision(
                projectile_transform.translation,
                projectile_transform.scale.truncate(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            );

            if let Some(collision) = collision {
                // If it's an enemy, destroy!
                if enemy_check.is_some() {
                    println!("Collided!");

                    // Enemy is destroyed
                    commands.entity(collider_entity).despawn();

                    // Projectile disappears too? Prevents "cutting through" a line of enemies all at once
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}

fn projectiles_collision(
    a_pos: Vec3,
    a_size: Vec2,
    b_pos: Vec3,
    b_size: Vec2,
) -> Option<Collision> {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth) = if a_min.x < b_min.x && a_max.x > b_min.x && a_max.x < b_max.x
        {
            (Collision::Left, b_min.x - a_max.x)
        } else if a_min.x > b_min.x && a_min.x < b_max.x && a_max.x > b_max.x {
            (Collision::Right, a_min.x - b_max.x)
        } else {
            (Collision::Inside, -f32::INFINITY)
        };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth) = if a_min.y < b_min.y && a_max.y > b_min.y && a_max.y < b_max.y
        {
            (Collision::Bottom, b_min.y - a_max.y)
        } else if a_min.y > b_min.y && a_min.y < b_max.y && a_max.y > b_max.y {
            (Collision::Top, a_min.y - b_max.y)
        } else {
            (Collision::Inside, -f32::INFINITY)
        };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        if y_depth.abs() < x_depth.abs() {
            Some(y_collision)
        } else {
            Some(x_collision)
        }
    } else {
        None
    }
}
