use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use super::grid_coords::GridCoords;

#[derive(Component)]
pub struct BounceEffect {
    pub intensity: f32,
    pub bounce_count: usize,
    pub decay: f32,
    pub z_index: i8,
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

/*
 * Per-step duration for the movement slide tween.
 * Set by the inputs plugin, read by apply_translate_effect.
 */
#[derive(Component, Clone, Copy)]
pub struct MovementSlide {
    pub duration_ms: u64,
}

/*
 * Requests an ease-out slide to the current tile when movement stops.
 * Inserted by the inputs plugin on release, consumed by apply_movement_settle.
 */
#[derive(Component)]
pub struct MovementSettle;

/// Stores the resting world position for entities whose Transform may be mid-tween.
/// Used by bounce/wave effects so they always return to the correct origin.
#[derive(Component)]
pub struct RestingTranslation(pub Vec3);

#[derive(Component)]
pub struct KnockbackEffect {
    pub direction: GridCoords,
}

#[derive(Component)]
pub struct IsKnockedBack;
