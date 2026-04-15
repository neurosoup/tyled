use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn ratio(&self) -> f32 {
        self.current / self.max
    }
}
