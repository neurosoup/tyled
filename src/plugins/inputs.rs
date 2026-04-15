/*
 * Plugin that handles player input and actions.
 */

use std::time::Duration;

use crate::prelude::*;
use bevy::{prelude::*, transform::commands};
use bevy_ecs_tiled::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use bevy_tweening::{
    AnimTarget, EntityCommandsTweeningExtensions, Tween, TweenAnim, TweenState, TweeningPlugin,
    lens::TransformPositionLens,
};
use leafwing_input_manager::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
    app.add_plugins(TweeningPlugin);
    app.add_systems(Startup, setup_timer);
    app.add_systems(PreUpdate, attach_players_actions);
    app.add_systems(Update, handle_players_input);
}

#[derive(Resource)]
pub struct InputTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(InputTimer(Timer::from_seconds(
        0.0625,
        TimerMode::Repeating,
    )));
}

fn attach_players_actions(
    mut commands: Commands,
    players: Query<(Entity, &Player), (Added<Player>, Without<InputMap<Action>>)>,
) {
    for (entity, player) in &players {
        commands
            .entity(entity)
            .insert((Action::default_input_map(&player),));
    }
}

fn handle_players_input(
    time: Res<Time>,
    mut timer: ResMut<InputTimer>,
    mut players: Query<
        (
            Entity,
            &ActionState<Action>,
            &GridCoords,
            &mut LookDirection,
        ),
        With<Player>,
    >,
    mut player_moved_writer: MessageWriter<EntityMoved>,
    mut beam_fired_writer: MessageWriter<BeamFired>,
) {
    timer.0.tick(time.delta());

    for (player_entity, action_state, player_grid_coords, mut look_direction) in &mut players {
        if action_state.just_pressed(&Action::Lock) {
            look_direction.toggle_lock();
        }

        if action_state.just_pressed(&Action::Shoot) {
            beam_fired_writer.write(BeamFired {
                owner: player_entity,
                origin: *player_grid_coords,
                direction: look_direction.to_grid_coords(),
            });
        }

        if !timer.0.is_finished() {
            continue;
        }

        if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
            let axis = action_state.clamped_axis_pair(&Action::Move);
            look_direction.look_at(axis);
            let direction = GridCoords::new(axis.x as i32, axis.y as i32);
            let position = *player_grid_coords + direction;
            player_moved_writer.write(EntityMoved {
                entity: player_entity,
                position,
            });
        }
    }
}
