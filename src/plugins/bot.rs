/*
 * Heuristic bot: each frame it reads the board and drives a bot-controlled
 * player's synthetic `ActionState` (no `InputMap`) through the same input
 * handler a human goes through. Policy: a paced deliberation beat decides,
 * in priority order, whether to fire, turn to aim, or path toward a target
 * tile — the chosen behaviour and reason are mirrored into `BotDecision` for
 * telemetry.
 */
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::plugins::beam::{is_position_claimed, resolve_fire};
use crate::plugins::damage::is_hostile_tile;
use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, attach_bot_state);
    app.add_systems(
        Update,
        bot_think
            .after(attach_bot_state)
            .before(crate::plugins::inputs::handle_characters_input)
            .run_if(in_state(RoundPhase::Playing)),
    );
}

/// The bot's current intent, mirrored each time it changes so telemetry can log
/// decisions on change rather than every frame.
#[derive(Component, Default, Clone, PartialEq)]
pub struct BotDecision {
    pub behaviour: String,
    pub why: String,
    pub move_x: f32,
    pub move_y: f32,
    pub shoot: bool,
}

#[derive(Component, Default)]
struct BotBrain {
    last_fire_secs: f32,
    shooting: bool,
    next_beat_secs: f32,
    target: Option<GridCoords>,
}

const CARDINALS: [GridCoords; 4] = [
    GridCoords { x: 0, y: 1 },
    GridCoords { x: 0, y: -1 },
    GridCoords { x: -1, y: 0 },
    GridCoords { x: 1, y: 0 },
];

fn attach_bot_state(mut commands: Commands, bots: Query<Entity, (Added<Bot>, Without<BotBrain>)>) {
    for entity in &bots {
        commands
            .entity(entity)
            .insert((BotBrain::default(), BotDecision::default()));
    }
}

fn bot_think(
    time: Res<Time>,
    config: Res<GameConfig>,
    map_info: Res<MapInfo>,
    claimed_query: Query<&ClaimedTile>,
    positions: Query<(Entity, &GridCoords), With<Player>>,
    mut bots: Query<
        (
            Entity,
            &GridCoords,
            &LookDirection,
            &mut ActionState<Action>,
            Option<&BeamCharges>,
            Option<&AbilityList>,
            &mut BotDecision,
            &mut BotBrain,
        ),
        With<Bot>,
    >,
) {
    let now = time.elapsed_secs();
    let cooldown_secs = config.bot.fire_cooldown_ms as f32 / 1000.0;
    let beat_secs = config.bot.think_interval_ms as f32 / 1000.0;
    let hostile_cost = config.bot.hostile_cost;
    let aggression = config.bot.aggression;
    let all: Vec<(Entity, GridCoords)> = positions.iter().map(|(e, c)| (e, *c)).collect();

    for (entity, coords, look, mut action_state, charges, abilities, mut decision, mut brain) in
        &mut bots
    {
        if now < brain.next_beat_secs {
            action_state.set_axis_pair(&Action::Move, Vec2::ZERO);
            action_state.release(&Action::Shoot);
            brain.shooting = false;
            continue;
        }
        brain.next_beat_secs = now + beat_secs;

        let coords = *coords;
        let opponent = all.iter().find(|(e, _)| *e != entity).map(|(_, c)| *c);
        let has_backfill = abilities.is_some_and(|list| list.0.contains(&AbilityDescriptor::Backfill));
        let has_charges = charges.map_or(true, |c| !c.is_empty());
        let fireable = resolve_fire(coords, has_backfill, &map_info, &claimed_query).is_some();

        let best_fire = fireable
            .then(|| {
                // Commit to the current facing until its line is exhausted, then rotate to
                // the longest remaining reach. Avoids swivelling between equal-reach directions.
                let facing = look.to_grid_coords();
                let facing_reach = reach(&map_info, &claimed_query, coords, facing);
                if facing_reach >= 1 {
                    Some((facing, facing_reach))
                } else {
                    CARDINALS
                        .into_iter()
                        .map(|dir| (dir, reach(&map_info, &claimed_query, coords, dir)))
                        .filter(|(_, n)| *n >= 1)
                        .max_by_key(|(_, n)| *n)
                }
            })
            .flatten();

        let (axis, behaviour, why, shoot) = if has_charges && fireable {
            // From an unclaimed tile every shot claims a tile: down a runway it claims the
            // farthest unclaimed tile in that line, and into a blocked neighbour it claims this
            // tile itself (a straight beam resolves on its origin). So fire whenever we can.
            match best_fire {
                // A runway exists but we're not facing it yet: turn in place to aim.
                Some((fire_dir, reach_n)) if look.to_grid_coords() != fire_dir => {
                    action_state.release(&Action::Shoot);
                    brain.shooting = false;
                    let unit = Vec2::new(fire_dir.x as f32, fire_dir.y as f32);
                    (
                        unit,
                        "aim",
                        format!("turning to face {}, reach {reach_n}", direction_name(fire_dir)),
                        false,
                    )
                }
                // Facing a runway (claim a far tile), or no runway at all (claim this tile by
                // firing along the current facing into its blocked neighbour): fire.
                _ => {
                    let ready = now - brain.last_fire_secs >= cooldown_secs && !brain.shooting;
                    if ready {
                        action_state.press(&Action::Shoot);
                        brain.last_fire_secs = now;
                        brain.shooting = true;
                    } else {
                        action_state.release(&Action::Shoot);
                        brain.shooting = false;
                    }
                    let (behaviour, why) = match best_fire {
                        Some((fire_dir, reach_n)) => (
                            if fires_toward_opponent(coords, fire_dir, opponent) {
                                "aggress"
                            } else {
                                "claim"
                            },
                            format!("firing {}, reach {reach_n}", direction_name(fire_dir)),
                        ),
                        None => ("claim", "claiming current tile".to_string()),
                    };
                    (Vec2::ZERO, behaviour, why, ready)
                }
            }
        } else {
            action_state.release(&Action::Shoot);
            brain.shooting = false;

            let reachable = dijkstra_first_steps(&map_info, &claimed_query, entity, coords, hostile_cost);
            // Any reachable unclaimed tile is a valid target: on arrival the bot can always
            // claim it (a runway shot, or firing into a blocked neighbour to claim the tile
            // itself), so it never strands on an unclaimable one.
            let unclaimed = |t: GridCoords| {
                t != coords
                    && reachable.contains_key(&t)
                    && !is_position_claimed(&map_info, &claimed_query, t)
            };
            let target = brain
                .target
                .filter(|t| unclaimed(*t))
                .or_else(|| {
                    reachable
                        .iter()
                        .filter(|(t, _)| !is_position_claimed(&map_info, &claimed_query, **t))
                        .min_by(|(a, (a_cost, _)), (b, (b_cost, _))| {
                            reposition_score(**a, *a_cost, opponent, aggression)
                                .total_cmp(&reposition_score(**b, *b_cost, opponent, aggression))
                                .then_with(|| manhattan(coords, **a).cmp(&manhattan(coords, **b)))
                        })
                        .map(|(t, _)| *t)
                })
                // Board fully claimed: pressure the opponent rather than freeze.
                .or_else(|| opponent.filter(|foe| reachable.contains_key(foe)));
            brain.target = target;

            match target.and_then(|t| reachable.get(&t).map(|(cost, step)| (t, *cost, *step))) {
                Some((t, cost, step)) => {
                    let behaviour = if aggression >= 0.5 { "aggress" } else { "reposition" };
                    let unit = Vec2::new(step.x as f32, step.y as f32);
                    (
                        unit,
                        behaviour,
                        format!("heading toward {t:?}, cost {cost}"),
                        false,
                    )
                }
                None => (Vec2::ZERO, "idle", "no reachable target".to_string(), false),
            }
        };

        action_state.set_axis_pair(&Action::Move, axis);

        let next = BotDecision {
            behaviour: behaviour.to_string(),
            why,
            move_x: axis.x,
            move_y: axis.y,
            shoot,
        };
        if *decision != next {
            *decision = next;
        }
    }
}

fn manhattan(a: GridCoords, b: GridCoords) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn direction_name(dir: GridCoords) -> &'static str {
    match (dir.x, dir.y) {
        (0, 1) => "up",
        (0, -1) => "down",
        (-1, 0) => "left",
        (1, 0) => "right",
        _ => "unknown",
    }
}

fn fires_toward_opponent(from: GridCoords, dir: GridCoords, opponent: Option<GridCoords>) -> bool {
    opponent.is_some_and(|foe| {
        let dx = (foe.x - from.x).signum();
        let dy = (foe.y - from.y).signum();
        (dir.x != 0 && dir.x.signum() == dx && dx != 0) || (dir.y != 0 && dir.y.signum() == dy && dy != 0)
    })
}

fn reposition_score(target: GridCoords, cost: u32, opponent: Option<GridCoords>, aggression: f32) -> f32 {
    let mut score = cost as f32;
    if aggression >= 0.5
        && let Some(foe) = opponent
    {
        score -= aggression * 10.0 / (1.0 + manhattan(target, foe) as f32);
    }
    score
}

/// Count of consecutive unclaimed on-ground tiles starting at `from + dir` and continuing in
/// `dir` until an obstacle (a claimed tile, forbidden area, or off-ground/edge).
fn reach(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    from: GridCoords,
    dir: GridCoords,
) -> u32 {
    let mut pos = from + dir;
    let mut count = 0;
    while map_info.on_ground(pos)
        && !map_info.on_forbidden_areas(pos)
        && !is_position_claimed(map_info, claimed_query, pos)
    {
        count += 1;
        pos = pos + dir;
    }
    count
}

/// Dijkstra over walkable ground tiles from `start`, 4-connected via `CARDINALS`.
fn dijkstra_first_steps(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    bot: Entity,
    start: GridCoords,
    hostile_cost: u32,
) -> HashMap<GridCoords, (u32, GridCoords)> {
    let mut dist: HashMap<GridCoords, u32> = HashMap::new();
    let mut first_step: HashMap<GridCoords, GridCoords> = HashMap::new();
    let mut heap: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();

    dist.insert(start, 0);
    heap.push(Reverse((0, start.x, start.y)));

    while let Some(Reverse((cost, x, y))) = heap.pop() {
        let coords = GridCoords::new(x, y);
        if cost > dist.get(&coords).copied().unwrap_or(u32::MAX) {
            continue;
        }
        for step in CARDINALS {
            let next = coords + step;
            if !map_info.on_ground(next) {
                continue;
            }
            let enter_cost = if is_hostile_tile(map_info, claimed_query, next, bot) {
                hostile_cost
            } else {
                1
            };
            let next_cost = cost + enter_cost;
            if next_cost < dist.get(&next).copied().unwrap_or(u32::MAX) {
                dist.insert(next, next_cost);
                let step_from_start = if coords == start { step } else { first_step[&coords] };
                first_step.insert(next, step_from_start);
                heap.push(Reverse((next_cost, next.x, next.y)));
            }
        }
    }

    dist.into_iter()
        .filter_map(|(coords, cost)| first_step.get(&coords).map(|step| (coords, (cost, *step))))
        .collect()
}
