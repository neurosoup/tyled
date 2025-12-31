use crate::components::{key_mapping::*, player::*};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (move_players_from_input, translate_players_from_grid_coords),
    );
}

const GRID_SIZE: i32 = 16;

fn move_players_from_input(
    mut players: Query<(&mut GridCoords, &KeyMapping), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut player_grid_coords, key_mapping) in &mut players {
        let movement_direction = if input.just_pressed(key_mapping.up) {
            GridCoords::new(0, 1)
        } else if input.just_pressed(key_mapping.down) {
            GridCoords::new(0, -1)
        } else if input.just_pressed(key_mapping.left) {
            GridCoords::new(-1, 0)
        } else if input.just_pressed(key_mapping.right) {
            GridCoords::new(1, 0)
        } else {
            continue; // No input for this player, continue to next
        };

        let destination = *player_grid_coords + movement_direction;
        *player_grid_coords = destination;
    }
}

fn translate_players_from_grid_coords(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in &mut grid_coords_entities {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}
