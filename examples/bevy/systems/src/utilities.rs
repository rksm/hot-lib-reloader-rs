use bevy::prelude::*;

pub(crate) fn is_outside_bounds(point: Vec2, bounds: (f32, f32, f32, f32)) -> bool {
    let (left, top, right, bottom) = bounds;
    point.x < left || point.x > right || point.y < bottom || point.y > top
}
