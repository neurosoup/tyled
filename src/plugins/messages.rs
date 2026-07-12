use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<EntityMoved>();
    app.add_message::<BeamFired>();
    app.add_message::<BeamResolved>();
    app.add_message::<TileClaimed>();
    app.add_message::<ChargeSpent>();
    app.add_message::<ChargeRegen>();
    app.add_message::<DamageableDied>();
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

// Fired when a tile's owner actually changes (a real flip), distinguishing it
// from a no-op resolve on an already-owned tile.
#[allow(dead_code)] // read by ability resolvers in Stage F2
#[derive(Message, Debug)]
pub struct TileClaimed {
    pub position: GridCoords,
    pub old_owner: Option<Entity>,
    pub new_owner: Entity,
}

// Fired when a player spends beam charges (currently one per shot, at fire time).
#[allow(dead_code)] // read by economy-ability resolvers in Stage F2
#[derive(Message, Debug)]
pub struct ChargeSpent {
    pub owner: Entity,
    pub amount: u32,
}

// Fired when a player regains beam charges. No emitter exists yet — the first
// regen source is Solar Panels (Slice 1). Declared now as part of the substrate.
#[allow(dead_code)] // emitted + read starting Slice 1 (Solar Panels)
#[derive(Message, Debug)]
pub struct ChargeRegen {
    pub owner: Entity,
    pub amount: u32,
}

#[derive(Message)]
pub struct DamageableDied {
    pub entity: Entity,
}
