use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Beam {
    pub owner: Entity,
    pub direction: GridCoords,
    pub head: GridCoords,
    pub speed: f32,
}
