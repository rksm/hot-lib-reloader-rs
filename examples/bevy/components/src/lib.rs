use bevy::prelude::*;

pub const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

#[derive(Component)]
pub struct OtherShip;

#[derive(Component)]
pub struct Bullet;

/// player component
#[derive(Component)]
pub struct Player {
    // pub movement_speed: f32,
    pub velocity: Vec3,
    pub rotation_speed: f32,
    pub shooting_timer: Option<Timer>,
}
