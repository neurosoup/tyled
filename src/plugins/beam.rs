/*
 * Plugin for beam behavior: spawning and stepping beams, and spending charges.
 * Emits `BeamResolved` when a beam stops; the claim plugin turns that into a
 * tile-ownership change.
 */
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_beam, resolve_parry, beam_step)
            .chain()
            .in_set(GameplaySet::Beam)
            .run_if(in_state(RoundPhase::Playing)),
    );
    // Ordered before `beam_step` — both mutate `Beam` unfiltered, a real ambiguity otherwise.
    #[cfg(feature = "dev")]
    app.add_systems(Update, resync_beam_step_timers.before(beam_step));
}

/// A beam's step duration at a given rally depth.
fn beam_step_duration(config: &GameConfig, parries: u32) -> f32 {
    let speed = config.parry.speed_multiplier.powi(parries as i32);
    (config.timing.beam_step_secs / speed).max(config.parry.min_step_secs)
}

/// A freshly seeded per-beam step timer, pre-elapsed so the beam steps on its spawn frame.
fn new_beam_step_timer(duration_secs: f32) -> Timer {
    let mut timer = Timer::from_seconds(duration_secs, TimerMode::Repeating);
    timer.set_elapsed(timer.duration());
    timer
}

/// Resyncs every live beam's step timer when `game_config.ron` hot-reloads.
#[cfg(feature = "dev")]
fn resync_beam_step_timers(config: Res<GameConfig>, mut beams: Query<&mut Beam>) {
    if !config.is_changed() {
        return;
    }
    for mut beam in &mut beams {
        let duration = beam_step_duration(&config, beam.parries);
        beam.step_timer
            .set_duration(std::time::Duration::from_secs_f32(duration));
    }
}

pub(crate) fn is_position_claimed(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    coords: GridCoords,
) -> bool {
    map_info
        .claimed_entities
        .get(&coords)
        .is_some_and(|e| claimed_query.get(*e).is_ok_and(|ct| ct.owner.is_some()))
}

/// The beam behavior for a shot, or `None` if firing is blocked.
pub(crate) fn resolve_fire(
    origin: GridCoords,
    can_override_block: bool,
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
) -> Option<BeamBehavior> {
    match (
        is_position_claimed(map_info, claimed_query, origin),
        can_override_block,
    ) {
        (true, false) => None,
        (true, true) => Some(BeamBehavior::Lance),
        (false, _) => Some(BeamBehavior::Straight),
    }
}

// Spends one charge per committed shot, only when a beam actually spawns —
// a fire blocked outright (standing on a claimed tile without Lance) costs
// nothing, but a spawned beam that finds nothing to claim still costs its charge.
fn spawn_beam(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut beam_fired_reader: MessageReader<BeamFired>,
    beams_query: Query<(&Beam, &GridCoords)>,
    ability_query: Query<&AbilityList>,
    claimed_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
    mut owner_state: Query<(&mut BeamCharges, &mut InFlightBeamCount)>,
    mut charge_spent_writer: MessageWriter<ChargeSpent>,
    mut ability_writer: MessageWriter<AbilityTriggered>,
) {
    for beam_fired_message in beam_fired_reader.read() {
        let owner_has_active_beam = beams_query.iter().any(|(beam, coords)| {
            if beam.caster != beam_fired_message.owner {
                return false;
            }
            // Horizontal new beam: overlapping if existing beam is on same row (Y) and horizontal
            if beam_fired_message.direction.x != 0 {
                return coords.y == beam_fired_message.origin.y && beam.direction.x != 0;
            }
            // Vertical new beam: overlapping if existing beam is on same column (X) and vertical
            coords.x == beam_fired_message.origin.x && beam.direction.y != 0
        });

        let has_lance = ability_query
            .get(beam_fired_message.owner)
            .is_ok_and(|list| list.0.contains(&AbilityDescriptor::Lance));
        let Some(behavior) = resolve_fire(
            beam_fired_message.origin,
            has_lance,
            &map_info,
            &claimed_query,
        ) else {
            continue;
        };

        if behavior == BeamBehavior::Lance {
            ability_writer.write(AbilityTriggered {
                owner: beam_fired_message.owner,
                ability: AbilityDescriptor::Lance,
            });
        }

        if let Ok((mut charges, mut in_flight)) = owner_state.get_mut(beam_fired_message.owner) {
            charges.current = charges.current.saturating_sub(1);
            in_flight.current += 1;
            charge_spent_writer.write(ChargeSpent {
                owner: beam_fired_message.owner,
                amount: 1,
            });
        }

        let mut entity_commands = commands.spawn((
            beam_fired_message.origin,
            Beam {
                owner: beam_fired_message.owner,
                caster: beam_fired_message.owner,
                direction: beam_fired_message.direction,
                parries: 0,
                behavior,
                step_timer: new_beam_step_timer(beam_step_duration(&config, 0)),
                awaiting_step: false,
                just_parried: false,
            },
        ));
        if !owner_has_active_beam {
            entity_commands.insert((
                WaveSource,
                BounceEffect {
                    intensity: 5.0,
                    bounce_count: 2,
                    decay: 0.3,
                    z_index: CLAIMED_TILE_Z_INDEX,
                },
            ));
        }
    }
}

/// Whether `position` is claimed by an entity other than `owner` (the beneficiary) and
/// `caster`'s (the original shooter's) `AbilityList` contains
/// [`AbilityDescriptor::BorderGrinder`].
fn border_grinder_flips_tile(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    ability_query: &Query<&AbilityList>,
    owner: Entity,
    caster: Entity,
    position: GridCoords,
) -> bool {
    let is_enemy_tile = map_info
        .claimed_entities
        .get(&position)
        .and_then(|entity| claimed_query.get(*entity).ok())
        .and_then(|claimed_tile| claimed_tile.owner)
        .is_some_and(|tile_owner| tile_owner != owner);

    is_enemy_tile
        && ability_query
            .get(caster)
            .is_ok_and(|list| list.0.contains(&AbilityDescriptor::BorderGrinder))
}

/// Ends a beam's lifetime: decrements `caster`'s in-flight count and despawns it.
fn end_beam(
    commands: &mut Commands,
    beam_entity: Entity,
    caster: Entity,
    in_flight: &mut Query<&mut InFlightBeamCount>,
) {
    if let Ok(mut count) = in_flight.get_mut(caster) {
        count.current = count.current.saturating_sub(1);
    }
    commands.entity(beam_entity).despawn();
}

/// Whether a beam immediately in front of `character`, which `character` is
/// facing, is currently parryable.
pub(crate) fn find_parryable_beam(
    coords: GridCoords,
    facing: GridCoords,
    character: Entity,
    beams: &Query<(Entity, &GridCoords, &Beam), Without<Character>>,
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    window_fraction: f32,
) -> Option<Entity> {
    for (entity, beam_coords, beam) in beams {
        // A second parry before the beam moves would let two close players loop every frame.
        if beam.owner == character || beam.awaiting_step {
            continue;
        }
        if *beam_coords + beam.direction != coords {
            continue;
        }
        // Must be facing the incoming beam, not just standing in its path.
        if facing != GridCoords::new(-beam.direction.x, -beam.direction.y) {
            continue;
        }
        if beam.step_timer.elapsed_secs()
            >= window_fraction * beam.step_timer.duration().as_secs_f32()
        {
            continue;
        }
        let threatens = match beam.behavior {
            BeamBehavior::Straight => !is_position_claimed(map_info, claimed_query, coords),
            BeamBehavior::Lance => is_position_claimed(map_info, claimed_query, coords),
        };
        if threatens {
            return Some(entity);
        }
    }
    None
}

// Reads `ParryTriggered`, reversing the beam and flipping its `owner` to the parrier.
fn resolve_parry(
    config: Res<GameConfig>,
    mut parry_triggered_reader: MessageReader<ParryTriggered>,
    mut beams: Query<&mut Beam>,
    mut beam_parried_writer: MessageWriter<BeamParried>,
) {
    let mut handled = std::collections::HashSet::new();
    for message in parry_triggered_reader.read() {
        if !handled.insert(message.beam) {
            continue;
        }
        let Ok(mut beam) = beams.get_mut(message.beam) else {
            continue;
        };
        if beam.owner == message.parrier {
            continue;
        }

        let previous_owner = beam.owner;
        beam.direction = GridCoords::new(-beam.direction.x, -beam.direction.y);
        beam.owner = message.parrier;
        beam.parries += 1;
        beam.step_timer = Timer::from_seconds(
            beam_step_duration(&config, beam.parries),
            TimerMode::Repeating,
        );
        beam.awaiting_step = true;
        beam.just_parried = true;

        beam_parried_writer.write(BeamParried {
            beam: message.beam,
            parrier: message.parrier,
            previous_owner,
            caster: beam.caster,
            new_direction: beam.direction,
            parries: beam.parries,
        });
    }
}

fn beam_step(
    mut commands: Commands,
    mut beams_query: Query<(Entity, &mut Beam, &mut GridCoords), Without<Character>>,
    claimed_query: Query<&ClaimedTile>,
    ability_query: Query<&AbilityList>,
    time: Res<Time>,
    map_info: Res<MapInfo>,
    mut beam_resolved_writer: MessageWriter<BeamResolved>,
    mut ability_writer: MessageWriter<AbilityTriggered>,
    mut in_flight: Query<&mut InFlightBeamCount>,
) {
    for (beam_entity, mut beam, mut position) in &mut beams_query {
        // The parry frame does not count, or one long frame would skip the whole wait.
        if std::mem::take(&mut beam.just_parried) {
            continue;
        }
        beam.step_timer.tick(time.delta());
        if !beam.step_timer.just_finished() {
            continue;
        }
        beam.awaiting_step = false;

        let next_position = *position + beam.direction;

        match beam.behavior {
            // +----------------------------+
            // | Lance                      |
            // | resolve on the first       |
            // | unclaimed tile ahead.      |
            // +----------------------------+
            BeamBehavior::Lance => {
                if !(map_info.on_ground(next_position)
                    || map_info.on_forbidden_areas(next_position))
                {
                    end_beam(&mut commands, beam_entity, beam.caster, &mut in_flight);
                    continue;
                }
                let is_next_unclaimed = map_info.on_ground(next_position)
                    && !map_info
                        .claimed_entities
                        .get(&next_position)
                        .is_some_and(|e| claimed_query.get(*e).is_ok_and(|ct| ct.owner.is_some()));
                if is_next_unclaimed {
                    beam_resolved_writer.write(BeamResolved {
                        position: next_position,
                        owner: beam.owner,
                    });
                    end_beam(&mut commands, beam_entity, beam.caster, &mut in_flight);
                    continue;
                }
                *position = next_position;
            }

            // +----------------------------+
            // | Straight: stop             |
            // | at the first blocked tile, |
            // | claim the last unclaimed   |
            // | tile before it.            |
            // +----------------------------+
            BeamBehavior::Straight => {
                // Out of map bounds rule
                if !(map_info.on_ground(next_position)
                    || map_info.on_forbidden_areas(next_position))
                {
                    // Move back to the last unclaimed position in case it's a forbidden area
                    while map_info.on_forbidden_areas(*position) {
                        *position -= beam.direction;
                    }
                    let is_position_claimed = map_info
                        .claimed_entities
                        .get(&*position)
                        .is_some_and(|claimed_entity| {
                            if let Ok(claimed_tile) = claimed_query.get(*claimed_entity) {
                                claimed_tile.owner.is_some()
                            } else {
                                false
                            }
                        });
                    if !is_position_claimed {
                        beam_resolved_writer.write(BeamResolved {
                            position: *position,
                            owner: beam.owner,
                        });
                    }
                    end_beam(&mut commands, beam_entity, beam.caster, &mut in_flight);
                    continue;
                }

                // Claimed tile check
                let is_next_already_claimed = map_info
                    .claimed_entities
                    .get(&next_position)
                    .is_some_and(|claimed_entity| {
                        if let Ok(claimed_tile) = claimed_query.get(*claimed_entity) {
                            claimed_tile.owner.is_some()
                        } else {
                            false
                        }
                    });

                if is_next_already_claimed
                    && border_grinder_flips_tile(
                        &map_info,
                        &claimed_query,
                        &ability_query,
                        beam.owner,
                        beam.caster,
                        next_position,
                    )
                {
                    ability_writer.write(AbilityTriggered {
                        owner: beam.caster,
                        ability: AbilityDescriptor::BorderGrinder,
                    });
                    beam_resolved_writer.write(BeamResolved {
                        position: next_position,
                        owner: beam.owner,
                    });
                    end_beam(&mut commands, beam_entity, beam.caster, &mut in_flight);
                    continue;
                }

                if is_next_already_claimed {
                    // Move back to the last unclaimed position in case it's a forbidden area
                    while map_info.on_forbidden_areas(*position) {
                        *position -= beam.direction;
                    }
                    let is_position_claimed = map_info
                        .claimed_entities
                        .get(&*position)
                        .is_some_and(|claimed_entity| {
                            if let Ok(claimed_tile) = claimed_query.get(*claimed_entity) {
                                claimed_tile.owner.is_some()
                            } else {
                                false
                            }
                        });
                    if !is_position_claimed {
                        beam_resolved_writer.write(BeamResolved {
                            position: *position,
                            owner: beam.owner,
                        });
                    }
                    end_beam(&mut commands, beam_entity, beam.caster, &mut in_flight);
                    continue;
                }

                // Advance
                *position = next_position;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_ecs_tiled::prelude::{TilemapGridSize, TilemapSize, TilemapTileSize};
    use std::time::Duration;

    /// A 4x1 strip of ground with one beam at x = 2 moving left, owned by the first player.
    fn parry_world() -> (App, Entity, Entity, Entity) {
        let mut app = App::new();
        app.add_message::<ParryTriggered>();
        app.add_message::<BeamParried>();
        app.add_message::<BeamResolved>();
        app.add_message::<AbilityTriggered>();
        app.insert_resource(
            ron::from_str::<GameConfig>(include_str!("../../assets/game_config.ron")).unwrap(),
        );
        let mut map_info = MapInfo {
            map_size: TilemapSize::new(4, 1),
            grid_size: TilemapGridSize::new(16.0, 16.0),
            tile_size: TilemapTileSize::new(16.0, 16.0),
            ..default()
        };
        for x in 0..4 {
            map_info
                .ground_entities
                .insert(GridCoords { x, y: 0 }, Entity::PLACEHOLDER);
        }
        app.insert_resource(map_info);
        let shooter = app.world_mut().spawn_empty().id();
        let parrier = app.world_mut().spawn_empty().id();
        let beam = app
            .world_mut()
            .spawn((
                GridCoords { x: 2, y: 0 },
                Beam {
                    owner: shooter,
                    caster: shooter,
                    direction: GridCoords { x: -1, y: 0 },
                    parries: 0,
                    behavior: BeamBehavior::Straight,
                    step_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                    awaiting_step: false,
                    just_parried: false,
                },
            ))
            .id();
        (app, shooter, parrier, beam)
    }

    #[test]
    fn parried_beam_waits_a_step_and_names_the_previous_owner() {
        let (mut app, shooter, parrier, beam) = parry_world();
        app.world_mut()
            .write_message(ParryTriggered { parrier, beam });
        app.world_mut().run_system_once(resolve_parry).unwrap();

        let state = app.world().get::<Beam>(beam).unwrap();
        assert!(state.awaiting_step);
        assert_eq!(state.step_timer.elapsed_secs(), 0.0);
        let parried: Vec<_> = app
            .world()
            .resource::<Messages<BeamParried>>()
            .iter_current_update_messages()
            .map(|message| (message.previous_owner, message.parrier))
            .collect();
        assert_eq!(parried, vec![(shooter, parrier)]);

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        app.insert_resource(time);
        // The parry frame is longer than a step, and the beam still stays.
        app.world_mut().run_system_once(beam_step).unwrap();
        assert!(app.world().get::<Beam>(beam).unwrap().awaiting_step);
        assert_eq!(
            app.world().get::<GridCoords>(beam),
            Some(&GridCoords { x: 2, y: 0 })
        );

        app.world_mut().run_system_once(beam_step).unwrap();

        assert!(!app.world().get::<Beam>(beam).unwrap().awaiting_step);
        assert_eq!(
            app.world().get::<GridCoords>(beam),
            Some(&GridCoords { x: 3, y: 0 })
        );
    }

    #[test]
    fn a_waiting_beam_cannot_be_parried() {
        let (mut app, _, parrier, beam) = parry_world();
        let probe = move |beams: Query<(Entity, &GridCoords, &Beam), Without<Character>>,
                          map_info: Res<MapInfo>,
                          claimed: Query<&ClaimedTile>| {
            find_parryable_beam(
                GridCoords { x: 1, y: 0 },
                GridCoords { x: 1, y: 0 },
                parrier,
                &beams,
                &map_info,
                &claimed,
                0.75,
            )
        };
        assert_eq!(app.world_mut().run_system_once(probe).unwrap(), Some(beam));

        app.world_mut().get_mut::<Beam>(beam).unwrap().awaiting_step = true;
        assert_eq!(app.world_mut().run_system_once(probe).unwrap(), None);
    }
}
