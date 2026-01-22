#![allow(unused_imports)]

use bevy::prelude::*;

mod components;
mod plugins;

/// Use this module instead of importing the `components`, `plugins`, `resources`, and `utils`
/// modules directly.
mod prelude {
    pub use super::*;
    pub(crate) use {components::*, plugins::*};
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(plugins::defaults::plugin);
        app.add_plugins(plugins::world::plugin);
        app.add_plugins(plugins::camera::plugin);
        app.add_plugins(plugins::character_controller::plugin);
        app.add_plugins(plugins::animations::plugin);
    }
}
