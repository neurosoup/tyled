/*
 * The round-start intro: the "3 · 2 · 1 · GO!" banner shown during
 * `RoundPhase::Starting`, drawn with the `text` plugin's `spawn_label` onto the
 * overlay camera (owned by the `camera` plugin) via the round's shared
 * `spawn_round_label` helper. Presentation only — the round state it reacts to
 * lives in the sibling `state` submodule.
 */
use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, lens::TransformScaleLens};

use super::spawn_round_label;
use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(RoundPhase::Starting), begin_intro_countdown);
    app.add_systems(
        Update,
        (
            advance_intro_countdown.run_if(in_state(RoundPhase::Starting)),
            // UNGATED on purpose: this fires while the state is already `Playing`.
            despawn_go_banner,
        ),
    );
}

/// The intro number currently on screen ("3"/"2"/"1"); swapped each step.
#[derive(Component)]
struct CountdownNumber;

/// The "GO!" banner. Scales up, then despawns on tween completion.
#[derive(Component)]
struct GoBanner;

/// Drives the intro number sequence: how long the current number has shown and
/// which number is next.
#[derive(Resource)]
struct IntroCountdown {
    timer: Timer,
    /// Number currently shown; counts 3 → 2 → 1, then 0 means "fire GO!".
    remaining: u8,
}

/// On entering `Starting`, show "3" immediately and start the step timer.
fn begin_intro_countdown(mut commands: Commands, font: Res<FontAtlas>, config: Res<GameConfig>) {
    let label = spawn_round_label(&mut commands, &font, &config.round.intro_count.to_string());
    commands.entity(label).insert(CountdownNumber);

    commands.insert_resource(IntroCountdown {
        // One number per second, so the intro lasts `intro_count` seconds.
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        remaining: config.round.intro_count,
    });
}

/// Each step: retire the current number and either show the next one, or — once
/// the numbers are exhausted — fire "GO!" and hand control to gameplay.
fn advance_intro_countdown(
    mut commands: Commands,
    time: Res<Time>,
    font: Res<FontAtlas>,
    config: Res<GameConfig>,
    mut intro: ResMut<IntroCountdown>,
    numbers: Query<Entity, With<CountdownNumber>>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    intro.timer.tick(time.delta());
    if !intro.timer.is_finished() {
        return;
    }

    // The current number's time is up.
    for entity in &numbers {
        commands.entity(entity).despawn();
    }
    intro.remaining = intro.remaining.saturating_sub(1);

    if intro.remaining > 0 {
        // Show the next number ("2", then "1").
        let label = spawn_round_label(&mut commands, &font, &intro.remaining.to_string());
        commands.entity(label).insert(CountdownNumber);
        return;
    }

    // Countdown finished: launch "GO!" and unfreeze gameplay in the same run. The
    // banner keeps animating into `Playing`; `despawn_go_banner` reaps it once it
    // has rushed off-screen. `ExponentialIn` holds it small then explodes it
    // outward, so it reads as charging toward the players and flying past.
    let go = spawn_round_label(&mut commands, &font, "GO!");
    commands.entity(go).insert((
        GoBanner,
        TweenAnim::new(Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_millis(config.round.go_linger_ms),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::splat(config.round.go_end_scale),
            },
        )),
    ));
    commands.remove_resource::<IntroCountdown>();
    next_phase.set(RoundPhase::Playing);
}

/// Despawns the "GO!" banner (and its glyph children) once its scale-up tween
/// completes. Ungated — by the time the tween finishes, the round is `Playing`.
fn despawn_go_banner(
    mut commands: Commands,
    mut completed: MessageReader<AnimCompletedEvent>,
    go_banners: Query<Entity, With<GoBanner>>,
) {
    for message in completed.read() {
        if let Ok(entity) = go_banners.get(message.anim_entity) {
            commands.entity(entity).despawn();
        }
    }
}
