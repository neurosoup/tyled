/*
 * Heuristic bot: each frame it reads the board and drives a bot-controlled
 * player's synthetic `ActionState` (no `InputMap`) through the same input
 * handler a human goes through. Policy: a paced deliberation beat decides,
 * in priority order, whether to fire, turn to aim, or path toward a target
 * tile — the chosen behaviour and reason are mirrored into `BotDecision` for
 * telemetry. A separate watcher reports loops (`BotLoopEnded`) and changes
 * nothing in how the bot plays.
 */
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;

use crate::plugins::beam::{is_position_claimed, resolve_fire};
use crate::plugins::damage::is_hostile_tile;
use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, attach_bot_state);
    app.add_systems(
        Update,
        bot_think
            .after(attach_bot_state)
            .in_set(GameplaySet::BotThink)
            .run_if(in_state(RoundPhase::Playing)),
    );
    app.init_resource::<RallyWatch>();
    app.add_systems(
        Update,
        watch_bot_loops
            .after(GameplaySet::Displacement)
            .before(GameplaySet::HudSync)
            .run_if(in_state(RoundPhase::Playing)),
    );
    app.add_systems(OnExit(RoundPhase::Playing), end_all_loops);
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
    /// Latest parry roll per beam.
    parry_rolls: ParryRolls,
}

/// Rally depth and result per beam.
type ParryRolls = HashMap<Entity, (u32, bool)>;

/// Whether to parry this beam at this depth. `roll` draws from the odds.
fn parry_roll(
    rolls: &mut ParryRolls,
    beam: Entity,
    depth: u32,
    chance: f32,
    decay: f32,
    roll: impl FnOnce(f64) -> bool,
) -> bool {
    // Rolling again on every frame would almost always end in a yes.
    if let Some(&(rolled_depth, wants)) = rolls.get(&beam)
        && rolled_depth == depth
    {
        return wants;
    }
    let odds = (chance * decay.powi(depth as i32)).clamp(0.0, 1.0);
    let wants = roll(odds as f64);
    rolls.insert(beam, (depth, wants));
    wants
}

/// A loop in progress, in real seconds since the app start.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Span {
    start: f32,
    last: f32,
}

/// What one bot has been repeating since its last claim or beam hit.
#[derive(Component, Default)]
struct LoopWatch {
    tile: Option<(GridCoords, f32)>,
    previous_tile: Option<GridCoords>,
    /// Visits in the current back-and-forth between two tiles.
    visits: u32,
    visits_start: f32,
    collisions: VecDeque<f32>,
    ping_pong: Option<Span>,
    ramming: Option<Span>,
}

impl LoopWatch {
    /// Notes the bot's tile, and returns a ping-pong loop that this tile ended.
    fn visit(&mut self, tile: GridCoords, now: f32, limit: u32) -> Option<Span> {
        if self.tile.is_some_and(|(current, _)| current == tile) {
            return None;
        }
        let mut ended = None;
        if self.previous_tile == Some(tile) {
            self.visits += 1;
        } else {
            ended = self.ping_pong.take();
            self.visits = if self.tile.is_some() { 2 } else { 1 };
            self.visits_start = self.tile.map_or(now, |(_, since)| since);
        }
        self.previous_tile = self.tile.map(|(current, _)| current);
        self.tile = Some((tile, now));
        if self.visits >= limit {
            self.ping_pong = Some(Span {
                start: self.visits_start,
                last: now,
            });
        }
        ended
    }

    fn collision(&mut self, now: f32, window: f32, limit: u32) {
        self.collisions.push_back(now);
        while self.collisions.front().is_some_and(|&at| now - at > window) {
            self.collisions.pop_front();
        }
        if let Some(span) = &mut self.ramming {
            span.last = now;
        } else if self.collisions.len() >= limit as usize {
            self.ramming = self.collisions.front().map(|&start| Span { start, last: now });
        }
    }

    /// Returns a collision loop that has been quiet for a whole window.
    fn expire(&mut self, now: f32, window: f32) -> Option<Span> {
        if self.ramming.is_some_and(|span| now - span.last > window) {
            self.collisions.clear();
            return self.ramming.take();
        }
        None
    }

    /// Forgets everything, and returns the loops that were in progress.
    fn finish(&mut self) -> impl Iterator<Item = (LoopPattern, Span)> + use<> {
        let done = std::mem::take(self);
        [
            (LoopPattern::PingPong, done.ping_pong),
            (LoopPattern::Collisions, done.ramming),
        ]
        .into_iter()
        .filter_map(|(pattern, span)| span.map(|span| (pattern, span)))
    }
}

/// Rallies in progress, per beam.
#[derive(Resource, Default)]
struct RallyWatch(HashMap<Entity, Rally>);

#[derive(Clone, Copy)]
struct Rally {
    span: Span,
    parries: u32,
}

fn loop_ended(player: Option<Entity>, pattern: LoopPattern, span: Span) -> BotLoopEnded {
    BotLoopEnded {
        player,
        pattern,
        start_secs: span.start,
        end_secs: span.last,
    }
}

fn watch_bot_loops(
    real: Res<Time<Real>>,
    config: Res<GameConfig>,
    mut claims: MessageReader<TileClaimed>,
    mut hits: MessageReader<DamageDealt>,
    mut collisions: MessageReader<CharacterCollision>,
    mut parries: MessageReader<BeamParried>,
    beams: Query<(), With<Beam>>,
    mut bots: Query<(Entity, &GridCoords, &mut LoopWatch), With<Bot>>,
    mut rallies: ResMut<RallyWatch>,
    mut ended: MessageWriter<BotLoopEnded>,
) {
    // Same zero as the telemetry `rt`.
    let now = real.startup().elapsed().as_secs_f32();
    let window = config.bot.loop_window_secs;

    let mut progressed: Vec<Entity> = claims
        .read()
        .filter(|claim| claim.old_owner != Some(claim.new_owner))
        .map(|claim| claim.new_owner)
        .collect();
    progressed.extend(
        hits.read()
            .filter(|hit| matches!(hit.source, DamageSource::BeamContact))
            .filter_map(|hit| hit.attacker),
    );
    // There are two characters, so every collision involves every bot.
    let collision_count = collisions.read().count();

    for (entity, coords, mut watch) in &mut bots {
        if progressed.contains(&entity) {
            for (pattern, span) in watch.finish() {
                ended.write(loop_ended(Some(entity), pattern, span));
            }
        }
        for _ in 0..collision_count {
            watch.collision(now, window, config.bot.loop_collisions);
        }
        if let Some(span) = watch.visit(*coords, now, config.bot.loop_pingpong_visits) {
            ended.write(loop_ended(Some(entity), LoopPattern::PingPong, span));
        }
        if let Some(span) = watch.expire(now, window) {
            ended.write(loop_ended(Some(entity), LoopPattern::Collisions, span));
        }
    }

    for parry in parries.read() {
        if !bots.contains(parry.parrier) {
            continue;
        }
        let rally = rallies.0.entry(parry.beam).or_insert(Rally {
            span: Span { start: now, last: now },
            parries: 0,
        });
        rally.span.last = now;
        rally.parries = parry.parries;
    }
    rallies.0.retain(|&beam, rally| {
        let alive = beams.contains(beam);
        if !alive && rally.parries >= config.bot.loop_rally_parries {
            ended.write(loop_ended(None, LoopPattern::Rally, rally.span));
        }
        alive
    });
}

/// Reports the loops still running when play stops, and starts the next round clean.
fn end_all_loops(
    config: Res<GameConfig>,
    mut bots: Query<(Entity, &mut LoopWatch)>,
    mut rallies: ResMut<RallyWatch>,
    mut ended: MessageWriter<BotLoopEnded>,
) {
    for (entity, mut watch) in &mut bots {
        for (pattern, span) in watch.finish() {
            ended.write(loop_ended(Some(entity), pattern, span));
        }
    }
    for (_, rally) in rallies.0.drain() {
        if rally.parries >= config.bot.loop_rally_parries {
            ended.write(loop_ended(None, LoopPattern::Rally, rally.span));
        }
    }
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
            .insert((BotBrain::default(), BotDecision::default(), LoopWatch::default()));
    }
}

fn bot_think(
    time: Res<Time>,
    config: Res<GameConfig>,
    map_info: Res<MapInfo>,
    claimed_query: Query<&ClaimedTile>,
    positions: Query<(Entity, &GridCoords), (With<Player>, With<Character>)>,
    beams: Query<(Entity, &GridCoords, &Beam), Without<Character>>,
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
    let jitter_ms = config.bot.think_interval_jitter_ms as i64;
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
        // Reflex layer: a parry window is tens of milliseconds wide, far narrower than the beat
        // interval below, so catching it can't wait for the next paced "think" — it's checked
        // every frame regardless of beat timing. The chance roll (cached per beam and rally depth,
        // shared with the brace/dodge call further down) is what makes this "opportunistic" rather
        // than a deterministic reflex: a beam the bot decided to ignore stays ignored here too.
        if let Some(beam) = find_parryable_beam(
            *coords,
            look.to_grid_coords(),
            entity,
            &beams,
            &map_info,
            &claimed_query,
            config.parry.window_fraction,
        ) {
            let depth = beams.get(beam).map_or(0, |(_, _, beam)| beam.parries);
            let wants_to_parry = parry_roll(
                &mut brain.parry_rolls,
                beam,
                depth,
                config.bot.parry_chance,
                config.bot.parry_rally_decay,
                |odds| rand::rng().random_bool(odds),
            );
            if wants_to_parry {
                action_state.set_axis_pair(&Action::Move, Vec2::ZERO);
                action_state.press(&Action::Shoot);
                brain.shooting = true;
                let next = BotDecision {
                    behaviour: "parry".to_string(),
                    why: "parrying incoming beam".to_string(),
                    move_x: 0.0,
                    move_y: 0.0,
                    shoot: true,
                };
                if *decision != next {
                    *decision = next;
                }
                continue;
            }
        }

        if now < brain.next_beat_secs {
            action_state.set_axis_pair(&Action::Move, Vec2::ZERO);
            action_state.release(&Action::Shoot);
            brain.shooting = false;
            continue;
        }
        let jitter_secs = if jitter_ms > 0 {
            rand::rng().random_range(-jitter_ms..=jitter_ms) as f32 / 1000.0
        } else {
            0.0
        };
        brain.next_beat_secs = now + (beat_secs + jitter_secs).max(0.0);
        brain.parry_rolls.retain(|beam, _| beams.contains(*beam));

        let coords = *coords;
        let opponent = all.iter().find(|(e, _)| *e != entity).map(|(_, c)| *c);
        let has_lance = abilities.is_some_and(|list| list.0.contains(&AbilityDescriptor::Lance));
        let has_border_grinder =
            abilities.is_some_and(|list| list.0.contains(&AbilityDescriptor::BorderGrinder));
        let has_charges = charges.map_or(true, |c| !c.is_empty());
        let behavior = resolve_fire(coords, has_lance, &map_info, &claimed_query);

        // Whether firing in `dir` right now would trigger a Border Grinder flip: the
        // immediate neighbor is completely blocked (no ordinary reach) and specifically
        // enemy-claimed.
        let is_border_grinder_flip = |dir: GridCoords| -> bool {
            has_border_grinder
                && reach(&map_info, &claimed_query, coords, dir) == 0
                && border_grinder_target(&map_info, &claimed_query, coords, dir, entity)
        };

        // Reach, but an exposed enemy frontier tile counts as reach 1 when the bot has
        // Border Grinder — a flip is at least as good as an ordinary reach-1 claim.
        let effective_reach = |dir: GridCoords| -> u32 {
            if is_border_grinder_flip(dir) {
                1
            } else {
                reach(&map_info, &claimed_query, coords, dir)
            }
        };

        let best_fire: Option<(GridCoords, u32)> = match behavior {
            Some(BeamBehavior::Straight) => {
                let facing = look.to_grid_coords();
                // An available flip always outranks an ordinary claim, however large — commit
                // to the current facing if it's already a flip, else take any flip direction.
                let flip_fire = is_border_grinder_flip(facing).then(|| (facing, 1)).or_else(|| {
                    CARDINALS
                        .into_iter()
                        .find(|&dir| is_border_grinder_flip(dir))
                        .map(|dir| (dir, 1))
                });
                flip_fire.or_else(|| {
                    // Commit to the current facing until its line is exhausted, then rotate to
                    // the longest remaining reach. Avoids swivelling between equal-reach directions.
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
                })
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

        // With Border Grinder, prefer chasing down a reachable enemy-claimed tile to flip over
        // merely claiming unclaimed ground — checked ahead of the ordinary fire-from-here branch
        // below, since a Lance-equipped bot's `can_fire` is almost always true (Lance fires from
        // any tile), so without this the bot would just keep claiming territory and never go
        // flip anything. Skipped whenever a flip is available from right here — the origin must
        // itself be unclaimed too (Border Grinder only ever triggers on a Straight-behavior
        // beam, which only fires from unclaimed ground; a Lance shot from claimed ground, even
        // one next to a hostile tile, never triggers it).
        let current_position_has_flip = !is_position_claimed(&map_info, &claimed_query, coords)
            && CARDINALS.into_iter().any(is_border_grinder_flip);
        let chase_target = (has_border_grinder && !current_position_has_flip)
            .then(|| {
                let reachable = dijkstra_first_steps(
                    &map_info,
                    &claimed_query,
                    entity,
                    coords,
                    hostile_cost,
                    opponent,
                );
                // A firing position, not the enemy tile itself — standing adjacent to a hostile
                // tile is enough to flip it (the existing `effective_reach` reach-0 trick fires
                // once we're there), and walking onto the enemy tile directly would just mean
                // stepping tile-by-tile through hostile ground taking chip damage the whole way.
                // Must be genuinely unclaimed, Lance or not: Border Grinder's flip check only
                // ever runs on a Straight-behavior beam (`beam.rs`), which only fires from
                // unclaimed ground — a Lance shot from claimed ground never triggers it at all.
                let is_firing_position = |t: GridCoords| {
                    t != coords
                        && !is_position_claimed(&map_info, &claimed_query, t)
                        && CARDINALS.into_iter().any(|dir| {
                            is_hostile_tile(&map_info, &claimed_query, t + dir, entity)
                        })
                        && !all.iter().any(|(other, pos)| *other != entity && *pos == t)
                };
                brain
                    .target
                    .filter(|t| reachable.contains_key(t) && is_firing_position(*t))
                    .or_else(|| {
                        reachable
                            .iter()
                            .filter(|(t, _)| is_firing_position(**t))
                            .min_by_key(|(_, (cost, _))| *cost)
                            .map(|(t, _)| *t)
                    })
                    .and_then(|t| reachable.get(&t).map(|(cost, step)| (t, *cost, *step)))
                    // Too far/costly a chase (e.g. deep hostile tunneling) isn't worth it —
                    // fall back to ordinary territory claiming instead.
                    .filter(|(_, cost, _)| *cost <= config.bot.chase_cost_threshold)
            })
            .flatten();

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
                let chase = astar_first_step(
                    &map_info,
                    &claimed_query,
                    entity,
                    coords,
                    foe,
                    hostile_cost,
                    Some(foe),
                );
                let step = if has_lance {
                    chase
                } else {
                    let safe = |t: GridCoords| {
                        t != foe
                            && map_info.on_ground(t)
                            && !is_hostile_tile(&map_info, &claimed_query, t, entity)
                    };
                    chase.filter(|s| safe(coords + *s)).or_else(|| {
                        // Next to the foe there is nothing left to approach.
                        (manhattan(coords, foe) > 1)
                            .then(|| {
                                CARDINALS
                                    .into_iter()
                                    .filter(|&d| safe(coords + d))
                                    .min_by_key(|&d| manhattan(coords + d, foe))
                            })
                            .flatten()
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
        } else if let Some(threat) =
            incoming_beam_threat(coords, &beams, entity, opponent, &map_info, &claimed_query)
        {
            // A hostile beam is bearing down this row/column. A parry is strictly better than a
            // dodge (it removes the threat and hands it back), but only worth going for when
            // there's enough runway to turn and still catch the window — a beam already adjacent
            // is too close to reorient for. Rolled once per rally depth and shared with the reflex
            // catch above, so a beam the bot committed to here is the one it'll actually swing at.
            let already_facing = look.to_grid_coords() == threat.needed_facing;
            let wants_to_brace = already_facing || threat.distance >= 2;
            let commits_to_parry = if wants_to_brace {
                parry_roll(
                    &mut brain.parry_rolls,
                    threat.beam,
                    threat.parries,
                    config.bot.parry_chance,
                    config.bot.parry_rally_decay,
                    |odds| rand::rng().random_bool(odds),
                )
            } else {
                // Too close to turn in time and not already facing: abandon any earlier
                // commitment to this beam so the reflex layer (which shares this cache) doesn't
                // still try to catch it the instant the window opens while we're busy dodging.
                brain.parry_rolls.remove(&threat.beam);
                false
            };

            action_state.release(&Action::Shoot);
            brain.shooting = false;
            if commits_to_parry && already_facing {
                // Already lined up: hold still rather than dodge away from a parry we're about
                // to land. The reflex check above fires the instant the window opens.
                (Vec2::ZERO, "brace", "holding to parry incoming beam".to_string(), false)
            } else if commits_to_parry {
                let unit = Vec2::new(threat.needed_facing.x as f32, threat.needed_facing.y as f32);
                (unit, "brace", "turning to parry incoming beam".to_string(), false)
            } else {
                let unit = Vec2::new(threat.dodge_step.x as f32, threat.dodge_step.y as f32);
                (unit, "dodge", "evading beam".to_string(), false)
            }
        } else if let Some((t, cost, step)) = chase_target {
            action_state.release(&Action::Shoot);
            brain.shooting = false;
            brain.target = Some(t);
            (
                Vec2::new(step.x as f32, step.y as f32),
                "border_grinder_chase",
                format!("heading to {t:?} to line up a Border Grinder flip, cost {cost}"),
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

            let reachable = dijkstra_first_steps(
                &map_info,
                &claimed_query,
                entity,
                coords,
                hostile_cost,
                opponent,
            );

            // A valid target is any reachable unclaimed tile (on arrival the bot can always
            // claim it). Only searched while charges remain — with none left, claiming is moot.
            let claim_target = has_charges.then(|| {
                let valid_target = |t: GridCoords| {
                    t != coords
                        && reachable.contains_key(&t)
                        && !is_position_claimed(&map_info, &claimed_query, t)
                        && !all.iter().any(|(other, pos)| *other != entity && *pos == t)
                };
                brain.target.filter(|t| valid_target(*t)).or_else(|| {
                    reachable
                        .iter()
                        .filter(|(t, _)| valid_target(**t))
                        .min_by(|(a, (a_cost, _)), (b, (b_cost, _))| {
                            reposition_score(**a, *a_cost, opponent, aggression)
                                .total_cmp(&reposition_score(**b, *b_cost, opponent, aggression))
                                .then_with(|| manhattan(coords, **a).cmp(&manhattan(coords, **b)))
                        })
                        .map(|(t, _)| *t)
                })
            });

            match claim_target
                .flatten()
                .and_then(|t| reachable.get(&t).map(|(cost, step)| (t, *cost, *step)))
            {
                Some((t, cost, step)) => {
                    brain.target = Some(t);
                    let behaviour = if aggression >= 0.5 { "aggress" } else { "reposition" };
                    let unit = Vec2::new(step.x as f32, step.y as f32);
                    (
                        unit,
                        behaviour,
                        format!("heading toward {t:?}, cost {cost}"),
                        false,
                    )
                }
                None => {
                    // Nothing to claim or flip right now — whether out of charges or because
                    // the board's fully divided with no unclaimed tile left — so pressuring the
                    // opponent is pure downside if we can't fire either way: hold safe ground, or
                    // retreat off hostile ground instead.
                    if !is_hostile_tile(&map_info, &claimed_query, coords, entity) {
                        brain.target = None;
                        (Vec2::ZERO, "hold", "nothing to claim; holding safe ground".to_string(), false)
                    } else {
                        let retreat = reachable
                            .iter()
                            .filter(|(t, _)| !is_hostile_tile(&map_info, &claimed_query, **t, entity))
                            .min_by_key(|(_, (cost, _))| *cost)
                            .map(|(t, (cost, step))| (*t, *cost, *step));
                        brain.target = retreat.map(|(t, _, _)| t);
                        match retreat {
                            Some((t, cost, step)) => (
                                Vec2::new(step.x as f32, step.y as f32),
                                "retreat",
                                format!("nothing to claim; retreating to {t:?}, cost {cost}"),
                                false,
                            ),
                            None => (
                                Vec2::ZERO,
                                "idle",
                                "nothing to claim; no safe retreat found".to_string(),
                                false,
                            ),
                        }
                    }
                }
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
/// `bot` — an exposed enemy frontier tile a Border Grinder beam can flip.
fn border_grinder_target(
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

/// A hostile beam travelling along `coords`' row or column toward it: how far out it is, which
/// way `coords` would need to face to parry it, and a perpendicular step onto a safe tile to
/// dodge it instead.
struct IncomingThreat {
    beam: Entity,
    /// Parries the beam has had so far.
    parries: u32,
    /// Chebyshev distance in tiles from the beam to `coords` along their shared line.
    distance: i32,
    needed_facing: GridCoords,
    dodge_step: GridCoords,
}

/// `None` if no such beam has a safe (on-ground, non-hostile) dodge tile at all — matching the
/// prior dodge-only behaviour, this skips a beam with nowhere safe to step rather than reporting
/// a threat the bot has no response to.
fn incoming_beam_threat(
    coords: GridCoords,
    beams: &Query<(Entity, &GridCoords, &Beam), Without<Character>>,
    self_entity: Entity,
    opponent: Option<GridCoords>,
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
) -> Option<IncomingThreat> {
    for (beam_entity, beam_pos, beam) in beams {
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
        let Some(dodge_step) = perpendicular.into_iter().find(|&p| {
            let dest = coords + p;
            Some(dest) != opponent
                && map_info.on_ground(dest)
                && !is_hostile_tile(map_info, claimed_query, dest, self_entity)
        }) else {
            continue;
        };
        return Some(IncomingThreat {
            beam: beam_entity,
            parries: beam.parries,
            distance: (coords.x - beam_pos.x).abs().max((coords.y - beam_pos.y).abs()),
            needed_facing: GridCoords::new(-dir.x, -dir.y),
            dodge_step,
        });
    }
    None
}

/// Dijkstra over walkable ground tiles from `start`, 4-connected via `CARDINALS`, never entering
/// the `blocked` tile.
fn dijkstra_first_steps(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    bot: Entity,
    start: GridCoords,
    hostile_cost: u32,
    blocked: Option<GridCoords>,
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
            if !map_info.on_ground(next) || Some(next) == blocked {
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
    blocked: Option<GridCoords>,
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
            if !map_info.on_ground(next) || Some(next) == blocked {
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_ecs_tiled::prelude::{TilemapGridSize, TilemapSize, TilemapTileSize};

    fn open_map(width: i32, height: i32) -> MapInfo {
        let mut map_info = MapInfo {
            map_size: TilemapSize::new(width as u32, height as u32),
            grid_size: TilemapGridSize::new(16.0, 16.0),
            tile_size: TilemapTileSize::new(16.0, 16.0),
            ..default()
        };
        for x in 0..width {
            for y in 0..height {
                map_info
                    .ground_entities
                    .insert(GridCoords { x, y }, Entity::PLACEHOLDER);
            }
        }
        map_info
    }

    #[test]
    fn paths_never_enter_the_blocked_tile() {
        let mut world = World::new();
        world.insert_resource(open_map(3, 2));
        let (reachable, step) = world
            .run_system_once(|map_info: Res<MapInfo>, claimed: Query<&ClaimedTile>| {
                let start = GridCoords::new(0, 0);
                let foe = GridCoords::new(1, 0);
                let reachable = dijkstra_first_steps(
                    &map_info,
                    &claimed,
                    Entity::PLACEHOLDER,
                    start,
                    40,
                    Some(foe),
                );
                let step = astar_first_step(
                    &map_info,
                    &claimed,
                    Entity::PLACEHOLDER,
                    start,
                    GridCoords::new(2, 0),
                    40,
                    Some(foe),
                );
                (reachable, step)
            })
            .unwrap();

        assert!(!reachable.contains_key(&GridCoords::new(1, 0)));
        assert_eq!(
            reachable.get(&GridCoords::new(2, 0)),
            Some(&(4, GridCoords::new(0, 1)))
        );
        assert_eq!(step, Some(GridCoords::new(0, 1)));
    }

    #[test]
    fn a_parry_is_rolled_once_per_depth_with_falling_odds() {
        let mut rolls = ParryRolls::default();
        let beam = Entity::PLACEHOLDER;
        let mut asked = Vec::new();
        for depth in [0, 0, 0, 1, 1, 2] {
            parry_roll(&mut rolls, beam, depth, 0.8, 0.5, |odds| {
                asked.push((odds * 1000.0).round() as u32);
                true
            });
        }
        assert_eq!(asked, vec![800, 400, 200]);
        assert!(parry_roll(&mut rolls, beam, 2, 0.8, 0.5, |_| false));
    }

    /// Walks back and forth between two tiles, one step every 0.1 s.
    fn ping_pong(watch: &mut LoopWatch, visits: u32) {
        for visit in 0..visits {
            let tile = GridCoords::new((visit % 2) as i32, 0);
            assert_eq!(watch.visit(tile, visit as f32 * 0.1, 10), None);
        }
    }

    #[test]
    fn ten_visits_between_two_tiles_make_a_loop() {
        let mut watch = LoopWatch::default();
        ping_pong(&mut watch, 9);
        assert!(watch.ping_pong.is_none());

        let mut watch = LoopWatch::default();
        ping_pong(&mut watch, 12);
        let ended = watch.visit(GridCoords::new(5, 5), 2.0, 10).unwrap();
        assert_eq!(ended.start, 0.0);
        assert!((ended.last - 1.1).abs() < 0.001);
    }

    #[test]
    fn steady_walking_is_not_a_loop() {
        let mut watch = LoopWatch::default();
        for x in 0..40 {
            assert_eq!(watch.visit(GridCoords::new(x, 0), x as f32, 10), None);
        }
        assert_eq!(watch.finish().count(), 0);
    }

    #[test]
    fn close_collisions_make_a_loop_that_ends_after_a_quiet_window() {
        let mut watch = LoopWatch::default();
        for at in [0.0, 0.4, 0.8, 1.2] {
            watch.collision(at, 3.0, 5);
        }
        assert!(watch.ramming.is_none());
        watch.collision(1.6, 3.0, 5);
        assert_eq!(watch.expire(2.0, 3.0), None);
        assert_eq!(watch.expire(5.0, 3.0), Some(Span { start: 0.0, last: 1.6 }));
    }

    #[test]
    fn spread_out_collisions_are_not_a_loop() {
        let mut watch = LoopWatch::default();
        for count in 0..10 {
            watch.collision(count as f32, 3.0, 5);
        }
        assert!(watch.ramming.is_none());
    }

    #[test]
    fn finishing_reports_the_running_loop_and_starts_clean() {
        let mut watch = LoopWatch::default();
        ping_pong(&mut watch, 12);
        let ended: Vec<_> = watch.finish().map(|(pattern, _)| pattern).collect();
        assert_eq!(ended, vec![LoopPattern::PingPong]);
        assert_eq!(watch.visits, 0);
    }
}
