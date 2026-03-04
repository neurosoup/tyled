use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_beam, beam_step));
}

/// System that spawns [`Beam`] entities in response to [`BeamFired`] messages.
fn spawn_beam(mut commands: Commands, mut beam_fired_reader: MessageReader<BeamFired>) {
    for beam_fired_message in beam_fired_reader.read() {
        commands.spawn(Beam {
            owner: beam_fired_message.owner,
            direction: beam_fired_message.direction,
            head: beam_fired_message.origin,
            speed: 1.0,
        });
    }
}

pub(crate) fn beam_step(
    mut commands: Commands,
    mut beams_query: Query<(Entity, &mut Beam)>,
    claimed_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
    mut tile_claimed_writer: MessageWriter<TileClaimed>,
    mut beam_moved_writer: MessageWriter<BeamMoved>,
) {
    for (beam_entity, mut beam) in &mut beams_query {
        let next_position = beam.head + beam.direction;

        // Out of map bounds rule
        if !map_info.on_ground(next_position) {
            info!("Beam stops at: {:?}", beam.head);
            tile_claimed_writer.write(TileClaimed {
                position: beam.head,
                owner: beam.owner,
            });
            commands.entity(beam_entity).despawn();
            continue;
        }

        // Colored tile check
        if let Some(tile_pos) = next_position.to_tile_pos(&map_info) {
            if let Some(tile_entity) = map_info.ground_entities.get(&tile_pos) {
                if let Ok(_) = claimed_query.get(*tile_entity) {
                    tile_claimed_writer.write(TileClaimed {
                        position: beam.head,
                        owner: beam.owner,
                    });
                    commands.entity(beam_entity).despawn();
                    continue;
                }
            }
        }

        // Advance
        beam.head = next_position;
        beam_moved_writer.write(BeamMoved {
            owner: beam.owner,
            position: next_position,
        });
    }
}
