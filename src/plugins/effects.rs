/*
 * This plugin handles effects applied to entities on the map.
 * For example, movement effects are applied to entities based on their current position and a target position when their GridCoords component changed.
 */
use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, CycleCompletedEvent, Delay, Sequence, Tween, TweenAnim,
    Tweenable,
    lens::{SpriteColorLens, TransformPositionLens},
};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_resting_translation
                .before(apply_bounce_effect)
                .before(apply_wave_effect),
            apply_knockback.before(apply_translate_effect),
            apply_translate_effect,
            apply_movement_settle,
            apply_death_effect.after(apply_knockback).after(GameplaySet::Damage),
            start_deferred_death_bounce,
            apply_wave_effect,
            apply_bounce_effect,
            apply_damage_effect,
            apply_illumination_effect,
            on_death_effect_completed,
            on_knockback_tween_completed,
            tick_knockback_lock,
            on_illumination_completed,
        ),
    );
    app.add_systems(OnExit(RoundPhase::Playing), clear_illumination_drivers);
}

pub fn create_movement_tween(
    start: Vec3,
    end: Vec3,
    duration_ms: u64,
    ease: EaseFunction,
) -> Tween {
    Tween::new(
        ease,
        Duration::from_millis(duration_ms),
        TransformPositionLens { start, end },
    )
}

pub fn create_bounce_tween(
    origin: Vec3,
    initial_intensity: f32,
    bounce_count: usize,
    decay: f32,
) -> impl Tweenable {
    let make_bounce = |height: f32, duration: Duration| {
        Tween::new(
            EaseFunction::QuadraticOut,
            duration,
            TransformPositionLens {
                start: origin,
                end: origin + Vec3::Y * height,
            },
        )
        .then(Tween::new(
            EaseFunction::BounceOut,
            duration,
            TransformPositionLens {
                start: origin + Vec3::Y * height,
                end: origin,
            },
        ))
    };

    (0..bounce_count)
        .map(|i| {
            let height = initial_intensity * decay.powi(i as i32);
            let duration = Duration::from_millis((300.0 * decay.powi(i as i32)) as u64);
            make_bounce(height, duration)
        })
        .reduce(|acc, next| acc.then(next))
        .unwrap()
}

pub fn create_color_flash_tween(duration_ms: u64) -> impl Tweenable {
    Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms / 4),
        SpriteColorLens {
            start: Color::srgb(1.0, 1.0, 1.0),
            end: Color::srgb(1.0, 0.0, 0.0),
        },
    )
    .then(Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(duration_ms),
        SpriteColorLens {
            start: Color::srgb(1.0, 0.0, 0.0),
            end: Color::srgb(1.0, 1.0, 1.0),
        },
    ))
}

pub fn create_illumination_tween(
    from: Color,
    tint: Color,
    fade_in_ms: u64,
    hold_ms: u64,
    fade_out_ms: u64,
) -> Sequence {
    let fade_in = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(fade_in_ms.max(1)),
        SpriteColorLens {
            start: from,
            end: tint,
        },
    );
    let fade_out = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(fade_out_ms.max(1)),
        SpriteColorLens {
            start: tint,
            end: Color::WHITE,
        },
    );

    let mut seq = Sequence::with_capacity(2).then(fade_in);
    if hold_ms > 0 {
        seq = seq.then(Delay::new(Duration::from_millis(hold_ms)));
    }
    seq.then(fade_out)
}

fn apply_knockback(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut query: Query<
        (Entity, &Transform, &mut GridCoords, &KnockbackEffect, Has<IsDead>),
        Added<KnockbackEffect>,
    >,
    map_info: Res<MapInfo>,
) {
    for (entity, transform, mut coords, knockback, is_dead) in &mut query {
        let target = *coords + knockback.direction;
        if !is_dead && map_info.on_ground(target) {
            let start = transform.translation;
            let destination = target.to_translation(&map_info);
            *coords = target;
            commands.entity(entity).insert((
                TweenAnim::new(create_movement_tween(
                    start,
                    destination,
                    config.effects.knockback_tween_ms,
                    EaseFunction::QuadraticOut,
                )),
                IsKnockedBack(Timer::new(
                    Duration::from_millis(config.effects.knockback_tween_ms),
                    TimerMode::Once,
                )),
                ActiveTransformEffect(TransformEffectKind::Knockback),
            ));
        }
        // Always remove, even when dead or blocked — otherwise this component
        // strands and permanently blocks future Transform effects on this entity.
        commands.entity(entity).remove::<KnockbackEffect>();
    }
}

fn apply_translate_effect(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut moving_objects: Query<
        (Entity, &Transform, &GridCoords, Option<&MovementSlide>),
        (
            Changed<GridCoords>,
            With<TranslateEffectTarget>,
            Without<KnockbackEffect>,
            Without<IsKnockedBack>,
            Without<IsDead>,
        ),
    >,
    map_info: Res<MapInfo>,
) {
    for (entity, transform, grid_coords, movement_slide) in &mut moving_objects {
        let destination = grid_coords.to_translation(&map_info);
        let duration_ms = movement_slide.map_or(config.timing.move_repeat_rate_ms, |s| s.duration_ms);

        commands.entity(entity).insert((
            TweenAnim::new(create_movement_tween(
                transform.translation,
                destination,
                duration_ms,
                EaseFunction::Linear,
            )),
            ActiveTransformEffect(TransformEffectKind::Translate),
        ));
    }
}

fn apply_movement_settle(
    mut commands: Commands,
    config: Res<GameConfig>,
    query: Query<
        (
            Entity,
            &Transform,
            &GridCoords,
            Has<IsKnockedBack>,
            Has<IsDead>,
            Has<KnockbackEffect>,
        ),
        Added<MovementSettle>,
    >,
    map_info: Res<MapInfo>,
) {
    for (entity, transform, grid_coords, is_knocked_back, is_dead, has_knockback_effect) in &query
    {
        if !is_knocked_back && !is_dead && !has_knockback_effect {
            let destination = grid_coords.to_translation(&map_info);
            commands.entity(entity).insert((
                TweenAnim::new(create_movement_tween(
                    transform.translation,
                    destination,
                    config.timing.move_repeat_rate_ms,
                    EaseFunction::QuadraticOut,
                )),
                ActiveTransformEffect(TransformEffectKind::Settle),
            ));
        }
        commands.entity(entity).remove::<MovementSettle>();
    }
}

/// Keeps a player's `RestingTranslation` aligned with its authoritative
/// `GridCoords`, so bounce origins are never read from a mid-tween interpolated
/// `Transform`.
fn sync_resting_translation(
    mut query: Query<
        (&GridCoords, &mut RestingTranslation),
        (Changed<GridCoords>, With<TranslateEffectTarget>),
    >,
    map_info: Res<MapInfo>,
) {
    for (coords, mut resting) in &mut query {
        resting.0 = coords.to_translation(&map_info);
    }
}

fn apply_damage_effect(
    mut commands: Commands,
    config: Res<GameConfig>,
    damageable_query: Query<
        (Entity, Option<&Children>),
        (With<DamageEffectTarget>, Changed<Health>),
    >,
    sprite_query: Query<&Sprite>,
) {
    for (_entity, children) in &damageable_query {
        if let Some(first_child) = children.and_then(|c| c.first()).copied() {
            if sprite_query.get(first_child).is_ok() {
                commands
                    .entity(first_child)
                    .insert(TweenAnim::new(create_color_flash_tween(
                        config.effects.damage_flash_ms,
                    )));
            }
        }
    }
}

// Reacts to DamageableDied event. Defers the bounce if knocked back — see
// `start_deferred_death_bounce`.
fn apply_death_effect(
    mut commands: Commands,
    mut damageable_died_reader: MessageReader<DamageableDied>,
    damageable_query: Query<
        Has<IsKnockedBack>,
        (With<DamageEffectTarget>, With<Health>, Without<IsDead>),
    >,
    knockback_pending: Query<(), With<KnockbackEffect>>,
) {
    for damageable_died_message in damageable_died_reader.read() {
        let Ok(is_knocked_back) = damageable_query.get(damageable_died_message.entity) else {
            continue;
        };
        let defer =
            is_knocked_back || knockback_pending.get(damageable_died_message.entity).is_ok();

        let mut entity_commands = commands.entity(damageable_died_message.entity);
        entity_commands.insert((
            BounceEffect {
                intensity: 8.0,
                bounce_count: 3,
                decay: 0.33,
                z_index: 1,
            },
            IsDead,
        ));
        if defer {
            entity_commands.insert(PendingDeathBounce);
        } else {
            entity_commands.insert(BounceEffectTarget);
        }
    }
}

/// Promotes a death bounce that was parked behind a knockback slide.
fn start_deferred_death_bounce(
    mut commands: Commands,
    pending: Query<
        Entity,
        (
            With<PendingDeathBounce>,
            With<BounceEffect>,
            Without<IsKnockedBack>,
            Without<KnockbackEffect>,
        ),
    >,
) {
    for entity in &pending {
        commands
            .entity(entity)
            .insert(BounceEffectTarget)
            .remove::<PendingDeathBounce>();
    }
}

// Once the death bounce finishes, hide the dead entity and clear its bounce
// rather than despawning it. In a round-based match the loser must survive to be
// restored by the round reset (`reset_round`, round `state` submodule); the
// `IsDead` marker is kept so the entity stays inert until then. Only players
// carry `DamageEffectTarget`, so this only ever hides players.
fn on_death_effect_completed(
    mut commands: Commands,
    mut anim_completed_reader: MessageReader<AnimCompletedEvent>,
    dead_entities: Query<&ActiveTransformEffect, (With<IsDead>, With<BounceEffect>)>,
) {
    for anim_completed_message in anim_completed_reader.read() {
        let Ok(active) = dead_entities.get(anim_completed_message.anim_entity) else {
            continue;
        };
        if active.0 != TransformEffectKind::Bounce {
            continue;
        }
        commands
            .entity(anim_completed_message.anim_entity)
            .insert(Visibility::Hidden)
            .remove::<(BounceEffect, ActiveTransformEffect)>();
    }
}

// Only clears the `ActiveTransformEffect` tag — `IsKnockedBack` itself is
// timer-driven, see `tick_knockback_lock`.
fn on_knockback_tween_completed(
    mut commands: Commands,
    mut anim_completed_reader: MessageReader<AnimCompletedEvent>,
    knocked_back: Query<&ActiveTransformEffect, With<IsKnockedBack>>,
) {
    for ev in anim_completed_reader.read() {
        let Ok(active) = knocked_back.get(ev.anim_entity) else {
            continue;
        };
        if active.0 != TransformEffectKind::Knockback {
            continue;
        }
        commands
            .entity(ev.anim_entity)
            .remove::<ActiveTransformEffect>();
    }
}

/// The sole remover of `IsKnockedBack`.
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

fn apply_wave_effect(
    mut commands: Commands,
    mut wave_source_query: Query<(&GridCoords, &BounceEffect), (With<WaveSource>, Changed<GridCoords>)>,
    mut effect_targets: Query<
        (Entity, &Transform, Option<&RestingTranslation>),
        With<WaveEffectTarget>,
    >,
    map_info: Res<MapInfo>,
) {
    for (source_coords, bounce_effect) in &mut wave_source_query {
        let Some(claimed_entity) = map_info.claimed_entities.get(source_coords) else {
            continue;
        };
        if let Ok((entity, transform, resting)) = effect_targets.get_mut(*claimed_entity) {
            let BounceEffect {
                intensity,
                bounce_count,
                decay,
                ..
            } = *bounce_effect;
            let origin = resting.map(|r| r.0).unwrap_or(transform.translation);
            commands
                .entity(entity)
                .insert(TweenAnim::new(create_bounce_tween(
                    origin,
                    intensity,
                    bounce_count,
                    decay,
                )));
        }
    }
}

fn apply_bounce_effect(
    mut commands: Commands,
    mut bounce_query: Query<
        (Entity, &Transform, &BounceEffect, Option<&RestingTranslation>),
        Added<BounceEffectTarget>,
    >,
) {
    for (entity, transform, bounce_effect, resting) in &mut bounce_query {
        let BounceEffect {
            intensity,
            bounce_count,
            decay,
            ..
        } = *bounce_effect;
        let origin = resting.map(|r| r.0).unwrap_or(transform.translation);
        commands
            .entity(entity)
            .insert((
                TweenAnim::new(create_bounce_tween(origin, intensity, bounce_count, decay)),
                ActiveTransformEffect(TransformEffectKind::Bounce),
            ))
            .remove::<BounceEffectTarget>();
    }
}

fn apply_illumination_effect(
    mut commands: Commands,
    config: Res<GameConfig>,
    beams: Query<(&GridCoords, &Beam), Changed<GridCoords>>,
    players: Query<&Player>,
    map_info: Res<MapInfo>,
    tile_sprites: Query<&Sprite, With<IlluminationEffectTarget>>,
    drivers: Query<(Entity, &IlluminationDriver)>,
) {
    for (grid_coords, beam) in &beams {
        let Ok(owner) = players.get(beam.owner) else {
            continue;
        };
        let Some(tile) = map_info.get_claimed_entity_by_position(*grid_coords) else {
            continue;
        };
        let Ok(sprite) = tile_sprites.get(tile) else {
            continue;
        };

        for (driver_entity, driver) in &drivers {
            if driver.tile == tile {
                commands.entity(driver_entity).despawn();
            }
        }

        let effects_config = &config.effects;
        let color = match owner.player_id {
            0 => effects_config.beam_illumination_color_p1,
            1 => effects_config.beam_illumination_color_p2,
            _ => effects_config.beam_illumination_color_p1,
        };
        let tint = Color::srgba(color[0], color[1], color[2], color[3]);
        commands.spawn((
            Name::new("IlluminationDriver"),
            IlluminationDriver { tile },
            AnimTarget::component::<Sprite>(tile),
            TweenAnim::new(create_illumination_tween(
                sprite.color,
                tint,
                effects_config.beam_illumination_fade_in_ms,
                effects_config.beam_illumination_hold_ms,
                effects_config.beam_illumination_fade_out_ms,
            )),
        ));
    }
}

fn on_illumination_completed(
    mut commands: Commands,
    mut anim_completed_reader: MessageReader<AnimCompletedEvent>,
    drivers: Query<Entity, With<IlluminationDriver>>,
) {
    for ev in anim_completed_reader.read() {
        if let Ok(entity) = drivers.get(ev.anim_entity) {
            commands.entity(entity).despawn();
        }
    }
}

fn clear_illumination_drivers(
    mut commands: Commands,
    drivers: Query<(Entity, &IlluminationDriver)>,
    mut tiles: Query<&mut Sprite, With<IlluminationEffectTarget>>,
) {
    for (driver_entity, driver) in &drivers {
        if let Ok(mut sprite) = tiles.get_mut(driver.tile) {
            sprite.color = Color::WHITE;
        }
        commands.entity(driver_entity).despawn();
    }
}
