---
id: TASK-11
title: Effect drivers and resolver refactor
status: In Progress
assignee: []
created_date: '2026-10-04 18:00'
updated_date: '2026-10-04 17:11'
labels: []
milestone: m-5
dependencies: []
references:
  - src/plugins/effects.rs
  - src/components/effects.rs
  - src/plugins/schedule.rs
  - src/plugins/round/state.rs
  - backlog/docs/doc-10 - 008-Effects-plugin.md
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Run every player effect tween through one resolver that owns each animated channel, instead of inserting `TweenAnim` directly and coordinating with gates (`ActiveTransformEffect`, `PendingDeathBounce`, `Without<IsKnockedBack>` filters).

Recommended model (Option B): every player effect runs as a **driver** entity (one tween, linked to its owner with a `DriverOf` / `EffectDrivers` relationship). A single resolver decides, per owner and channel, whether a new effect starts, replaces, queues behind, or is dropped.

Fallback (Option A): same requests, same `decide()` and same fold, but the resolver inserts `TweenAnim` directly on the player root and keeps the record in `ActiveTransformEffect { kind, then }` (reliable because the resolver is its only writer). Only Stage 3's spawn and completion code differ. Switch to A if Stage 3 has driver-specific trouble.

Why: the transform-effect interplay already caused a freeze bug (af40482), the ownership rules take long prose in doc-10, and DECKBUILDING adds more effects on the same channel (Barrier repel, Body Blocker, drag ordering).

Out of scope: the glow effect (`apply_glow_effect`, lit overlays) has no tweens and must not change. Tiles keep direct `TweenAnim` (see Stage 5).

Line numbers below were taken on 2026-10-04 at commit d9c08fd; re-check them before starting.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
### Stage 1: fix the knockback ordering on its own (do this first)

Goal: `apply_knockback` always sees a `KnockbackEffect` inserted in the same frame.

Problem today: `apply_knockback` is only ordered before `apply_translate_effect` and `apply_death_effect` (`effects.rs:103,106`). The marker comes through Commands from `GameplaySet::Movement` (`controller.rs:52,142,145,157,160`) and `GameplaySet::Damage` (`damage.rs:80`). Whether it is visible this frame or next depends on where the executor places the system.

Changes:
- `src/plugins/schedule.rs`: add `GameplaySet::Displacement` to the enum and to the `.chain()`, between `Damage` and `Presentation`. Doc comment: "Applies knockback after damage and before presentation."
- `src/plugins/effects.rs:103`: `apply_knockback.in_set(GameplaySet::Displacement).before(apply_translate_effect)`. The ordered edge makes Bevy insert a sync point after Movement and Damage. Do not add a `RoundPhase` gate.
- `apply_knockback` (`effects.rs:250-283`): also skip the slide when `Health.current <= 0.0` (query `(…, Has<IsDead>, Option<&Health>)`). Once the ordering is deterministic, a lethal beam hit would otherwise always slide before `IsDead` lands; today it usually does not. See design question 4.

Must stay the same: drag re-locks on every beam step; entry damage still fires after a knockback (`apply_owned_tile_entry_damage` now sees the `GridCoords` change one frame later, which is fine because `Changed` is measured since its last run).

Verify: `cargo check`, `cargo test --features dev`. Manual (user): drag, death during a drag, collision bump.

Rollback: revert the one commit.

### Stage 2: driver base, sprite effects only (no behaviour change)

New types (`src/components/effects.rs`):
```rust
/// The animated value an effect writes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectChannel { RootTranslation, SpriteScale, SpriteColor }

/// One effect request or running effect, with the data needed to build its tween.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EffectKind {
    Translate { ms: u64 },
    Settle { ms: u64 },
    Knockback { ms: u64 },
    DeathBounce { intensity: f32, bounce_count: usize, decay: f32 },
    ParryPunch { sprite: Entity, peak: f32, secs: f32 },
    DamageFlash { sprite: Entity, ms: u64 },
}
impl EffectKind { pub fn channel(&self) -> EffectChannel; pub fn rank(&self) -> u8; }

/// A carrier entity that runs one effect tween on its owner's channel.
#[derive(Component, Debug)]
pub struct EffectDriver { pub owner: Entity, pub kind: EffectKind, pub then: Option<EffectKind> }

/// Links a driver to the entity whose effect it runs.
#[derive(Component)]
#[relationship(relationship_target = EffectDrivers)]
pub struct DriverOf(pub Entity);

/// The live effect drivers of an entity.
#[derive(Component, Default)]
#[relationship_target(relationship = DriverOf, linked_spawn)]
pub struct EffectDrivers(Vec<Entity>);
```
- Rank: Translate 0, Settle 1, Knockback 2, DeathBounce 3; sprite kinds 0.
- Do not use `ChildOf` for the driver link: `sprite_child` reads `children.first()` (`effects.rs:366-369`).
- Translate, Settle and Knockback carry no destination: a driver builds its tween when it starts, from the current `Transform.translation` to `GridCoords.to_translation(&map_info)`. DeathBounce starts from `RestingTranslation` (as `effects.rs:572`).

New message: `EffectRequest { owner: Entity, kind: EffectKind }`, internal to effects, registered in `effects::plugin` with `app.add_message`. Note it in doc-9 as internal (moving it to `messages.rs` is a convention call).

New internal sets, inside `GameplaySet::Presentation`: `configure_sets(Update, (EffectsSet::Request, EffectsSet::Complete, EffectsSet::Resolve).chain().in_set(GameplaySet::Presentation))`, plus `EffectsSet::Resolve.before(bevy_tweening::AnimationSystem::AnimationUpdate)`.

Pure functions (unit-tested):
```rust
enum Decision { Start, Replace, Queue, Drop }
fn decide(incoming: &EffectKind, current: Option<&EffectKind>) -> Decision;
fn merge_queued(existing: Option<EffectKind>, incoming: EffectKind) -> Option<EffectKind>;
struct ChannelPlan { spawn: Option<(EffectKind, Option<EffectKind>)>, new_then: Option<Option<EffectKind>> }
fn fold_requests(live: Option<(EffectKind, Option<EffectKind>)>, requests: &[EffectKind]) -> ChannelPlan;
```

Systems:
- `resolve_effect_requests` (Resolve): read every `EffectRequest`; drop non-DeathBounce requests on owners with `IsDead`; group by (owner, channel); call `fold_requests` against the live driver (found via `EffectDrivers` + `EffectDriver::kind.channel()`); apply the plan (despawn old + spawn new, or update `then` in place, or nothing).
- `start_effect` (helper): build the `TweenAnim` and spawn `(Name::new("Fx:<Kind>"), EffectDriver, DriverOf(owner), AnimTarget::component::<Transform|Sprite>(target), TweenAnim)`. Target is the owner for RootTranslation, the sprite for the sprite channels.
- `on_effect_completed` (Complete): read `AnimCompletedEvent`, look up `EffectDriver` from `anim_entity`, despawn it, call the per-kind completion hook (no-op in this stage), and if `then` is set call `start_effect(then)`. Runs before Resolve with a sync point between.

Ported: `apply_damage_effect` (`effects.rs:371-396`) writes `DamageFlash { sprite, ms }`; `apply_parry_scale_effect` (`592-618`) writes `ParryPunch` and still always removes `ParryScaleEffectTarget`. Both move to `EffectsSet::Request`.

Removed: `ParryScaleDriver`, `DamageFlashDriver` (`components/effects.rs:153-163`), `on_damage_flash_completed` (`effects.rs:398-408`), `on_parry_scale_completed` (`620-630`).

Must stay the same: a repeated hit restarts the flash, a repeated parry restarts the punch (sprite channels are always Replace); flash and punch on the same sprite run together; sprite drivers are not cleared on round reset (despawning one halfway would leave the sprite red or scaled).

Verify: `cargo check`, `cargo test --features dev`. Unit tests: sprite-channel `decide` always Replace; same-frame double request gives one spawn. One headless integration test (`MinimalPlugins` + `TweeningPlugin` + resolver): two `DamageFlash` requests for one owner in one frame give exactly one `EffectDriver`, linked in `EffectDrivers`. Dev-only (`#[cfg(feature = "dev")]`) system that warns when an owner has two drivers on one channel. Manual (user): parry punch and damage flash overlapping; two quick parries; two quick hits.

Rollback: revert the stage.

### Stage 3: player position effects through the resolver (one atomic commit)

Every write to a player's `Transform.translation` tween goes through the resolver, and the visual gates are deleted. Must be one commit: mixing direct `TweenAnim` writers and driver writers on the same field resolves by bevy_tweening query order (`lib.rs:2159-2161`), which is arbitrary.

Ported (all to `EffectsSet::Request`):
- `apply_translate_effect` (`effects.rs:285-314`): query becomes `(Entity, Option<&MovementSlide>), (Changed<GridCoords>, With<TranslateEffectTarget>)`; the `Without<KnockbackEffect/IsKnockedBack/IsDead>` filters go; writes `Translate { ms }`.
- `apply_movement_settle` (`316-348`): `Has` checks go; writes `Settle { ms: move_repeat_rate_ms }`; still always removes `MovementSettle`.
- `apply_knockback` (`250-283`): keeps its gameplay half (dead / zero-health check, ground check, `*coords = target`, insert `IsKnockedBack(Timer)`); the `TweenAnim` and `ActiveTransformEffect` inserts become a `Knockback { ms: knockback_tween_ms }` request; still always removes `KnockbackEffect`.
- `apply_death_effect` (`412-444`): inserts `IsDead` and writes `DeathBounce { 8.0, 3, 0.33 }`; no more `BounceEffect`, `PendingDeathBounce`, `BounceEffectTarget` on players; `knockback_pending` query goes.
- `apply_bounce_effect` (`558-581`): becomes the tile-only claim bounce; stops inserting `ActiveTransformEffect`.
- DeathBounce completion hook: insert `Visibility::Hidden` on the owner (replaces `on_death_effect_completed`).
- New `clear_root_effect_drivers` on `OnExit(RoundPhase::Outcome)`: despawn RootTranslation drivers, including any queued `then`. No ordering needed against `reset_round` (`round/state.rs:83`).

Removed: `ActiveTransformEffect`, `TransformEffectKind`, `PendingDeathBounce` (`components/effects.rs:128-142`); `start_deferred_death_bounce` (`447-465`), `on_death_effect_completed` (`472-489`), `on_knockback_tween_completed` (`493-509`); in `round/state.rs:365-376` drop `PendingDeathBounce`, `ActiveTransformEffect` and `TweenAnim` from the removal tuple and their imports at `:36` (keep `BounceEffect` and `BounceEffectTarget` there).

Kept unchanged: `IsKnockedBack` + `tick_knockback_lock` (`512-523`, the gameplay input lock, read by `inputs.rs:81`), `KnockbackEffect`, `MovementSlide`, `RestingTranslation`, `sync_resting_translation`.

Must stay the same: translate linear over `MovementSlide`, settle QuadraticOut, knockback QuadraticOut over `knockback_tween_ms`; drag re-aims the slide and refreshes the lock every step; death during a drag bounces only after the last slide; bounce origin is `RestingTranslation`; player hidden after the bounce; after a reset the player appears at spawn instantly.

Verify: `cargo check`, `cargo test --features dev`. Unit tests: the full `decide()` table; `fold_requests` cases (same-frame Translate+Knockback, Knockback+DeathBounce, Settle+Translate, drag replace carrying `then`, DeathBounce dropping Translate); `merge_queued`. Integration test: Knockback then DeathBounce, step time past `knockback_tween_ms`, assert the DeathBounce driver exists. Manual (user):
1. Walk and stop: smooth settle.
2. Collision bump.
3. Dragged across several tiles: no snapping, input locked until the drag ends.
4. Die during a drag: slide finishes, then bounce, then hidden.
5. Die standing still.
6. Parry punch and damage flash during a knockback.
7. Round reset after a death and after a mid-drag timeout: player at spawn and can move.
8. Hitstop during a drag: slide and lock slow down together.

Rollback: revert to the Stage 2 state; Option A swaps in for the spawn and completion code.

### Stage 4 (optional, later): move the gameplay half of knockback out of effects

Move coordinates, the dead and ground checks, the lock and `tick_knockback_lock` into `src/plugins/damage.rs` in `GameplaySet::Displacement` (DECKBUILDING.md:439 already names damage as knockback owner). Optionally replace the `KnockbackEffect` marker with a `KnockbackRequested { entity, direction }` message from `controller.rs` and `damage.rs`; damage writes `EntityKnockedBack { entity }`, effects turns it into a `Knockback` request. Needed before Barrier (#33) and Body Blocker.

Verify: same as Stage 3. Rollback: revert; Stage 3 works without it.

### Stage 5: tiles stay on direct `TweenAnim`

`apply_wave_effect` (`525-556`) and the claim bounce (`animations.rs:99` → `apply_bounce_effect`) keep inserting `TweenAnim` directly: one channel, always replace, which the slot gives for free. Drivers would add a spawn and despawn per beam step. No work.

### Policy table (RootTranslation)

| Incoming ↓ / current → | none | Translate or Settle | Knockback | DeathBounce |
|---|---|---|---|---|
| Translate or Settle | Start | Replace | Queue | Drop |
| Knockback | Start | Replace | Replace | Drop |
| DeathBounce | Start | Replace | Queue | Drop |

ParryPunch and DamageFlash (own channels): always Start or Replace.

Fold rules:
- One queued slot per driver. `merge_queued`: a new queued item replaces the old one if its rank is higher or equal (DeathBounce beats any queued movement; a later Translate or Settle replaces an earlier one).
- On Replace, the old driver's `then` goes through `decide(old_then, Some(new_kind))`: Queue keeps it on the new driver, Drop discards it. A DeathBounce queued behind a drag therefore survives every drag step.
- Same frame: group by (owner, channel), stable-sort by rank ascending (equal rank keeps write order, latest wins), fold through `decide` from the live (kind, then). At most one spawn per channel per frame. Results: Translate + Knockback → Knockback; Knockback + DeathBounce → Knockback with DeathBounce queued; Settle + Translate → Settle.
- Owners with `IsDead`: drop every request except DeathBounce before the fold.
- Translate under Knockback is Queue, not Drop: the lock timer and the tween are not ordered, so input can unlock a frame before the slide ends; a queued Translate rebuilds from the current position and does nothing if the player has not moved.

### Docs per stage

- Stage 1: doc-24 (GameplaySet chain); doc-10 (`apply_knockback` workflow, "Apply Knockback", "Transform effect ownership" wording); doc-11 Damage (knockback picked up in Displacement, entry damage one frame later); CLAUDE.md `schedule` row if it lists sets.
- Stage 2: doc-10 (replace the ParryScaleDriver and damage-flash sections with "Effect drivers and resolver"; update mermaid CRUD charts); doc-9 (internal `EffectRequest`).
- Stage 3: doc-10 (rewrite "Transform effect ownership" as the policy table and fold rules; delete the sections for `ActiveTransformEffect`, `PendingDeathBounce`, `start_deferred_death_bounce`, `on_knockback_tween_completed`, `on_death_effect_completed`; `IsKnockedBack` is now only the input lock; add `clear_root_effect_drivers`); doc-15 Round (reset list; effects clears its own drivers); doc-2 Input (`MovementSettle` wording); doc-5 Animations (claim bounce has no tag); doc-24 (new `OnExit(Outcome)` system).
- Stage 4: doc-11, doc-4 Controller, doc-10, doc-9, DECKBUILDING.md knockback mentions (lines 29 and 765).

### Open design questions (for game-designer)

1. Death bounce waits for the slide to finish, or for the lock to expire? Blocks Stage 3. Default: the slide (driver completion), identical today because both use `knockback_tween_ms`.
2. Stun duration separate from slide duration? Blocks Stage 4 and Barrier. Default: one knob; if split later, add `knockback_lock_ms` with a higher/lower comment.
3. Does being dragged block parrying? Blocks nothing here (input gate at `inputs.rs:81`). Default: keep it blocked.
4. Does a lethal hit still play its last knockback slide before the death bounce? Blocks Stage 1's health check. Default: no slide.

### Risks and rollback

- Stage 1: entry damage after a knockback one frame later; Presentation may see effect markers in a different order. Rollback: one commit.
- Stage 2: the completion → follow-up → resolve sync point (mitigated by chained sets and the dev invariant warning); drivers get stuck if the target loses `Transform` or `Sprite` (`bevy_tweening` `lib.rs:2304-2307`), which sprites never do today. Rollback: Stage 1 state.
- Stage 3 (riskiest): any writer left on direct insert means two writers on one field; Settle-wins-same-frame is a small visible difference; reset timing; relationship despawn when an owner is despawned (players never are today). Rollback: Stage 2 state, or Option A.
- Stage 4: ownership moves across plugins; keep docs in sync. Rollback: revert.
- All stages: never run the game yourself; the user runs each manual checklist.
<!-- SECTION:PLAN:END -->
