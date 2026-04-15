/*
 * This plugin translates player input → movement on the map.
 */

use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, move_players);
}

fn move_players(
    mut entity_moved_reader: MessageReader<EntityMoved>,
    mut players: Query<&mut GridCoords, With<Player>>,
    map_info: Res<MapInfo>,
) {
    for entity_moved_message in entity_moved_reader.read() {
        let entity = entity_moved_message.entity;
        let position = entity_moved_message.position;

        if map_info.on_ground(position) {
            if let Ok(mut player_grid_coords) = players.get_mut(entity) {
                *player_grid_coords = position;
            }
        }
    }
}
