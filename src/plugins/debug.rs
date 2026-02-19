use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new());

    // Tiled debug
    app.add_plugins(TiledDebugPluginGroup);
}
