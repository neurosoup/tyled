/*
 Plugin for initializing sprite animations and attaching them to entities.
*/
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
            animate_player,
            animate_claimed_tile,
            animate_beam_charges,
            initialize_player_animations,
            initialize_claimed_tile_animations,
            initialize_digit_animations,
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

// handles[from][to] — valid for all from != to in 0..10
#[derive(Resource, Clone)]
struct DigitAnimations {
    handles: [[Handle<Animation>; 10]; 10],
}

impl DigitAnimations {
    fn get(&self, from: u8, to: u8) -> Option<Handle<Animation>> {
        if from == to || from >= 10 || to >= 10 {
            return None;
        }
        Some(self.handles[from as usize][to as usize].clone())
    }
}

fn animate_beam_charges(
    players: Query<(&Player, &BeamCharges), Changed<BeamCharges>>,
    mut digits_query: Query<(Entity, &Player, &mut Digit)>,
    children_query: Query<&Children>,
    mut sprite_animations: Query<&mut SpritesheetAnimation>,
    digit_animations: If<Res<DigitAnimations>>,
) {
    for (player, beam_charges) in &players {
        for (entity, digit_player, mut digit) in &mut digits_query {
            if digit_player.player_id != player.player_id {
                continue;
            }

            let divisor = 10u32.pow(digit.position as u32);
            let to = ((beam_charges.current / divisor) % 10) as u8;

            let from = digit.value;
            let Some(handle) = digit_animations.get(from, to) else {
                continue;
            };
            digit.value = to;

            for descendant in children_query.iter_descendants(entity) {
                let Ok(mut animation) = sprite_animations.get_mut(descendant) else {
                    continue;
                };
                animation.switch(handle);
                break;
            }
        }
    }
}

fn animate_claimed_tile(
    mut commands: Commands,
    mut beam_resolved_reader: MessageReader<BeamResolved>,
    players_query: Query<&Player, With<Character>>,
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

        let handle = match player.player_id {
            0 => claimed_tile_animations.to_player_one.clone(),
            1 => claimed_tile_animations.to_player_two.clone(),
            _ => claimed_tile_animations.unclaimed.clone(),
        };
        animation.switch(handle);
        commands
            .entity(*claimed_tile_entity)
            .insert(BounceEffectTarget);
    }
}

fn animate_player(
    // Parent entity: has Player, LookDirection
    players: Query<(Entity, &Player, &LookDirection)>,
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

fn initialize_claimed_tile_animations(
    mut commands: Commands,
    unclaimed_tiles: Query<(Entity, &ClaimedTile), Added<ClaimedTile>>,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<Animation>>,
) {
    for (entity, _) in &unclaimed_tiles {
        let image = assets.load("atlas1.png");
        let spritesheet = Spritesheet::new(&image, 40, 32);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 40, 32, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        const FLIP_FRAME_MS: u32 = 20;

        let unclaimed_animation_handle =
            animations.add(spritesheet.create_animation().add_cell(0, 3).build());
        commands.insert_resource(ClaimedTileAnimations {
            unclaimed: unclaimed_animation_handle.clone(),
            to_player_one: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(4, 0..=11)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .build(),
            ),
            to_player_two: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(5, 0..=11)
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
                z_index: CLAIMED_TILE_Z_INDEX,
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

fn initialize_player_animations(
    mut commands: Commands,
    mut messages: MessageReader<TiledEvent<ObjectCreated>>,
    mut animations: ResMut<Assets<Animation>>,
    // The Sprite lives on a child entity, so we only need Entity + Player here
    players: Query<(Entity, &Player), With<Character>>,
    // Used to traverse the hierarchy with iter_descendants
    children_query: Query<&Children>,
    // Read-only access to find the sprite child
    sprites: Query<&Sprite>,
) {
    for message in messages.read() {
        let Ok((entity, player)) = players.get(message.origin) else {
            continue;
        };

        // Walk descendants to find the child that carries the Sprite
        let mut sprite_and_image: Option<(Entity, Handle<Image>)> = None;
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(sprite) = sprites.get(descendant) {
                sprite_and_image = Some((descendant, sprite.image.clone()));
                break;
            }
        }

        let Some((sprite_entity, image)) = sprite_and_image else {
            info!(
                "Cannot attach player animations: Sprite child not found for player {}",
                player.player_id
            );
            return;
        };

        let spritesheet = Spritesheet::new(&image, 40, 32);

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

pub fn initialize_digit_animations(
    mut commands: Commands,
    mut messages: MessageReader<TiledEvent<ObjectCreated>>,
    digits_query: Query<Entity, With<Digit>>,
    children_query: Query<&Children>,
    sprites: Query<&Sprite>,
    mut animations: ResMut<Assets<Animation>>,
) {
    for message in messages.read() {
        let Ok(entity) = digits_query.get(message.origin) else {
            continue;
        };

        let mut sprite_and_image: Option<(Entity, Handle<Image>)> = None;
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(sprite) = sprites.get(descendant) {
                sprite_and_image = Some((descendant, sprite.image.clone()));
                break;
            }
        }

        let Some((sprite_entity, image)) = sprite_and_image else {
            info!("Cannot attach digit animations: Sprite child not found");
            continue;
        };

        let spritesheet = Spritesheet::new(&image, 40, 32);

        const FRAME_MS: u32 = 100;

        let mut make_anim = |from: usize, to: usize| -> Handle<Animation> {
            let (builder, direction) = match (from, to) {
                // 9→0: col 39 (last of 9) then cols 0-3 (all of 0)
                (9, 0) => (
                    spritesheet
                        .create_animation()
                        .add_cell(39, 8)
                        .add_partial_row(8, 0..=3),
                    AnimationDirection::Forwards,
                ),
                // 0→9: same cells as 9→0 played backwards → 3, 2, 1, 0, 39
                (0, 9) => (
                    spritesheet
                        .create_animation()
                        .add_cell(39, 8)
                        .add_partial_row(8, 0..=3),
                    AnimationDirection::Backwards,
                ),
                _ => {
                    let direction = if to > from {
                        AnimationDirection::Forwards
                    } else {
                        AnimationDirection::Backwards
                    };
                    let (start, end) = (from.min(to) * 4 + 3, from.max(to) * 4 + 3);
                    (
                        spritesheet
                            .create_animation()
                            .add_partial_row(8, start..=end),
                        direction,
                    )
                }
            };
            animations.add(
                builder
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .set_direction(direction)
                    .set_easing(Easing::In(EasingVariety::Quintic))
                    .build(),
            )
        };

        let handles = std::array::from_fn(|from| {
            std::array::from_fn(|to| {
                if from == to {
                    return Handle::default();
                }
                make_anim(from, to)
            })
        });

        let initial_handle = handles[9][0].clone();
        commands.insert_resource(DigitAnimations { handles });

        commands
            .entity(sprite_entity)
            .insert(SpritesheetAnimation::new(initial_handle));
    }
}
