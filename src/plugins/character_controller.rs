use crate::prelude::{key_mapping::*, player::*, walkable::*, world::*};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            attach_player_controls,
            move_player_from_input,
            translate_from_grid_coords,
        ),
    );
}

fn attach_player_controls(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player), (Added<Player>, Without<KeyMapping>)>,
) {
    for (entity, player) in &mut players {
        let key_mapping = match player.player_id {
            0 => KeyMapping::wasd(),
            1 => KeyMapping::arrow_keys(),
            _ => KeyMapping::default(),
        };

        commands.entity(entity).insert(key_mapping);
    }
}

fn move_player_from_input(
    mut players: Query<(&mut GridCoords, &KeyMapping), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    level_walkables: Res<LevelWalkables>,
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
        if level_walkables.in_walkable(&destination) {
            *player_grid_coords = destination;
        }
    }
}

fn translate_from_grid_coords(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in &mut grid_coords_entities {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}
