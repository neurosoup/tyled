/*
 * Applies damage to entities when they move onto a claimed tile owned by another entity.
 */
use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_timer);
    app.add_systems(Update, (apply_owned_tile_damage, apply_beam_damage));
}

#[derive(Resource)]
pub struct DamageTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(DamageTimer(Timer::from_seconds(
        0.500,
        TimerMode::Repeating,
    )));
}

fn apply_beam_damage(
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    damageables_query: Query<(Entity, &GridCoords, &mut Health)>,
    mut beams_query: Query<&Beam, Changed<GridCoords>>,
) {
}

fn apply_owned_tile_damage(
    time: Res<Time>,
    mut timer: ResMut<DamageTimer>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    mut characters: Query<(Entity, &GridCoords, &mut Health), With<Character>>,
    claimed_tiles: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    for (entity, position, mut health) in &mut characters {
        if health.current <= 0.0 {
            continue;
        }
        if let Some(claimed_entity) = map_info.get_claimed_entity_by_position(*position) {
            if let Ok(claimed_tile) = claimed_tiles.get(claimed_entity) {
                if claimed_tile.owner.is_some_and(|owner| owner != entity) {
                    health.current -= 1.0;
                    if health.current <= 0.0 {
                        damageable_died_writer.write(DamageableDied { entity });
                    }
                }
            }
        }
    }
}
