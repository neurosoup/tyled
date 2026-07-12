---
id: doc-12
title: '[010] Abilities plugin'
type: other
created_date: '2026-07-12 12:00'
updated_date: '2026-07-12 12:00'
---
# Abilities Plugin

The home of the beam-ability deckbuilding substrate. Its job is to let each player carry an ordered list of draftable abilities that later modify beam behaviour, the charge economy, and tile contests through resolver systems.

This is **Stage F1** of a staged rollout, and F1 is *substrate only — no new content*. In this stage the plugin merely registers the ability component types so the `bevy-inspector-egui` world inspector can display each player's (empty) loadout. The `on_resolve` / `on_claim` descriptor resolvers — the systems that read the drafted abilities and change beam outcomes — land in **Stage F2** and will be added here.

## Concepts

Two distinct types make up the substrate (the split is intentional):

- `AbilityList(Vec<AbilityDescriptor>)` (`src/components/abilities.rs`) — the **per-player, persistent, draftable** list. It is attached (empty) to every player in the Maps plugin's `initialize_players`. Straight Shot is the *implicit baseline* applied first and is **not** stored here, so an empty list means "Straight-only" — the layer-1 balancing control. Later stages append drafted descriptors.
- `AbilityDescriptor` (`src/components/abilities.rs`) — a single draftable ability, kept as pure, `Reflect`-serialisable data (no `Entity` or runtime handles) so a future loadout can be authored in RON, hot-reloaded via `file_watcher`, and persisted across sessions. Stage F1 declares one variant, `Backfill`, which is not yet attached to any player (it is wired in Stage F2).

Related but owned by the Beam plugin: `BeamBehavior { Straight, Backfill }` (`src/components/beam.rs`) is the **per-beam, transient** resolved execution mode carried on each `Beam` (it replaced the former `Beam::inverted` bool). In Stage F2 a player's `AbilityList` containing `AbilityDescriptor::Backfill` will drive selection of `BeamBehavior::Backfill` at spawn time; in Stage F1 every beam is `Straight`.

## Plugin workflow

- Startup phase
    - (none)
- Registration (at plugin build)
    - `register_type::<AbilityList>()` and `register_type::<AbilityDescriptor>()` — makes the loadout visible in the world inspector.
- Update phase
    - (none yet — ability resolver systems arrive in Stage F2.)

## Plugin Systems

None in Stage F1. The plugin only registers reflected component types. The `on_resolve` / `on_claim` resolvers, and the messages they consume (`TileClaimed`, `ChargeSpent`, and later `ChargeRegen`, all declared in the Messages plugin), are added in Stage F2.

## Components, Resources and Messages CRUD

Stage F1 introduces no systems, so there are no read/write flows to diagram yet. The component definitions:

- `AbilityList` — `#[derive(Component, Reflect, Default, Clone)]`, attached to each player by `initialize_players` (Maps plugin) with an empty `Vec`.
- `AbilityDescriptor` — `#[derive(Reflect, Clone, Debug)]`, one variant `Backfill` (unattached in F1).

CRUD/mermaid diagrams will be added alongside the resolver systems in Stage F2.
