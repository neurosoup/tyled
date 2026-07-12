use bevy::prelude::*;

/// The ordered list of beam-behavior/economy abilities a player has drafted.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub struct AbilityList(pub Vec<AbilityDescriptor>);

/// A single draftable ability. Kept as pure, `Reflect`-serializable data (no
/// `Entity` or runtime handles) so a future loadout can be authored in RON,
/// hot-reloaded via `file_watcher`, and persisted across sessions.
#[derive(Reflect, Clone, Debug)]
pub enum AbilityDescriptor {
    #[allow(dead_code)] // attached to players + mapped to BeamBehavior in Stage F2
    Backfill,
}
