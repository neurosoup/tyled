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
pub struct CurrentLevel;
