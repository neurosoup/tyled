/*
 * This plugin translates player input → movement on the map.
 */

use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, *};

use crate::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, move_characters.before(super::beam::beam_step));
}

fn move_characters(
    mut commands: Commands,
    mut entity_moved_reader: MessageReader<EntityMoved>,
    mut characters: Query<&mut GridCoords, With<Character>>,
    map_info: Res<MapInfo>,
    beams_query: Query<(&GridCoords, &Beam), Without<Character>>,
) {
    for entity_moved_message in entity_moved_reader.read() {
        let entity = entity_moved_message.entity;
        let position = entity_moved_message.position;

        let beam_hit = beams_query
            .iter()
            .find(|(bp, b)| **bp == position && b.owner != entity);

        if let Some((_, beam)) = beam_hit {
            commands
                .entity(entity)
                .insert(KnockbackEffect { direction: beam.direction });
        } else if map_info.on_ground(position) {
            if let Ok(mut grid_coords) = characters.get_mut(entity) {
                *grid_coords = position;
            }
        }
    }
}
