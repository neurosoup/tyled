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
    app.add_systems(Update, apply_damage);
}

#[derive(Resource)]
pub struct DamageTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(DamageTimer(Timer::from_seconds(
        0.500,
        TimerMode::Repeating,
    )));
}

fn apply_damage(
    time: Res<Time>,
    mut timer: ResMut<DamageTimer>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    player_entities: Query<(Entity, &GridCoords), With<Player>>,
    mut damageable_entities: Query<&mut Health>,
    claimed_entities: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    // Apply damage to damageable entities that are in claimed areas but not owned by the player
    for (entity, position) in &player_entities {
        if let Some(claimed_entity) = map_info.get_claimed_entity_by_position(*position) {
            if let Ok(claimed_tile) = claimed_entities.get(claimed_entity) {
                if claimed_tile.owner.is_some_and(|owner| owner != entity) {
                    if let Ok(mut health) = damageable_entities.get_mut(entity) {
                        if health.current <= 0.0 {
                            return;
                        }
                        health.current -= 1.0;
                        if health.current <= 0.0 {
                            damageable_died_writer.write(DamageableDied { entity });
                        }
                    }
                }
            }
        }
    }
}
