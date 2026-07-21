/*
 * Play-telemetry sink: records human/bot actions and per-round outcomes to a
 * single gitignored JSONL file (`play_trace.jsonl`, one JSON object per line).
 *
 * At startup the previous session's traces are rotated into numbered backups
 * (`play_trace.1.jsonl`, `play_trace.2.jsonl`, …, bounded by `telemetry.history`)
 * before a fresh `play_trace.jsonl` is opened. It opens with a
 * self-describing `session` header line. `action` records are written on change
 * per player entity while `Playing`; one `outcome` record is written per round
 * on entering `Outcome`. Every record is tagged `controller` ("human"|"bot").
 * All systems are gated behind the `telemetry.enabled` config toggle.
 */
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::Serialize;

use crate::plugins::damage::is_hostile_tile;
use crate::prelude::*;

const TRACE_PATH: &str = "play_trace.jsonl";

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ActionSnapshots>();
    app.add_systems(Startup, open_sink.run_if(telemetry_enabled));
    app.add_systems(
        Update,
        (record_actions, record_decisions)
            .run_if(in_state(RoundPhase::Playing))
            .run_if(telemetry_enabled),
    );
    app.add_systems(
        OnEnter(RoundPhase::Outcome),
        record_outcome.run_if(telemetry_enabled),
    );
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
    telemetry_enabled: bool,
    bot_p1: bool,
    bot_p2: bool,
    loadout_p1: Vec<String>,
    loadout_p2: Vec<String>,
}

#[derive(Serialize)]
struct ActionRecord {
    kind: &'static str,
    controller: &'static str,
    player_id: u8,
    t: f32,
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
    behaviour: String,
    why: String,
    move_x: f32,
    move_y: f32,
    shoot: bool,
}

#[derive(Serialize)]
struct OutcomeRecord {
    kind: &'static str,
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

fn open_sink(mut commands: Commands, config: Res<GameConfig>, loadouts: Res<PlayerLoadouts>) {
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
    sink.write(&SessionRecord {
        kind: "session",
        telemetry_enabled: config.telemetry.enabled,
        bot_p1: config.controllers.is_bot(0),
        bot_p2: config.controllers.is_bot(1),
        loadout_p1: loadout_names(&loadouts.player1),
        loadout_p2: loadout_names(&loadouts.player2),
    });
    let _ = sink.writer.flush();
    commands.insert_resource(sink);
}

fn record_actions(
    time: Res<Time>,
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
            t: time.elapsed_secs(),
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
    time: Res<Time>,
    sink: Option<ResMut<TelemetrySink>>,
    decisions: Query<(&Player, &BotDecision), (With<Bot>, Changed<BotDecision>)>,
) {
    let Some(mut sink) = sink else {
        return;
    };

    for (player, decision) in &decisions {
        sink.write(&DecisionRecord {
            kind: "decision",
            controller: "bot",
            player_id: player.player_id,
            t: time.elapsed_secs(),
            behaviour: decision.behaviour.clone(),
            why: decision.why.clone(),
            move_x: decision.move_x,
            move_y: decision.move_y,
            shoot: decision.shoot,
        });
    }
}

fn record_outcome(
    config: Res<GameConfig>,
    result: Res<RoundResult>,
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

    sink.write(&OutcomeRecord {
        kind: "outcome",
        winner: result.winner,
        reason,
        round_duration_secs: config.round.round_duration_secs.saturating_sub(remaining),
        players: player_records,
    });
    let _ = sink.writer.flush();
}
