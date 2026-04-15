/*
 * This plugin handles effects applied to entities on the map.
 * For example, movement effects are applied to entities based on their current position and a target position when their GridCoords component changed.
 */
use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_tweening::{
    AnimCompletedEvent, CycleCompletedEvent, Tween, TweenAnim, Tweenable,
    lens::TransformPositionLens,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_translate_effect,
            apply_wave_effect,
            apply_bounce_effect,
            apply_death_effect,
            on_death_effect_completed,
        ),
    );
}

pub fn create_movement_tween(start: Vec3, end: Vec3) -> Tween {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        TransformPositionLens { start, end },
    )
}

pub fn create_bounce_sequence(
    origin: Vec3,
    initial_intensity: f32,
    bounce_count: usize,
    decay: f32,
) -> impl Tweenable {
    let make_bounce = |height: f32, duration: Duration| {
        Tween::new(
            EaseFunction::QuadraticOut,
            duration,
            TransformPositionLens {
                start: origin,
                end: origin + Vec3::Y * height,
            },
        )
        .then(Tween::new(
            EaseFunction::BounceOut,
            duration,
            TransformPositionLens {
                start: origin + Vec3::Y * height,
                end: origin,
            },
        ))
    };

    (0..bounce_count)
        .map(|i| {
            let height = initial_intensity * decay.powi(i as i32);
            let duration = Duration::from_millis((300.0 * decay.powi(i as i32)) as u64);
            make_bounce(height, duration)
        })
        .reduce(|acc, next| acc.then(next))
        .unwrap()
}

fn apply_translate_effect(
    mut commands: Commands,
    mut moving_objects: Query<
        (Entity, &Transform, &GridCoords),
        (Changed<GridCoords>, With<TranslateEffectTarget>),
    >,
    map_info: Res<MapInfo>,
) {
    for (entity, transform, grid_coords) in &mut moving_objects {
        let destination = grid_coords.to_translation(&map_info);

        commands
            .entity(entity)
            .insert(TweenAnim::new(create_movement_tween(
                transform.translation,
                destination,
            )));
    }
}

// Reacts to PlayerDied event.
fn apply_death_effect(
    mut commands: Commands,
    mut damageable_died_reader: MessageReader<DamageableDied>,
    damageable_query: Query<&Health, With<DamageEffectTarget>>,
) {
    // Despawn players who have 0 health
    for damageable_died_message in damageable_died_reader.read() {
        if let Ok(_) = damageable_query.get(damageable_died_message.entity) {
            // Implement the death effect here
            commands.entity(damageable_died_message.entity).insert((
                BounceEffect {
                    intensity: 8.0,
                    bounce_count: 3,
                    decay: 0.33,
                    z_index: 1,
                },
                BounceEffectTarget,
                IsDead,
            ));
        }
    }
}

// Apply post-death effects associated with a death effect (e.g., despawn entities with IsDead)
fn on_death_effect_completed(
    mut commands: Commands,
    mut anim_completed_reader: MessageReader<AnimCompletedEvent>,
    dead_entities: Query<Entity, (With<IsDead>, With<BounceEffect>)>,
) {
    for anim_completed_message in anim_completed_reader.read() {
        if let Ok(entity) = dead_entities.get(anim_completed_message.anim_entity) {
            commands.entity(entity).despawn();
        }
    }
}

fn apply_wave_effect(
    mut commands: Commands,
    mut waves_query: Query<(&GridCoords, &BounceEffect), Changed<GridCoords>>,
    mut effect_targets: Query<(Entity, &GridCoords), With<WaveEffectTarget>>,
    map_info: Res<MapInfo>,
) {
    for (targeted_coords, bounce_effect) in &mut waves_query {
        let Some(claimed_entity) = map_info.claimed_entities.get(targeted_coords) else {
            continue;
        };
        if let Ok((entity, grid_coords)) = effect_targets.get_mut(*claimed_entity) {
            let BounceEffect {
                intensity,
                bounce_count,
                decay,
                z_index,
            } = *bounce_effect;
            commands
                .entity(entity)
                .insert(TweenAnim::new(create_bounce_sequence(
                    grid_coords.to_translation_with_z_index(&map_info, z_index),
                    intensity,
                    bounce_count,
                    decay,
                )));
        }
    }
}

fn apply_bounce_effect(
    mut commands: Commands,
    mut bounce_query: Query<(Entity, &GridCoords, &BounceEffect), Added<BounceEffectTarget>>,
    map_info: Res<MapInfo>,
) {
    for (entity, grid_coords, bounce_effect) in &mut bounce_query {
        let BounceEffect {
            intensity,
            bounce_count,
            decay,
            z_index,
        } = *bounce_effect;
        commands
            .entity(entity)
            .insert(TweenAnim::new(create_bounce_sequence(
                grid_coords.to_translation_with_z_index(&map_info, z_index),
                intensity,
                bounce_count,
                decay,
            )))
            .remove::<BounceEffectTarget>();
    }
}
