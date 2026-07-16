# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Model Assignment Rules

- Architecture decisions and reviews: Use Opus
- Implementation tasks (new features, refactors): Use Sonnet  
- Simple edits, formatting, renaming, commits and push: Use Haiku

# About this project

Tyled is a 2-player local real-time strategy game built with Bevy 0.18. Players shoot beams to claim tiles; claimed tiles damage opponents who walk on them. See README.md for full gameplay rules.

## Tooling

Prefer the `rust-analyzer` LSP (via the `LSP` tool) over `grep` when navigating code — go-to-definition, find-references, and hover give accurate, type-aware results without false positives from text search.

## Commands

```bash
cargo run          # Run the game (with hot reloading via file_watcher feature)
cargo build        # Debug build
cargo check        # Type-check without building
```

The project uses **Rust nightly** (see `rust-toolchain.toml`) and is configured to link with `clang`/`lld` for faster compile times (`.cargo/config.toml`).

## Architecture

### Plugin structure

`AppPlugin` in `src/lib.rs` registers all plugins in order. Each plugin is a `pub(crate) fn plugin(app: &mut App)` in `src/plugins/`. Plugin registration order matters for system ordering.

| Plugin | Responsibility |
|--------|---------------|
| `defaults` | Bevy `DefaultPlugins` with nearest-neighbor filtering |
| `messages` | Registers all game-wide `Message` types |
| `maps` | Loads Tiled maps, populates `MapInfo` resource, initializes players/tiles/HP bars |
| `round` | Feature folder (`src/plugins/round/`, one `plugin()` entry). Submodules: `state` — the `RoundPhase` state machine (`Loading`/`Starting`/`Playing`/`Outcome` — gameplay systems run only `in_state(Playing)`) + the global `Countdown` resource (started on `MapCreated`, ticked only while `Playing`); `intro` — the "3 · 2 · 1 · GO!" round-start banner shown `in_state(Starting)`, drawn via `text`'s `spawn_label` onto the overlay camera. Shared `spawn_round_label` helper in `round/mod.rs` |
| `camera` | Spawns all cameras: pixel-perfect main camera via `bevy_smooth_pixel_camera` (`PixelCamera`, layer 2 viewport, order 2) with dynamic zoom snapping to `ZOOM_LEVELS` (`[1/4, 1/3, 1/2, 1]`), HUD camera on `RenderLayers(1)` order 3, and the fixed overlay camera on `RenderLayers(3)` order 4 (no clear, `FixedVertical`). Owns the `OVERLAY_RENDER_LAYER` const |
| `text` | Bitmap-font text-rendering service: loads the shared font atlas (`assets/font.png`) into the `FontAtlas` resource and provides `spawn_label(text, transform, render_layers)` to compose a string into per-glyph sprites (`src/plugins/text.rs`) |
| `inputs` | `leafwing-input-manager` setup; translates player input to `EntityMoved`/`BeamFired` messages; gates `BeamFired` when player's `BeamCharges` is exhausted |
| `controller` | Reads `EntityMoved` messages, validates against `MapInfo`, updates player `GridCoords` |
| `beam` | Steps `Beam` entities (invisible logical tracers) each tick, resolves them via `BeamResolved` messages, decrements `BeamCharges` on the firing player — the beam is *visually* represented by a shock wave of bouncing tiles (`BounceEffect`) rather than a visible projectile |
| `claim` | Reads `BeamResolved`, mutates the authoritative `ClaimedTile::owner`, emits `TileClaimed`; the single home for tile-ownership changes (and future `on_resolve`/`on_claim` ability resolvers) |
| `damage` | Ticks every 500ms; damages players standing on opponent-owned tiles; emits `DamageableDied` |
| `effects` | Tweening effects: movement slide (`TranslateEffectTarget`), bounce (`BounceEffect`/`WaveEffect`), damage flash (`DamageEffectTarget`), death bounce |
| `animations` | `bevy_spritesheet_animation` setup; attaches and switches world-space player/tile sprite animations |
| `hud` | All HUD animations (render layer 1): lerps HP bar `scale.x` to match `Health.ratio()`; drives the rolling-odometer numeric counters (beam charges, claimed-tile percentage, round countdown) |
| `debug` | `bevy-inspector-egui` world inspector (always enabled) |

### Communication pattern

Systems communicate via **messages** (from `bevy_ecs_tiled`), not direct queries across plugins. All message types are declared in `src/plugins/messages.rs` and registered in the `messages` plugin:

- `EntityMoved { entity, position }` — player wants to move
- `BeamFired { owner, origin, direction }` — player fired a beam
- `BeamResolved { position, owner }` — beam landed on a tile
- `DamageableDied { entity }` — an entity's HP hit zero

### Grid coordinate system

`GridCoords` (`src/components/grid_coords.rs`) is the game's logical position type. It wraps `(i32, i32)` and provides conversions to/from Bevy's `TilePos` and world-space `Vec3` via `MapInfo`. The `to_translation_with_z_index` method encodes depth using `y + z_index * z_offset` to fake 2.5D layering.

### MapInfo resource

`MapInfo` (`src/plugins/maps.rs`) is the central spatial index. It is populated once after the `CurrentLevel` map fires a `MapCreated` event, and holds:
- `ground_entities`: valid walkable positions
- `claimed_entities`: one `ClaimedTile` entity per ground tile (always present, owner is `None` until claimed)
- `forbidden_areas`: impassable tiles (beam passes through but cannot resolve there)

### Cameras (and adding a new one)

The game renders two Tiled maps simultaneously — the game board (default render layer, main camera → off-screen texture → ViewportCamera) and the HUD (render layer 1, HUD camera with a fixed top viewport). **Every camera is spawned in one place — `initialize_cameras` (`src/plugins/camera.rs`) — so render-layer and render-order allocations stay coherent and auditable in a single file.** Four cameras are active at runtime:

| Camera | Layer | Order | Role |
|--------|-------|-------|------|
| Main (`PixelCamera`) | default (no `RenderLayers`) | 0 | Renders game world to off-screen texture |
| ViewportCamera (auto-spawned) | 2 | 2 | Composites off-screen texture onto window with subpixel smoothing |
| HUD camera | 1 | 3 | Renders HUD strip at top of window |
| Overlay camera | 3 | 4 | Full-window screen-space round-phase banners (content owned by the `overlay` plugin) |

Render layers have named consts in `camera.rs`: `HUD_RENDER_LAYER` (1), `LEVEL_RENDER_LAYER` (2), `OVERLAY_RENDER_LAYER` (3). HUD entities carry `Propagate(RenderLayers::from_layers(&[1]))` so all their children are also on layer 1.

**To add a camera**, spawn it in `initialize_cameras` and:
1. Give it a **unique render layer** (add a new `*_RENDER_LAYER` const) and a **unique `order`** that stacks correctly — higher order composites on top (main viewport 2 < HUD 3 < overlay 4).
2. **Attach an explicit `RenderLayers` to it** — this is mandatory for *every* camera except the main one. `update_camera` isolates the main world camera as the only `Camera2d` *without* a `RenderLayers` (`Single<…, Without<RenderLayers>>`); a second layer-less camera makes that query match more than one entity and panics.
3. For a camera that draws *on top of* the world (HUD, overlay), use `clear_color: ClearColorConfig::None` so it doesn't wipe what's beneath, and `Msaa::Off` to match the others (mismatched sample counts trigger a wgpu validation error).
4. Content for a dedicated camera lives in its own plugin/feature (e.g. the `hud` plugin, or the round `intro` submodule for the overlay camera); only the camera entity itself belongs in `camera.rs`.

### Effect components

Effects are driven by marker components rather than events:
- `TranslateEffectTarget` — any entity with this + `GridCoords` gets a slide tween when `GridCoords` changes
- `WaveEffectTarget` — `ClaimedTile` entities; bounce when a `BounceEffect` source moves onto their tile
- `BounceEffectTarget` — one-shot bounce; removed after tween starts
- `DamageEffectTarget` — color flash on child sprite when `Health` changes; death bounce + `IsDead` on `DamageableDied`

### Controls (hardcoded)

| Action | Player 1 | Player 2 |
|--------|----------|----------|
| Move | WASD | Arrow keys |
| Lock direction | Q | Right Shift |
| Shoot | Tab | / |

Input ticks at 75ms intervals; beam step ticks at 62.5ms (0.0625s).

### Documentation

Each plugin and component has a corresponding doc in `backlog/docs/`. These docs are kept in sync with the code and are authoritative: use them as the primary source for understanding a plugin's systems, queries, message flows, and component lifecycle before reading the source. When modifying a plugin or component — adding/removing systems, changing queries, altering message fields, renaming things — update the matching doc to keep the workflow descriptions, system descriptions, and CRUD/mermaid diagrams in sync.

### HUD numeric counters (adding a new one)

Every HUD number is a **rolling-odometer digit group**: one `Digit` Tiled object per decimal place (`Digit::position` = 0 for ones, 1 for tens, 2 for hundreds…), all sharing a **marker component** that binds the group to a value source. `initialize_digit_animations` (`hud.rs`) auto-attaches a `SpritesheetAnimation` to *every* `With<Digit>` entity and builds the shared `DigitAnimations` from→to table once; each `animate_*` system just picks a target value and calls `animate_digit`, which is idempotent (no-ops when the shown digit already matches, so it can run every frame without change-detection).

**Reflect / Tiled generation:** marker + custom components derive `#[derive(Component, Reflect, Default)]` + `#[reflect(Component, Default)]`. Bevy 0.18 **auto-registers** `Reflect` types (no `register_type` call), and `bevy_ecs_tiled`'s `user_properties` feature exports the whole registry to `tiled_types_export.json` on every startup. So the loop for a new component is: define it → `cargo run` once to regenerate the export → in Tiled, re-import types and select the new class as an object property. The type appears as its full path, e.g. `tyled::components::countdown::CountdownDigit`.

To add a counter:
1. Define a zero-sized marker in `src/components/` (mirror `BeamChargesDigit` / `CountdownDigit`) and re-export it in `components/mod.rs`.
2. `cargo run` once so the marker lands in `tiled_types_export.json`.
3. In Tiled (`assets/hud2.tmx`, `Numbers` object group): duplicate a digit object per decimal place, set each `Digit.position`, and add the marker class. Include `Player` (with `player_id`) **only** for per-player counters; omit it for global ones.
4. Add an `animate_*` system in `hud.rs` and register it in the plugin. Per-player counters call `animate_digits_for_player::<Marker>` (filters by `Player`, value from a domain component gated on `Changed<…>`); global counters query `With<Marker>` and call `animate_digit` directly (value from a resource, e.g. `Countdown`). The value is always owned by a domain plugin — hud only reads it.

### Asset pipeline

Sprites are Aseprite/Pixelorama files exported to PNG. Tiled maps (`.tmx`) and tilesets (`.tsx`) reference these PNGs. `bevy_ecs_tiled` loads them via `asset_server`; custom Tiled object properties map to Bevy components via `#[derive(Reflect)]` with `#[reflect(Component)]`.

## Planned: Beam Ability Deckbuilding System

`DECKBUILDING.md` (repo root) is an actively-developed design plan to rework beam behaviors and the beam-charges economy into a Balatro-style deckbuilding system — draftable abilities built on triggers/enablers/payoffs/stacks vocabulary, with a growing roster, four archetypes, an upcoming parry mechanic, and a staged multi-session rollout. Not yet implemented. Load that file before starting any session that touches beam behavior, beam charges, tile claiming/contesting, or ability/draft systems, and keep it updated as design decisions are made or sessions land — it is the authoritative source for this effort, not this file.
