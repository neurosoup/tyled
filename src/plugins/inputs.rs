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
    app.add_systems(Startup, setup_input_timer);
    app.add_systems(PreUpdate, attach_players_actions);
    app.add_systems(
        Update,
        (handle_characters_input, tick_turning).run_if(in_state(RoundPhase::Playing)),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, resync_input_timer);
}

#[derive(Resource)]
pub struct InputTimer(Timer);

fn setup_input_timer(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(InputTimer(Timer::from_seconds(
        config.timing.input_tick_secs,
        TimerMode::Repeating,
    )));
}

#[cfg(feature = "dev")]
fn resync_input_timer(config: Res<GameConfig>, timer: Option<ResMut<InputTimer>>) {
    if config.is_changed()
        && let Some(mut timer) = timer
    {
        timer
            .0
            .set_duration(Duration::from_secs_f32(config.timing.input_tick_secs));
    }
}

fn attach_players_actions(
    mut commands: Commands,
    players: Query<(Entity, &Player), (Added<Player>, Without<InputMap<Action>>, With<Character>)>,
) {
    for (entity, player) in &players {
        commands
            .entity(entity)
            .insert((Action::default_input_map(&player),));
    }
}

fn handle_characters_input(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut timer: ResMut<InputTimer>,
    mut characters: Query<
        (
            Entity,
            &ActionState<Action>,
            &GridCoords,
            &mut LookDirection,
            Option<&BeamCharges>,
            Option<&IsTurning>,
        ),
        (With<Character>, Without<IsKnockedBack>),
    >,
    mut entity_moved_writer: MessageWriter<EntityMoved>,
    mut beam_fired_writer: MessageWriter<BeamFired>,
) {
    timer.0.tick(time.delta());

    for (entity, action_state, grid_coords, mut look_direction, beam_charges, turning) in
        &mut characters
    {
        if action_state.pressed(&Action::Lock) {
            look_direction.lock();
        } else {
            look_direction.unlock();
        }

        if action_state.just_pressed(&Action::Shoot) {
            let has_charges = beam_charges.map_or(true, |c| !c.is_empty());
            if has_charges {
                beam_fired_writer.write(BeamFired {
                    owner: entity,
                    origin: *grid_coords,
                    direction: look_direction.to_grid_coords(),
                });
            }
        }

        if action_state.axis_pair(&Action::Move) != Vec2::ZERO {
            let axis = action_state.clamped_axis_pair(&Action::Move);

            // On a detected facing change (while unlocked), start or restart a turn
            // and skip movement this frame: the character turns in place first.
            if let Some(desired) = look_direction.would_look_at(axis) {
                let current_target = turning.map(|t| t.target).or(look_direction.direction);
                if let Some(from) = turning.map(|t| t.from).or(look_direction.direction)
                    && Some(desired) != current_target
                    && desired != from
                {
                    commands
                        .entity(entity)
                        .insert(IsTurning::new(from, desired, config.timing.turn_step_ms));
                    continue;
                }
            }

            // Same direction, but a turn is still finishing — keep movement blocked.
            if turning.is_some() {
                continue;
            }

            if timer.0.is_finished() {
                look_direction.look_at(axis);
                let direction = GridCoords::new(axis.x as i32, axis.y as i32);
                let position = *grid_coords + direction;
                entity_moved_writer.write(EntityMoved { entity, position });
            }
        }
    }
}

fn tick_turning(
    mut commands: Commands,
    time: Res<Time>,
    mut turners: Query<(Entity, &mut LookDirection, &mut IsTurning)>,
) {
    for (entity, mut look_direction, mut turning) in &mut turners {
        turning.timer.tick(time.delta());
        if !turning.timer.is_finished() {
            continue;
        }

        if let Some(reached) = turning.remaining.pop_front() {
            turning.from = reached;
        }

        if turning.remaining.is_empty() {
            // Commit the new facing only once the turn finishes; until then
            // LookDirection stays at the original heading so mid-turn shots fire there.
            look_direction.direction = Some(turning.target);
            commands.entity(entity).remove::<IsTurning>();
        } else {
            turning.timer.reset();
        }
    }
}
