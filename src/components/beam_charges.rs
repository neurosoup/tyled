use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct BeamChargesDigit;

#[derive(Component)]
pub struct BeamCharges {
    pub current: u32,
    pub max: u32,
}

impl BeamCharges {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn is_empty(&self) -> bool {
        self.current == 0
    }
}
