/*
 * Owns the round-level game state. Currently this is the round countdown timer:
 * a global, player-agnostic count from 120 seconds down to 0. The `Countdown`
 * resource holds the authoritative remaining-seconds value; the HUD plugin only
 * reads it to drive the on-screen digits. Reaching 0 has no consumer yet.
 */
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (start_countdown, tick_countdown));
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
