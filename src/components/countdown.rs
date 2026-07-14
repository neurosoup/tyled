use bevy::prelude::*;

/// Marks a `Digit` sprite that displays the round countdown timer. Unlike the
/// per-player counter markers, countdown digits are not tied to a `Player`.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct CountdownDigit;
