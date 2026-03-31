use crate::prelude::*;
use bevy::prelude::*;

pub const CLAIMED_TILE_Z_INDEX: i8 = 2;

#[derive(Component, Debug)]
pub struct ClaimedTile {
    pub owner: Option<Entity>,
}
