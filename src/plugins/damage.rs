/*
 * Applies damage to entities when they move onto a claimed tile owned by another entity.
 */
use std::time::Duration;

use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_timer);
    app.add_systems(
        Update,
        apply_knockback
            .in_set(GameplaySet::Displacement)
            .run_if(in_state(RoundPhase::Playing)),
    );
    app.add_systems(Update, tick_knockback_lock);
    app.add_systems(
        Update,
        (
            (apply_owned_tile_entry_damage, apply_owned_tile_damage).chain(),
            apply_beam_damage,
            apply_collision_damage,
        )
            .in_set(GameplaySet::Damage)
            .run_if(in_state(RoundPhase::Playing)),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, resync_damage_timer);
}

#[derive(Resource)]
pub struct DamageTimer(Timer);

fn setup_timer(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(DamageTimer(Timer::from_seconds(
        config.damage.tick_secs,
        TimerMode::Repeating,
    )));
}

#[cfg(feature = "dev")]
fn resync_damage_timer(config: Res<GameConfig>, timer: Option<ResMut<DamageTimer>>) {
    if config.is_changed()
        && let Some(mut timer) = timer
    {
        timer
            .0
            .set_duration(Duration::from_secs_f32(config.damage.tick_secs));
    }
}

/// Moves each knocked-back entity one tile.
fn apply_knockback(
    mut commands: Commands,
    mut requested_reader: MessageReader<KnockbackRequested>,
    mut knocked_back_writer: MessageWriter<EntityKnockedBack>,
    config: Res<GameConfig>,
    mut query: Query<(&mut GridCoords, Has<IsDead>, Option<&Health>)>,
    map_info: Res<MapInfo>,
) {
    let mut latest: EntityHashMap<GridCoords> = EntityHashMap::default();
    for request in requested_reader.read() {
        latest.insert(request.entity, request.direction);
    }
    for (entity, direction) in latest {
        let Ok((mut coords, is_dead, health)) = query.get_mut(entity) else {
            continue;
        };
        let target = *coords + direction;
        let is_lethal = health.is_some_and(|health| health.current <= 0.0);
        if is_dead || is_lethal || !map_info.on_ground(target) {
            continue;
        }
        *coords = target;
        commands.entity(entity).insert(IsKnockedBack(Timer::new(
            Duration::from_millis(config.effects.knockback_tween_ms),
            TimerMode::Once,
        )));
        knocked_back_writer.write(EntityKnockedBack { entity });
    }
}

/// Ticks the knockback input lock and removes it when it ends.
fn tick_knockback_lock(
    mut commands: Commands,
    time: Res<Time>,
    mut knocked: Query<(Entity, &mut IsKnockedBack)>,
) {
    for (entity, mut knockback) in &mut knocked {
        knockback.0.tick(time.delta());
        if knockback.0.is_finished() {
            commands.entity(entity).remove::<IsKnockedBack>();
        }
    }
}

fn deal_damage(
    entity: Entity,
    health: &mut Health,
    amount: f32,
    writer: &mut MessageWriter<DamageableDied>,
) {
    health.current -= amount;
    if health.current <= 0.0 {
        writer.write(DamageableDied { entity });
    }
}

fn apply_beam_damage(
    mut knockback_writer: MessageWriter<KnockbackRequested>,
    config: Res<GameConfig>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    mut damageables_query: Query<(Entity, &GridCoords, &mut Health)>,
    beams_query: Query<(&Beam, &GridCoords), Changed<GridCoords>>,
) {
    for (beam, beam_position) in &beams_query {
        for (entity, position, mut health) in &mut damageables_query {
            if health.current <= 0.0 {
                continue;
            }
            if position == beam_position && beam.owner != entity {
                deal_damage(
                    entity,
                    &mut health,
                    config.damage.beam_contact,
                    &mut damageable_died_writer,
                );
                knockback_writer.write(KnockbackRequested {
                    entity,
                    direction: beam.direction,
                });
            }
        }
    }
}

fn apply_collision_damage(
    mut collision_reader: MessageReader<CharacterCollision>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    config: Res<GameConfig>,
    mut characters_query: Query<&mut Health, With<Character>>,
) {
    for collision in collision_reader.read() {
        if let Ok(mut health) = characters_query.get_mut(collision.loser) {
            if health.current <= 0.0 {
                continue;
            }
            deal_damage(
                collision.loser,
                &mut health,
                config.damage.collision,
                &mut damageable_died_writer,
            );
        }
    }
}

/// Returns true when `position` holds a claimed tile owned by someone other than `character`.
pub(crate) fn is_hostile_tile(
    map_info: &MapInfo,
    claimed_tiles: &Query<&ClaimedTile>,
    position: GridCoords,
    character: Entity,
) -> bool {
    map_info
        .get_claimed_entity_by_position(position)
        .and_then(|claimed_entity| claimed_tiles.get(claimed_entity).ok())
        .is_some_and(|claimed_tile| {
            claimed_tile.owner.is_some_and(|owner| owner != character)
        })
}

/// Damages a character the moment it moves onto a hostile tile. Driven by change detection on
/// `GridCoords`, so it cannot phase-miss a tile the way the 500ms poll can when a player crosses
/// faster than the poll samples. Also fires when a character is knocked onto a hostile tile.
fn apply_owned_tile_entry_damage(
    config: Res<GameConfig>,
    mut timer: ResMut<DamageTimer>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    mut characters_query: Query<
        (Entity, &GridCoords, &mut PreviousGridCoords, &mut Health),
        (With<Character>, Changed<GridCoords>),
    >,
    claimed_tiles_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    for (entity, position, mut previous, mut health) in &mut characters_query {
        if health.current <= 0.0 {
            continue;
        }
        let came_from = previous.0;
        previous.0 = *position;
        // Only spike on a fresh incursion: don't re-charge a step taken entirely
        // within enemy territory.
        if is_hostile_tile(&map_info, &claimed_tiles_query, *position, entity)
            && !is_hostile_tile(&map_info, &claimed_tiles_query, came_from, entity)
        {
            deal_damage(
                entity,
                &mut health,
                config.damage.on_enter,
                &mut damageable_died_writer,
            );
            // Restart the standing clock so the poll below can't double-hit this same frame.
            timer.0.reset();
        }
    }
}

/// Damages characters that remain on a hostile tile, ticking every 500ms. Also covers the case
/// the entry system can't: a tile becoming hostile *beneath* a stationary player (their
/// `GridCoords` never changes, so no entry event fires).
fn apply_owned_tile_damage(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut timer: ResMut<DamageTimer>,
    mut damageable_died_writer: MessageWriter<DamageableDied>,
    mut characters_query: Query<(Entity, &GridCoords, &mut Health), With<Character>>,
    claimed_tiles_query: Query<&ClaimedTile>,
    map_info: Res<MapInfo>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    for (entity, position, mut health) in &mut characters_query {
        if health.current <= 0.0 {
            continue;
        }
        if is_hostile_tile(&map_info, &claimed_tiles_query, *position, entity) {
            deal_damage(
                entity,
                &mut health,
                config.damage.standing,
                &mut damageable_died_writer,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs_tiled::prelude::{TilemapGridSize, TilemapSize, TilemapTileSize};

    #[test]
    fn two_knockbacks_in_one_frame_move_one_tile_in_the_last_direction() {
        let mut app = App::new();
        app.add_message::<KnockbackRequested>();
        app.add_message::<EntityKnockedBack>();
        app.insert_resource(
            ron::from_str::<GameConfig>(include_str!("../../assets/game_config.ron")).unwrap(),
        );
        let mut map_info = MapInfo {
            map_size: TilemapSize::new(4, 4),
            grid_size: TilemapGridSize::new(16.0, 16.0),
            tile_size: TilemapTileSize::new(16.0, 16.0),
            ..default()
        };
        for x in 0..4 {
            for y in 0..4 {
                map_info
                    .ground_entities
                    .insert(GridCoords { x, y }, Entity::PLACEHOLDER);
            }
        }
        app.insert_resource(map_info);
        app.add_systems(Update, apply_knockback);
        let entity = app.world_mut().spawn(GridCoords { x: 1, y: 1 }).id();
        for direction in [GridCoords { x: 1, y: 0 }, GridCoords { x: 0, y: 1 }] {
            app.world_mut()
                .write_message(KnockbackRequested { entity, direction });
        }
        app.update();
        assert_eq!(
            app.world().get::<GridCoords>(entity),
            Some(&GridCoords { x: 1, y: 2 })
        );
        assert!(app.world().get::<IsKnockedBack>(entity).is_some());
    }
}
