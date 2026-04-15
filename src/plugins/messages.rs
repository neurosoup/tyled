use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<EntityMoved>();
    app.add_message::<BeamFired>();
    app.add_message::<BeamResolved>();
}

// Fired when an entity moved from one grid position to another
#[derive(Message)]
pub struct EntityMoved {
    pub entity: Entity,
    pub position: GridCoords,
}

#[derive(Message)]
pub struct BeamFired {
    pub owner: Entity,
    pub origin: GridCoords,
    pub direction: GridCoords,
}

#[derive(Message, Debug)]
pub struct BeamResolved {
    pub position: GridCoords,
    pub owner: Entity,
}

#[derive(Message)]
pub struct PlayerDied {
    pub player: Entity,
}
