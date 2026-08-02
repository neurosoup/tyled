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
    positions: Query<(Entity, &GridCoords), (With<Player>, With<Character>)>,
    beams: Query<(&GridCoords, &Beam), Without<Character>>,
    mut bots: Query<
        (
            Entity,
            &Player,
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

    for (
        entity,
        player,
        coords,
        look,
        mut action_state,
        charges,
        abilities,
        mut decision,
        mut brain,
    ) in &mut bots
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
        let has_lance = abilities.is_some_and(|list| list.0.contains(&AbilityDescriptor::Lance));
        let has_overpen =
            abilities.is_some_and(|list| list.0.contains(&AbilityDescriptor::Overpenetration));
        let has_charges = charges.map_or(true, |c| !c.is_empty());
        let behavior = resolve_fire(coords, has_lance, &map_info, &claimed_query);

        // Reach, but an exposed enemy frontier tile counts as reach 1 when the bot has
        // Overpenetration — a flip is at least as good as an ordinary reach-1 claim.
        let effective_reach = |dir: GridCoords| -> u32 {
            let n = reach(&map_info, &claimed_query, coords, dir);
            if n == 0 && has_overpen && overpen_target(&map_info, &claimed_query, coords, dir, entity)
            {
                1
            } else {
                n
            }
        };

        let best_fire: Option<(GridCoords, u32)> = match behavior {
            Some(BeamBehavior::Straight) => {
                // Commit to the current facing until its line is exhausted, then rotate to
                // the longest remaining reach. Avoids swivelling between equal-reach directions.
                let facing = look.to_grid_coords();
                let facing_reach = effective_reach(facing);
                if facing_reach >= 1 {
                    Some((facing, facing_reach))
                } else {
                    CARDINALS
                        .into_iter()
                        .map(|dir| (dir, effective_reach(dir)))
                        .filter(|(_, n)| *n >= 1)
                        .max_by_key(|(_, n)| *n)
                }
            }
            Some(BeamBehavior::Lance) => {
                let facing = look.to_grid_coords();
                let strike = opponent
                    .and_then(|foe| {
                        if lance_hits_enemy(&map_info, &claimed_query, coords, foe, facing) {
                            Some((facing, manhattan(coords, foe) as u32))
                        } else {
                            CARDINALS
                                .into_iter()
                                .find(|&dir| lance_hits_enemy(&map_info, &claimed_query, coords, foe, dir))
                                .map(|dir| (dir, manhattan(coords, foe) as u32))
                        }
                    });
                strike.or_else(|| {
                    // Prefer the current facing if it lands; otherwise the first landing direction in
                    // a fixed, neutral cardinal order (not ranked by strike depth or distance).
                    lance_landing(&map_info, &claimed_query, coords, facing)
                        .map(|land| (facing, land))
                        .or_else(|| {
                            CARDINALS.into_iter().find_map(|dir| {
                                lance_landing(&map_info, &claimed_query, coords, dir)
                                    .map(|land| (dir, land))
                            })
                        })
                        .map(|(dir, land)| (dir, manhattan(coords, land) as u32))
                })
            }
            None => None,
        };

        let can_fire = has_charges
            && match behavior {
                Some(BeamBehavior::Straight) => true,
                Some(BeamBehavior::Lance) => best_fire.is_some(),
                None => false,
            };

        let strike_mode = config.bot.strike_for(player.player_id);

        let (axis, behaviour, why, shoot) = if let (true, Some(foe)) = (strike_mode, opponent) {
            // Offense-focused: fire in any direction whose shot geometrically reaches the
            // opponent (line-of-fire, not adjacency); otherwise seek their row/column. Territory
            // is irrelevant here. `behavior` is `None` when firing is blocked (a claimed tile
            // without Lance), so a Straight hunter standing on a claimed tile can't strike,
            // whereas a Lance hunter can pierce it — the crux of the experiment.
            let facing = look.to_grid_coords();
            let fire_dir = has_charges
                .then_some(behavior)
                .flatten()
                .and_then(|shot| {
                    if shot_hits_enemy(shot, &map_info, &claimed_query, coords, foe, facing) {
                        Some(facing)
                    } else {
                        CARDINALS.into_iter().find(|&dir| {
                            shot_hits_enemy(shot, &map_info, &claimed_query, coords, foe, dir)
                        })
                    }
                });

            if let Some(dir) = fire_dir {
                if facing != dir {
                    action_state.release(&Action::Shoot);
                    brain.shooting = false;
                    (
                        Vec2::new(dir.x as f32, dir.y as f32),
                        "aim",
                        format!("turning to strike {}", direction_name(dir)),
                        false,
                    )
                } else {
                    let ready = now - brain.last_fire_secs >= cooldown_secs && !brain.shooting;
                    if ready {
                        action_state.press(&Action::Shoot);
                        brain.last_fire_secs = now;
                        brain.shooting = true;
                    } else {
                        action_state.release(&Action::Shoot);
                        brain.shooting = false;
                    }
                    let label = if behavior == Some(BeamBehavior::Lance) {
                        "strike_lance"
                    } else {
                        "strike_straight"
                    };
                    (Vec2::ZERO, label, format!("striking {}", direction_name(dir)), ready)
                }
            } else {
                // No line yet: chase the opponent. A* gives a stable, goal-directed path toward
                // them, and a strike triggers the instant the chase opens a line of fire.
                //
                // A striker *without* Lance can't fire from a claimed tile, so if the chase
                // parks it on an enemy tile it just soaks damage waiting for the foe to move — so
                // it never steps onto (or idles on) an enemy tile, skirting to the nearest safe
                // tile toward the foe instead. A Lance striker *can* fire from a claimed tile
                // (its own or the enemy's), so it chases freely — entering enemy territory to line
                // up a pierce is a real capability, not a trap.
                action_state.release(&Action::Shoot);
                brain.shooting = false;
                let chase =
                    astar_first_step(&map_info, &claimed_query, entity, coords, foe, hostile_cost);
                let step = if has_lance {
                    chase
                } else {
                    let safe = |t: GridCoords| {
                        map_info.on_ground(t)
                            && !is_hostile_tile(&map_info, &claimed_query, t, entity)
                    };
                    chase.filter(|s| safe(coords + *s)).or_else(|| {
                        CARDINALS
                            .into_iter()
                            .filter(|&d| safe(coords + d))
                            .min_by_key(|&d| manhattan(coords + d, foe))
                    })
                };
                match step {
                    Some(step) => (
                        Vec2::new(step.x as f32, step.y as f32),
                        "hunt",
                        format!("chasing {foe:?}"),
                        false,
                    ),
                    None => (Vec2::ZERO, "idle", "no safe approach to foe".to_string(), false),
                }
            }
        } else if let Some(step) =
            incoming_beam_dodge(coords, &beams, entity, &map_info, &claimed_query)
        {
            // A hostile beam is bearing down this row/column: step perpendicular off the line.
            action_state.release(&Action::Shoot);
            brain.shooting = false;
            (
                Vec2::new(step.x as f32, step.y as f32),
                "dodge",
                "evading beam".to_string(),
                false,
            )
        } else if can_fire {
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
                            if behavior == Some(BeamBehavior::Lance)
                                && opponent.is_some_and(|foe| {
                                    lance_hits_enemy(&map_info, &claimed_query, coords, foe, fire_dir)
                                })
                            {
                                "lance_strike"
                            } else if behavior == Some(BeamBehavior::Lance) {
                                "lance"
                            } else if fires_toward_opponent(coords, fire_dir, opponent) {
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

/// Whether the immediate next tile in `dir` from `from` is claimed by an entity other than
/// `bot` — an exposed enemy frontier tile an Overpenetration beam can flip.
fn overpen_target(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    from: GridCoords,
    dir: GridCoords,
    bot: Entity,
) -> bool {
    let pos = from + dir;
    map_info
        .claimed_entities
        .get(&pos)
        .and_then(|entity| claimed_query.get(*entity).ok())
        .and_then(|claimed_tile| claimed_tile.owner)
        .is_some_and(|owner| owner != bot)
}

/// Landing tile of a Lance shot from `from` in `dir`: pierces claimed/forbidden tiles and
/// resolves on the first unclaimed ground tile ahead; `None` if it leaves the map first (fizzle).
fn lance_landing(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    from: GridCoords,
    dir: GridCoords,
) -> Option<GridCoords> {
    let mut pos = from + dir;
    loop {
        if !(map_info.on_ground(pos) || map_info.on_forbidden_areas(pos)) {
            return None;
        }
        if map_info.on_ground(pos) && !is_position_claimed(map_info, claimed_query, pos) {
            return Some(pos);
        }
        pos = pos + dir;
    }
}

/// Whether a shot of the given `behavior` fired from `from` in `dir` geometrically reaches `foe`.
/// A Straight shot is blocked by the first claimed tile (`is_position_claimed` keys off *any*
/// owner); a Lance shot pierces claimed/forbidden tiles to the first unclaimed cell.
fn shot_hits_enemy(
    behavior: BeamBehavior,
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    from: GridCoords,
    foe: GridCoords,
    dir: GridCoords,
) -> bool {
    match behavior {
        BeamBehavior::Lance => lance_hits_enemy(map_info, claimed_query, from, foe, dir),
        BeamBehavior::Straight => {
            let mut pos = from + dir;
            loop {
                if !(map_info.on_ground(pos) || map_info.on_forbidden_areas(pos)) {
                    return false;
                }
                if pos == foe {
                    return true;
                }
                if map_info.on_ground(pos) && is_position_claimed(map_info, claimed_query, pos) {
                    return false;
                }
                pos = pos + dir;
            }
        }
    }
}

/// Whether a Lance shot from `from` in `dir` reaches `foe` — landing on their tile (an
/// unclaimed cell) or passing its head over them (a claimed cell) — before resolving elsewhere
/// or leaving the map.
fn lance_hits_enemy(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    from: GridCoords,
    foe: GridCoords,
    dir: GridCoords,
) -> bool {
    let mut pos = from + dir;
    loop {
        if !(map_info.on_ground(pos) || map_info.on_forbidden_areas(pos)) {
            return false;
        }
        if pos == foe {
            return true;
        }
        if map_info.on_ground(pos) && !is_position_claimed(map_info, claimed_query, pos) {
            return false;
        }
        pos = pos + dir;
    }
}

/// If a hostile beam is travelling along `coords`' row or column toward it, returns a perpendicular
/// step onto a safe (on-ground, non-hostile) tile to escape the line, else `None`.
fn incoming_beam_dodge(
    coords: GridCoords,
    beams: &Query<(&GridCoords, &Beam), Without<Character>>,
    self_entity: Entity,
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
) -> Option<GridCoords> {
    for (beam_pos, beam) in beams {
        if beam.owner == self_entity {
            continue;
        }
        let dir = beam.direction;
        // On the beam's line and ahead of it in its travel direction.
        let on_line = (dir.x != 0 && beam_pos.y == coords.y && (coords.x - beam_pos.x).signum() == dir.x)
            || (dir.y != 0 && beam_pos.x == coords.x && (coords.y - beam_pos.y).signum() == dir.y);
        if !on_line {
            continue;
        }
        let perpendicular = if dir.x != 0 {
            [GridCoords::new(0, 1), GridCoords::new(0, -1)]
        } else {
            [GridCoords::new(1, 0), GridCoords::new(-1, 0)]
        };
        if let Some(step) = perpendicular.into_iter().find(|&p| {
            let dest = coords + p;
            map_info.on_ground(dest) && !is_hostile_tile(map_info, claimed_query, dest, self_entity)
        }) {
            return Some(step);
        }
    }
    None
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

/// A* from `start` to `goal` over walkable ground (4-connected via `CARDINALS`), entering an
/// enemy-owned tile costs `hostile_cost`. Returns the first step of a shortest path, or `None` if
/// `goal` is unreachable. Goal-directed (Manhattan heuristic, admissible since every step costs
/// ≥ 1), so chasing a single target yields a stable, direct path rather than a flood.
fn astar_first_step(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    bot: Entity,
    start: GridCoords,
    goal: GridCoords,
    hostile_cost: u32,
) -> Option<GridCoords> {
    if start == goal {
        return None;
    }
    let heuristic = |c: GridCoords| manhattan(c, goal) as u32;
    let mut g_score: HashMap<GridCoords, u32> = HashMap::new();
    let mut first_step: HashMap<GridCoords, GridCoords> = HashMap::new();
    // (f = g + h, g, x, y)
    let mut heap: BinaryHeap<Reverse<(u32, u32, i32, i32)>> = BinaryHeap::new();

    g_score.insert(start, 0);
    heap.push(Reverse((heuristic(start), 0, start.x, start.y)));

    while let Some(Reverse((_f, cost, x, y))) = heap.pop() {
        let coords = GridCoords::new(x, y);
        // Accept reaching the goal *or* any tile cardinally adjacent to it: the goal itself may be
        // a non-walkable tile (e.g. a player-start marker outside `ground_entities`), and adjacency
        // is where a strike lines up anyway. `first_step[coords]` is `None` only when already there.
        if coords == goal || manhattan(coords, goal) == 1 {
            return first_step.get(&coords).copied();
        }
        if cost > g_score.get(&coords).copied().unwrap_or(u32::MAX) {
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
            if next_cost < g_score.get(&next).copied().unwrap_or(u32::MAX) {
                g_score.insert(next, next_cost);
                let step_from_start = if coords == start { step } else { first_step[&coords] };
                first_step.insert(next, step_from_start);
                heap.push(Reverse((next_cost + heuristic(next), next_cost, next.x, next.y)));
            }
        }
    }
    None
}
