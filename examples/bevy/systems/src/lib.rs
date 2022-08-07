mod utilities;

use bevy::{prelude::*, sprite::collide_aabb};
use components::*;
use rand::{thread_rng, Rng};

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // player
    let ship_handle = asset_server.load("textures/simplespace/ship_C.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: ship_handle,
            ..default()
        })
        .insert(Player {
            velocity: Vec3::ZERO,
            rotation_speed: f32::to_radians(180.0),
            shooting_timer: None,
        });
}

#[no_mangle]
pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
    time: Res<Time>,
) {
    const SPEED: f32 = 300.0;

    let (ship, mut transform) = query.single_mut();

    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement_factor += 1.0;
    }

    // update the ship rotation around the Z axis (perpendicular to the 2D plane of the screen)
    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_seconds());

    // get the ship's forward vector by applying the current rotation to the ships initial facing vector
    let movement_direction = transform.rotation * Vec3::Y;
    // get the distance the ship will move based on direction, the ship's movement speed and delta time
    let movement_distance = movement_factor * SPEED * time.delta_seconds();
    // create the change in translation using the new movement direction and distance
    let translation_delta = movement_direction * movement_distance;
    // update the ship translation with our new translation delta
    transform.translation += translation_delta;

    // bound the ship within the invisible level bounds
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

#[no_mangle]
pub fn player_shooting_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    const SIZE: f32 = 10.0;

    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(tfm) = query.get_single() {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: *tfm,
                    sprite: Sprite {
                        color: Color::rgb(0.9, 0.8, 0.0),
                        custom_size: Some(Vec2::new(SIZE, SIZE)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Bullet);
        }
    }
}

#[no_mangle]
pub fn bullet_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Bullet>>,
    cam: Query<&Camera>,
    time: Res<Time>,
) {
    let screen_size = cam.single().logical_viewport_size().unwrap() * 0.5;
    let speed = 800.0;
    for (entity, mut tfm) in &mut query {
        let x = tfm
            .rotation
            .mul_vec3(Vec3::new(0.0, speed * time.delta_seconds(), 0.0));
        tfm.translation += x;

        if utilities::is_outside_bounds(
            tfm.translation.truncate(),
            (
                (-screen_size.x),
                screen_size.y,
                screen_size.x,
                (-screen_size.y),
            ),
        ) {
            log::info!("pufff");
            commands.entity(entity).despawn();
        }
    }
}

#[no_mangle]
pub fn bullet_hit_system(
    mut commands: Commands,
    bullet_query: Query<&Transform, With<Bullet>>,
    ship_query: Query<(Entity, &Transform), With<OtherShip>>,
) {
    for bullet_tfm in bullet_query.iter() {
        for (entity, ship_tfm) in ship_query.iter() {
            if collide_aabb::collide(
                bullet_tfm.translation,
                Vec2::new(10.0, 10.0),
                ship_tfm.translation,
                Vec2::new(30.0, 30.0),
            )
            .is_some()
            {
                log::info!("BUUMMMM");
                commands.entity(entity).despawn();
            }
        }
    }
}

#[no_mangle]
pub fn spawn_other_ships(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    others: Query<(Entity, &Transform), With<OtherShip>>,
    cam: Query<&Camera>,
) {
    const MARGIN: f32 = 30.0;
    const MIN_SHIP_COUNT: usize = 10;

    let screen_size = cam.single().logical_viewport_size().unwrap() * 0.5;
    let mut other_ships_count = 0;

    for (entity, tfm) in others.iter() {
        if utilities::is_outside_bounds(
            tfm.translation.truncate(),
            (
                (-screen_size.x) - MARGIN,
                screen_size.y + MARGIN,
                screen_size.x + MARGIN,
                (-screen_size.y) - MARGIN,
            ),
        ) {
            commands.entity(entity).despawn();
        } else {
            other_ships_count += 1;
        }
    }

    if other_ships_count < MIN_SHIP_COUNT {
        let x = if thread_rng().gen::<bool>() {
            thread_rng().gen_range(((-screen_size.x) - MARGIN)..(-screen_size.x))
        } else {
            thread_rng().gen_range(screen_size.x..(screen_size.x + MARGIN))
        };
        let y = if thread_rng().gen::<bool>() {
            thread_rng().gen_range(((-screen_size.y) - MARGIN)..(-screen_size.y))
        } else {
            thread_rng().gen_range(screen_size.y..(screen_size.y + MARGIN))
        };
        let dir = thread_rng().gen_range(0.0f32..360.0f32);
        let mut transform = Transform::from_xyz(x, y, 0.0);
        transform.rotate_z(dir.to_radians());

        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/simplespace/enemy_A.png"),
                transform,
                ..default()
            })
            .insert(OtherShip);
    }
}

#[no_mangle]
pub fn move_other_ships(time: Res<Time>, mut query: Query<&mut Transform, With<OtherShip>>) {
    const SPEED: f32 = 100.0;
    for mut tfm in &mut query {
        let x = tfm
            .rotation
            .mul_vec3(Vec3::new(0.0, SPEED * time.delta_seconds(), 0.0));

        tfm.translation += x;
    }
}
