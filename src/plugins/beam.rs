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
    app.add_systems(Startup, (setup_beam_step_timer, setup_solar_panels_timer));
    app.add_systems(
        Update,
        (
            spawn_beam,
            beam_step,
            spend_charge_on_fire,
            regen_charges_from_solar_panels,
        )
            .run_if(in_state(RoundPhase::Playing)),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, (resync_beam_step_timer, resync_solar_panels_timer));
}

#[derive(Resource)]
pub struct BeamStepTimer(Timer);

fn setup_beam_step_timer(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(BeamStepTimer(Timer::from_seconds(
        config.timing.beam_step_secs,
        TimerMode::Repeating,
    )));
}

#[cfg(feature = "dev")]
fn resync_beam_step_timer(config: Res<GameConfig>, timer: Option<ResMut<BeamStepTimer>>) {
    if config.is_changed()
        && let Some(mut timer) = timer
    {
        timer.0.set_duration(std::time::Duration::from_secs_f32(
            config.timing.beam_step_secs,
        ));
    }
}

#[derive(Resource)]
pub struct SolarPanelsTimer(Timer);

fn setup_solar_panels_timer(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(SolarPanelsTimer(Timer::from_seconds(
        config.economy.solar_panels_tick_secs,
        TimerMode::Repeating,
    )));
}

#[cfg(feature = "dev")]
fn resync_solar_panels_timer(config: Res<GameConfig>, timer: Option<ResMut<SolarPanelsTimer>>) {
    if config.is_changed()
        && let Some(mut timer) = timer
    {
        timer.0.set_duration(std::time::Duration::from_secs_f32(
            config.economy.solar_panels_tick_secs,
        ));
    }
}

fn regen_charges_from_solar_panels(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut solar_panels_timer: ResMut<SolarPanelsTimer>,
    mut players: Query<(Entity, &AbilityList, &ClaimedTileCount, &mut BeamCharges)>,
    mut charge_regen_writer: MessageWriter<ChargeRegen>,
) {
    solar_panels_timer.0.tick(time.delta());
    if !solar_panels_timer.0.is_finished() {
        return;
    }
    for (entity, abilities, tile_count, mut charges) in &mut players {
        if !abilities.0.contains(&AbilityDescriptor::SolarPanels) {
            continue;
        }
        let gained = tile_count.current / config.economy.solar_panels_tiles_per_charge;
        if gained == 0 {
            continue;
        }
        charges.current = (charges.current + gained).min(charges.max);
        charge_regen_writer.write(ChargeRegen {
            owner: entity,
            amount: gained,
        });
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
    match (is_position_claimed(map_info, claimed_query, origin), can_override_block) {
        (true, false) => None,
        (true, true) => Some(BeamBehavior::Lance),
        (false, _) => Some(BeamBehavior::Straight),
    }
}

/// System that shakes unclaimed tile entities in response to [`BeamFired`] messages.
fn spawn_beam(
    mut commands: Commands,
    mut beam_fired_reader: MessageReader<BeamFired>,
    beams_query: Query<(&Beam, &GridCoords)>,
    ability_query: Query<&AbilityList>,
    claimed_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    for beam_fired_message in beam_fired_reader.read() {
        let owner_has_active_beam = beams_query.iter().any(|(beam, coords)| {
            if beam.owner != beam_fired_message.owner {
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

        let mut entity_commands = commands.spawn((
            beam_fired_message.origin,
            Beam {
                owner: beam_fired_message.owner,
                direction: beam_fired_message.direction,
                speed: 1.0,
                behavior,
            },
        ));
        if !owner_has_active_beam {
            entity_commands.insert(BounceEffect {
                intensity: 5.0,
                bounce_count: 2,
                decay: 0.3,
                z_index: CLAIMED_TILE_Z_INDEX,
            });
        }
    }
}

/// Whether `position` is claimed by an entity other than `owner` and `owner`'s
/// `AbilityList` contains [`AbilityDescriptor::Overpenetration`] — i.e. the beam should
/// punch through this enemy tile instead of stopping short of it.
fn overpenetrates_enemy_tile(
    map_info: &MapInfo,
    claimed_query: &Query<&ClaimedTile>,
    ability_query: &Query<&AbilityList>,
    owner: Entity,
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
            .get(owner)
            .is_ok_and(|list| list.0.contains(&AbilityDescriptor::Overpenetration))
}

pub(crate) fn beam_step(
    mut commands: Commands,
    mut beams_query: Query<(Entity, &Beam, &mut GridCoords)>,
    claimed_query: Query<&ClaimedTile>,
    ability_query: Query<&AbilityList>,
    time: Res<Time>,
    mut beam_step_timer: ResMut<BeamStepTimer>,
    map_info: Res<MapInfo>,
    mut beam_resolved_writer: MessageWriter<BeamResolved>,
) {
    beam_step_timer.0.tick(time.delta());
    if !beam_step_timer.0.is_finished() {
        return;
    }
    for (beam_entity, beam, mut position) in &mut beams_query {
        let next_position = *position + beam.direction;

        match beam.behavior {
            // +----------------------------+
            // | Lance                   |
            // | resolve on the first       |
            // | unclaimed tile ahead.      |
            // +----------------------------+
            BeamBehavior::Lance => {
                if !(map_info.on_ground(next_position)
                    || map_info.on_forbidden_areas(next_position))
                {
                    commands.entity(beam_entity).despawn();
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
                    commands.entity(beam_entity).despawn();
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
                    commands.entity(beam_entity).despawn();
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
                    && overpenetrates_enemy_tile(
                        &map_info,
                        &claimed_query,
                        &ability_query,
                        beam.owner,
                        next_position,
                    )
                {
                    beam_resolved_writer.write(BeamResolved {
                        position: next_position,
                        owner: beam.owner,
                    });
                    commands.entity(beam_entity).despawn();
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
                    commands.entity(beam_entity).despawn();
                    continue;
                }

                // Advance
                *position = next_position;
            }
        }
    }
}

// Spend one charge per committed shot at fire time (not on resolve). A shot
// that finds nothing to claim can still cost a charge — e.g. a Lance beam
// that reaches the map edge without finding an unclaimed tile. Each
// `BeamFired` spawns exactly one beam, so this is exactly one charge per shot.
fn spend_charge_on_fire(
    mut beam_fired_reader: MessageReader<BeamFired>,
    mut beam_charges: Query<&mut BeamCharges>,
    mut charge_spent_writer: MessageWriter<ChargeSpent>,
) {
    for message in beam_fired_reader.read() {
        if let Ok(mut charges) = beam_charges.get_mut(message.owner) {
            charges.current = charges.current.saturating_sub(1);
            charge_spent_writer.write(ChargeSpent {
                owner: message.owner,
                amount: 1,
            });
        }
    }
}
