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
 * The `Countdown` resource is a global, player-agnostic count from the configured
 * round length down to 0. It is inserted on `MapCreated` (so the
 * HUD shows the starting value during the intro), only ticks down in `Playing`,
 * and reaching 0 triggers the timeout branch of round resolution.
 *
 * Round resolution (the two-vector win model): `resolve_kill` ends the round the
 * instant a player dies (highest priority); `resolve_timeout` ends it when the
 * countdown hits 0; `resolve_charge_exhaustion` ends it when every player has
 * spent their last charge and no beam is still in flight — both backstops
 * resolve by tile count → HP → seat. Either sets
 * `RoundResult`, bumps `MatchScore`, and enters `Outcome`. Leaving `Outcome`
 * runs `reset_round`, an in-place full wipe of board + charges + health +
 * positions (honouring the `RoundResetExceptions` carve-out) before the loop
 * returns to `Starting`.
 */
use bevy::platform::collections::HashMap;
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
    /// The round has ended; the `outcome` submodule shows the win banner, then
    /// loops back to `Starting` after a full reset (`reset_round`).
    Outcome,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<RoundPhase>();
    app.init_resource::<RoundResult>();
    app.init_resource::<MatchScore>();
    app.init_resource::<RoundResetExceptions>();
    app.add_systems(
        Update,
        (
            start_countdown,
            start_round_on_map_created.run_if(in_state(RoundPhase::Loading)),
            (
                tick_countdown,
                resolve_kill,
                resolve_timeout,
                resolve_charge_exhaustion,
            )
                .run_if(in_state(RoundPhase::Playing)),
        ),
    );
    // A full board + charge reset runs on leaving `Outcome`, so the very first
    // `Loading → Starting` (fresh board) is never touched by it.
    app.add_systems(OnExit(RoundPhase::Outcome), reset_round);
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
    fn new(start_secs: u32) -> Self {
        Self {
            remaining: start_secs,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

/// (Re)starts the countdown at full duration whenever a map is created, so the
/// display visibly begins at the starting value once the HUD digits are ready.
fn start_countdown(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
) {
    for _ in map_created_reader.read() {
        commands.insert_resource(Countdown::new(config.round.round_duration_secs));
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

/// The winner of the most recently resolved round. `None` is a draw (only a
/// defensive fallback — every resolution path breaks ties down to a single
/// seat). Read by the `outcome` submodule to label the win banner.
#[derive(Resource, Default)]
pub struct RoundResult {
    pub winner: Option<u8>,
}

/// Running per-player tally of rounds won across the match. Accumulates for the
/// life of the match: a round boundary is a full board reset, but the score
/// persists.
#[derive(Resource, Default)]
pub struct MatchScore {
    pub wins: [u32; 2],
}

/// Tiles that keep their owner across the round reset instead of reverting to
/// unclaimed — a generic carve-out hook for future burst-claim abilities. Empty
/// today; `reset_round` already applies it so those abilities need no
/// reset-code retrofit when they land.
#[derive(Resource, Default)]
pub struct RoundResetExceptions(pub HashMap<GridCoords, Entity>);

/// Records `winner` and credits the match score, then enters `Outcome`.
fn conclude_round(
    winner: Option<u8>,
    result: &mut RoundResult,
    score: &mut MatchScore,
    next_phase: &mut NextState<RoundPhase>,
) {
    result.winner = winner;
    if let Some(id) = winner
        && let Some(slot) = score.wins.get_mut(id as usize)
    {
        *slot += 1;
    }
    next_phase.set(RoundPhase::Outcome);
}

/// Kill vector (highest priority): the instant a player's HP hits zero the round
/// ends. The surviving player wins; a same-frame mutual kill is broken by tile
/// count, then seat (lower `player_id`).
fn resolve_kill(
    mut died_reader: MessageReader<DamageableDied>,
    players: Query<(Entity, &Player, &ClaimedTileCount)>,
    mut result: ResMut<RoundResult>,
    mut score: ResMut<MatchScore>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    let dead: Vec<Entity> = died_reader.read().map(|message| message.entity).collect();
    if dead.is_empty() {
        return;
    }

    let survivors: Vec<&Player> = players
        .iter()
        .filter(|(entity, ..)| !dead.contains(entity))
        .map(|(_, player, _)| player)
        .collect();

    let winner = match survivors.as_slice() {
        // A single survivor takes the round outright.
        [player] => Some(player.player_id),
        // No survivor (mutual death): break by tile count, then seat.
        [] => players
            .iter()
            .max_by(|(_, a_player, a_count), (_, b_player, b_count)| {
                a_count
                    .current
                    .cmp(&b_count.current)
                    .then(b_player.player_id.cmp(&a_player.player_id))
            })
            .map(|(_, player, _)| player.player_id),
        // Two+ survivors means no relevant player actually died: ignore.
        _ => return,
    };

    conclude_round(winner, &mut result, &mut score, &mut next_phase);
}

/// Ranks players by tile count, then HP, then seat (lower `player_id`) and
/// returns the leader's `player_id`. The shared tiebreak used by both backstop
/// vectors (timeout and charge exhaustion).
fn winner_by_standing<'a>(
    players: impl Iterator<Item = (&'a Player, &'a ClaimedTileCount, &'a Health)>,
) -> Option<u8> {
    players
        .max_by(|(a_player, a_count, a_health), (b_player, b_count, b_health)| {
            a_count
                .current
                .cmp(&b_count.current)
                .then(a_health.current.total_cmp(&b_health.current))
                .then(b_player.player_id.cmp(&a_player.player_id))
        })
        .map(|(player, ..)| player.player_id)
}

/// Timeout vector (backstop): when the countdown reaches zero the round ends,
/// resolved by tile count, then HP, then seat (lower `player_id`). A same-frame
/// kill preempts this — if any death fired this frame, defer to `resolve_kill`.
fn resolve_timeout(
    mut died_reader: MessageReader<DamageableDied>,
    countdown: Option<Res<Countdown>>,
    players: Query<(&Player, &ClaimedTileCount, &Health)>,
    mut result: ResMut<RoundResult>,
    mut score: ResMut<MatchScore>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    if died_reader.read().next().is_some() {
        return;
    }
    let Some(countdown) = countdown else {
        return;
    };
    if countdown.remaining > 0 {
        return;
    }

    let winner = winner_by_standing(players.iter());
    conclude_round(winner, &mut result, &mut score, &mut next_phase);
}

/// Charge-exhaustion vector (backstop): when every player has spent their last
/// charge and no beam is still in flight, neither side can act, so the round
/// ends immediately rather than idling until the timeout.
fn resolve_charge_exhaustion(
    mut died_reader: MessageReader<DamageableDied>,
    countdown: Option<Res<Countdown>>,
    beams: Query<(), With<Beam>>,
    players: Query<(&Player, &ClaimedTileCount, &Health, &BeamCharges)>,
    mut result: ResMut<RoundResult>,
    mut score: ResMut<MatchScore>,
    mut next_phase: ResMut<NextState<RoundPhase>>,
) {
    if died_reader.read().next().is_some() {
        return;
    }
    if countdown.is_some_and(|countdown| countdown.remaining == 0) {
        return;
    }
    if !beams.is_empty() {
        return;
    }

    let mut any_player = false;
    let all_exhausted = players.iter().all(|(.., charges)| {
        any_player = true;
        charges.is_empty()
    });
    if !any_player || !all_exhausted {
        return;
    }

    let winner = winner_by_standing(
        players
            .iter()
            .map(|(player, count, health, _)| (player, count, health)),
    );
    conclude_round(winner, &mut result, &mut score, &mut next_phase);
}

/// Full in-place round reset, run on leaving `Outcome`. Wipes tile ownership
/// (except `RoundResetExceptions`), restores every player to full HP/charges and
/// its spawn tile, revives the loser (removes `IsDead`, unhides it), clears any
/// in-flight beams, and restarts the countdown.
fn reset_round(
    mut commands: Commands,
    config: Res<GameConfig>,
    exceptions: Res<RoundResetExceptions>,
    mut players: Query<(
        Entity,
        &SpawnPoint,
        &mut Health,
        &mut BeamCharges,
        &mut ClaimedTileCount,
    )>,
    mut tiles: Query<(&GridCoords, &mut ClaimedTile)>,
    beams: Query<Entity, With<Beam>>,
) {
    // Reset tile ownership, keeping only the carve-out tiles.
    for (coords, mut tile) in &mut tiles {
        tile.owner = exceptions.0.get(coords).copied();
    }

    // Owned-tile count each player retains through the wipe (0 unless a carve-out
    // tile is theirs).
    let mut reserved_counts: HashMap<Entity, u32> = HashMap::new();
    for owner in exceptions.0.values() {
        *reserved_counts.entry(*owner).or_default() += 1;
    }

    for (entity, spawn, mut health, mut charges, mut count) in &mut players {
        health.current = health.max;
        charges.current = charges.max;
        count.current = reserved_counts.get(&entity).copied().unwrap_or(0);
        commands
            .entity(entity)
            .insert((spawn.0, PreviousGridCoords(spawn.0), Visibility::Visible))
            .remove::<(IsDead, IsTurning)>();
    }

    // Clear any beams still in flight when the round ended.
    for beam in &beams {
        commands.entity(beam).despawn();
    }

    // `start_countdown` is keyed on `MapCreated`, which does not re-fire in place,
    // so restart the timer here.
    commands.insert_resource(Countdown::new(config.round.round_duration_secs));
}
