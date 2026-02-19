use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub const GRID_SIZE: i32 = 16;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(TiledPlugin::default());

    app.init_resource::<LevelLookup>();

    app.add_systems(Startup, load_level);
    app.add_systems(Update, evt_object_created);
    // app.add_systems(PreUpdate, build_level_lookup);
}

fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("level0.tmx")),
        TilemapAnchor::Center,
    ));
}

fn evt_object_created(
    mut object_events: MessageReader<TiledEvent<ObjectCreated>>,
    mut object_query: Query<(&Name, &Transform), With<TiledObject>>,
) {
    for e in object_events.read() {
        let Ok((name, transform)) = object_query.get_mut(e.origin) else {
            return;
        };
        info!("=> Received TiledObjectCreated event for object '{}'", name);
        info!("Transform = {:?}", transform);
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
