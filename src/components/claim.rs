use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct ClaimedTile {
    pub owner: Entity,
}
