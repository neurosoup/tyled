use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use bevy_tweening::{Tween, TweenAnim, lens::TransformPositionLens};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, attach_player_movement_tween);
    app.add_systems(Update, translate_from_grid_coords);
}

fn create_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
}

fn attach_player_movement_tween(
    mut commands: Commands,
    players: Query<(Entity, &GridCoords, &Transform), Added<Player>>,
) {
    for (entity, grid_coords, transform) in &players {
        let initial_pos = grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
            .extend(transform.translation.z);

        commands
            .entity(entity)
            .insert((TweenAnim::new(create_tween(initial_pos, initial_pos))
                .with_destroy_on_completed(false),));
    }
}

fn translate_from_grid_coords(
    mut messages: MessageWriter<PlayerMovedEvent>,
    mut grid_coords_entities: Query<
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
    for (entity, transform, grid_coords, mut anim, player) in &mut grid_coords_entities {
        if player.is_some() {
            messages.write(PlayerMovedEvent {
                player: entity,
                to_grid_position: *grid_coords,
            });
        }
        let destination = grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
            .extend(transform.translation.z);
        anim.set_tweenable(create_tween(transform.translation, destination))
            .unwrap();
    }
}
