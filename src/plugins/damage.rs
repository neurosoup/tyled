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
    app.add_systems(
        Update,
        (
            (apply_owned_tile_entry_damage, apply_owned_tile_damage).chain(),
            apply_beam_damage,
        )
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
    mut commands: Commands,
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
                commands.entity(entity).insert(KnockbackEffect {
                    direction: beam.direction,
                });
            }
        }
    }
}

/// Returns true when `position` holds a claimed tile owned by someone other than `character`.
fn is_hostile_tile(
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
