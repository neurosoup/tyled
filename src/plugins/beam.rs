use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_beam_step_timer);
    app.add_systems(Update, (spawn_beam, beam_step));
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
    players_query: Query<&Player>,
) {
    for beam_fired_message in beam_fired_reader.read() {
        if let Ok(_) = players_query.get(beam_fired_message.owner) {
            commands.spawn((
                beam_fired_message.origin,
                Beam {
                    owner: beam_fired_message.owner,
                    direction: beam_fired_message.direction,
                    speed: 1.0,
                },
                WaveEffect {
                    intensity: 2.0,
                    bounce_count: 5,
                    decay: 0.5,
                },
            ));
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
        if !map_info.on_ground(next_position) {
            info!("Beam stops at: {:?}", *position);
            beam_resolved_writer.write(BeamResolved {
                position: *position,
                owner: beam.owner,
            });
            commands.entity(beam_entity).despawn();
            continue;
        }

        // +------------------------+
        // | Claimed tile check     |
        // +------------------------+
        let is_next_already_claimed = map_info
            .ground_entities
            .get(&next_position)
            .is_some_and(|tile_entity| claimed_query.get(*tile_entity).is_ok());
        if is_next_already_claimed {
            beam_resolved_writer.write(BeamResolved {
                position: *position,
                owner: beam.owner,
            });
            commands.entity(beam_entity).despawn();
            continue;
        }

        // Advance
        *position = next_position;
    }
}
