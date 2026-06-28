use bevy::prelude::*;
use bevy::window::PresentMode;

pub(crate) fn plugin(app: &mut App) {
    let plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        });

    app.add_plugins(plugins);
}
