/*
 * Plugin for beam behavior and claim tile when beam is resolved.
 */
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_beam_step_timer);
    app.add_systems(Update, (spawn_beam, beam_step, claim_tile));
}

#[derive(Resource)]
pub struct BeamStepTimer(Timer);

fn setup_beam_step_timer(mut commands: Commands) {
    commands.insert_resource(BeamStepTimer(Timer::from_seconds(
        0.0625,
        TimerMode::Repeating,
    )));
}

/// System that shakes unclaimed tile entities in response to [`BeamFired`] messages.
fn spawn_beam(
    mut commands: Commands,
    mut beam_fired_reader: MessageReader<BeamFired>,
    beams_query: Query<&Beam>,
) {
    for beam_fired_message in beam_fired_reader.read() {
        let owner_has_active_beam = beams_query
            .iter()
            .any(|beam| beam.owner == beam_fired_message.owner);

        let mut entity_commands = commands.spawn((
            beam_fired_message.origin,
            Beam {
                owner: beam_fired_message.owner,
                direction: beam_fired_message.direction,
                speed: 1.0,
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

pub(crate) fn beam_step(
    mut commands: Commands,
    mut beams_query: Query<(Entity, &Beam, &mut GridCoords)>,
    claimed_query: Query<&ClaimedTile>,
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

        // +--------------------------+
        // | Out of map bounds rule   |
        // +--------------------------+
        if !(map_info.on_ground(next_position) || map_info.on_forbidden_areas(next_position)) {
            // Move back to the last unclaimed position in case it's a forbidden area
            while map_info.on_forbidden_areas(*position) {
                *position -= beam.direction;
            }
            let is_position_claimed =
                map_info
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

        // +------------------------+
        // | Claimed tile check     |
        // +------------------------+
        let is_next_already_claimed =
            map_info
                .claimed_entities
                .get(&next_position)
                .is_some_and(|claimed_entity| {
                    if let Ok(claimed_tile) = claimed_query.get(*claimed_entity) {
                        claimed_tile.owner.is_some()
                    } else {
                        false
                    }
                });

        if is_next_already_claimed {
            // Move back to the last unclaimed position in case it's a forbidden area
            while map_info.on_forbidden_areas(*position) {
                *position -= beam.direction;
            }
            let is_position_claimed =
                map_info
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

fn claim_tile(
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    mut claimed_tiles: Query<&mut ClaimedTile>,
    players: Query<&Player, With<Character>>,
    mut beam_charges: Query<&mut BeamCharges>,
    mut charges_changed_writer: MessageWriter<BeamChargesChanged>,
    map_info: Res<MapInfo>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        // Query the player id
        let Ok(player) = players.get(tile_claimed_message.owner) else {
            panic!("Did not find the player associated with the claim")
        };

        if let Some(claimed_entity) = map_info
            .claimed_entities
            .get(&tile_claimed_message.position)
        {
            if let Ok(mut claimed_tile) = claimed_tiles.get_mut(*claimed_entity) {
                claimed_tile.owner = Some(tile_claimed_message.owner);

                if let Ok(mut charges) = beam_charges.get_mut(tile_claimed_message.owner) {
                    charges.current = charges.current.saturating_sub(1);
                    charges_changed_writer.write(BeamChargesChanged {
                        player_id: player.player_id,
                        current: charges.current,
                        max: charges.max,
                    });
                }
            }
        }
    }
}
