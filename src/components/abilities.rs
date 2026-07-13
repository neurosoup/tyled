use bevy::prelude::*;

/// The ordered list of beam-behavior/economy abilities a player has drafted.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub struct AbilityList(pub Vec<AbilityDescriptor>);

/// A single draftable ability. Kept as pure, `Reflect`-serializable data (no
/// `Entity` or runtime handles) so a future loadout can be authored in RON,
/// hot-reloaded via `file_watcher`, and persisted across sessions.
#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
pub enum AbilityDescriptor {
    Backfill,
}

/// Hardcoded per-player starting loadouts (Stage F2 — no draft UI yet).
///
/// The single place to assign or swap P1/P2 kits between runs, the substrate
/// the balancing testing protocol rides on: an empty list is Straight-only
/// (the layer-1 control), a `Backfill` entry reproduces today's contextual
/// inverted mode. Read by `initialize_players` when attaching each player's
/// [`AbilityList`].
#[derive(Resource, Clone)]
pub struct PlayerLoadouts {
    pub player1: Vec<AbilityDescriptor>,
    pub player2: Vec<AbilityDescriptor>,
}

impl PlayerLoadouts {
    /// The drafted abilities for a given `player_id` (0 = P1, 1 = P2).
    pub fn for_player(&self, player_id: u8) -> Vec<AbilityDescriptor> {
        match player_id {
            0 => self.player1.clone(),
            _ => self.player2.clone(),
        }
    }
}
