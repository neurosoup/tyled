/*
 * Global game-pace modulation: a brief slowdown on a landed parry, snapping down
 * fast then easing back to normal speed.
 */
use bevy::prelude::*;

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<HitStop>();
    app.add_systems(
        Update,
        trigger_hitstop.in_set(GameplaySet::Presentation),
    );
    app.add_systems(Update, recover_hitstop.after(trigger_hitstop));
    // A round ending mid-slowdown must not bleed into the next round's intro.
    app.add_systems(OnExit(RoundPhase::Outcome), force_restore_on_round_reset);
}

#[derive(Resource, Default)]
pub(crate) struct HitStop {
    /// Ticked on `Time<Real>`.
    timer: Timer,
    /// A copy of the factor, saved when the slowdown starts. Keeps the recovery
    /// stable even if the config value changes while it's running.
    factor: f32,
}

/// Arm the recovery countdown.
fn arm(hitstop: &mut HitStop, factor: f32, recovery_secs: f32) {
    hitstop.factor = factor;
    hitstop.timer = Timer::from_seconds(recovery_secs, TimerMode::Once);
}

/// Immediately restores normal speed and finishes the recovery timer.
fn force_restore_on_round_reset(mut time_virtual: ResMut<Time<Virtual>>, mut hitstop: ResMut<HitStop>) {
    time_virtual.set_relative_speed(1.0);
    hitstop.timer.finish();
}

// Reads `BeamParried` to trigger a hitstop.
fn trigger_hitstop(
    mut beam_parried_reader: MessageReader<BeamParried>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut hitstop: ResMut<HitStop>,
    config: Res<GameConfig>,
) {
    for _ in beam_parried_reader.read() {
        time_virtual.set_relative_speed(config.hitstop.factor);
        arm(&mut hitstop, config.hitstop.factor, config.hitstop.recovery_secs);
    }
}

// Bring the speed back to normal from a hitstop.
fn recover_hitstop(
    time_real: Res<Time<Real>>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut hitstop: ResMut<HitStop>,
) {
    if hitstop.timer.is_finished() {
        return;
    }
    hitstop.timer.tick(time_real.delta());
    let duration = hitstop.timer.duration().as_secs_f32();
    let progress = if duration > 0.0 {
        (hitstop.timer.elapsed_secs() / duration).min(1.0)
    } else {
        1.0
    };
    let eased = progress.powi(3);
    if hitstop.timer.is_finished() {
        time_virtual.set_relative_speed(1.0);
    } else {
        time_virtual.set_relative_speed(hitstop.factor + (1.0 - hitstop.factor) * eased);
    }
}
