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
pub struct HPBar;

#[derive(Component, Default)]
pub struct CurrentLevel;

#[derive(Component)]
pub struct HudMap;

#[derive(Component)]
pub struct IsDead;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Character;

#[derive(Component, Default)]
pub struct Bot;
