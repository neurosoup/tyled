use std::time::Duration;

use crate::prelude::*;
use bevy::{prelude::*, transform::commands};
use bevy_ecs_ldtk::{prelude::*, utils::*};
use bevy_spritesheet_animation::prelude::*;
use bevy_tweening::{
    AnimTarget, EntityCommandsTweeningExtensions, Tween, TweenAnim, TweenState, TweeningPlugin,
    lens::TransformPositionLens,
};
use leafwing_input_manager::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
    app.add_plugins(TweeningPlugin);
    app.add_systems(Startup, setup_input_timer);
    app.add_systems(PreUpdate, attach_players_actions);
    app.add_systems(Update, handle_players_input);
}

#[derive(Resource)]
pub struct InputTimer(Timer);

fn setup_input_timer(mut commands: Commands) {
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
    mut input_timer: ResMut<InputTimer>,
    mut players: Query<(&ActionState<Action>, &mut GridCoords, &mut LookDirection), With<Player>>,
    level_lookup: Res<LevelLookup>,
) {
    input_timer.0.tick(time.delta());

    for (action_state, mut player_grid_coords, mut look_direction) in &mut players {
        if action_state.just_pressed(&Action::Lock) {
            look_direction.toggle_lock();
        }

        if !input_timer.0.is_finished() {
            return;
        }

        if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
            let axis = action_state.clamped_axis_pair(&Action::Move);
            look_direction.look_at(axis);
            let direction = GridCoords::new(axis.x as i32, axis.y as i32);
            let destination = *player_grid_coords + direction;
            if level_lookup.on_ground(&destination) {
                *player_grid_coords = destination;
            }
        }
    }
}
