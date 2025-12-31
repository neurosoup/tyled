use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);
    app.insert_resource(LevelSelection::index(0));
    app.add_systems(Startup, load_ltdk_world);
}

fn load_ltdk_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("tyled.ldtk").into(),
        ..Default::default()
    });
}
