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
| `camera` | Pixel-perfect main camera via `bevy_smooth_pixel_camera` (`PixelCamera`, layer 2 viewport, order 2) with dynamic zoom snapping to `ZOOM_LEVELS` (`[1/4, 1/3, 1/2, 1]`) + HUD camera on `RenderLayers(1)`, order 3 |
| `inputs` | `leafwing-input-manager` setup; translates player input to `EntityMoved`/`BeamFired` messages; gates `BeamFired` when player's `BeamCharges` is exhausted |
| `controller` | Reads `EntityMoved` messages, validates against `MapInfo`, updates player `GridCoords` |
| `beam` | Steps `Beam` entities (invisible logical tracers) each tick, resolves them via `BeamResolved` messages, decrements `BeamCharges` on the firing player — the beam is *visually* represented by a shock wave of bouncing tiles (`BounceEffect`) rather than a visible projectile |
| `claim` | Reads `BeamResolved`, mutates the authoritative `ClaimedTile::owner`, emits `TileClaimed`; the single home for tile-ownership changes (and future `on_resolve`/`on_claim` ability resolvers) |
| `damage` | Ticks every 500ms; damages players standing on opponent-owned tiles; emits `DamageableDied` |
| `effects` | Tweening effects: movement slide (`TranslateEffectTarget`), bounce (`BounceEffect`/`WaveEffect`), damage flash (`DamageEffectTarget`), death bounce |
| `animations` | `bevy_spritesheet_animation` setup; attaches and switches player/tile sprite animations; lerps HP bar `scale.x` to match `Health.ratio()` |
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

### Two maps, three cameras

The game renders two Tiled maps simultaneously:
- `level1.tmx` — the game board (default render layer, main camera → off-screen texture → ViewportCamera)
- `hud.tmx` — HP bar containers (render layer 1, HUD camera with a fixed top viewport)

Three cameras are active at runtime:

| Camera | Layer | Order | Role |
|--------|-------|-------|------|
| Main (`PixelCamera`) | default (no `RenderLayers`) | 0 | Renders game world to off-screen texture |
| ViewportCamera (auto-spawned) | 2 | 2 | Composites off-screen texture onto window with subpixel smoothing |
| HUD camera | 1 | 3 | Renders HUD strip at top of window |

HUD entities carry `Propagate(RenderLayers::from_layers(&[1]))` so all their children are also on layer 1.

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

### Asset pipeline

Sprites are Aseprite/Pixelorama files exported to PNG. Tiled maps (`.tmx`) and tilesets (`.tsx`) reference these PNGs. `bevy_ecs_tiled` loads them via `asset_server`; custom Tiled object properties map to Bevy components via `#[derive(Reflect)]` with `#[reflect(Component)]`.

## Planned: Beam Ability Deckbuilding System

`DECKBUILDING.md` (repo root) is an actively-developed design plan to rework beam behaviors and the beam-charges economy into a Balatro-style deckbuilding system — draftable abilities built on triggers/enablers/payoffs/stacks vocabulary, with a growing roster, four archetypes, an upcoming parry mechanic, and a staged multi-session rollout. Not yet implemented. Load that file before starting any session that touches beam behavior, beam charges, tile claiming/contesting, or ability/draft systems, and keep it updated as design decisions are made or sessions land — it is the authoritative source for this effort, not this file.
