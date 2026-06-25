#![cfg(feature = "dev")]

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};
use bevy_smooth_pixel_camera::components::ViewportCamera;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new());
    // Prevent bevy_egui from auto-assigning to the first camera (which renders to an off-screen
    // texture via PixelCamera). We manually target ViewportCamera instead, which composites the
    // final image to the full window.
    app.insert_resource(EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    });
    // app.add_systems(Update, attach_egui_to_viewport_camera);

    // Tiled debug
    // app.add_plugins(TiledDebugPluginGroup);
}

fn attach_egui_to_viewport_camera(
    mut commands: Commands,
    viewport_cameras: Query<Entity, Added<ViewportCamera>>,
) {
    for entity in &viewport_cameras {
        commands.entity(entity).insert(PrimaryEguiContext);
    }
}
