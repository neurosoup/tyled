/*
 * Play-telemetry sink: records human/bot actions, game events and per-round
 * outcomes to a single gitignored JSONL file (`play_trace.jsonl`, one JSON object per line).
 *
 * At startup the previous session's traces are rotated into numbered backups
 * (`play_trace.1.jsonl`, `play_trace.2.jsonl`, …, bounded by `telemetry.history`)
 * before a fresh `play_trace.jsonl` is opened. It opens with a self-describing
 * `session` header line that anchors the clocks to the Unix epoch.
 *
 * Every record after the header carries two clocks, both in seconds since app launch:
 * `t` is game time (it slows down during hitstop), and `rt` is real time. Use `rt`
 * to place a record on a video timeline.
 *
 * Record kinds: `action`, `decision`, `event`, `outcome`. An `event` record has an
 * `event` name plus its own fields. All systems are gated behind the
 * `telemetry.enabled` config toggle.
 */
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::Serialize;

use crate::plugins::damage::is_hostile_tile;
use crate::plugins::maps::LEVEL_PATH;
use crate::prelude::*;

const TRACE_PATH: &str = "play_trace.jsonl";

/// Trace shape version. 2: clock anchor, `rt`, events. 3: parry `attacker`, `rt` from app start. 4: `bot_loop`.
const SCHEMA_VERSION: u32 = 4;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ActionSnapshots>();
    app.init_resource::<TelemetryRound>();
    app.add_systems(Startup, open_sink.run_if(telemetry_enabled));
    // Recorders run after all gameplay sets, so the last frame of a round is recorded.
    app.add_systems(
        Update,
        (
            record_actions,
            record_decisions,
            record_knockbacks,
            record_claims,
            record_hits,
            record_parries,
            record_abilities,
        )
            .after(GameplaySet::HudSync)
            .run_if(in_state(RoundPhase::Playing))
            .run_if(telemetry_enabled),
    );
    app.add_systems(
        Update,
        (record_charges_empty, record_bot_loops)
            .after(GameplaySet::HudSync)
            .run_if(telemetry_enabled),
    );
    app.add_systems(
        OnEnter(RoundPhase::Playing),
        record_round_start.run_if(telemetry_enabled),
    );
    app.add_systems(
        OnEnter(RoundPhase::Outcome),
        record_outcome.run_if(telemetry_enabled),
    );
    app.add_systems(Last, record_session_end.run_if(telemetry_enabled));
}

fn telemetry_enabled(config: Res<GameConfig>) -> bool {
    config.telemetry.enabled
}

#[derive(Resource)]
struct TelemetrySink {
    writer: BufWriter<File>,
}

impl TelemetrySink {
    fn write<T: Serialize>(&mut self, record: &T) {
        if let Ok(line) = serde_json::to_string(record) {
            let _ = writeln!(self.writer, "{line}");
        }
    }
}

/// Index of the current round, counted from 1. Zero before the first round starts.
#[derive(Resource, Default)]
struct TelemetryRound(u32);

/// Both clocks at one instant.
#[derive(Clone, Copy)]
struct Stamp {
    t: f32,
    rt: f32,
}

#[derive(SystemParam)]
struct Clock<'w> {
    time: Res<'w, Time>,
    real: Res<'w, Time<Real>>,
}

impl Clock<'_> {
    fn stamp(&self) -> Stamp {
        Stamp {
            t: self.time.elapsed_secs(),
            // Same zero as `rt_at_write`: `Time<Real>::elapsed` starts later, at the first frame.
            rt: self.real.startup().elapsed().as_secs_f32(),
        }
    }
}

/// The last action state written per player, so records are emitted on change
/// rather than every frame.
#[derive(Resource, Default)]
struct ActionSnapshots(HashMap<Entity, ActionSnapshot>);

#[derive(PartialEq, Clone)]
struct ActionSnapshot {
    x: i32,
    y: i32,
    facing: Option<String>,
    shoot: bool,
    charges_current: Option<u32>,
    on_hostile: bool,
}

#[derive(Serialize)]
struct SessionRecord {
    kind: &'static str,
    schema_version: u32,
    unix_ms: u64,
    t_at_write: f32,
    rt_at_write: f32,
    map: &'static str,
    telemetry_enabled: bool,
    bot_p1: bool,
    bot_p2: bool,
    loadout_p1: Vec<String>,
    loadout_p2: Vec<String>,
    rounds: u32,
    after_last_round: String,
}

#[derive(Serialize)]
struct ActionRecord {
    kind: &'static str,
    controller: &'static str,
    player_id: u8,
    t: f32,
    rt: f32,
    x: i32,
    y: i32,
    facing: Option<String>,
    shoot: bool,
    charges_current: Option<u32>,
    charges_max: Option<u32>,
    on_hostile: bool,
}

#[derive(Serialize)]
struct DecisionRecord {
    kind: &'static str,
    controller: &'static str,
    player_id: u8,
    t: f32,
    rt: f32,
    behaviour: String,
    why: String,
    move_x: f32,
    move_y: f32,
    shoot: bool,
}

/// One `event` record: the shared fields plus the fields of `data`.
#[derive(Serialize)]
struct EventRecord<T: Serialize> {
    kind: &'static str,
    event: &'static str,
    t: f32,
    rt: f32,
    #[serde(flatten)]
    data: T,
}

fn event<T: Serialize>(name: &'static str, stamp: Stamp, data: T) -> EventRecord<T> {
    EventRecord {
        kind: "event",
        event: name,
        t: stamp.t,
        rt: stamp.rt,
        data,
    }
}

#[derive(Serialize)]
struct RoundStartData {
    round: u32,
    players: Vec<RoundStartPlayer>,
}

#[derive(Serialize)]
struct RoundStartPlayer {
    player_id: u8,
    controller: &'static str,
    loadout: Vec<String>,
}

#[derive(Serialize)]
struct ClaimData {
    player_id: u8,
    tiles_claimed: u32,
    tile_pct_after: f32,
    x: i32,
    y: i32,
}

#[derive(Serialize)]
struct HitData {
    attacker: Option<u8>,
    victim: u8,
    damage: f32,
    hp_after: f32,
    cause: &'static str,
}

#[derive(Serialize)]
struct KnockbackData {
    victim: u8,
    tiles: u32,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
}

#[derive(Serialize)]
struct ParryData {
    defender: u8,
    attacker: Option<u8>,
    caster: Option<u8>,
    success: bool,
    parries: u32,
}

#[derive(Serialize)]
struct BotLoopData {
    player_id: Option<u8>,
    pattern: &'static str,
    start_rt: f32,
    end_rt: f32,
}

#[derive(Serialize)]
struct AbilityData {
    player_id: u8,
    ability: String,
}

#[derive(Serialize)]
struct ChargesEmptyData {
    player_id: u8,
}

#[derive(Serialize)]
struct SessionEndData {
    round: u32,
}

#[derive(Serialize)]
struct OutcomeRecord {
    kind: &'static str,
    t: f32,
    rt: f32,
    round: u32,
    winner: Option<u8>,
    reason: &'static str,
    round_duration_secs: u32,
    players: Vec<OutcomePlayer>,
}

#[derive(Serialize)]
struct OutcomePlayer {
    player_id: u8,
    controller: &'static str,
    tile_pct: f32,
    tile_count: u32,
    hp: f32,
    charges_spent: u32,
    loadout: Vec<String>,
}

fn controller_tag(bot: Option<&Bot>) -> &'static str {
    if bot.is_some() { "bot" } else { "human" }
}

fn loadout_names(descriptors: &[AbilityDescriptor]) -> Vec<String> {
    descriptors
        .iter()
        .map(|descriptor| format!("{descriptor:?}"))
        .collect()
}

fn player_id_of(players: &Query<&Player>, entity: Entity) -> Option<u8> {
    players.get(entity).ok().map(|player| player.player_id)
}

/// Merges claims of the same player into one burst: `(player_id, tiles, first_x, first_y)`,
/// in the order the players first appear.
fn group_bursts(claims: &[(u8, i32, i32)]) -> Vec<(u8, u32, i32, i32)> {
    let mut bursts: Vec<(u8, u32, i32, i32)> = Vec::new();
    for &(player_id, x, y) in claims {
        if let Some(burst) = bursts.iter_mut().find(|burst| burst.0 == player_id) {
            burst.1 += 1;
        } else {
            bursts.push((player_id, 1, x, y));
        }
    }
    bursts
}

/// Shifts the previous session's trace aside into `play_trace.N.jsonl` backups,
/// keeping `history` total trace files (current + rotated backups), so each run
/// starts clean without discarding recent sessions.
fn rotate_traces(history: u32) {
    if history <= 1 {
        return;
    }
    let backups = history - 1;
    let backup = |n: u32| format!("play_trace.{n}.jsonl");
    let _ = std::fs::remove_file(backup(backups));
    for n in (1..backups).rev() {
        let _ = std::fs::rename(backup(n), backup(n + 1));
    }
    let _ = std::fs::rename(TRACE_PATH, backup(1));
}

fn open_sink(
    mut commands: Commands,
    config: Res<GameConfig>,
    loadouts: Res<PlayerLoadouts>,
    clock: Clock,
) {
    rotate_traces(config.telemetry.history);

    let Ok(file) = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(TRACE_PATH)
    else {
        warn!("telemetry: could not open {TRACE_PATH}");
        return;
    };

    let mut sink = TelemetrySink {
        writer: BufWriter::new(file),
    };
    // Both clocks are read at the same instant. `rt_at_write` is the exact time since the app start.
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64);
    sink.write(&SessionRecord {
        kind: "session",
        schema_version: SCHEMA_VERSION,
        unix_ms,
        t_at_write: clock.time.elapsed_secs(),
        rt_at_write: clock.stamp().rt,
        map: LEVEL_PATH,
        telemetry_enabled: config.telemetry.enabled,
        bot_p1: config.controllers.is_bot(0),
        bot_p2: config.controllers.is_bot(1),
        loadout_p1: loadout_names(&loadouts.player1),
        loadout_p2: loadout_names(&loadouts.player2),
        rounds: config.launch.rounds,
        after_last_round: format!("{:?}", config.launch.after_last_round),
    });
    let _ = sink.writer.flush();
    commands.insert_resource(sink);
}

fn record_actions(
    clock: Clock,
    map_info: Res<MapInfo>,
    sink: Option<ResMut<TelemetrySink>>,
    mut snapshots: ResMut<ActionSnapshots>,
    claimed_query: Query<&ClaimedTile>,
    players: Query<(
        Entity,
        &Player,
        &GridCoords,
        &LookDirection,
        &ActionState<Action>,
        Option<&BeamCharges>,
        Option<&Bot>,
    )>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for (entity, player, coords, look, action_state, charges, bot) in &players {
        let facing = look.direction.map(|direction| format!("{direction:?}"));
        let shoot = action_state.pressed(&Action::Shoot);
        let on_hostile = is_hostile_tile(&map_info, &claimed_query, *coords, entity);

        let snapshot = ActionSnapshot {
            x: coords.x,
            y: coords.y,
            facing: facing.clone(),
            shoot,
            charges_current: charges.map(|c| c.current),
            on_hostile,
        };

        if snapshots.0.get(&entity) == Some(&snapshot) {
            continue;
        }
        snapshots.0.insert(entity, snapshot);

        sink.write(&ActionRecord {
            kind: "action",
            controller: controller_tag(bot),
            player_id: player.player_id,
            t: stamp.t,
            rt: stamp.rt,
            x: coords.x,
            y: coords.y,
            facing,
            shoot,
            charges_current: charges.map(|c| c.current),
            charges_max: charges.map(|c| c.max),
            on_hostile,
        });
    }
}

fn record_decisions(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    decisions: Query<(&Player, &BotDecision), (With<Bot>, Changed<BotDecision>)>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for (player, decision) in &decisions {
        sink.write(&DecisionRecord {
            kind: "decision",
            controller: "bot",
            player_id: player.player_id,
            t: stamp.t,
            rt: stamp.rt,
            behaviour: decision.behaviour.clone(),
            why: decision.why.clone(),
            move_x: decision.move_x,
            move_y: decision.move_y,
            shoot: decision.shoot,
        });
    }
}

fn record_round_start(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut round: ResMut<TelemetryRound>,
    players: Query<(&Player, &AbilityList, Option<&Bot>)>,
) {
    round.0 += 1;
    let Some(mut sink) = sink else {
        return;
    };

    let mut records: Vec<RoundStartPlayer> = players
        .iter()
        .map(|(player, abilities, bot)| RoundStartPlayer {
            player_id: player.player_id,
            controller: controller_tag(bot),
            loadout: loadout_names(&abilities.0),
        })
        .collect();
    records.sort_by_key(|player| player.player_id);

    sink.write(&event(
        "round_start",
        clock.stamp(),
        RoundStartData {
            round: round.0,
            players: records,
        },
    ));
    let _ = sink.writer.flush();
}

fn record_claims(
    clock: Clock,
    map_info: Res<MapInfo>,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<TileClaimed>,
    players: Query<(&Player, &ClaimedTileCount)>,
) {
    let Some(mut sink) = sink else {
        return;
    };

    let mut claims: Vec<(u8, i32, i32)> = Vec::new();
    for message in reader.read() {
        if message.old_owner == Some(message.new_owner) {
            continue;
        }
        if let Ok((player, _)) = players.get(message.new_owner) {
            claims.push((player.player_id, message.position.x, message.position.y));
        }
    }

    let total_tiles = map_info.ground_entities.len().max(1) as f32;
    let stamp = clock.stamp();
    for (player_id, tiles_claimed, x, y) in group_bursts(&claims) {
        let owned = players
            .iter()
            .find(|(player, _)| player.player_id == player_id)
            .map_or(0, |(_, count)| count.current);
        sink.write(&event(
            "claim",
            stamp,
            ClaimData {
                player_id,
                tiles_claimed,
                tile_pct_after: owned as f32 / total_tiles * 100.0,
                x,
                y,
            },
        ));
    }
}

fn record_hits(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<DamageDealt>,
    players: Query<&Player>,
    healths: Query<&Health>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for message in reader.read() {
        let Some(victim) = player_id_of(&players, message.entity) else {
            continue;
        };
        sink.write(&event(
            "hit",
            stamp,
            HitData {
                attacker: message.attacker.and_then(|a| player_id_of(&players, a)),
                victim,
                damage: message.amount,
                hp_after: healths
                    .get(message.entity)
                    .map_or(0.0, |health| health.current),
                cause: match message.source {
                    DamageSource::BeamContact => "beam",
                    DamageSource::Collision => "collision",
                    DamageSource::TileEntry | DamageSource::TileStanding => "hostile_tile",
                },
            },
        ));
    }
}

fn record_knockbacks(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<EntityKnockedBack>,
    players: Query<(&Player, &GridCoords)>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for message in reader.read() {
        let Ok((player, coords)) = players.get(message.entity) else {
            continue;
        };
        sink.write(&event(
            "knockback",
            stamp,
            KnockbackData {
                victim: player.player_id,
                tiles: 1,
                x: coords.x,
                y: coords.y,
                dx: message.direction.x,
                dy: message.direction.y,
            },
        ));
    }
}

fn record_parries(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<BeamParried>,
    players: Query<&Player>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for message in reader.read() {
        let Some(defender) = player_id_of(&players, message.parrier) else {
            continue;
        };
        sink.write(&event(
            "parry",
            stamp,
            ParryData {
                defender,
                attacker: player_id_of(&players, message.previous_owner),
                caster: player_id_of(&players, message.caster),
                success: true,
                parries: message.parries,
            },
        ));
    }
}

/// Runs in every phase to catch loops that end with the round.
fn record_bot_loops(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<BotLoopEnded>,
    players: Query<&Player>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for message in reader.read() {
        sink.write(&event(
            "bot_loop",
            stamp,
            BotLoopData {
                player_id: message.player.and_then(|p| player_id_of(&players, p)),
                pattern: match message.pattern {
                    LoopPattern::PingPong => "ping_pong",
                    LoopPattern::Collisions => "collisions",
                    LoopPattern::Rally => "rally",
                },
                start_rt: message.start_secs,
                end_rt: message.end_secs,
            },
        ));
    }
}

fn record_abilities(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reader: MessageReader<AbilityTriggered>,
    players: Query<&Player>,
) {
    let Some(mut sink) = sink else {
        return;
    };
    let stamp = clock.stamp();

    for message in reader.read() {
        let Some(player_id) = player_id_of(&players, message.owner) else {
            continue;
        };
        sink.write(&event(
            "ability_used",
            stamp,
            AbilityData {
                player_id,
                ability: format!("{:?}", message.ability),
            },
        ));
    }
}

/// Writes one record each time a player's charges reach zero. Runs in every phase,
/// so the refill at a round reset clears the memory for the next round.
fn record_charges_empty(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    mut reported: Local<HashSet<Entity>>,
    players: Query<(Entity, &Player, &BeamCharges), Changed<BeamCharges>>,
) {
    let Some(mut sink) = sink else {
        return;
    };

    for (entity, player, charges) in &players {
        if charges.current > 0 {
            reported.remove(&entity);
        } else if reported.insert(entity) {
            sink.write(&event(
                "charges_empty",
                clock.stamp(),
                ChargesEmptyData {
                    player_id: player.player_id,
                },
            ));
        }
    }
}

fn record_session_end(
    clock: Clock,
    sink: Option<ResMut<TelemetrySink>>,
    round: Res<TelemetryRound>,
    mut exits: MessageReader<AppExit>,
) {
    if exits.read().next().is_none() {
        return;
    }
    let Some(mut sink) = sink else {
        return;
    };
    sink.write(&event(
        "session_end",
        clock.stamp(),
        SessionEndData { round: round.0 },
    ));
    let _ = sink.writer.flush();
}

fn record_outcome(
    clock: Clock,
    config: Res<GameConfig>,
    result: Res<RoundResult>,
    round: Res<TelemetryRound>,
    countdown: Option<Res<Countdown>>,
    map_info: Res<MapInfo>,
    sink: Option<ResMut<TelemetrySink>>,
    players: Query<(
        &Player,
        &ClaimedTileCount,
        &Health,
        &BeamCharges,
        &AbilityList,
        Option<&Bot>,
    )>,
) {
    let Some(mut sink) = sink else {
        return;
    };

    let remaining = countdown.map_or(0, |c| c.remaining);
    let any_dead = players.iter().any(|(.., health, _, _, _)| health.current <= 0.0);
    let reason = if any_dead {
        "kill"
    } else if remaining == 0 {
        "timeout"
    } else {
        "charge_exhaustion"
    };

    let total_tiles = map_info.ground_entities.len().max(1) as f32;

    let mut player_records: Vec<OutcomePlayer> = players
        .iter()
        .map(|(player, count, health, charges, abilities, bot)| OutcomePlayer {
            player_id: player.player_id,
            controller: controller_tag(bot),
            tile_pct: count.current as f32 / total_tiles * 100.0,
            tile_count: count.current,
            hp: health.current,
            charges_spent: charges.max.saturating_sub(charges.current),
            loadout: loadout_names(&abilities.0),
        })
        .collect();
    player_records.sort_by_key(|player| player.player_id);

    let stamp = clock.stamp();
    sink.write(&OutcomeRecord {
        kind: "outcome",
        t: stamp.t,
        rt: stamp.rt,
        round: round.0,
        winner: result.winner,
        reason,
        round_duration_secs: config.round.round_duration_secs.saturating_sub(remaining),
        players: player_records,
    });
    let _ = sink.writer.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_of_one_player_merge_into_one_burst() {
        let bursts = group_bursts(&[(1, 4, 5), (0, 1, 1), (1, 9, 9), (1, 2, 2)]);
        assert_eq!(bursts, vec![(1, 3, 4, 5), (0, 1, 1, 1)]);
    }

    #[test]
    fn event_record_keeps_shared_fields_and_flattens_data() {
        let record = event(
            "hit",
            Stamp { t: 1.5, rt: 2.5 },
            HitData {
                attacker: Some(0),
                victim: 1,
                damage: 5.0,
                hp_after: 20.0,
                cause: "beam",
            },
        );
        let value = serde_json::to_value(&record).unwrap();
        assert_eq!(value["kind"], "event");
        assert_eq!(value["event"], "hit");
        assert_eq!(value["t"], 1.5);
        assert_eq!(value["rt"], 2.5);
        assert_eq!(value["victim"], 1);
        assert_eq!(value["cause"], "beam");
    }

    #[test]
    fn session_record_carries_the_clock_anchor() {
        let record = SessionRecord {
            kind: "session",
            schema_version: SCHEMA_VERSION,
            unix_ms: 1_700_000_000_000,
            t_at_write: 0.5,
            rt_at_write: 0.75,
            map: LEVEL_PATH,
            telemetry_enabled: true,
            bot_p1: true,
            bot_p2: true,
            loadout_p1: vec![],
            loadout_p2: vec![],
            rounds: 3,
            after_last_round: "Stay".to_string(),
        };
        let value = serde_json::to_value(&record).unwrap();
        for key in ["schema_version", "unix_ms", "t_at_write", "rt_at_write", "map"] {
            assert!(value.get(key).is_some(), "missing {key}");
        }
        assert_eq!(value["schema_version"], 4);
    }

    #[test]
    fn bot_loop_record_carries_its_time_range() {
        let record = event(
            "bot_loop",
            Stamp { t: 1.0, rt: 9.0 },
            BotLoopData {
                player_id: None,
                pattern: "rally",
                start_rt: 2.5,
                end_rt: 8.5,
            },
        );
        let value = serde_json::to_value(&record).unwrap();
        assert_eq!(value["event"], "bot_loop");
        assert!(value["player_id"].is_null());
        assert_eq!(value["start_rt"], 2.5);
        assert_eq!(value["end_rt"], 8.5);
    }

    #[test]
    fn record_time_counts_from_the_app_start() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(Time::<Real>::default());
        std::thread::sleep(std::time::Duration::from_millis(5));

        let rt = world
            .run_system_once(|clock: Clock| clock.stamp().rt)
            .unwrap();

        assert!(rt >= 0.005, "rt was {rt}");
    }
}
