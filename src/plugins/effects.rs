use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_tweening::{Tween, TweenAnim, Tweenable, lens::TransformPositionLens};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (apply_translate_effect, apply_wave_effect));
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

        // anim.set_tweenable(create_movement_tween(transform.translation, destination))
        //     .unwrap();
    }
}

fn apply_wave_effect(
    mut commands: Commands,
    mut waves_query: Query<(&GridCoords, &WaveEffect), Changed<GridCoords>>,
    mut claimed_query: Query<(Entity, &GridCoords), With<WaveEffectTarget>>,
    map_info: Res<MapInfo>,
) {
    for (targeted_coords, shaker_effect) in &mut waves_query {
        let Some(claimed_entity) = map_info.claimed_entities.get(targeted_coords) else {
            continue;
        };
        if let Ok((entity, grid_coords)) = claimed_query.get_mut(*claimed_entity) {
            let WaveEffect {
                intensity,
                bounce_count,
                decay,
            } = *shaker_effect;
            commands
                .entity(entity)
                .insert(TweenAnim::new(create_bounce_sequence(
                    grid_coords.to_translation_with_z_index(&map_info, CLAIMED_TILE_Z_INDEX),
                    intensity,
                    bounce_count,
                    decay,
                )));
        }
    }
}
