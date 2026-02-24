use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{Tween, TweenAnim, lens::TransformPositionLens};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, translate_objects);
}

pub fn create_movement_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
}

fn translate_objects(
    mut messages: MessageWriter<PlayerMovedEvent>,
    map_lookup: Res<MapLookup>,
    mut moving_objects: Query<
        (
            Entity,
            &Transform,
            &GridCoords,
            &mut TweenAnim,
            Option<&Player>,
        ),
        Changed<GridCoords>,
    >,
) {
    for (entity, transform, grid_coords, mut anim, player) in &mut moving_objects {
        info!(
            "Translating object {} to tile position {:?}",
            entity, grid_coords
        );
        let tile_pos = TilePos::from(*grid_coords);
        let destination = (tile_pos.center_in_world(
            &map_lookup.map_size,
            &map_lookup.grid_size,
            &map_lookup.tile_size,
            &map_lookup.map_type,
            &map_lookup.map_anchor,
        ) - Vec2::new(map_lookup.player_size.x, map_lookup.player_size.y) / 2.0)
            .extend(0.0);

        anim.set_tweenable(create_movement_tween(transform.translation, destination))
            .unwrap();

        if player.is_some() {
            messages.write(PlayerMovedEvent {
                player: entity,
                to_grid_coords: *grid_coords,
            });
        }
    }
}
