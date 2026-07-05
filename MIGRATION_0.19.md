# Bevy 0.19 migration — staging branch (`bevy-0.19`)

**Status: this branch does NOT compile yet. That is expected.**

`Cargo.toml` bumps `bevy` to 0.19 and the four ready ecosystem crates, but `bevy_ecs_tiled`
and `bevy_smooth_pixel_camera` still require Bevy 0.18. Cargo does not hard-fail resolution —
it lets **two versions of `bevy_ecs` coexist** in the graph (0.18.1 via the two blocked crates,
0.19.0 via `bevy` + the bumped crates). Compilation then fails with trait-mismatch errors, e.g.
`bevy_ecs_tiled`'s `TiledEvent<ObjectCreated>` "is not a `Message`" (it implements the 0.18
`Message` trait, our systems expect the 0.19 one). This is the blocker manifesting; it clears
once both blocked crates depend on Bevy 0.19.

## Done on this branch

- Dropped `bevy_ecs_ldtk` (see `drop-ldtk` branch / PR — no 0.19 release, was unused).
- Bumped to 0.19-ready versions:
  - `bevy` 0.18 → **0.19**
  - `bevy-inspector-egui` 0.36 → **0.37**
  - `bevy_spritesheet_animation` 6.1 → **7.0**
  - `bevy_tweening` 0.15 → **0.16**
  - `leafwing-input-manager` 0.20 → **0.21**
- Left the two blockers pinned to 0.18 with `# BLOCKED` comments in `Cargo.toml`.

## Blockers (recheck on crates.io)

| Crate | Needs | Watch |
|---|---|---|
| `bevy_ecs_tiled` | first release depending on `bevy 0.19` (currently 0.12.0 → bevy 0.18) | also its `bevy_ecs_tilemap` dep must hit 0.19 |
| `bevy_smooth_pixel_camera` | release depending on `bevy 0.19` (currently 0.4.1 → bevy 0.18.1) | — |

## Flip checklist (do when both blockers publish 0.19 support)

1. Bump both blocked pins in `Cargo.toml`, delete the `# BLOCKED` comments.
2. `cargo update && cargo check --features dev` — then work through real compiler errors below.

### Code deltas to verify against Bevy 0.19 docs + the compiler

These are *unverified* — the build can't run until step 1, so let the compiler confirm each.
Most of the 0.18→0.19 guide (Resources-as-Components, scene/BSN, text→parley, materials,
atmosphere, render-graph) does **not** apply: 2D game, no meshes/text/materials/scenes/audio,
and no `#[derive(Resource)]` type is also a `Component`.

- **Import paths** (moved before, may move again): `bevy::camera::Viewport`,
  `bevy::camera::visibility::RenderLayers` (`src/plugins/camera.rs:7`),
  `bevy::render::view::Msaa` (`camera.rs:9`), `bevy::sprite::Anchor` (`src/plugins/maps.rs:8`),
  `bevy::platform::collections::HashMap` (`maps.rs:8`).
- **`If<Res<T>>`** system param — `src/plugins/animations.rs:69,120,157,158`.
- **`HierarchyPropagatePlugin::<RenderLayers>` + `Propagate`** — `camera.rs:6,59,93`.
- **`OrthographicProjection::default_2d()`** — `camera.rs`.
- **Feature flags:** `audio` no longer implied by `2d` — we use no audio and build
  `default-features = false, features = ["2d"]`, so expected no-op; confirm no missing-feature error.
- **Message API** (`add_message` / `MessageReader` / `MessageWriter`, `src/plugins/messages.rs`) —
  confirm unchanged in 0.19.

### Bumped-crate breaking-change spot-checks

- **`bevy_spritesheet_animation` 6.1 → 7.0** (major bump, changelog lists no API breaks): re-check
  `Spritesheet::new`, the builder API (`create_animation`/`add_cell`/`add_partial_row`/
  `set_repetitions`/`set_duration`/`set_direction`/`set_easing`/`build`), `AnimationDuration`,
  `AnimationDirection`, `Easing`/`EasingVariety` in `src/plugins/animations.rs`.
- **`bevy_tweening` 0.15 → 0.16**, **`leafwing-input-manager` 0.20 → 0.21**,
  **`bevy-inspector-egui` 0.36 → 0.37** (feature-flag reorg) — confirm current usage compiles.
- **`bevy_ecs_tiled`** 0.19 release may rename `TiledEvent`/`MapCreated`/`ObjectCreated` or change
  `user_properties` reflection registration. Confirm `#[reflect(Component)]` types still
  auto-register (there are no explicit `register_type` calls today).

3. Full build + runtime verification: `cargo run` and exercise movement (WASD / arrows), beam fire
   (Tab / `/`) + tile claim shock-wave, damage tick on opponent tiles, HP-bar lerp, death bounce,
   camera zoom-snap + pixel-perfect compositing, and the `dev` egui world inspector.
4. Update any affected `backlog/docs/` plugin docs, then merge `bevy-0.19` → `main`.
