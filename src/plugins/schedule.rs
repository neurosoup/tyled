/*
 * Single place declaring the cross-plugin gameplay pipeline order: bot decisions feed input,
 * input feeds movement, movement feeds the beam step, beams feed tile claims, claims feed the
 * charge economy, economy feeds damage, and damage feeds HUD sync. Plugins tag their own systems
 * into these sets locally via `.in_set(...)` rather than naming another plugin's system directly.
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
            GameplaySet::HudSync,
        )
            .chain(),
    );
}
