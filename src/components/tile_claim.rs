use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Beam {
    pub owner: Entity,
    pub direction: GridCoords,
    pub head: GridCoords,
    pub speed: f32,
}

#[derive(Component)]
pub struct ClaimedTile {
    pub owner: Entity,
}
