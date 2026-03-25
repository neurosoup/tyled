use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::TweenAnim;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_beam, beam_step));
}

/// System that spawns [`Beam`] entities in response to [`BeamFired`] messages.
fn spawn_beam(
    mut commands: Commands,
    mut beam_fired_reader: MessageReader<BeamFired>,
    players_query: Query<&Player>,
    map_info: Res<MapInfo>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for beam_fired_message in beam_fired_reader.read() {
        if let Ok(player) = players_query.get(beam_fired_message.owner) {
            let translation = beam_fired_message
                .origin
                .to_translation_with_z_index(&map_info, -2);

            let texture: Handle<Image> = asset_server.load("claimed-tiles.png");
            let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 32), 32, 1, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            commands.spawn((
                beam_fired_message.origin,
                Transform::from_translation(translation),
                Beam {
                    owner: beam_fired_message.owner,
                    direction: beam_fired_message.direction,
                    speed: 1.0,
                },
                Sprite::from_atlas_image(
                    texture,
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: match player.player_id {
                            0 => 0,
                            1 => 1,
                            _ => 0,
                        },
                    },
                ),
                TweenAnim::new(create_movement_tween(translation, translation))
                    .with_destroy_on_completed(false),
            ));
        }
    }
}

pub(crate) fn beam_step(
    mut commands: Commands,
    mut beams_query: Query<(Entity, &Beam, &mut GridCoords)>,
    claimed_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
    mut beam_resolved_writer: MessageWriter<BeamResolved>,
) {
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
        // | Colored tile check     |
        // +------------------------+
        let is_next_already_claimed = next_position
            .to_tile_pos(&map_info)
            .and_then(|tile_pos| map_info.ground_entities.get(&tile_pos))
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
