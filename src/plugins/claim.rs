use std::ops::Add;

use crate::prelude::*;
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, on_tile_claimed);
}

fn on_tile_claimed(
    mut commands: Commands,
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    players_query: Query<&Player>,
    asset_server: Res<AssetServer>,
    map_info: Res<MapInfo>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        let Some(tile_entity) = map_info.ground_entities.get(&tile_claimed_message.position) else {
            continue;
        };

        let Ok(player) = players_query.get(tile_claimed_message.owner) else {
            continue;
        };
    }
}
