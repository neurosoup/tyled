use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Beam {
    pub owner: Entity,
    pub direction: GridCoords,
    pub speed: f32,
    pub behavior: BeamBehavior,
}

/// The resolved per-beam execution mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BeamBehavior {
    #[default]
    Straight,
    Backfill,
}
