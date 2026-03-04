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
            let translation =
                beam_fired_message.origin.to_translation(&map_info) + Vec3::new(0.0, 0.0, -0.1);

            let texture: Handle<Image> = asset_server.load("grid_tiles2-Sheet.png");
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
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
                            0 => 6,
                            1 => 7,
                            _ => 6,
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
    mut tile_claimed_writer: MessageWriter<TileClaimed>,
) {
    for (beam_entity, beam, mut position) in &mut beams_query {
        let next_position = *position + beam.direction;

        // Out of map bounds rule
        if !map_info.on_ground(next_position) {
            info!("Beam stops at: {:?}", *position);
            tile_claimed_writer.write(TileClaimed {
                position: *position,
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
                        position: *position,
                        owner: beam.owner,
                    });
                    commands.entity(beam_entity).despawn();
                    continue;
                }
            }
        }

        // Advance
        *position = next_position;
    }
}
