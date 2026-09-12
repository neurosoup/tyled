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

// Show the amount of damage taken
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct DamageBar;

/// Marker attached in Tiled to each HUD bar showing a player's claimed-tile
/// share of the board. Fills left→right for P1, right→left for P2.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TerritoryBar;

/// Marker attached in Tiled to each HUD bar showing a player's claimed-tiles
/// plus beam-charges share of the board. Same mirrored fill direction as
/// `TerritoryBar`.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ChargesBar;

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
