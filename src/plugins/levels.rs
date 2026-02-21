use crate::prelude::*;
use bevy::{ecs::name, prelude::*};
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_tiled::prelude::*;

pub const GRID_SIZE: i32 = 16;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default());

    app.init_resource::<LevelLookup>();

    app.add_systems(Startup, load_level);
    app.add_systems(Update, attach_tile_pos_to_objects);
    // app.add_systems(PreUpdate, build_level_lookup);
}

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("level0.tmx")),
        TilemapAnchor::Center,
    ));
}

fn attach_tile_pos_to_objects(
    mut commands: Commands,
    mut object_events: MessageReader<TiledEvent<ObjectCreated>>,
    object_query: Query<(&Transform, &ChildOf), With<TiledObject>>,
    layer_query: Query<&ChildOf, With<TiledLayer>>,
    tilemap_query: Query<
        (
            &TilemapTileSize,
            &TilemapGridSize,
            &TilemapSize,
            &TilemapType,
            &TilemapAnchor,
            &TiledMapReference,
        ),
        With<TiledTilemap>,
    >,
) {
    for e in object_events.read() {
        // Get the object's transform and child_of components
        let Ok((transform, child_of)) = object_query.get(e.origin) else {
            continue;
        };

        // Navigate parent hierarchy: object → object layer → map
        let layer_entity = child_of.parent();
        let Ok(layer_child_of) = layer_query.get(layer_entity) else {
            continue;
        };
        let map_entity = layer_child_of.parent();

        // Find the first tilemap belonging to the same map as this object
        let Some((tile_size, grid_size, map_size, tilemap_type, tilemap_anchor, _)) = tilemap_query
            .iter()
            .find(|(.., map_ref)| map_ref.0 == map_entity)
        else {
            continue;
        };

        // Get the tile position under the object and insert it into the entity
        if let Some(tile_pos) = TilePos::from_world_pos(
            &transform.translation.truncate(),
            map_size,
            grid_size,
            tile_size,
            tilemap_type,
            tilemap_anchor,
        ) {
            commands.entity(e.origin).insert(TilePos::from(tile_pos));
        }
    }
}

// fn build_level_lookup(
//     mut level_lookup: ResMut<LevelLookup>,
//     mut level_messages: MessageReader<LevelEvent>,
//     ground_cells: Query<&GridCoords, With<Ground>>,
//     ldtk_project_entities: Query<&LdtkProjectHandle>,
//     ldtk_project_assets: Res<Assets<LdtkProject>>,
// ) -> Result {
//     for level_event in level_messages.read() {
//         if let LevelEvent::Spawned(level_iid) = level_event {
//             let ldtk_project = ldtk_project_assets
//                 .get(ldtk_project_entities.single()?)
//                 .expect("LdtkProject should be loaded when level is spawned");
//             let level = ldtk_project
//                 .get_raw_level_by_iid(level_iid.get())
//                 .expect("spawned level should exist in project");

//             let ground_locations = ground_cells.iter().copied().collect();

//             let new_level_lookup = LevelLookup {
//                 ground_locations,
//                 level_width: level.px_wid / GRID_SIZE,
//                 level_height: level.px_hei / GRID_SIZE,
//             };

//             *level_lookup = new_level_lookup;
//         }
//     }
//     Ok(())
// }
