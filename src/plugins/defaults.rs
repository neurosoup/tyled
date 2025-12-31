use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
}
