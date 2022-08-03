use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Player;

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn()
        .insert_bundle(bevy::prelude::SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                color: Color::RED,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player::default());
}

#[no_mangle]
pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Right) {
        transform.translation.x += 10.0;
    }

    if keyboard_input.pressed(KeyCode::Left) {
        transform.translation.x -= 10.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        transform.translation.y += 10.0;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        transform.translation.y -= 10.0;
    }
}
