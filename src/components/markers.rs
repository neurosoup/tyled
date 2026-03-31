use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ForbiddenArea;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Ground;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct HUD;

#[derive(Component)]
pub struct WaveEffect {
    pub intensity: f32,
    pub bounce_count: usize,
    pub decay: f32,
}

#[derive(Component)]
pub struct TranslateEffectTarget;

#[derive(Component)]
pub struct WaveEffectTarget;
