/*
 * Round-level game state: the `RoundPhase` state machine and the round countdown
 * timer.
 *
 * `RoundPhase` is the project's state machine for a single round:
 *   Loading  → waiting for the level map to be created
 *   Starting → the "3 · 2 · 1 · GO!" intro countdown (gameplay frozen)
 *   Playing  → live gameplay
 *   Outcome  → the round is over (win banner) [not yet wired]
 * Live-gameplay systems across the input/movement/beam/damage plugins gate on
 * `in_state(RoundPhase::Playing)`, so the intro and outcome screens freeze the
 * world for free. The intro countdown itself is rendered by the sibling `intro`
 * submodule.
 *
 * The `Countdown` resource is a global, player-agnostic count from
 * `Countdown::START_SECONDS` down to 0. It is inserted on `MapCreated` (so the
 * HUD shows the starting value during the intro) but only ticks down in
 * `Playing`. Reaching 0 has no consumer yet.
 */
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::prelude::*;

/// The lifecycle of a single round. This is the project's first Bevy `States`
/// type; gameplay systems gate on `Playing` so non-play phases freeze the world.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RoundPhase {
    /// Waiting for the level map to be created and players/tiles initialized.
    #[default]
    Loading,
    /// The "3 · 2 · 1 · GO!" intro countdown. Gameplay is frozen.
    Starting,
    /// Live gameplay.
    Playing,
    /// The round has ended (win banner). Not yet wired.
    Outcome,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<RoundPhase>();
    app.add_systems(
        Update,
        (
            start_countdown,
            start_round_on_map_created.run_if(in_state(RoundPhase::Loading)),
            tick_countdown.run_if(in_state(RoundPhase::Playing)),
        ),
    );
}

/// Enters `Starting` once the current level's map is created. The
/// `in_state(Loading)` guard makes this fire exactly once and ignore the HUD
/// map's `MapCreated` message.
fn start_round_on_map_created(
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    current_level_query: Query<(), (With<TiledMap>, With<CurrentLevel>)>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    for map_created_message in map_created_reader.read() {
        if current_level_query.get(map_created_message.origin).is_ok() {
            next_phase.set(RoundPhase::Starting);
            return;
        }
    }
}

/// Global round countdown, in whole seconds. Not tied to any player.
#[derive(Resource)]
pub struct Countdown {
    pub remaining: u32,
    timer: Timer,
}

impl Countdown {
    const START_SECONDS: u32 = 180;

    fn new() -> Self {
        Self {
            remaining: Self::START_SECONDS,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

/// (Re)starts the countdown at full duration whenever a map is created, so the
/// display visibly begins at the starting value once the HUD digits are ready.
fn start_countdown(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
) {
    for _ in map_created_reader.read() {
        commands.insert_resource(Countdown::new());
    }
}

/// Decrements `remaining` by one every second until it hits zero, then holds.
fn tick_countdown(time: Res<Time>, countdown: Option<ResMut<Countdown>>) {
    let Some(mut countdown) = countdown else {
        return;
    };

    if countdown.remaining == 0 {
        return;
    }

    countdown.timer.tick(time.delta());
    if !countdown.timer.is_finished() {
        return;
    }

    countdown.remaining -= 1;
}
