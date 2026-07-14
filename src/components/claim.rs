use crate::prelude::*;
use bevy::prelude::*;

pub const CLAIMED_TILE_Z_INDEX: i8 = 2;

#[derive(Component, Debug)]
pub struct ClaimedTile {
    pub owner: Option<Entity>,
}

/// Authoritative per-player count of currently owned tiles. Code-inserted on
/// each player entity; maintained by the claim plugin as tile ownership flips.
#[derive(Component, Default)]
pub struct ClaimedTileCount {
    pub current: u32,
}

/// Marker attached in Tiled to each HUD digit object that displays a player's
/// claimed-tile percentage (counterpart to `BeamChargesDigit`).
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ClaimedTilesDigit;
