use std::time::Duration;

use crate::prelude::{actions::*, player::*, walkable::*, world::*};
use bevy::{prelude::*, transform::commands};
use bevy_ecs_ldtk::{prelude::*, utils::*};
use bevy_tweening::{
    AnimTarget, EntityCommandsTweeningExtensions, Tween, TweenAnim, TweenState, TweeningPlugin,
    lens::TransformPositionLens,
};
use leafwing_input_manager::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
    app.add_plugins(TweeningPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            attach_player_controls,
            handle_movement_input,
            translate_from_grid_coords,
        ),
    );
}

#[derive(Resource)]
struct MoveTimer(Timer);

fn create_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
}

fn setup(mut commands: Commands) {
    commands.insert_resource(MoveTimer(Timer::from_seconds(0.0625, TimerMode::Repeating)));
}

fn attach_player_controls(
    mut commands: Commands,
    players: Query<
        (Entity, &GridCoords, &Transform, &Player),
        (Added<Player>, Without<InputMap<Action>>),
    >,
) {
    for (entity, grid_coords, transform, player) in &players {
        let initial_pos = grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
            .extend(transform.translation.z);

        commands.entity(entity).insert((
            Action::default_input_map(&player),
            TweenAnim::new(create_tween(initial_pos, initial_pos)).with_destroy_on_completed(false),
        ));
    }
}

fn handle_movement_input(
    time: Res<Time>,
    mut move_timer: ResMut<MoveTimer>,
    mut players: Query<(&ActionState<Action>, &mut GridCoords, &mut LookDirection), With<Player>>,
    level_walkables: Res<LevelWalkables>,
) {
    move_timer.0.tick(time.delta());

    for (action_state, mut player_grid_coords, mut look_direction) in &mut players {
        if action_state.just_pressed(&Action::Lock) {
            look_direction.toggle_lock();
        }

        if !move_timer.0.is_finished() {
            return;
        }

        if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
            let axis = action_state.clamped_axis_pair(&Action::Move);
            look_direction.look_at(axis);
            let direction = GridCoords::new(axis.x as i32, axis.y as i32);
            let destination = *player_grid_coords + direction;
            if level_walkables.in_walkable(&destination) {
                *player_grid_coords = destination;
            }
        }
    }
}

fn translate_from_grid_coords(
    mut grid_coords_entities: Query<(&Transform, &GridCoords, &mut TweenAnim), Changed<GridCoords>>,
) {
    for (transform, grid_coords, mut anim) in &mut grid_coords_entities {
        let destination = grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
            .extend(transform.translation.z);
        anim.set_tweenable(create_tween(transform.translation, destination))
            .unwrap();
    }
}
