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
            animate_unclaimed_tile,
            tick_unclaim_reverts,
            initialize_player_animations,
            initialize_claimed_tile_animations,
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
    from_player_one: Handle<Animation>,
    from_player_two: Handle<Animation>,
}

const UNCLAIM_CASCADE_SECS: f32 = 3.0;

#[derive(Component)]
struct UnclaimRevert {
    timer: Timer,
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

fn animate_unclaimed_tile(
    mut commands: Commands,
    claimed_query: Query<
        (Entity, &GridCoords, &ClaimedTile, &SpritesheetAnimation),
        Changed<ClaimedTile>,
    >,
    claimed_tile_animations: If<Res<ClaimedTileAnimations>>,
    map_info: Res<MapInfo>,
) {
    for (entity, grid_coords, claimed_tile, animation) in &claimed_query {
        // `owner` has already been cleared by the time this runs, so `None` marks
        // a tile that just became unclaimed; a still-owned tile (a fresh claim or
        // a retained reset exception) keeps its color and is skipped here.
        if claimed_tile.owner.is_some() {
            continue;
        }

        let is_colored = animation.animation == claimed_tile_animations.to_player_one
            || animation.animation == claimed_tile_animations.to_player_two;
        if !is_colored {
            continue;
        }

        commands.entity(entity).insert(UnclaimRevert {
            timer: Timer::from_seconds(unclaim_delay(*grid_coords, &map_info), TimerMode::Once),
        });
    }
}

fn unclaim_delay(grid_coords: GridCoords, map_info: &MapInfo) -> f32 {
    let center_x = (map_info.map_size.x as f32 - 1.0) / 2.0;
    let center_y = (map_info.map_size.y as f32 - 1.0) / 2.0;
    let offset_x = grid_coords.x as f32 - center_x;
    let offset_y = grid_coords.y as f32 - center_y;
    let distance = (offset_x * offset_x + offset_y * offset_y).sqrt();
    let max_distance = (center_x * center_x + center_y * center_y).sqrt().max(1.0);
    (distance / max_distance) * UNCLAIM_CASCADE_SECS
}

fn tick_unclaim_reverts(
    mut commands: Commands,
    time: Res<Time>,
    mut reverts_query: Query<(
        Entity,
        &ClaimedTile,
        &mut SpritesheetAnimation,
        &mut UnclaimRevert,
    )>,
    claimed_tile_animations: If<Res<ClaimedTileAnimations>>,
) {
    for (entity, claimed_tile, mut animation, mut revert) in &mut reverts_query {
        revert.timer.tick(time.delta());
        if !revert.timer.is_finished() {
            continue;
        }

        // Skip the revert if the tile was re-claimed while its delay ran down,
        // so a freshly claimed tile is not wrongly un-colored.
        if claimed_tile.owner.is_none() {
            if animation.animation == claimed_tile_animations.to_player_one {
                animation.switch(claimed_tile_animations.from_player_one.clone());
            } else if animation.animation == claimed_tile_animations.to_player_two {
                animation.switch(claimed_tile_animations.from_player_two.clone());
            }
        }

        commands.entity(entity).remove::<UnclaimRevert>();
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
        let Some(direction) = look_direction.direction else {
            continue;
        };

        for descendant in children_query.iter_descendants(entity) {
            let Ok((mut sprite, mut animation)) = sprites.get_mut(descendant) else {
                continue;
            };

            let (target_handle, flip_x) = match player.player_id {
                0 => match direction {
                    Direction::Up => (player_one_animations.idle_up.clone(), sprite.flip_x),
                    Direction::Down => (player_one_animations.idle_down.clone(), sprite.flip_x),
                    Direction::Left => (player_one_animations.idle_x.clone(), false),
                    Direction::Right => (player_one_animations.idle_x.clone(), true),
                },
                1 => match direction {
                    Direction::Up => (player_two_animations.idle_up.clone(), sprite.flip_x),
                    Direction::Down => (player_two_animations.idle_down.clone(), sprite.flip_x),
                    Direction::Left => (player_two_animations.idle_x.clone(), false),
                    Direction::Right => (player_two_animations.idle_x.clone(), true),
                },
                _ => panic!("Invalid player ID"),
            };

            // switch() always resets to frame 0, so only call it when the animation changes
            if animation.animation != target_handle {
                animation.switch(target_handle);
            }
            sprite.flip_x = flip_x;

            // Only one sprite child expected per player, stop after the first match
            break;
        }
    }
}

fn initialize_claimed_tile_animations(
    mut commands: Commands,
    mut unclaimed_tiles: Query<(Entity, &ClaimedTile, &mut Transform), Added<ClaimedTile>>,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<Animation>>,
) {
    for (entity, _, mut transform) in &mut unclaimed_tiles {
        // Sprite is 16×32 over a 16×16 grid cell; shift up so the bottom aligns
        // with the cell bottom and the top 16px overflow above.
        transform.translation.y += 8.0;
        commands
            .entity(entity)
            .insert(RestingTranslation(transform.translation));
        let image = assets.load("tiles.png");
        let spritesheet = Spritesheet::new(&image, 12, 12);
        let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 32), 12, 12, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        const FLIP_FRAME_MS: u32 = 20;

        let unclaimed_animation_handle =
            animations.add(spritesheet.create_animation().add_cell(0, 2).build());
        commands.insert_resource(ClaimedTileAnimations {
            unclaimed: unclaimed_animation_handle.clone(),
            to_player_one: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(0, 0..=7)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .build(),
            ),
            to_player_two: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(1, 0..=7)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .build(),
            ),
            from_player_one: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(0, 0..=7)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .set_direction(AnimationDirection::Backwards)
                    .build(),
            ),
            from_player_two: animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(1, 0..=7)
                    .set_repetitions(AnimationRepeat::Times(1))
                    .set_duration(AnimationDuration::PerFrame(FLIP_FRAME_MS))
                    .set_direction(AnimationDirection::Backwards)
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

        let spritesheet = Spritesheet::new(&image, 12, 12);
        const FRAME_MS: u32 = 200;

        let idle_left_right_animation_handle = match player.player_id {
            0 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(1, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
            1 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(5, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
            _ => panic!("Invalid player ID"),
        };

        let idle_down_animation_handle = match player.player_id {
            0 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(0, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
            1 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(4, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
            _ => panic!("Invalid player ID"),
        };

        let idle_up_animation_handle = match player.player_id {
            0 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(2, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
            1 => animations.add(
                spritesheet
                    .create_animation()
                    .add_partial_row(6, 0..=3)
                    .set_duration(AnimationDuration::PerFrame(FRAME_MS))
                    .build(),
            ),
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
