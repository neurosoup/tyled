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
    app.add_systems(PreUpdate, attach_players_actions);
    app.add_systems(
        Update,
        (handle_characters_input, tick_turning).run_if(in_state(RoundPhase::Playing)),
    );
}

/// Per-character auto-repeat state driving the tap-vs-hold movement behaviour.
#[derive(Component, Default)]
struct MoveRepeat {
    timer: Timer,
    held_axis: Vec2,
    moving: bool,
}

fn attach_players_actions(
    mut commands: Commands,
    config: Res<GameConfig>,
    players: Query<(Entity, &Player), (Added<Player>, Without<InputMap<Action>>, With<Character>)>,
) {
    for (entity, player) in &players {
        commands.entity(entity).insert((
            Action::default_input_map(&player),
            MoveRepeat::default(),
            MovementSlide {
                duration_ms: config.timing.move_repeat_rate_ms,
            },
        ));
    }
}

fn handle_characters_input(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut characters: Query<
        (
            Entity,
            &ActionState<Action>,
            &GridCoords,
            &mut LookDirection,
            &mut MoveRepeat,
            &mut MovementSlide,
            Option<&BeamCharges>,
            Option<&IsTurning>,
        ),
        (With<Character>, Without<IsKnockedBack>),
    >,
    mut entity_moved_writer: MessageWriter<EntityMoved>,
    mut beam_fired_writer: MessageWriter<BeamFired>,
) {
    for (
        entity,
        action_state,
        grid_coords,
        mut look_direction,
        mut move_repeat,
        mut movement_slide,
        beam_charges,
        turning,
    ) in &mut characters
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

        // Released: clear the repeat state so the next press steps immediately. Runs
        // before turn handling so releasing mid-turn leaves no queued step. If a move
        // was in flight, ease its final slide out to rest instead of stopping abruptly.
        let axis = action_state.clamped_axis_pair(&Action::Move);
        if axis == Vec2::ZERO {
            if move_repeat.moving {
                commands.entity(entity).insert(MovementSettle);
            }
            move_repeat.held_axis = Vec2::ZERO;
            move_repeat.moving = false;
            continue;
        }

        let from_rest = move_repeat.held_axis == Vec2::ZERO;

        // On a detected facing change (while unlocked), commit the new facing immediately.
        if let Some(desired) = look_direction.would_look_at(axis) {
            let current_target = turning.map(|t| t.target).or(look_direction.direction);
            if let Some(from) = turning.map(|t| t.from).or(look_direction.direction)
                && Some(desired) != current_target
                && desired != from
            {
                look_direction.direction = Some(desired);
                commands
                    .entity(entity)
                    .insert(IsTurning::new(from, desired, config.timing.turn_step_ms));
                // Turning out of rest turns in place without stepping: hold the step back so a
                // quick release just turns, while continuing to hold starts walking after the
                // delay. While already moving, fall through so mid-run turns keep moving.
                if from_rest {
                    move_repeat.held_axis = axis;
                    move_repeat.timer = Timer::new(
                        Duration::from_millis(config.timing.move_repeat_delay_ms),
                        TimerMode::Once,
                    );
                    continue;
                }
            }
        }

        // Stepping out of rest is immediate (no lag); once moving, every step — including a
        // change of direction — waits for the delay, then repeats at the rate.
        let should_step = if from_rest {
            true
        } else {
            move_repeat.timer.tick(time.delta());
            move_repeat.timer.is_finished()
        };

        if should_step {
            let direction = GridCoords::new(axis.x as i32, axis.y as i32);
            entity_moved_writer.write(EntityMoved {
                entity,
                position: *grid_coords + direction,
            });

            // The slide lasts until the next step is due, so the first tile glides straight
            // into the cruise with no idle gap: the first move out of rest spans the longer
            // delay, later steps run at the shorter repeat rate. Slides are linear; the only
            // easing is the ease-out on stop, applied on release via MovementSettle.
            let base_ms = if move_repeat.moving {
                config.timing.move_repeat_rate_ms
            } else {
                config.timing.move_repeat_delay_ms
            };
            // A cardinal step covers less ground than a diagonal, so shorten its interval by
            // 1/√2 to match the diagonal's apparent (world) speed.
            let interval = if axis.x == 0.0 || axis.y == 0.0 {
                (base_ms as f32 * std::f32::consts::FRAC_1_SQRT_2).round() as u64
            } else {
                base_ms
            };
            *movement_slide = MovementSlide { duration_ms: interval };
            move_repeat.held_axis = axis;
            move_repeat.moving = true;
            move_repeat.timer = Timer::new(Duration::from_millis(interval), TimerMode::Once);
        }
    }
}

fn tick_turning(mut commands: Commands, time: Res<Time>, mut turners: Query<(Entity, &mut IsTurning)>) {
    for (entity, mut turning) in &mut turners {
        turning.timer.tick(time.delta());
        if !turning.timer.is_finished() {
            continue;
        }

        if let Some(reached) = turning.remaining.pop_front() {
            turning.from = reached;
        }

        if turning.remaining.is_empty() {
            commands.entity(entity).remove::<IsTurning>();
        } else {
            turning.timer.reset();
        }
    }
}
