use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, on_player_moved);
}

fn on_player_moved(
    mut player_moved_reader: MessageReader<PlayerMoved>,
    mut players: Query<&mut GridCoords, With<Player>>,
    map_info: Res<MapInfo>,
) {
    for player_moved_message in player_moved_reader.read() {
        let entity = player_moved_message.player;
        let position = player_moved_message.position;

        if map_info.on_ground(position) {
            if let Ok(mut player_grid_coords) = players.get_mut(entity) {
                *player_grid_coords = position;
            }
        }
    }
}
