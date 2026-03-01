use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_beam, beam_step, on_tile_claimed));
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

fn beam_step(
    mut commands: Commands,
    mut beams: Query<(Entity, &mut Beam)>,
    map_info: Res<MapInfo>,
    mut tile_claimed_writer: MessageWriter<TileClaimed>,
    mut beam_moved_writer: MessageWriter<BeamMoved>,
) {
    for (beam_entity, mut beam) in &mut beams {
        let next_position = beam.head + beam.direction;

        // Out of map bounds rule
        if !map_info.on_ground(next_position) {
            tile_claimed_writer.write(TileClaimed {
                position: next_position,
                owner: beam.owner,
            });
            commands.entity(beam_entity).despawn();
            continue;
        }

        // Advance
        beam.head = next_position;
        beam_moved_writer.write(BeamMoved {
            owner: beam.owner,
            position: next_position,
        });
    }
}

fn on_tile_claimed(
    mut commands: Commands,
    mut tile_claimed_reader: MessageReader<TileClaimed>,
    map_info: Res<MapInfo>,
) {
    for tile_claimed_message in tile_claimed_reader.read() {
        if let Some(tile_pos) = tile_claimed_message.position.to_tile_pos(&map_info) {
            if let Some(tile_entity) = map_info.ground_entities.get(&tile_pos) {
                commands.entity(*tile_entity).insert(ClaimedTile {
                    owner: tile_claimed_message.owner,
                });
            }
        }
    }
}
