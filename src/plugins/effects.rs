/*
 * This plugin handles effects applied to entities on the map.
 * For example, movement effects are applied to entities based on their current position and a target position when their GridCoords component changed.
 */
use std::collections::HashMap;
use std::time::Duration;

use crate::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    prelude::*,
    render::render_resource::TextureFormat,
};
use bevy_ecs_tiled::prelude::{TilePos, TileTextureIndex, TilemapId, TilemapTexture};
use bevy_spritesheet_animation::plugin::AnimationSystemSet;
use bevy_tweening::{
    AnimCompletedEvent, AnimTarget, CycleCompletedEvent, Tween, TweenAnim, Tweenable,
    lens::{SpriteColorLens, TransformPositionLens, TransformScaleLens},
};

/// Local z of the sprite lit overlay; must stay below 1.0 so it never covers the next tile row.
const SPRITE_LIT_OVERLAY_Z: f32 = 0.5;

/// Z of the tilemap lit overlay, above the floor and below the claimed tiles.
const TILEMAP_LIT_OVERLAY_Z: f32 = 0.5;

/// The ground atlas is a 12 by 12 grid of cells.
const GROUND_ATLAS_COLUMNS: u32 = 12;

/// A lit copy of one source atlas and the settings it was last built with.
struct LitAtlas {
    lit: Handle<Image>,
    source: Option<Handle<Image>>,
    built_with: Option<(f32, f32)>,
}

impl LitAtlas {
    fn new(images: &Assets<Image>) -> Self {
        Self {
            lit: images.reserve_handle(),
            source: None,
            built_with: None,
        }
    }

    fn rebuild(
        &mut self,
        wanted: (f32, f32),
        images: &mut Assets<Image>,
        events: &[AssetEvent<Image>],
        name: &str,
    ) {
        let Some(source) = self.source.clone() else {
            return;
        };
        let source_changed = events
            .iter()
            .any(|event| event.is_modified(source.id()) || event.is_loaded_with_dependencies(source.id()));
        if !source_changed && self.built_with == Some(wanted) {
            return;
        }
        let Some(source_image) = images.get(source.id()) else {
            return;
        };
        self.built_with = Some(wanted);
        let Some(image) = make_lit_image(source_image, wanted.0, wanted.1) else {
            warn!("The {name} atlas has an unsupported format, so its glow is disabled");
            return;
        };
        if let Err(error) = images.insert(self.lit.id(), image) {
            error!("Failed to store the lit {name} atlas: {error}");
        }
    }
}

/// The lit atlases for the claimed tiles and the ground, plus the shared ground cell layout.
#[derive(Resource)]
struct LitAtlases {
    tiles: LitAtlas,
    ground: LitAtlas,
    ground_layout: Option<Handle<TextureAtlasLayout>>,
}

impl FromWorld for LitAtlases {
    fn from_world(world: &mut World) -> Self {
        let images = world.resource::<Assets<Image>>();
        Self {
            tiles: LitAtlas::new(images),
            ground: LitAtlas::new(images),
            ground_layout: None,
        }
    }
}

/// Ordering of the effect driver pipeline inside `GameplaySet::Presentation`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectsSet {
    /// Systems that write `EffectRequest` messages.
    Request,
    /// Finishes completed drivers and starts their queued follow-ups.
    Complete,
    /// Turns this frame's requests into drivers.
    Resolve,
}

/// A request to run one effect on an owner entity.
#[derive(Message, Clone, Copy, Debug)]
pub struct EffectRequest {
    pub owner: Entity,
    pub kind: EffectKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Decision {
    Start,
    Replace,
    Queue,
    Drop,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct ChannelPlan {
    spawn: Option<(EffectKind, Option<EffectKind>)>,
    new_then: Option<Option<EffectKind>>,
}

type LiveEffect = (EffectKind, Option<EffectKind>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<LitAtlases>();
    app.add_message::<EffectRequest>();
    app.configure_sets(
        Update,
        (
            EffectsSet::Request,
            EffectsSet::Complete,
            EffectsSet::Resolve,
        )
            .chain()
            .in_set(GameplaySet::Presentation),
    );
    app.configure_sets(
        Update,
        EffectsSet::Resolve.before(bevy_tweening::AnimationSystem::AnimationUpdate),
    );
    app.add_systems(
        Update,
        (
            apply_damage_effect.in_set(EffectsSet::Request),
            apply_parry_scale_effect.in_set(EffectsSet::Request),
            on_effect_completed.in_set(EffectsSet::Complete),
            resolve_effect_requests.in_set(EffectsSet::Resolve),
        ),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, warn_duplicate_channel_drivers.after(EffectsSet::Resolve));
    app.add_systems(
        Update,
        (
            sync_resting_translation
                .before(apply_bounce_effect)
                .before(apply_wave_effect),
            apply_knockback
                .in_set(GameplaySet::Displacement)
                .before(apply_translate_effect),
            apply_translate_effect,
            apply_movement_settle,
            apply_death_effect.after(apply_knockback).in_set(GameplaySet::Presentation),
            start_deferred_death_bounce,
            apply_wave_effect,
            apply_bounce_effect,
            spawn_sprite_lit_overlays,
            build_lit_atlas,
            apply_glow_effect
                .after(spawn_sprite_lit_overlays)
                .after(spawn_tilemap_lit_overlays),
            update_lit_overlays.after(apply_glow_effect),
            on_death_effect_completed,
            on_knockback_tween_completed,
            tick_knockback_lock,
            trigger_parry_scale_effect.in_set(GameplaySet::Presentation),
        ),
    );
    app.add_systems(
        Update,
        spawn_tilemap_lit_overlays.run_if(resource_changed::<MapInfo>),
    );
    app.add_systems(
        PostUpdate,
        sync_lit_overlay_frames.after(AnimationSystemSet),
    );
    app.add_systems(OnExit(RoundPhase::Playing), clear_glow);
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

pub fn create_parry_scale_tween(peak: f32, duration_secs: f32) -> Tween {
    Tween::new(
        EaseFunction::CubicIn,
        Duration::from_secs_f32(duration_secs),
        TransformScaleLens {
            start: Vec3::splat(peak),
            end: Vec3::ONE,
        },
    )
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

/// Returns a lighter, less saturated copy of `source`, keeping alpha.
fn make_lit_image(source: &Image, lightness: f32, chroma: f32) -> Option<Image> {
    if !matches!(
        source.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm
    ) || source.data.is_none()
    {
        return None;
    }
    let mut image = source.clone();
    for y in 0..image.height() {
        for x in 0..image.width() {
            let Ok(color) = image.get_color_at(x, y) else {
                continue;
            };
            let alpha = color.alpha();
            if alpha == 0.0 {
                continue;
            }
            let mut oklch = Oklcha::from(color);
            oklch.lightness += (1.0 - oklch.lightness) * lightness;
            oklch.chroma *= chroma;
            let srgba = Srgba::from(oklch);
            let lit = Color::srgba(
                srgba.red.clamp(0.0, 1.0),
                srgba.green.clamp(0.0, 1.0),
                srgba.blue.clamp(0.0, 1.0),
                alpha,
            );
            let _ = image.set_color_at(x, y, lit);
        }
    }
    Some(image)
}

fn apply_knockback(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut query: Query<
        (Entity, &Transform, &mut GridCoords, &KnockbackEffect, Has<IsDead>, Option<&Health>),
        Added<KnockbackEffect>,
    >,
    map_info: Res<MapInfo>,
) {
    for (entity, transform, mut coords, knockback, is_dead, health) in &mut query {
        let target = *coords + knockback.direction;
        let is_lethal = health.is_some_and(|health| health.current <= 0.0);
        if !is_dead && !is_lethal && map_info.on_ground(target) {
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

/// Decides what an incoming effect does given the effect currently running on its channel.
fn decide(incoming: &EffectKind, current: Option<&EffectKind>) -> Decision {
    use EffectKind::*;
    let Some(current) = current else {
        return Decision::Start;
    };
    if incoming.channel() != current.channel() {
        return Decision::Start;
    }
    match (incoming, current) {
        (_, DeathBounce { .. }) => Decision::Drop,
        (Translate { .. } | Settle { .. } | DeathBounce { .. }, Knockback { .. }) => {
            Decision::Queue
        }
        _ => Decision::Replace,
    }
}

/// Merges a new queued effect into the single queued slot.
fn merge_queued(existing: Option<EffectKind>, incoming: EffectKind) -> Option<EffectKind> {
    match existing {
        Some(existing) if existing.rank() > incoming.rank() => Some(existing),
        _ => Some(incoming),
    }
}

/// Folds one frame of requests for one channel into at most one spawn or one queue update.
fn fold_requests(live: Option<LiveEffect>, requests: &[EffectKind]) -> ChannelPlan {
    let mut sorted = requests.to_vec();
    sorted.sort_by_key(|kind| kind.rank());
    let mut state = live;
    let mut spawned = false;
    for incoming in &sorted {
        match decide(incoming, state.as_ref().map(|(kind, _)| kind)) {
            Decision::Start => {
                state = Some((*incoming, None));
                spawned = true;
            }
            Decision::Replace => {
                let then = state
                    .and_then(|(_, then)| then)
                    .filter(|then| decide(then, Some(incoming)) == Decision::Queue);
                state = Some((*incoming, then));
                spawned = true;
            }
            Decision::Queue => {
                if let Some((_, then)) = &mut state {
                    *then = merge_queued(*then, *incoming);
                }
            }
            Decision::Drop => {}
        }
    }
    if spawned {
        return ChannelPlan {
            spawn: state,
            new_then: None,
        };
    }
    let new_then = state
        .zip(live)
        .filter(|((_, then), (_, live_then))| then != live_then)
        .map(|((_, then), _)| then);
    ChannelPlan {
        spawn: None,
        new_then,
    }
}

/// Spawns the driver entity that runs one effect tween.
fn start_effect(
    commands: &mut Commands,
    owner: Entity,
    kind: EffectKind,
    then: Option<EffectKind>,
) {
    let driver = (
        Name::new(kind.name()),
        EffectDriver { owner, kind, then },
        DriverOf(owner),
    );
    match kind {
        EffectKind::ParryPunch { sprite, peak, secs } => {
            commands.spawn((
                driver,
                AnimTarget::component::<Transform>(sprite),
                TweenAnim::new(create_parry_scale_tween(peak, secs)),
            ));
        }
        EffectKind::DamageFlash { sprite, ms } => {
            commands.spawn((
                driver,
                AnimTarget::component::<Sprite>(sprite),
                TweenAnim::new(create_color_flash_tween(ms)),
            ));
        }
        EffectKind::Translate { .. }
        | EffectKind::Settle { .. }
        | EffectKind::Knockback { .. }
        | EffectKind::DeathBounce { .. } => {}
    }
}

fn resolve_effect_requests(
    mut commands: Commands,
    mut requests: MessageReader<EffectRequest>,
    dead: Query<(), With<IsDead>>,
    owners: Query<&EffectDrivers>,
    mut drivers: Query<&mut EffectDriver>,
) {
    let mut grouped: HashMap<(Entity, EffectChannel), Vec<EffectKind>> = HashMap::new();
    for request in requests.read() {
        if request.kind.channel() == EffectChannel::RootTranslation
            && dead.contains(request.owner)
            && !matches!(request.kind, EffectKind::DeathBounce { .. })
        {
            continue;
        }
        grouped
            .entry((request.owner, request.kind.channel()))
            .or_default()
            .push(request.kind);
    }
    for ((owner, channel), kinds) in grouped {
        let live_driver = owners.get(owner).ok().and_then(|live| {
            live.iter().find(|driver| {
                drivers
                    .get(*driver)
                    .is_ok_and(|driver| driver.kind.channel() == channel)
            })
        });
        let live = live_driver.and_then(|driver| {
            drivers
                .get(driver)
                .ok()
                .map(|driver| (driver.kind, driver.then))
        });
        let plan = fold_requests(live, &kinds);
        if let Some((kind, then)) = plan.spawn {
            if let Some(driver) = live_driver {
                commands.entity(driver).despawn();
            }
            start_effect(&mut commands, owner, kind, then);
        } else if let (Some(then), Some(driver)) = (plan.new_then, live_driver) {
            if let Ok(mut driver) = drivers.get_mut(driver) {
                driver.then = then;
            }
        }
    }
}

fn on_effect_completed(
    mut commands: Commands,
    mut anim_completed_reader: MessageReader<AnimCompletedEvent>,
    drivers: Query<&EffectDriver>,
) {
    for event in anim_completed_reader.read() {
        let Ok(driver) = drivers.get(event.anim_entity) else {
            continue;
        };
        commands.entity(event.anim_entity).despawn();
        if let Some(then) = driver.then {
            start_effect(&mut commands, driver.owner, then, None);
        }
    }
}

#[cfg(feature = "dev")]
fn warn_duplicate_channel_drivers(owners: Query<(Entity, &EffectDrivers)>, drivers: Query<&EffectDriver>) {
    for (owner, owned) in &owners {
        let mut seen: Vec<EffectChannel> = Vec::new();
        for driver in owned.iter().filter_map(|driver| drivers.get(driver).ok()) {
            let channel = driver.kind.channel();
            if seen.contains(&channel) {
                warn!("Entity {owner} has two effect drivers on {channel:?}");
            }
            seen.push(channel);
        }
    }
}

fn apply_damage_effect(
    mut requests: MessageWriter<EffectRequest>,
    config: Res<GameConfig>,
    damageable_query: Query<
        (Entity, Option<&Children>),
        (With<DamageEffectTarget>, Changed<Health>),
    >,
    sprite_query: Query<&Sprite>,
) {
    for (entity, children) in &damageable_query {
        if let Some(sprite) = sprite_child(children, &sprite_query) {
            requests.write(EffectRequest {
                owner: entity,
                kind: EffectKind::DamageFlash {
                    sprite,
                    ms: config.effects.damage_flash_ms,
                },
            });
        }
    }
}

/// The first child entity that actually carries a `Sprite`.
fn sprite_child(children: Option<&Children>, sprite_query: &Query<&Sprite>) -> Option<Entity> {
    let first_child = children.and_then(|c| c.first()).copied()?;
    sprite_query.get(first_child).is_ok().then_some(first_child)
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

fn trigger_parry_scale_effect(
    mut commands: Commands,
    mut beam_parried_reader: MessageReader<BeamParried>,
) {
    for message in beam_parried_reader.read() {
        commands.entity(message.parrier).insert(ParryScaleEffectTarget);
    }
}

fn apply_parry_scale_effect(
    mut requests: MessageWriter<EffectRequest>,
    mut commands: Commands,
    config: Res<GameConfig>,
    parry_query: Query<(Entity, Option<&Children>), Added<ParryScaleEffectTarget>>,
    sprite_query: Query<&Sprite>,
) {
    for (entity, children) in &parry_query {
        if let Some(sprite) = sprite_child(children, &sprite_query) {
            requests.write(EffectRequest {
                owner: entity,
                kind: EffectKind::ParryPunch {
                    sprite,
                    peak: config.parry.scale_punch_peak,
                    secs: config.parry.scale_punch_secs,
                },
            });
        }
        commands.entity(entity).remove::<ParryScaleEffectTarget>();
    }
}

fn spawn_sprite_lit_overlays(
    mut commands: Commands,
    mut atlases: ResMut<LitAtlases>,
    tiles: Query<
        (Entity, &Sprite),
        (With<GlowEffectTarget>, Without<LitOverlayLink>),
    >,
) {
    for (tile, sprite) in &tiles {
        if atlases.tiles.source.is_none() {
            atlases.tiles.source = Some(sprite.image.clone());
        }
        let overlay = commands
            .spawn((
                Name::new("SpriteLitOverlay"),
                SpriteLitOverlay,
                GlowPulses::default(),
                Sprite {
                    image: atlases.tiles.lit.clone(),
                    texture_atlas: sprite.texture_atlas.clone(),
                    color: Color::WHITE.with_alpha(0.0),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, SPRITE_LIT_OVERLAY_Z),
                Visibility::Hidden,
                ChildOf(tile),
            ))
            .id();
        commands.entity(tile).insert(LitOverlayLink(overlay));
    }
}

fn spawn_tilemap_lit_overlays(
    mut commands: Commands,
    mut atlases: ResMut<LitAtlases>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    map_info: Res<MapInfo>,
    cells: Query<(&TilePos, &TilemapId, &TileTextureIndex), Without<LitOverlayLink>>,
    tilemap_textures: Query<&TilemapTexture>,
) {
    for cell in map_info
        .ground_entities
        .values()
        .chain(map_info.forbidden_areas.values())
        .copied()
    {
        let Ok((tile_pos, tilemap_id, texture_index)) = cells.get(cell) else {
            continue;
        };
        let Ok(TilemapTexture::Single(source)) = tilemap_textures.get(tilemap_id.0) else {
            warn_once!("The ground tilemap does not use a single texture, so its glow is disabled");
            continue;
        };
        if atlases.ground.source.is_none() {
            atlases.ground.source = Some(source.clone());
        }
        let layout = atlases
            .ground_layout
            .get_or_insert_with(|| {
                layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::new(map_info.tile_size.x as u32, map_info.tile_size.y as u32),
                    GROUND_ATLAS_COLUMNS,
                    GROUND_ATLAS_COLUMNS,
                    None,
                    None,
                ))
            })
            .clone();
        let position = GridCoords::from(*tile_pos).to_world_pos(&map_info);
        let overlay = commands
            .spawn((
                Name::new("TilemapLitOverlay"),
                TilemapLitOverlay,
                GlowPulses::default(),
                Sprite {
                    image: atlases.ground.lit.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout,
                        index: texture_index.0 as usize,
                    }),
                    color: Color::WHITE.with_alpha(0.0),
                    ..default()
                },
                Transform::from_translation(position.extend(TILEMAP_LIT_OVERLAY_Z)),
                Visibility::Hidden,
                ChildOf(tilemap_id.0),
            ))
            .id();
        commands.entity(cell).insert(LitOverlayLink(overlay));
    }
}

fn build_lit_atlas(
    mut atlases: ResMut<LitAtlases>,
    config: Res<GameConfig>,
    mut images: ResMut<Assets<Image>>,
    mut image_events: MessageReader<AssetEvent<Image>>,
) {
    let events: Vec<_> = image_events.read().copied().collect();
    let wanted = (
        config.effects.beam_glow_lightness,
        config.effects.beam_glow_chroma,
    );
    atlases.tiles.rebuild(wanted, &mut images, &events, "tile");
    atlases.ground.rebuild(wanted, &mut images, &events, "ground");
}

fn apply_glow_effect(
    config: Res<GameConfig>,
    beams: Query<(&GridCoords, &Beam), Changed<GridCoords>>,
    map_info: Res<MapInfo>,
    links: Query<&LitOverlayLink>,
    mut overlays: Query<&mut GlowPulses>,
) {
    let effects_config = &config.effects;
    let fade_in = effects_config.beam_glow_fade_in_ms as f32 / 1000.0;
    let hold = effects_config.beam_glow_hold_ms as f32 / 1000.0;
    let fade_out = effects_config.beam_glow_fade_out_ms as f32 / 1000.0;
    let neighbor_delay = effects_config.beam_glow_neighbor_delay_ms as f32 / 1000.0;

    let mut push = |position: GridCoords, peak: f32, delay: f32| {
        let claimed = map_info.get_claimed_entity_by_position(position);
        let ground = map_info
            .ground_entities
            .get(&position)
            .or(map_info.forbidden_areas.get(&position))
            .copied();
        for entity in [claimed, ground].into_iter().flatten() {
            let Ok(link) = links.get(entity) else {
                continue;
            };
            let Ok(mut pulses) = overlays.get_mut(link.0) else {
                continue;
            };
            pulses.0.push(GlowPulse {
                elapsed: 0.0,
                delay,
                peak,
                fade_in,
                hold,
                fade_out,
            });
        }
    };

    for (grid_coords, beam) in &beams {
        push(*grid_coords, 1.0, 0.0);
        for direction in [
            GridCoords::new(1, 0),
            GridCoords::new(-1, 0),
            GridCoords::new(0, 1),
            GridCoords::new(0, -1),
        ] {
            push(
                *grid_coords + direction,
                effects_config.beam_glow_neighbor_peak,
                neighbor_delay,
            );
        }
        if beam.direction.x != 0 || beam.direction.y != 0 {
            push(
                *grid_coords + beam.direction + beam.direction,
                effects_config.beam_glow_neighbor_peak,
                neighbor_delay,
            );
        }
    }
}

fn update_lit_overlays(
    time: Res<Time>,
    mut overlays: Query<(&mut GlowPulses, &mut Sprite, &mut Visibility)>,
) {
    let delta = time.delta_secs();
    for (mut pulses, mut sprite, mut visibility) in &mut overlays {
        if pulses.0.is_empty() && *visibility == Visibility::Hidden {
            continue;
        }
        let mut strength = 0.0_f32;
        pulses.0.retain_mut(|pulse| {
            pulse.elapsed += delta;
            match pulse.alpha() {
                Some(alpha) => {
                    strength = strength.max(alpha);
                    true
                }
                None => false,
            }
        });
        let color = Color::WHITE.with_alpha(strength);
        if sprite.color != color {
            sprite.color = color;
        }
        visibility.set_if_neq(if strength > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn sync_lit_overlay_frames(
    tiles: Query<&Sprite, (With<GlowEffectTarget>, Without<SpriteLitOverlay>)>,
    mut overlays: Query<(&ChildOf, &mut Sprite, &Visibility), With<SpriteLitOverlay>>,
) {
    for (child_of, mut sprite, visibility) in &mut overlays {
        if *visibility == Visibility::Hidden {
            continue;
        }
        let Ok(tile_sprite) = tiles.get(child_of.parent()) else {
            continue;
        };
        let (Some(tile_atlas), Some(overlay_atlas)) =
            (&tile_sprite.texture_atlas, &sprite.texture_atlas)
        else {
            continue;
        };
        if tile_atlas.index != overlay_atlas.index {
            let index = tile_atlas.index;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = index;
            }
        }
    }
}

fn clear_glow(
    mut overlays: Query<(&mut GlowPulses, &mut Sprite, &mut Visibility)>,
) {
    for (mut pulses, mut sprite, mut visibility) in &mut overlays {
        pulses.0.clear();
        sprite.color = Color::WHITE.with_alpha(0.0);
        *visibility = Visibility::Hidden;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension};
    use bevy_tweening::TweeningPlugin;

    const T: EffectKind = EffectKind::Translate { ms: 10 };
    const S: EffectKind = EffectKind::Settle { ms: 10 };
    const K: EffectKind = EffectKind::Knockback { ms: 10 };
    const D: EffectKind = EffectKind::DeathBounce {
        intensity: 8.0,
        bounce_count: 3,
        decay: 0.33,
    };

    fn punch() -> EffectKind {
        EffectKind::ParryPunch {
            sprite: Entity::PLACEHOLDER,
            peak: 1.5,
            secs: 0.2,
        }
    }

    fn flash(ms: u64) -> EffectKind {
        EffectKind::DamageFlash {
            sprite: Entity::PLACEHOLDER,
            ms,
        }
    }

    #[test]
    fn decide_root_table() {
        use Decision::*;
        let rows = [
            (T, [Start, Replace, Queue, Drop]),
            (S, [Start, Replace, Queue, Drop]),
            (K, [Start, Replace, Replace, Drop]),
            (D, [Start, Replace, Queue, Drop]),
        ];
        for (incoming, expected) in rows {
            assert_eq!(decide(&incoming, None), expected[0], "{incoming:?} none");
            assert_eq!(decide(&incoming, Some(&T)), expected[1], "{incoming:?} T");
            assert_eq!(decide(&incoming, Some(&S)), expected[1], "{incoming:?} S");
            assert_eq!(decide(&incoming, Some(&K)), expected[2], "{incoming:?} K");
            assert_eq!(decide(&incoming, Some(&D)), expected[3], "{incoming:?} D");
        }
    }

    #[test]
    fn decide_sprite_channels_always_replace() {
        assert_eq!(decide(&punch(), None), Decision::Start);
        assert_eq!(decide(&punch(), Some(&punch())), Decision::Replace);
        assert_eq!(decide(&flash(1), None), Decision::Start);
        assert_eq!(decide(&flash(1), Some(&flash(2))), Decision::Replace);
    }

    #[test]
    fn merge_queued_keeps_higher_rank() {
        assert_eq!(merge_queued(None, T), Some(T));
        assert_eq!(merge_queued(Some(T), S), Some(S));
        assert_eq!(merge_queued(Some(S), T), Some(S));
        assert_eq!(merge_queued(Some(T), D), Some(D));
        assert_eq!(merge_queued(Some(D), S), Some(D));
        assert_eq!(merge_queued(Some(S), S), Some(S));
    }

    #[test]
    fn fold_translate_and_knockback_gives_knockback() {
        let plan = fold_requests(None, &[K, T]);
        assert_eq!(plan.spawn, Some((K, None)));
        assert_eq!(plan.new_then, None);
    }

    #[test]
    fn fold_knockback_and_death_queues_death() {
        let plan = fold_requests(None, &[D, K]);
        assert_eq!(plan.spawn, Some((K, Some(D))));
    }

    #[test]
    fn fold_settle_and_translate_gives_settle() {
        let plan = fold_requests(None, &[S, T]);
        assert_eq!(plan.spawn, Some((S, None)));
    }

    #[test]
    fn fold_drag_replace_carries_queued_death() {
        let plan = fold_requests(Some((K, Some(D))), &[K]);
        assert_eq!(plan.spawn, Some((K, Some(D))));
    }

    #[test]
    fn fold_replace_discards_dropped_then() {
        let plan = fold_requests(Some((T, Some(T))), &[S]);
        assert_eq!(plan.spawn, Some((S, None)));
    }

    #[test]
    fn fold_death_drops_translate() {
        let plan = fold_requests(Some((D, None)), &[T]);
        assert_eq!(plan, ChannelPlan::default());
    }

    #[test]
    fn fold_queue_updates_then_in_place() {
        let plan = fold_requests(Some((K, None)), &[T]);
        assert_eq!(plan.spawn, None);
        assert_eq!(plan.new_then, Some(Some(T)));
        let plan = fold_requests(Some((K, Some(D))), &[T]);
        assert_eq!(plan, ChannelPlan::default());
    }

    #[test]
    fn fold_same_frame_sprite_requests_give_one_spawn() {
        let plan = fold_requests(None, &[flash(1), flash(2)]);
        assert_eq!(plan.spawn, Some((flash(2), None)));
    }

    #[test]
    fn two_flash_requests_in_one_frame_give_one_driver() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TweeningPlugin));
        app.add_message::<EffectRequest>();
        app.add_systems(Update, resolve_effect_requests);
        let sprite = app.world_mut().spawn(Sprite::default()).id();
        let owner = app.world_mut().spawn_empty().id();
        for ms in [100, 200] {
            app.world_mut().write_message(EffectRequest {
                owner,
                kind: EffectKind::DamageFlash { sprite, ms },
            });
        }
        app.update();
        let linked = app.world().get::<EffectDrivers>(owner).unwrap().iter().count();
        assert_eq!(linked, 1);
        let drivers = app
            .world_mut()
            .query::<&EffectDriver>()
            .iter(app.world())
            .count();
        assert_eq!(drivers, 1);
    }

    fn image_from(pixels: &[[u8; 4]]) -> Image {
        Image::new(
            Extent3d {
                width: pixels.len() as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels.iter().flatten().copied().collect(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }

    #[test]
    fn identity_at_zero_lightness_full_chroma() {
        let source = image_from(&[[200, 40, 90, 255], [10, 120, 250, 128]]);
        let lit = make_lit_image(&source, 0.0, 1.0).unwrap();
        for x in 0..2 {
            let a = source.get_color_at(x, 0).unwrap().to_srgba();
            let b = lit.get_color_at(x, 0).unwrap().to_srgba();
            assert!((a.red - b.red).abs() < 0.01);
            assert!((a.green - b.green).abs() < 0.01);
            assert!((a.blue - b.blue).abs() < 0.01);
        }
    }

    #[test]
    fn alpha_is_preserved() {
        let source = image_from(&[[200, 40, 90, 77]]);
        let lit = make_lit_image(&source, 0.5, 0.5).unwrap();
        assert_eq!(lit.data.as_ref().unwrap()[3], 77);
    }

    #[test]
    fn red_gets_lighter_and_less_saturated() {
        let source = image_from(&[[255, 0, 0, 255]]);
        let lit = make_lit_image(&source, 0.4, 0.6).unwrap();
        let before = Oklcha::from(source.get_color_at(0, 0).unwrap());
        let after = Oklcha::from(lit.get_color_at(0, 0).unwrap());
        assert!(after.lightness > before.lightness);
        assert!(after.chroma < before.chroma);
    }

    #[test]
    fn transparent_pixels_stay_transparent() {
        let source = image_from(&[[12, 34, 56, 0]]);
        let lit = make_lit_image(&source, 0.5, 0.5).unwrap();
        assert_eq!(lit.data.as_ref().unwrap(), &vec![12, 34, 56, 0]);
    }

    #[test]
    fn unsupported_format_returns_none() {
        let mut source = image_from(&[[0, 0, 0, 255]]);
        source.texture_descriptor.format = TextureFormat::R8Unorm;
        assert!(make_lit_image(&source, 0.5, 0.5).is_none());
    }
}
