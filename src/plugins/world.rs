use crate::prelude::{player::*, walkable::*};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub const GRID_SIZE: i32 = 16;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);

    app.insert_resource(LevelSelection::index(0));
    app.init_resource::<LevelWalkables>();

    app.register_ldtk_entity::<PlayerBundle>("Player1");
    app.register_ldtk_entity::<PlayerBundle>("Player2");
    app.register_ldtk_int_cell::<WalkableBundle>(1);

    app.add_systems(Startup, load_ltdk_world);
    app.add_systems(Update, cache_walkable_locations);
}

fn load_ltdk_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tyled.ldtk").into(),
        ..Default::default()
    });
}

fn cache_walkable_locations(
    mut level_walkables: ResMut<LevelWalkables>,
    mut level_messages: MessageReader<LevelEvent>,
    walkables: Query<&GridCoords, With<Walkable>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_messages.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single()?)
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");

            let walkable_locations = walkables.iter().copied().collect();

            let new_level_walkables = LevelWalkables {
                walkable_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_walkables = new_level_walkables;
        }
    }
    Ok(())
}
