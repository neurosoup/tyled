use std::ops::Add;

use crate::prelude::*;
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, claim_tile);
}

fn claim_tile(
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    mut claimed_query: Query<&mut ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        if let Some(claimed_entity) = map_info
            .claimed_entities
            .get(&tile_claimed_message.position)
        {
            if let Ok(mut claimed_tile) = claimed_query.get_mut(*claimed_entity) {
                claimed_tile.owner = Some(tile_claimed_message.owner);
            }
        }
    }
}
