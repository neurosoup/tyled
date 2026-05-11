use crate::prelude::*;
use bevy::prelude::*;

pub struct Digits {
    pub count: u8,
    pub value: u32,
}

impl Digits {
    pub fn new(count: u8) -> Self {
        Self { count, value: 0 }
    }

    pub fn digit_at(&self, position: u8) -> u8 {
        let divisor = 10u32.pow(position as u32);
        ((self.value / divisor) % 10) as u8
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Digit {
    pub position: u8,
    pub value: u8,
}
