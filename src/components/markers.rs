use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Ground;
