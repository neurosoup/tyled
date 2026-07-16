use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Player {
    pub player_id: u8,
}

/// The grid position a player was created at, captured once in
/// `initialize_players`. The round reset restores players here, since the Tiled
/// transform has since been overwritten by movement.
#[derive(Component)]
pub struct SpawnPoint(pub GridCoords);

/// The tile a character occupied before its most recent move.
#[derive(Component, Clone, Copy)]
pub struct PreviousGridCoords(pub GridCoords);
