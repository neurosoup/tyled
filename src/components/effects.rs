use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component)]
pub struct BounceEffect {
    pub intensity: f32,
    pub bounce_count: usize,
    pub decay: f32,
}

#[derive(Component)]
pub struct BounceEffectTarget;

#[derive(Component)]
pub struct WaveEffectTarget;

#[derive(Component)]
pub struct TranslateEffectTarget;
