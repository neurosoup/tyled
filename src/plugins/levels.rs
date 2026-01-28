use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub const GRID_SIZE: i32 = 16;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);

    app.insert_resource(LevelSelection::index(0));
    app.init_resource::<LevelLookup>();

    app.register_ldtk_entity::<PlayerBundle>("Player1");
    app.register_ldtk_entity::<PlayerBundle>("Player2");
    app.register_ldtk_int_cell::<GroundCellBundle>(1);

    app.add_systems(Startup, load_ltdk_project);
    app.add_systems(PreUpdate, build_level_lookup);
}

fn load_ltdk_project(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tyled.ldtk").into(),
        ..Default::default()
    });
}

fn build_level_lookup(
    mut level_lookup: ResMut<LevelLookup>,
    mut level_messages: MessageReader<LevelEvent>,
    ground_cells: Query<&GridCoords, With<Ground>>,
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

            let ground_locations = ground_cells.iter().copied().collect();

            let new_level_lookup = LevelLookup {
                ground_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_lookup = new_level_lookup;
        }
    }
    Ok(())
}
