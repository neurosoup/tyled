/*
 Plugin for all HUD animations (rendered on the HUD camera / render layer 1).

 Owns the HP-bar animation (`animate_hp`) and the generic numeric-counter
 machinery: rolling-odometer digit sprites via `DigitAnimations` /
 `initialize_digit_animations`, plus one `animate_*` system per counter. Each
 counter's *value* is maintained by its own domain plugin (beam charges by the
 beam plugin, claimed-tile count by the claim plugin); this plugin only reads
 those values and drives the HUD sprites.
*/
use crate::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_spritesheet_animation::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            animate_hp,
            animate_beam_charges,
            animate_claimed_tiles,
            initialize_digit_animations,
        ),
    );
}

fn animate_hp(
    players: Query<(&Health, &Player), With<DamageEffectTarget>>,
    mut hp_bars: Query<(&Player, &mut Transform), With<HPBar>>,
) {
    for (health, player) in &players {
        for (hp_bar_player, mut transform) in &mut hp_bars {
            if hp_bar_player.player_id == player.player_id {
                let ratio = health.ratio();
                transform.scale.x = transform.scale.x.lerp(ratio, 0.05);
                if transform.scale.x <= 0.001 {
                    transform.scale.x = 0.0;
                }
            }
        }
    }
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
    mut digits_query: Query<(Entity, &Player, &mut Digit), With<BeamChargesDigit>>,
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

fn animate_claimed_tiles(
    players: Query<(&Player, &ClaimedTileCount), Changed<ClaimedTileCount>>,
    mut digits_query: Query<(Entity, &Player, &mut Digit), With<ClaimedTilesDigit>>,
    children_query: Query<&Children>,
    mut sprite_animations: Query<&mut SpritesheetAnimation>,
    digit_animations: If<Res<DigitAnimations>>,
    map_info: Res<MapInfo>,
) {
    let total = map_info.ground_entities.len() as u32;
    if total == 0 {
        return;
    }

    for (player, count) in &players {
        // Owned-tile count as a rounded percentage of the whole board (0..=100).
        let percent = (count.current * 100 + total / 2) / total;

        for (entity, digit_player, mut digit) in &mut digits_query {
            if digit_player.player_id != player.player_id {
                continue;
            }

            let divisor = 10u32.pow(digit.position as u32);
            let to = ((percent / divisor) % 10) as u8;

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

        let spritesheet = Spritesheet::new(&image, 40, 3);

        const FRAME_MS: u32 = 100;
        const DIGIT_ROW: usize = 2;

        let mut make_anim = |from: usize, to: usize| -> Handle<Animation> {
            let (builder, direction) = match (from, to) {
                // 9→0: col 39 (last of 9) then cols 0-3 (all of 0)
                (9, 0) => (
                    spritesheet
                        .create_animation()
                        .add_cell(39, DIGIT_ROW)
                        .add_partial_row(DIGIT_ROW, 0..=3),
                    AnimationDirection::Forwards,
                ),
                // 0→9: same cells as 9→0 played backwards → 3, 2, 1, 0, 39
                (0, 9) => (
                    spritesheet
                        .create_animation()
                        .add_cell(39, DIGIT_ROW)
                        .add_partial_row(DIGIT_ROW, 0..=3),
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
                            .add_partial_row(DIGIT_ROW, start..=end),
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
