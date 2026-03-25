use std::ops::Add;

use crate::prelude::*;
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, on_tile_claimed);
}

fn on_tile_claimed(
    mut commands: Commands,
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    players_query: Query<&Player>,
    asset_server: Res<AssetServer>,
    map_info: Res<MapInfo>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        let Some(tile_pos) = tile_claimed_message.position.to_tile_pos(&map_info) else {
            continue;
        };

        let Some(tile_entity) = map_info.ground_entities.get(&tile_pos) else {
            continue;
        };

        let Ok(player) = players_query.get(tile_claimed_message.owner) else {
            continue;
        };

        commands.entity(*tile_entity).insert(ClaimedTile {
            owner: tile_claimed_message.owner,
        });

        let texture: Handle<Image> = asset_server.load("claimed-tiles.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 32), 32, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let tile_transform = tile_pos
            .center_in_world(
                &map_info.map_size,
                &map_info.grid_size,
                &map_info.tile_size,
                &map_info.map_type,
                &map_info.map_anchor,
            )
            // Place at the same z-index as the tile
            .extend(-2.0 * map_info.z_offset);
        // + Vec3::new(0.0, 8.0, 0.0);

        commands.spawn((
            Name::new(format!("ClaimedBy {}", player.player_id)),
            Transform::from_translation(tile_transform),
            Anchor::from(Vec2::new(-0.04, -0.22)),
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
        ));
    }
}
