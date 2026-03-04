use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<PlayerMoved>();
    app.add_message::<BeamFired>();
    app.add_message::<TileClaimed>();
}

// Fired when a player moved from one grid position to another
#[derive(Message)]
pub struct PlayerMoved {
    pub player: Entity,
    pub position: GridCoords,
}

#[derive(Message)]
pub struct BeamFired {
    pub owner: Entity,
    pub origin: GridCoords,
    pub direction: GridCoords,
}

#[derive(Message, Debug)]
pub struct TileClaimed {
    pub position: GridCoords,
    pub owner: Entity,
}
