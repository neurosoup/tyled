use crate::prelude::*;
use bevy::{ecs::name, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::TweenAnim;
use std::collections::HashSet;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default());

    app.init_resource::<MapLookup>();

    app.add_systems(Startup, load_map);
    app.add_systems(Update, on_map_created);
}

#[derive(Default, Resource)]
pub struct MapLookup {
    pub ground_locations: HashSet<TilePos>,
    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub tile_size: TilemapTileSize,
    pub map_type: TilemapType,
    pub map_anchor: TilemapAnchor,
    pub player_size: TilemapTileSize,
}

impl MapLookup {
    pub fn on_ground(&self, grid_coords: &GridCoords) -> bool {
        let tile_pos = TilePos::from(*grid_coords);
        tile_pos.within_map_bounds(&self.map_size) && self.ground_locations.contains(&tile_pos)
    }
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("level0.tmx")),
        TilemapAnchor::Center,
    ));
}

fn on_map_created(
    mut commands: Commands,
    mut messages: MessageReader<TiledEvent<MapCreated>>,
    mut map_lookup: ResMut<MapLookup>,
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
    ground_tiles_query: Query<&TilePos, With<Ground>>,
    players_query: Query<(Entity, &Player, &Transform), With<TiledObject>>,
) {
    for _ in messages.read() {
        let Some((_, tile_size, grid_size, map_size, map_type, map_anchor)) =
            tilemap_query.iter().find(|(name, ..)| name.0 == "MapTiles")
        else {
            continue;
        };

        // Initialize map lookup
        let ground_locations = ground_tiles_query.iter().copied().collect();
        *map_lookup = MapLookup {
            ground_locations,
            map_size: *map_size,
            grid_size: *grid_size,
            tile_size: *tile_size,
            map_type: *map_type,
            map_anchor: *map_anchor,
            player_size: TilemapTileSize::new(24.0, 24.0),
        };

        // Initialize players
        for (entity, player, transform) in &players_query {
            let look_direction = LookDirection::new(match player.player_id {
                0 => Direction::Down,
                1 => Direction::Up,
                _ => Direction::Down,
            });

            if let Some(grid_coords) =
                GridCoords::from_world_pos(&(transform.translation.truncate()), &map_lookup)
            {
                commands.entity(entity).insert((
                    grid_coords,
                    look_direction,
                    (TweenAnim::new(create_movement_tween(
                        transform.translation,
                        transform.translation,
                    ))
                    .with_destroy_on_completed(false),),
                ));
            }
        }
    }
}
