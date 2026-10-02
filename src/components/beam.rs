use crate::prelude::*;
use bevy::prelude::*;

/// `owner` is the beneficiary of claims/damage/trail colour and flips on a landed parry;
/// `caster` is the original shooter and never changes.
#[derive(Component)]
pub struct Beam {
    pub owner: Entity,
    pub caster: Entity,
    pub direction: GridCoords,
    /// Rally depth; derives the current speed.
    pub parries: u32,
    pub behavior: BeamBehavior,
    /// Per-beam step cadence.
    pub step_timer: Timer,
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
