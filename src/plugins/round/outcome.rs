/*
 * The round-end outcome: the win banner shown during `RoundPhase::Outcome`,
 * drawn with the `text` plugin's `spawn_label` onto the overlay camera via the
 * round's shared `spawn_round_label` helper — the same presentation machinery as
 * the sibling `intro` submodule. Presentation only; the round resolution and
 * reset it reacts to live in the sibling `state` submodule (which sets
 * `RoundResult` and enters `Outcome`, and runs `reset_round` on the way out).
 *
 * The banner lingers for `OUTCOME_LINGER_SECS`, long enough for the loser's
 * death bounce (~0.9s) to finish, then loops the round back to `Starting`.
 */
use bevy::prelude::*;

use super::spawn_round_label;
use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(RoundPhase::Outcome), show_outcome_banner);
    app.add_systems(
        Update,
        advance_outcome.run_if(in_state(RoundPhase::Outcome)),
    );
    app.add_systems(OnExit(RoundPhase::Outcome), despawn_outcome_banner);
}

/// The win banner label; despawned when the round loops.
#[derive(Component)]
struct OutcomeBanner;

/// Counts down the banner's on-screen linger.
#[derive(Resource)]
struct OutcomeTimer(Timer);

/// On entering `Outcome`, show who won and start the linger timer.
fn show_outcome_banner(
    mut commands: Commands,
    font: Res<FontAtlas>,
    result: Res<RoundResult>,
    config: Res<GameConfig>,
) {
    let text = match result.winner {
        Some(0) => "P1 WINS",
        Some(_) => "P2 WINS",
        None => "DRAW",
    };
    let banner = spawn_round_label(&mut commands, &font, text);
    commands.entity(banner).insert(OutcomeBanner);

    commands.insert_resource(OutcomeTimer(Timer::from_seconds(
        config.round.outcome_linger_secs,
        TimerMode::Once,
    )));
}

/// Once the linger elapses, loop back to `Starting`. Leaving `Outcome` triggers
/// the state submodule's `reset_round`, then `Starting` re-runs the intro.
fn advance_outcome(
    time: Res<Time>,
    mut timer: ResMut<OutcomeTimer>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_phase.set(RoundPhase::Starting);
    }
}

/// Reaps the banner (and its glyph children) and the timer when the round loops.
fn despawn_outcome_banner(
    mut commands: Commands,
    banners: Query<Entity, With<OutcomeBanner>>,
) {
    for banner in &banners {
        commands.entity(banner).despawn();
    }
    commands.remove_resource::<OutcomeTimer>();
}
