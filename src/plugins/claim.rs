use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_inspector_egui::egui::Grid;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (on_tile_claimed, spawn_claimed_tiles).chain());
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

fn spawn_claimed_tiles(
    mut commands: Commands,
    claimed_tiles_query: Query<(&ClaimedTile, &TilePos), Added<ClaimedTile>>,
    players_query: Query<&Player>,
    asset_server: Res<AssetServer>,
    map_info: Res<MapInfo>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (claimed_tile, tile_pos) in &claimed_tiles_query {
        info!("Claimed tile: {:?}", claimed_tile);
        if let Ok(player) = players_query.get(claimed_tile.owner) {
            let texture: Handle<Image> = asset_server.load("grid_tiles2-Sheet.png");
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 1, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            info!("Claimed tile: {:?}", tile_pos);
            let tile_transform = tile_pos
                .center_in_world(
                    &map_info.map_size,
                    &map_info.grid_size,
                    &map_info.tile_size,
                    &map_info.map_type,
                    &map_info.map_anchor,
                )
                .extend(-0.01);
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
                            _ => 0,
                        },
                    },
                ),
            ));
        }
    }
}
