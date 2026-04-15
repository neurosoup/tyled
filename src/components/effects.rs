use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component)]
pub struct BounceEffect {
    pub intensity: f32,
    pub bounce_count: usize,
    pub decay: f32,
}

/*
 * Simple bounce effect target component.
 * Used in conjunction with BounceEffect component.
 */
#[derive(Component)]
pub struct BounceEffectTarget;

/*
 * Wave effect target component.
 * Used in conjunction with BounceEffect to create wave-like effects.
 */
#[derive(Component)]
pub struct WaveEffectTarget;

/*
 * Translate effect target component.
 * Used in conjunction with GridCoords component (Changed event).
 */
#[derive(Component)]
pub struct TranslateEffectTarget;

/*
 * Damage effect target component.
 * Used in conjunction with Health component (Changed event).
 */
#[derive(Component)]
pub struct DamageEffectTarget;
