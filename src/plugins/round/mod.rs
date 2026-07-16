/*
 * The round feature: everything scoped to a single round of the match, split by
 * role into two submodules:
 *   `state` — the `RoundPhase` state machine (Loading/Starting/Playing/Outcome),
 *             the `Countdown` timer, and the phase transitions. Gameplay systems
 *             elsewhere gate on `in_state(RoundPhase::Playing)`.
 *   `intro` — the round-start "3 · 2 · 1 · GO!" banner shown during `Starting`.
 *             A future win banner for `Outcome` would join as a sibling submodule.
 *
 * Round-phase banners are drawn with the `text` plugin's `spawn_label` onto the
 * overlay camera (owned by the `camera` plugin); `spawn_round_label` centres a
 * label on that camera for any round submodule to use.
 */
mod intro;
mod state;

pub use state::*;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    state::plugin(app);
    intro::plugin(app);
}

/// Spawns a banner label centred on the overlay camera (origin + overlay layer),
/// shared by the round's presentation submodules.
fn spawn_round_label(commands: &mut Commands, font: &FontAtlas, text: &str) -> Entity {
    spawn_label(
        commands,
        font,
        text,
        Transform::default(),
        RenderLayers::layer(OVERLAY_RENDER_LAYER),
    )
}
