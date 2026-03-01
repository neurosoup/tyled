use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{Tween, TweenAnim, lens::TransformPositionLens};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (translate_objects, move_player));
}

pub fn create_movement_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
}

fn move_player(
    mut messages: MessageReader<PlayerMoved>,
    mut players: Query<&mut GridCoords, With<Player>>,
    map_info: Res<MapInfo>,
) {
    for message in messages.read() {
        let entity = message.player;
        let position = message.position;

        if map_info.on_ground(position) {
            if let Ok(mut player_grid_coords) = players.get_mut(entity) {
                *player_grid_coords = position;
            }
        }
    }
}

fn translate_objects(
    map_info: Res<MapInfo>,
    mut moving_objects: Query<(&Transform, &GridCoords, &mut TweenAnim), Changed<GridCoords>>,
) {
    for (transform, grid_coords, mut anim) in &mut moving_objects {
        let destination = grid_coords.to_translation(&map_info, IVec2::new(24, 24));

        anim.set_tweenable(create_movement_tween(transform.translation, destination))
            .unwrap();
    }
}
