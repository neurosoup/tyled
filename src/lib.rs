#![allow(unused_imports)]

use bevy::prelude::*;

mod components;
mod plugins;

/// Use this module instead of importing the `components`, `plugins`, `resources`, and `utils`
/// modules directly.
mod prelude {
    // pub use super::*;
    pub use crate::{components::*, plugins::*};
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(plugins::defaults::plugin);
        app.add_plugins(plugins::messages::plugin);
        app.add_plugins(plugins::maps::plugin);
        app.add_plugins(plugins::camera::plugin);
        app.add_plugins(plugins::inputs::plugin);
        app.add_plugins(plugins::movements::plugin);
        app.add_plugins(plugins::animations::plugin);
        app.add_plugins(plugins::beam::plugin);
        app.add_plugins(plugins::claim::plugin);
        app.add_plugins(plugins::debug::plugin);
    }
}
