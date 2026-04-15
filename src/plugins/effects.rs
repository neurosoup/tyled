/*
 * This plugin handles effects applied to entities on the map.
 * For example, movement effects are applied to entities based on their current position and a target position when their GridCoords component changed.
 */
use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_tweening::{Tween, TweenAnim, Tweenable, lens::TransformPositionLens};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_translate_effect,
            apply_wave_effect,
            apply_bounce_effect,
            apply_damage_effect,
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

fn apply_damage_effect(
    mut commands: Commands,
    map_info: Res<MapInfo>,
    damaged_players: Query<(Entity, &Health, &Player), With<DamageEffectTarget>>,
    mut hp_bars: Query<(&HPBar, &GridCoords, &mut Transform)>,
) {
    for (player_entity, health, player) in &damaged_players {
        // Resize HP bars regarding player health
        for (hp_bar, grid_coords, mut transform) in &mut hp_bars {
            if hp_bar.player_id == player.player_id {
                let translation = grid_coords.to_translation(&map_info);
                let direction = match player.player_id {
                    0 => -1.0,
                    1 => 1.0,
                    _ => 0.0,
                };
                // transform.scale.x =  direction * (health.current / health.max);
                transform.translation = translation;
            }
        }

        // Despawn players who have 0 health
        if health.current <= 0.0 {
            commands.entity(player_entity).despawn();
        }
    }
}

fn apply_wave_effect(
    mut commands: Commands,
    mut waves_query: Query<(&GridCoords, &BounceEffect), Changed<GridCoords>>,
    mut effect_targets: Query<(Entity, &GridCoords), With<WaveEffectTarget>>,
    map_info: Res<MapInfo>,
) {
    for (targeted_coords, shaker_effect) in &mut waves_query {
        let Some(claimed_entity) = map_info.claimed_entities.get(targeted_coords) else {
            continue;
        };
        if let Ok((entity, grid_coords)) = effect_targets.get_mut(*claimed_entity) {
            let BounceEffect {
                intensity,
                bounce_count,
                decay,
            } = *shaker_effect;
            commands
                .entity(entity)
                .insert(TweenAnim::new(create_bounce_sequence(
                    //TODO: The z-index should be based on the entity's wave effect target z-index.
                    grid_coords.to_translation_with_z_index(&map_info, CLAIMED_TILE_Z_INDEX),
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
        } = *bounce_effect;
        commands
            .entity(entity)
            .insert(TweenAnim::new(create_bounce_sequence(
                grid_coords.to_translation_with_z_index(&map_info, CLAIMED_TILE_Z_INDEX),
                intensity,
                bounce_count,
                decay,
            )))
            .remove::<BounceEffectTarget>();
    }
}
