/*
 Plugin for all HUD animations (rendered on the HUD camera / render layer 1).
*/
use std::time::Duration;

use crate::prelude::*;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_spritesheet_animation::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            animate_hp,
            animate_damage_bar,
            animate_territory_bar,
            animate_charges_bar,
            arm_damage_echo_delay,
            tick_damage_echo_delay,
            animate_beam_charges,
            animate_claimed_tiles,
            animate_countdown,
            initialize_digit_animations,
        ),
    );
}

/// Marks a `DamageBar` as holding off before it resumes catching up to its
/// target ratio; removed once the delay elapses.
#[derive(Component)]
struct DamageEchoDelay(Timer);

/// Arms (or restarts) `DamageEchoDelay` on every `DamageBar` matching a
/// player whose `Health` just changed.
fn arm_damage_echo_delay(
    mut commands: Commands,
    players: Query<&Player, (With<DamageEffectTarget>, Changed<Health>)>,
    damage_bars: Query<(Entity, &Player), With<DamageBar>>,
    config: Res<GameConfig>,
) {
    for player in &players {
        for (bar_entity, bar_player) in &damage_bars {
            if bar_player.player_id == player.player_id {
                commands.entity(bar_entity).insert(DamageEchoDelay(
                    Timer::new(
                        Duration::from_millis(config.animation.damage_bar_delay_ms),
                        TimerMode::Once,
                    ),
                ));
            }
        }
    }
}

fn tick_damage_echo_delay(
    mut commands: Commands,
    mut delays: Query<(Entity, &mut DamageEchoDelay)>,
    time: Res<Time>,
) {
    for (entity, mut delay) in &mut delays {
        if delay.0.tick(time.delta()).is_finished() {
            commands.entity(entity).remove::<DamageEchoDelay>();
        }
    }
}

/// Nudges every bar matching `F` that belongs to `player_id` toward `ratio` at
/// `decay_rate`, snapping to `0.0` once both the bar and its target are there.
///
/// Also snaps `scale.x` to the nearest whole-pixel width, so the bar doesn't
/// shimmer under nearest-neighbor filtering.
fn nudge_bar_for_player<F: QueryFilter>(
    player_id: u8,
    ratio: f32,
    bars: &mut Query<(&Player, &mut Transform), F>,
    decay_rate: f32,
    delta_secs: f32,
    bar_pixel_width: f32,
) {
    for (bar_player, mut transform) in &mut *bars {
        if bar_player.player_id == player_id {
            transform
                .scale
                .x
                .smooth_nudge(&ratio, decay_rate, delta_secs);
            transform.scale.x =
                (transform.scale.x * bar_pixel_width).round() / bar_pixel_width;
            if ratio <= 0.001 && transform.scale.x <= 0.001 {
                transform.scale.x = 0.0;
            }
        }
    }
}

fn animate_hp(
    players: Query<(&Health, &Player), With<DamageEffectTarget>>,
    mut hp_bars: Query<(&Player, &mut Transform), With<HPBar>>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    for (health, player) in &players {
        nudge_bar_for_player(
            player.player_id,
            health.ratio(),
            &mut hp_bars,
            config.animation.hp_bar_decay_rate,
            time.delta_secs(),
            HP_BAR_PIXEL_WIDTH,
        );
    }
}

fn animate_damage_bar(
    players: Query<(&Health, &Player), With<DamageEffectTarget>>,
    mut damage_bars: Query<(&Player, &mut Transform), (With<DamageBar>, Without<DamageEchoDelay>)>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    for (health, player) in &players {
        nudge_bar_for_player(
            player.player_id,
            health.ratio(),
            &mut damage_bars,
            config.animation.damage_bar_decay_rate,
            time.delta_secs(),
            HP_BAR_PIXEL_WIDTH,
        );
    }
}

/// Drives the territory bar toward a player's claimed-tile share of the board.
fn animate_territory_bar(
    players: Query<(&Player, &ClaimedTileCount)>,
    mut bars: Query<(&Player, &mut Transform), With<TerritoryBar>>,
    map_info: Res<MapInfo>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let total = map_info.ground_entities.len();
    if total == 0 {
        return;
    }

    for (player, count) in &players {
        let ratio = count.current as f32 / total as f32;
        nudge_bar_for_player(
            player.player_id,
            ratio,
            &mut bars,
            config.animation.territory_bar_decay_rate,
            time.delta_secs(),
            TERRITORY_BAR_PIXEL_WIDTH,
        );
    }
}

/// Drives the charges bar toward a player's claimed-tiles-plus-beam-charges
/// share of the board, clamped to `1.0`. Not `Changed`-gated, same reasoning
/// as `animate_territory_bar`.
///
/// Counts each player's in-flight `Beam`s alongside `claimed` and `charges`
/// so the charges bar stays in sync with the territory bar instead of
/// dipping on every shot and popping back on a hit.
fn animate_charges_bar(
    players: Query<(Entity, &Player, &ClaimedTileCount, &BeamCharges)>,
    beams: Query<&Beam>,
    mut bars: Query<(&Player, &mut Transform), With<ChargesBar>>,
    map_info: Res<MapInfo>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let total = map_info.ground_entities.len();
    if total == 0 {
        return;
    }

    for (entity, player, count, charges) in &players {
        let in_flight = beams.iter().filter(|beam| beam.owner == entity).count() as u32;
        let ratio = ((count.current + charges.current + in_flight) as f32 / total as f32).min(1.0);
        nudge_bar_for_player(
            player.player_id,
            ratio,
            &mut bars,
            config.animation.charges_bar_decay_rate,
            time.delta_secs(),
            TERRITORY_BAR_PIXEL_WIDTH,
        );
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

/// Drives a single digit sprite to display the decimal place selected by its
/// `Digit::position`, switching its child `SpritesheetAnimation` to the from→to
/// rolling-odometer animation. Idempotent: when the digit already shows the
/// target value, `from == to`, `DigitAnimations::get` returns `None`, and this
/// is a no-op — so callers may run every frame without spurious switches.
fn animate_digit(
    entity: Entity,
    digit: &mut Digit,
    value: u32,
    children_query: &Query<&Children>,
    sprite_animations: &mut Query<&mut SpritesheetAnimation>,
    digit_animations: &DigitAnimations,
) {
    let divisor = 10u32.pow(digit.position as u32);
    let to = ((value / divisor) % 10) as u8;

    let from = digit.value;
    let Some(handle) = digit_animations.get(from, to) else {
        return;
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

/// Drives every digit sprite marked `M` for the given player to display `value`
/// (one decimal digit per `Digit::position`).
fn animate_digits_for_player<M: Component>(
    player_id: u8,
    value: u32,
    digits: &mut Query<(Entity, &Player, &mut Digit), With<M>>,
    children_query: &Query<&Children>,
    sprite_animations: &mut Query<&mut SpritesheetAnimation>,
    digit_animations: &DigitAnimations,
) {
    for (entity, digit_player, mut digit) in digits.iter_mut() {
        if digit_player.player_id != player_id {
            continue;
        }

        animate_digit(
            entity,
            &mut digit,
            value,
            children_query,
            sprite_animations,
            digit_animations,
        );
    }
}

fn animate_beam_charges(
    players: Query<(&Player, &BeamCharges), Changed<BeamCharges>>,
    mut digits: Query<(Entity, &Player, &mut Digit), With<BeamChargesDigit>>,
    children_query: Query<&Children>,
    mut sprite_animations: Query<&mut SpritesheetAnimation>,
    digit_animations: If<Res<DigitAnimations>>,
) {
    for (player, beam_charges) in &players {
        animate_digits_for_player(
            player.player_id,
            beam_charges.current,
            &mut digits,
            &children_query,
            &mut sprite_animations,
            &digit_animations,
        );
    }
}

fn animate_claimed_tiles(
    players: Query<(&Player, &ClaimedTileCount), Changed<ClaimedTileCount>>,
    mut digits: Query<(Entity, &Player, &mut Digit), With<ClaimedTilesDigit>>,
    children_query: Query<&Children>,
    mut sprite_animations: Query<&mut SpritesheetAnimation>,
    digit_animations: If<Res<DigitAnimations>>,
) {
    for (player, count) in &players {
        animate_digits_for_player(
            player.player_id,
            count.current,
            &mut digits,
            &children_query,
            &mut sprite_animations,
            &digit_animations,
        );
    }
}

/// Drives the round countdown digits from the global `Countdown` resource. The
/// countdown is player-agnostic, so this reads a resource rather than a
/// per-`Player` value and drives the digits directly (no `Player` filter).
///
/// Runs every frame with no change-detection gate: `Countdown` is mutated every
/// frame by its tick system (marking it changed constantly), so a `Changed`/
/// `is_changed` gate would buy nothing. Correctness comes from `drive_digit`
/// being idempotent — the animation only switches on the second it actually
/// changes.
fn animate_countdown(
    countdown: Option<Res<Countdown>>,
    mut digits: Query<(Entity, &mut Digit), With<CountdownDigit>>,
    children_query: Query<&Children>,
    mut sprite_animations: Query<&mut SpritesheetAnimation>,
    digit_animations: If<Res<DigitAnimations>>,
) {
    let Some(countdown) = countdown else {
        return;
    };

    for (entity, mut digit) in digits.iter_mut() {
        animate_digit(
            entity,
            &mut digit,
            countdown.remaining,
            &children_query,
            &mut sprite_animations,
            &digit_animations,
        );
    }
}

pub fn initialize_digit_animations(
    mut commands: Commands,
    mut messages: MessageReader<TiledEvent<ObjectCreated>>,
    digits_query: Query<Entity, With<Digit>>,
    children_query: Query<&Children>,
    sprites: Query<&Sprite>,
    mut animations: ResMut<Assets<Animation>>,
    config: Res<GameConfig>,
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

        let frame_ms = config.animation.digit_roll_frame_ms;
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
                    .set_duration(AnimationDuration::PerFrame(frame_ms))
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
