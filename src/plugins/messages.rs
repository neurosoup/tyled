use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<PlayerMovedEvent>();
}

// Fired when a player moved from one grid position to another
#[derive(Message)]
pub struct PlayerMovedEvent {
    pub player: Entity,
    pub to_grid_position: GridCoords,
}
