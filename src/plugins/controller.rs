/*
 * This plugin translates player input → movement on the map.
 */

use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        move_characters
            .in_set(GameplaySet::Movement)
            .run_if(in_state(RoundPhase::Playing)),
    );
}

fn move_characters(
    mut entity_moved_reader: MessageReader<EntityMoved>,
    mut characters: Query<(Entity, &mut GridCoords, &Transform), With<Character>>,
    map_info: Res<MapInfo>,
    beams_query: Query<(&GridCoords, &Beam), Without<Character>>,
    mut collision_writer: MessageWriter<CharacterCollision>,
    mut knockback_writer: MessageWriter<KnockbackRequested>,
) {
    let moves: Vec<(Entity, GridCoords)> = entity_moved_reader
        .read()
        .map(|message| (message.entity, message.position))
        .collect();

    let snapshot: Vec<(Entity, GridCoords, Vec3)> = characters
        .iter()
        .map(|(entity, coords, transform)| (entity, *coords, transform.translation))
        .collect();

    let mut resolved: Vec<Entity> = Vec::new();

    for &(entity, position) in &moves {
        if resolved.contains(&entity) {
            continue;
        }

        let beam_hit = beams_query
            .iter()
            .find(|(bp, b)| **bp == position && b.owner != entity);

        if let Some((_, beam)) = beam_hit {
            knockback_writer.write(KnockbackRequested {
                entity,
                direction: beam.direction,
            });
            continue;
        }

        if !map_info.on_ground(position) {
            continue;
        }

        let occupant = snapshot
            .iter()
            .find(|(other, coords, _)| *other != entity && *coords == position)
            .map(|(other, _, _)| *other);

        let Some(occupant) = occupant else {
            if let Ok((_, mut grid_coords, _)) = characters.get_mut(entity) {
                *grid_coords = position;
            }
            continue;
        };

        if resolved.contains(&occupant) {
            continue;
        }

        resolve_collision(
            &map_info,
            &snapshot,
            &moves,
            entity,
            occupant,
            &mut collision_writer,
            &mut knockback_writer,
        );
        resolved.push(entity);
        resolved.push(occupant);
    }
}

struct CollisionSide {
    entity: Entity,
    current: GridCoords,
    destination: GridCoords,
    distance: f32,
}

fn collision_side(
    entity: Entity,
    snapshot: &[(Entity, GridCoords, Vec3)],
    moves: &[(Entity, GridCoords)],
    map_info: &MapInfo,
) -> CollisionSide {
    let (current, translation) = snapshot
        .iter()
        .find(|(other, _, _)| *other == entity)
        .map(|(_, coords, transform)| (*coords, *transform))
        .expect("collision side entity must be a live Character");

    let destination = moves
        .iter()
        .find(|(other, _)| *other == entity)
        .map(|(_, position)| *position)
        .unwrap_or(current);

    let distance = (destination.to_world_pos(map_info) - translation.xy()).length();

    CollisionSide {
        entity,
        current,
        destination,
        distance,
    }
}

fn resolve_collision(
    map_info: &MapInfo,
    snapshot: &[(Entity, GridCoords, Vec3)],
    moves: &[(Entity, GridCoords)],
    mover: Entity,
    occupant: Entity,
    collision_writer: &mut MessageWriter<CharacterCollision>,
    knockback_writer: &mut MessageWriter<KnockbackRequested>,
) {
    const EPSILON: f32 = 1e-3;

    let mover_side = collision_side(mover, snapshot, moves, map_info);
    let occupant_side = collision_side(occupant, snapshot, moves, map_info);

    if (mover_side.distance - occupant_side.distance).abs() < EPSILON {
        knockback_writer.write(KnockbackRequested {
            entity: mover_side.entity,
            direction: mover_side.current - mover_side.destination,
        });
        knockback_writer.write(KnockbackRequested {
            entity: occupant_side.entity,
            direction: occupant_side.current - occupant_side.destination,
        });
        return;
    }

    let (winner, loser) = if mover_side.distance > occupant_side.distance {
        (mover_side, occupant_side)
    } else {
        (occupant_side, mover_side)
    };

    knockback_writer.write(KnockbackRequested {
        entity: winner.entity,
        direction: winner.current - winner.destination,
    });
    knockback_writer.write(KnockbackRequested {
        entity: loser.entity,
        direction: winner.destination - winner.current,
    });
    collision_writer.write(CharacterCollision { loser: loser.entity });
}
