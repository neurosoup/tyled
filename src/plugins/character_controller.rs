use crate::prelude::{actions::*, player::*, walkable::*, world::*};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
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
    mut players: Query<(Entity, &mut Player), (Added<Player>, Without<InputMap<Action>>)>,
) {
    for (entity, player) in &mut players {
        commands
            .entity(entity)
            .insert(Action::default_input_map(&player));
    }
}

fn move_player_from_input(
    mut players: Query<(&ActionState<Action>, &mut GridCoords), With<Player>>,
    level_walkables: Res<LevelWalkables>,
) {
    for (action_state, mut player_grid_coords) in &mut players {
        if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
            let axis = action_state.clamped_axis_pair(&Action::Move);
            let direction = GridCoords::new(axis.x as i32, axis.y as i32);
            let destination = *player_grid_coords + direction;
            if level_walkables.in_walkable(&destination) {
                *player_grid_coords = destination;
            }
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
