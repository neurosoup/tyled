use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::TweenAnim;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default());

    app.init_resource::<MapInfo>();

    app.add_systems(Startup, load_map);
    app.add_systems(Update, (initialize_map_info, initialize_players).chain());
}

#[derive(Default, Resource)]
pub struct MapInfo {
    pub ground_entities: HashMap<TilePos, Entity>,
    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub tile_size: TilemapTileSize,
    pub map_type: TilemapType,
    pub map_anchor: TilemapAnchor,
}

impl MapInfo {
    pub fn on_ground(&self, grid_coords: GridCoords) -> bool {
        let tile_pos = TilePos::from(grid_coords);
        tile_pos.within_map_bounds(&self.map_size) && self.ground_entities.contains_key(&tile_pos)
    }
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("level0.tmx")),
        TilemapAnchor::Center,
    ));
}

fn initialize_map_info(
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    mut map_info: ResMut<MapInfo>,
    tilemap_query: Query<
        (
            &TiledName,
            &TilemapTileSize,
            &TilemapGridSize,
            &TilemapSize,
            &TilemapType,
            &TilemapAnchor,
        ),
        With<TiledTilemap>,
    >,
    ground_tiles_query: Query<(Entity, &TilePos), With<Ground>>,
) {
    for _ in map_created_reader.read() {
        let Some((_, tile_size, grid_size, map_size, map_type, map_anchor)) =
            tilemap_query.iter().find(|(name, ..)| name.0 == "MapTiles")
        else {
            continue;
        };

        let ground_entities = ground_tiles_query
            .iter()
            .map(|(entity, tile_pos)| (*tile_pos, entity))
            .collect();

        *map_info = MapInfo {
            ground_entities,
            map_size: *map_size,
            grid_size: *grid_size,
            tile_size: *tile_size,
            map_type: *map_type,
            map_anchor: *map_anchor,
        };
    }
}

fn initialize_players(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    map_info: Res<MapInfo>,
    players_query: Query<(Entity, &Player, &Transform), With<TiledObject>>,
    children_query: Query<&Children>,
) {
    for _ in map_created_reader.read() {
        for (entity, player, transform) in &players_query {
            let look_direction = LookDirection::new(match player.player_id {
                0 => Direction::Down,
                1 => Direction::Up,
                _ => Direction::Down,
            });

            if let Some(grid_coords) =
                GridCoords::from_world_pos(&(transform.translation.truncate()), &map_info)
            {
                commands.entity(entity).insert((
                    grid_coords,
                    look_direction,
                    TweenAnim::new(create_movement_tween(
                        transform.translation,
                        transform.translation,
                    ))
                    .with_destroy_on_completed(false),
                ));

                if let Ok(children) = children_query.get(entity) {
                    if let Some(&first_child) = children.first() {
                        commands
                            .entity(first_child)
                            .insert(Anchor::from(Vec2::new(0.0, 0.0)));
                    }
                }
            }
        }
    }
}
