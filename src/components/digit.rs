use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Digit {
    pub position: u8,
    pub value: u8,
}
