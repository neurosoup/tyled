---
id: doc-12
title: '[010] Abilities plugin'
type: other
created_date: '2026-07-12 12:00'
updated_date: '2026-08-05 12:00'
---
# Abilities Plugin

The home of the beam-ability substrate. Its job is to let each player carry an ordered list of draftable abilities that modify beam behaviour, the charge economy, and tile contests through resolver systems.

The plugin registers the ability component types (so the `bevy-inspector-egui` world inspector can display each player's loadout) *and* owns `PlayerLoadouts`, the hardcoded per-player starting kits read by the Maps plugin's `initialize_players`. There is no draft UI, so loadouts are assigned in code and swapped between runs by editing `PlayerLoadouts`. The `Lance` ability is handed to players via `PlayerLoadouts` and, in the Beam plugin, both drives `BeamBehavior::Lance` selection **and** is what lets a player fire at all from an already-claimed tile — without it, firing from a claimed tile is refused (no beam, no charge). Beam-behavior selection and this fire gate (both via `resolve_fire`, reading a player's `AbilityList` at fire/spawn time) are Beam plugin concerns; this plugin holds no resolver systems.

## Concepts

Three types make up the substrate (the split is intentional):

- `AbilityList(Vec<AbilityDescriptor>)` (`src/components/abilities.rs`) — the **per-player, persistent, draftable** list. It is attached to every player in the Maps plugin's `initialize_players` from `PlayerLoadouts`. Straight Shot is the *implicit baseline* applied first and is **not** stored here, so an empty list means "Straight-only" — the layer-1 balancing control. Later stages append drafted descriptors.
- `AbilityDescriptor` (`src/components/abilities.rs`) — a single draftable ability, kept as pure, `Reflect`-serialisable data (no `Entity` or runtime handles) so a future loadout can be authored in RON, hot-reloaded via `file_watcher`, and persisted across sessions. Declares three variants: `Lance`, `Overpenetration`, `SolarPanels`.
- `PlayerLoadouts` (`src/components/abilities.rs`) — a **resource** holding the hardcoded P1/P2 starting descriptor lists (there is no draft UI). Inserted by this plugin's build — currently **empty for P1 (Straight-only) and `[Overpenetration]` for P2** — and read by `initialize_players` via `for_player(player_id)`. It is the single place to assign or swap kits between runs; adding a descriptor to a player's list grants that player the ability.

Related but owned by the Beam plugin: `BeamBehavior { Straight, Lance }` (`src/components/beam.rs`) is the **per-beam, transient** resolved execution mode carried on each `Beam`. `spawn_beam` selects `BeamBehavior::Lance` when a beam is fired from already-claimed ground **and** the firing player's `AbilityList` contains `AbilityDescriptor::Lance`; from unclaimed ground it stays `Straight`. Firing from a claimed tile **without** `Lance` is refused outright — no beam, no charge (see the Beam plugin doc).

The other two variants are also resolved entirely in the Beam plugin, reading a player's `AbilityList`: `Overpenetration` lets a `Straight`-behavior beam punch through an enemy-claimed tile it would otherwise stop short of, flipping that tile instead of stopping at its border (see `beam.rs`'s `overpenetrates_enemy_tile`; the Bot plugin's decision logic also weighs chasing down an Overpenetration flip). `SolarPanels` passively regenerates a player's `BeamCharges` on a fixed tick, gaining one charge per `config.economy.solar_panels_tiles_per_charge` tiles the player currently owns (`beam.rs`'s `regen_charges_from_solar_panels`).

## Plugin workflow

- Startup phase
    - (none)
- Registration (at plugin build)
    - `register_type::<AbilityList>()` and `register_type::<AbilityDescriptor>()` — makes the loadout visible in the world inspector.
    - `insert_resource(PlayerLoadouts { .. })` — the hardcoded default kits (currently empty for P1, `[Overpenetration]` for P2).
- Update phase
    - (none — this plugin has no Update systems.)

## Plugin Systems

None. The plugin only registers reflected component types and inserts the `PlayerLoadouts` resource. None of the three abilities need a claim-side resolver here — all their behavior (beam-behavior selection, the fire gate, the Overpenetration flip check, and the SolarPanels regen tick) is spawn/step-side logic living in the Beam plugin.

## Components, Resources and Messages CRUD

The plugin has no systems, so there are no per-frame read/write flows to diagram. Definitions and where they are used:

- `AbilityList` — `#[derive(Component, Reflect, Default, Clone)]`, attached to each player by `initialize_players` (Maps plugin) from `PlayerLoadouts::for_player`. Read in the Beam plugin (by `spawn_beam` to select `BeamBehavior`) and in the Input plugin (by `handle_characters_input` to gate firing), both via `resolve_fire`.
- `AbilityDescriptor` — `#[derive(Reflect, Clone, Debug, PartialEq, Eq)]`, three variants: `Lance`, `Overpenetration`, `SolarPanels`.
- `PlayerLoadouts` — `#[derive(Resource, Clone)]`, inserted here, read by `initialize_players` (Maps plugin).
