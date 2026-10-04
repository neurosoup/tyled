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

/// Marks a tile that glows when a beam crosses it.
#[derive(Component)]
pub struct GlowEffectTarget;

/// Points a sprite entity or a tilemap cell entity at its lit overlay.
#[derive(Component)]
pub struct LitOverlayLink(pub Entity);

/// Marks the lit overlay that is a child of the tile sprite it lights.
#[derive(Component)]
pub struct SpriteLitOverlay;

/// Marks the lit overlay drawn over a tilemap cell.
#[derive(Component)]
pub struct TilemapLitOverlay;

/// The glow pulses currently active on an overlay.
#[derive(Component, Default)]
pub struct GlowPulses(pub Vec<GlowPulse>);

/// One glow pulse with a delay, a fade-in, a hold and a fade-out, in seconds.
#[derive(Clone, Copy, Debug)]
pub struct GlowPulse {
    pub elapsed: f32,
    pub delay: f32,
    pub peak: f32,
    pub fade_in: f32,
    pub hold: f32,
    pub fade_out: f32,
}

impl GlowPulse {
    /// The current strength, or `None` once the pulse has finished.
    pub fn alpha(&self) -> Option<f32> {
        let fade_in = self.fade_in.max(0.001);
        let fade_out = self.fade_out.max(0.001);
        let t = self.elapsed - self.delay;
        if t < 0.0 {
            return Some(0.0);
        }
        if t < fade_in {
            return Some(self.peak * EaseFunction::QuadraticOut.sample_clamped(t / fade_in));
        }
        let t = t - fade_in;
        if t < self.hold {
            return Some(self.peak);
        }
        let t = t - self.hold;
        if t < fade_out {
            return Some(self.peak * (1.0 - EaseFunction::QuadraticIn.sample_clamped(t / fade_out)));
        }
        None
    }
}

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

/// Locks input processing while an entity is knocked back.
#[derive(Component)]
pub struct IsKnockedBack(pub Timer);

/// Which effect currently owns an entity's `Transform` `TweenAnim` slot.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ActiveTransformEffect(pub TransformEffectKind);

/// Priority order (highest first): Bounce > Knockback > {Settle, Translate}.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransformEffectKind {
    Translate,
    Settle,
    Knockback,
    Bounce,
}

/// A death bounce that is waiting for an in-flight knockback slide to finish.
#[derive(Component)]
pub struct PendingDeathBounce;

/// Marks entities allowed to act as a wave source for `apply_wave_effect`.
#[derive(Component)]
pub struct WaveSource;

/// One-shot landed-parry scale punch, inserted on the parrier's root entity.
#[derive(Component)]
pub struct ParryScaleEffectTarget;

/// Drives the scale-punch tween on a sprite entity from a separate carrier entity.
#[derive(Component)]
pub struct ParryScaleDriver {
    pub sprite: Entity,
}

/// Drives the damage color-flash tween on a sprite entity from a separate carrier entity.
#[derive(Component)]
pub struct DamageFlashDriver {
    pub sprite: Entity,
}
