/*
 * Single place declaring the cross-plugin gameplay pipeline order.
 * Plugins tag their own systems into these sets locally via `.in_set(...)` rather than naming
 * another plugin's system directly.
 */
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    BotThink,
    Input,
    Movement,
    Beam,
    Claim,
    Economy,
    Damage,
    /// Applies knockback after damage and before presentation.
    Displacement,
    Presentation,
    RoundResolution,
    HudSync,
}

pub(crate) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            GameplaySet::BotThink,
            GameplaySet::Input,
            GameplaySet::Movement,
            GameplaySet::Beam,
            GameplaySet::Claim,
            GameplaySet::Economy,
            GameplaySet::Damage,
            GameplaySet::Displacement,
            GameplaySet::Presentation,
            GameplaySet::RoundResolution,
            GameplaySet::HudSync,
        )
            .chain(),
    );
}

/// Marks the systems that write `NextState<RoundPhase>`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoundPhaseWriter;
