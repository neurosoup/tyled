use bevy::prelude::*;
use serde::Deserialize;

/// The ordered list of beam-behavior/economy abilities a player has drafted.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub struct AbilityList(pub Vec<AbilityDescriptor>);

/// A single draftable ability.
#[derive(Reflect, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum AbilityDescriptor {
    Lance,
    BorderGrinder,
    SolarPanels,
}

impl AbilityDescriptor {
    /// Finds an ability by name, ignoring case, `_` and `-` (for command-line flags).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace(['_', '-'], "").as_str() {
            "lance" => Some(Self::Lance),
            "bordergrinder" => Some(Self::BorderGrinder),
            "solarpanels" => Some(Self::SolarPanels),
            _ => None,
        }
    }
}

/// Hardcoded per-player starting loadouts.
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
