use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use bevy_tweening::{Tween, Tweenable, lens::TransformPositionLens};

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(SpritesheetAnimationPlugin);
    app.add_systems(
        Update,
        (
            update_players_animation,
            update_claimed_tile_animation,
            attach_player_animations,
            attach_claimed_tile_animations,
        ),
    );
}

#[derive(Resource, Clone)]
struct PlayerOneAnimations {
    idle_x: Handle<Animation>,
    idle_up: Handle<Animation>,
    idle_down: Handle<Animation>,
}

#[derive(Resource, Clone)]
struct PlayerTwoAnimations {
    idle_x: Handle<Animation>,
    idle_up: Handle<Animation>,
    idle_down: Handle<Animation>,
}

#[derive(Resource, Clone)]
struct ClaimedTileAnimations {
    unclaimed: Handle<Animation>,
    to_player_one: Handle<Animation>,
    to_player_two: Handle<Animation>,
}

fn update_claimed_tile_animation(
    mut commands: Commands,
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    players_query: Query<&Player>,
    mut claimed_query: Query<&mut SpritesheetAnimation, With<ClaimedTile>>,
    map_info: Res<MapInfo>,
    claimed_tile_animations: If<Res<ClaimedTileAnimations>>,
) {
    for tile_claimed_message in beam_resolved_reader.read() {
        let Some(claimed_tile_entity) = map_info
            .claimed_entities
            .get(&tile_claimed_message.position)
        else {
            continue;
        };

        let Ok(player) = players_query.get(tile_claimed_message.owner) else {
            continue;
        };

        let Ok(mut animation) = claimed_query.get_mut(*claimed_tile_entity) else {
            continue;
        };

        match player.player_id {
            0 => {
                animation.switch(claimed_tile_animations.to_player_one.clone());
                commands
                    .entity(*claimed_tile_entity)
                    .insert(BounceEffectTarget);
            }

            1 => {
                animation.switch(claimed_tile_animations.to_player_two.clone());
                commands
                    .entity(*claimed_tile_entity)
                    .insert(BounceEffectTarget);
            }
            _ => {
                animation.switch(claimed_tile_animations.unclaimed.clone());
                commands
                    .entity(*claimed_tile_entity)
                    .insert(BounceEffectTarget);
            }
        }
    }
}

fn update_players_animation(
    // Parent entity: has Player, LookDirection
    players: Query<(Entity, &Player, &LookDirection), With<TiledObject>>,
    // Used to traverse the hierarchy with iter_descendants
    children_query: Query<&Children>,
    // Child entity: has Sprite and SpritesheetAnimation (both must co-locate)
    mut sprites: Query<(&mut Sprite, &mut SpritesheetAnimation)>,
    player_one_animations: If<Res<PlayerOneAnimations>>,
    player_two_animations: If<Res<PlayerTwoAnimations>>,
) {
    for (entity, player, look_direction) in &players {
        for descendant in children_query.iter_descendants(entity) {
            let Ok((mut sprite, mut animation)) = sprites.get_mut(descendant) else {
                continue;
            };

            match player.player_id {
                0 => match look_direction.direction {
                    Some(Direction::Up) => {
                        animation.switch(player_one_animations.idle_up.clone());
                    }
                    Some(Direction::Down) => {
                        animation.switch(player_one_animations.idle_down.clone());
                    }
                    Some(Direction::Left) => {
                        animation.switch(player_one_animations.idle_x.clone());
                        sprite.flip_x = true;
                    }
                    Some(Direction::Right) => {
                        animation.switch(player_one_animations.idle_x.clone());
                        sprite.flip_x = false;
                    }
                    None => {}
                },
                1 => match look_direction.direction {
                    Some(Direction::Up) => {
                        animation.switch(player_two_animations.idle_up.clone());
                    }
                    Some(Direction::Down) => {
                        animation.switch(player_two_animations.idle_down.clone());
                    }
                    Some(Direction::Left) => {
                        animation.switch(player_two_animations.idle_x.clone());
                        sprite.flip_x = true;
                    }
                    Some(Direction::Right) => {
                        animation.switch(player_two_animations.idle_x.clone());
                        sprite.flip_x = false;
                    }
                    None => {}
                },
                _ => panic!("Invalid player ID"),
            }

            // Only one sprite child expected per player, stop after the first match
            break;
        }
    }
}

fn attach_claimed_tile_animations(
    mut commands: Commands,
    unclaimed_tiles: Query<(Entity, &ClaimedTile), Added<ClaimedTile>>,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<Animation>>,
) {
    for (entity, _) in &unclaimed_tiles {
        let image = assets.load("plates.png");
        let spritesheet = Spritesheet::new(&image, 12, 3);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 32), 12, 3, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        const FLIP_FRAME_MS: u32 = 20;

        let unclaimed_animation_handle =
            animations.add(spritesheet.create_animation().add_cell(0, 0).build());
        commands.insert_resource(ClaimedTileAnimations {
            unclaimed: unclaimed_animation_handle.clone(),
            to_player_one: animations.add(
                spritesheet
                    .create_animation()
                    .add_row(1)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .build(),
            ),
            to_player_two: animations.add(
                spritesheet
                    .create_animation()
                    .add_row(2)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .build(),
            ),
        });

        commands.entity(entity).insert((
            BounceEffect {
                intensity: 8.0,
                bounce_count: 1,
                decay: 1.0,
            },
            SpritesheetAnimation::new(unclaimed_animation_handle.clone()),
            Sprite::from_atlas_image(
                image.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                },
            ),
        ));
    }
}

fn attach_player_animations(
    mut commands: Commands,
    mut messages: MessageReader<TiledEvent<ObjectCreated>>,
    mut animations: ResMut<Assets<Animation>>,
    // The Sprite lives on a child entity, so we only need Entity + Player here
    players: Query<(Entity, &Player), With<TiledObject>>,
    // Used to traverse the hierarchy with iter_descendants
    children_query: Query<&Children>,
    // Read-only access to find the sprite child
    sprites: Query<&Sprite>,
) {
    for message in messages.read() {
        let Ok((entity, player)) = players.get(message.origin) else {
            info!("Player not found for entity {:?}", message.origin);
            return;
        };

        // Walk descendants to find the child that carries the Sprite
        let mut sprite_entity_and_image: Option<(Entity, Handle<Image>)> = None;
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(sprite) = sprites.get(descendant) {
                sprite_entity_and_image = Some((descendant, sprite.image.clone()));
                break;
            }
        }

        let Some((sprite_entity, image)) = sprite_entity_and_image else {
            info!("Sprite child not found for player {}", player.player_id);
            return;
        };

        let spritesheet = Spritesheet::new(&image, 32, 16);

        let idle_left_right_animation_handle = match player.player_id {
            0 => animations.add(spritesheet.create_animation().add_cell(3, 0).build()),
            1 => animations.add(spritesheet.create_animation().add_cell(3, 1).build()),
            _ => panic!("Invalid player ID"),
        };

        let idle_down_animation_handle = match player.player_id {
            0 => animations.add(spritesheet.create_animation().add_cell(0, 0).build()),
            1 => animations.add(spritesheet.create_animation().add_cell(0, 1).build()),
            _ => panic!("Invalid player ID"),
        };

        let idle_up_animation_handle = match player.player_id {
            0 => animations.add(spritesheet.create_animation().add_cell(2, 0).build()),
            1 => animations.add(spritesheet.create_animation().add_cell(2, 1).build()),
            _ => panic!("Invalid player ID"),
        };

        if player.player_id == 0 {
            commands.insert_resource(PlayerOneAnimations {
                idle_x: idle_left_right_animation_handle.clone(),
                idle_down: idle_down_animation_handle,
                idle_up: idle_up_animation_handle,
            });
        } else if player.player_id == 1 {
            commands.insert_resource(PlayerTwoAnimations {
                idle_x: idle_left_right_animation_handle.clone(),
                idle_down: idle_down_animation_handle,
                idle_up: idle_up_animation_handle,
            });
        } else {
            panic!("Invalid player ID");
        }

        // SpritesheetAnimation must go on the SAME entity as Sprite (the child)
        commands
            .entity(sprite_entity)
            .insert(SpritesheetAnimation::new(idle_left_right_animation_handle));
    }
}
