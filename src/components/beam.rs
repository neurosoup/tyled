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
    Lance,
}

/// Authoritative per-player count of that player's beams currently in flight.
#[derive(Component, Default)]
pub struct InFlightBeamCount {
    pub current: u32,
}
