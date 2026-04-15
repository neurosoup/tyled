/*
 * Plugin that handles map loading and map-related resources and initializations.
 */
use std::time::Duration;

use crate::prelude::*;
use bevy::{
    camera::visibility::RenderLayers, platform::collections::HashMap, prelude::*, sprite::Anchor,
};
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default());

    app.init_resource::<MapInfo>();

    app.add_systems(Startup, load_maps);
    app.add_systems(
        Update,
        (
            initialize_map_info,
            (
                initialize_players,
                initialize_claimed_tiles,
                initialize_hp_bars,
            ),
        )
            .chain(),
    );
}

#[derive(Default, Resource, Debug, Clone)]
pub struct MapInfo {
    pub ground_entities: HashMap<GridCoords, Entity>,
    pub claimed_entities: HashMap<GridCoords, Entity>,
    pub forbidden_areas: HashMap<GridCoords, Entity>,
    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub tile_size: TilemapTileSize,
    pub map_type: TilemapType,
    pub map_anchor: TilemapAnchor,
    pub z_offset: f32,
}

impl MapInfo {
    pub fn on_ground(&self, grid_coords: GridCoords) -> bool {
        let tile_pos = TilePos::from(grid_coords);
        tile_pos.within_map_bounds(&self.map_size)
            && self.ground_entities.contains_key(&grid_coords)
    }

    pub fn on_forbidden_areas(&self, grid_coords: GridCoords) -> bool {
        let tile_pos = TilePos::from(grid_coords);
        tile_pos.within_map_bounds(&self.map_size)
            && self.forbidden_areas.contains_key(&grid_coords)
    }

    pub fn get_claimed_entity_by_position(&self, grid_coords: GridCoords) -> Option<Entity> {
        let tile_pos = TilePos::from(grid_coords);
        if tile_pos.within_map_bounds(&self.map_size) {
            self.claimed_entities.get(&grid_coords).copied()
        } else {
            None
        }
    }
}

fn load_maps(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading maps");

    commands.spawn((
        TiledMap(asset_server.load("level0.tmx")),
        CurrentLevel,
        TilemapAnchor::Center,
    ));

    commands.spawn((
        TiledMap(asset_server.load("hud.tmx")),
        HudMap,
        TilemapAnchor::Center,
    ));
}

fn initialize_map_info(
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    mut map_info: ResMut<MapInfo>,
    map_query: Query<&TiledMapLayerZOffset, (With<TiledMap>, With<CurrentLevel>)>,
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
    ground_tiles_query: Query<(Entity, &TilePos), (With<Ground>, Without<ForbiddenArea>)>,
    forbidden_areas_query: Query<(Entity, &TilePos), With<ForbiddenArea>>,
) {
    for map_created_message in map_created_reader.read() {
        // Skip maps that are not the current level
        let Ok(z_offset) = map_query.get(map_created_message.origin) else {
            continue;
        };
        let Some((_, tile_size, grid_size, map_size, map_type, map_anchor)) =
            tilemap_query.iter().find(|(name, ..)| name.0 == "Atlas")
        else {
            panic!("Atlas tilemap not found");
        };
        let ground_entities = ground_tiles_query
            .iter()
            .map(|(entity, tile_pos)| (GridCoords::from(*tile_pos), entity))
            .collect();

        let forbidden_areas = forbidden_areas_query
            .iter()
            .map(|(entity, tile_pos)| (GridCoords::from(*tile_pos), entity))
            .collect();

        *map_info = MapInfo {
            ground_entities,
            forbidden_areas,
            claimed_entities: HashMap::new(),
            map_size: *map_size,
            grid_size: *grid_size,
            tile_size: *tile_size,
            map_type: *map_type,
            map_anchor: *map_anchor,
            z_offset: z_offset.0,
        };
    }
}

fn initialize_hp_bars(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    map_info: Res<MapInfo>,
    map_query: Query<Entity, (With<TiledMap>, With<CurrentLevel>)>,
    hp_bars_query: Query<(Entity, &HPBar, &Transform, Option<&Children>)>,
    mut sprite_query: Query<&mut Sprite, Without<HPBar>>,
) {
    // Refers to the hud map where HP containers are located: 20 tiles wide by 16x32 tiles tall
    let hp_container_width = 20.0 * 16.0 - 4.0;
    let hp_container_height = 32.0;

    for map_created_message in map_created_reader.read() {
        // Skip maps that are not the current level
        let Ok(_) = map_query.get(map_created_message.origin) else {
            continue;
        };

        for (entity, hp_bar, transform, children) in &hp_bars_query {
            if let Some(grid_coords) =
                GridCoords::from_world_pos(&(transform.translation.truncate()), &map_info)
            {
                let player_one_offset = match hp_bar.player_id {
                    0 => Vec3::X,
                    _ => Vec3::ZERO,
                };

                commands.entity(entity).insert((
                    grid_coords,
                    Transform::from_translation(transform.translation + player_one_offset),
                ));

                if let Some(first_child) = children.and_then(|c| c.first()).copied() {
                    let anchor_x = 0.5;
                    let offset_direction = match hp_bar.player_id {
                        0 => 1.0,
                        1 => -1.0,
                        _ => 0.0,
                    };
                    commands
                        .entity(first_child)
                        .insert((Anchor::from(Vec2::new(anchor_x * offset_direction, -0.25)),));
                    if let Ok(mut sprite) = sprite_query.get_mut(first_child) {
                        sprite.custom_size =
                            Some(Vec2::new(hp_container_width, hp_container_height));
                    }
                }
            }
        }
    }
}

fn initialize_players(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    map_info: Res<MapInfo>,
    map_query: Query<Entity, (With<TiledMap>, With<CurrentLevel>)>,
    players_query: Query<(Entity, &Player, &Transform)>,
    children_query: Query<&Children>,
) {
    for map_created_message in map_created_reader.read() {
        // Skip maps that are not the current level
        let Ok(_) = map_query.get(map_created_message.origin) else {
            continue;
        };

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
                    TranslateEffectTarget,
                    DamageEffectTarget,
                    Health {
                        current: 20.0,
                        max: 100.0,
                    },
                ));

                if let Ok(children) = children_query.get(entity) {
                    if let Some(&first_child) = children.first() {
                        commands
                            .entity(first_child)
                            .insert(Anchor::from(Vec2::new(-0.05, -0.33)));
                    }
                }
            }
        }
    }
}

fn initialize_claimed_tiles(
    mut commands: Commands,
    mut map_created_reader: MessageReader<TiledEvent<MapCreated>>,
    mut map_info: ResMut<MapInfo>,
    map_query: Query<Entity, (With<TiledMap>, With<CurrentLevel>)>,
) {
    for map_created_message in map_created_reader.read() {
        // Skip maps that are not the current level
        let Ok(_) = map_query.get(map_created_message.origin) else {
            continue;
        };

        // Collect keys first to avoid holding a borrow on map_info
        let grid_coords_list: Vec<_> = map_info.ground_entities.keys().copied().collect();

        for grid_coords in grid_coords_list {
            let tile_transform =
                grid_coords.to_translation_with_z_index(&map_info, CLAIMED_TILE_Z_INDEX);
            let entity = commands
                .spawn((
                    Name::new("Unclaimed"),
                    ClaimedTile { owner: None },
                    WaveEffectTarget,
                    grid_coords,
                    Transform::from_translation(tile_transform),
                    Anchor::from(Vec2::new(-0.02, 0.18)),
                ))
                .id();

            map_info.claimed_entities.insert(grid_coords, entity);
        }
    }
}
