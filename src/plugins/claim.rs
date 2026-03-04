use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, on_tile_claimed);
}

fn on_tile_claimed(
    mut commands: Commands,
    mut tile_claimed_reader: MessageReader<TileClaimed>,
    players_query: Query<&Player>,
    asset_server: Res<AssetServer>,
    map_info: Res<MapInfo>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for tile_claimed_message in tile_claimed_reader.read() {
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

        let texture: Handle<Image> = asset_server.load("grid_tiles2-Sheet.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let tile_transform = tile_pos
            .center_in_world(
                &map_info.map_size,
                &map_info.grid_size,
                &map_info.tile_size,
                &map_info.map_type,
                &map_info.map_anchor,
            )
            .extend(-0.1);

        commands.spawn((
            Name::new(format!("Player {}", player.player_id)),
            Transform::from_translation(tile_transform),
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
        ));
    }
}
