use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{Tween, TweenAnim, lens::TransformPositionLens};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (translate_objects, on_player_moved));
}

pub fn create_movement_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
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

fn translate_objects(
    mut moving_objects: Query<(&Transform, &GridCoords, &mut TweenAnim), Changed<GridCoords>>,
    map_info: Res<MapInfo>,
) {
    for (transform, grid_coords, mut anim) in &mut moving_objects {
        let destination = grid_coords.to_translation(&map_info, IVec2::new(24, 24));

        anim.set_tweenable(create_movement_tween(transform.translation, destination))
            .unwrap();
    }
}
