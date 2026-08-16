---
id: doc-23
title: '[021] Charge plugin'
type: other
created_date: '2026-08-13 12:00'
updated_date: '2026-08-13 12:00'
---
# Charge Plugin

Owns `BeamCharges` regen/refund/cost-policy resolvers — the home for any beam ability that modifies a player's charge pool without being a beam-behavior or tile-ownership effect. Today that's just Solar Panels' regen tick; the abilities plugin doc (`backlog/docs/doc-12 - 010-Abilities-plugin.md`) and DECKBUILDING.md's charge-economy roster (§3) queue several more (Salvage, Tithe, Frugal Frontier, Battery Cap) that will land here as their stages ship.

Registered in `AppPlugin` between the Claim and Damage plugins, so its regen tick reads each player's `ClaimedTileCount` after the Claim plugin has written it that frame.

## Plugin workflow

- Startup phase
    - Setup Solar Panels Timer:
        - Reads:
            - `GameConfig` resource (`config.charge.solar_panels_tick_secs`)
        - Writes:
            - Inserts the `SolarPanelsTimer` resource
- Update phase (dev builds only, `#[cfg(feature = "dev")]`)
    - Resync Solar Panels Timer:
        - Reads:
            - `GameConfig` resource, gated on `is_changed()` (hot-reload)
        - Writes:
            - `SolarPanelsTimer`'s duration
- Update phase (gated `run_if(in_state(RoundPhase::Playing))`)
    - Regen Charges From Solar Panels:
        - Reads:
            - `Time` resource (to tick `SolarPanelsTimer`)
            - `GameConfig` resource (`config.charge.solar_panels_tiles_per_charge`)
            - Each player's `AbilityList` (gated on `AbilityDescriptor::SolarPanels`) and `ClaimedTileCount`
        - Writes:
            - Each qualifying player's `BeamCharges::current` (capped at `BeamCharges::max`)
            - Emits a `ChargeRegen` message (`owner`, `amount`) per player who actually gained a charge

## Plugin Systems

### Setup Solar Panels Timer

Runs once at `Startup`. Inserts `SolarPanelsTimer`, a repeating `Timer` seeded from `config.charge.solar_panels_tick_secs`. Mirrors the Beam plugin's `setup_beam_step_timer` pattern.

### Resync Solar Panels Timer

Dev-only (`#[cfg(feature = "dev")]`), same hot-reload pattern as the Beam plugin's `resync_beam_step_timer`: when `GameConfig` changes (the asset watcher picks up an edit to `assets/game_config.ron`), rewrites `SolarPanelsTimer`'s duration to match the current `solar_panels_tick_secs` without resetting its elapsed progress.

### Regen Charges From Solar Panels

Ticks `SolarPanelsTimer`; no-ops until it finishes. On a finished tick, iterates every player with an `AbilityList`, `ClaimedTileCount`, and `BeamCharges`, skipping any player whose `AbilityList` doesn't contain `AbilityDescriptor::SolarPanels`. For each remaining player, computes `gained = tile_count.current / config.charge.solar_panels_tiles_per_charge` (integer division — regen scales in whole-charge steps per tile-count bracket, not continuously), skips if `gained == 0`, otherwise adds `gained` to `BeamCharges::current` (capped at `BeamCharges::max`) and emits a `ChargeRegen { owner, amount: gained }` message. `ChargeRegen` has no consumers yet — it's the ability-system hook for anything that reacts to a regen event (e.g. a future HUD pulse or an ability that piggybacks on regen ticks), same role `TileClaimed`/`ChargeSpent` play for their respective triggers.

## Components, Resources and Messages CRUD

### Read GameConfig (timer setup/resync)

Used in the following systems:
- **setup_solar_panels_timer**: seeds `SolarPanelsTimer`'s duration from `config.charge.solar_panels_tick_secs`
- **resync_solar_panels_timer** (dev-only): rewrites the duration on config hot-reload

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update (dev)`")):::system-group
setup_timer["`**setup_solar_panels_timer**`"]
resync_timer["`**resync_solar_panels_timer**`"]

startup -.-> setup_timer
update -.-> resync_timer

game_config_res@{ shape: doc, label: "GameConfig" }

setup_timer ---> |reads charge.solar_panels_tick_secs| game_config_res
resync_timer ---> |reads charge.solar_panels_tick_secs, is_changed| game_config_res
```

### Write SolarPanelsTimer

Used in the following systems:
- **setup_solar_panels_timer**: inserts the resource at `Startup`
- **resync_solar_panels_timer** (dev-only): rewrites its duration on config hot-reload
- **regen_charges_from_solar_panels**: ticks it every frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update`")):::system-group
setup_timer["`**setup_solar_panels_timer**`"]
resync_timer["`**resync_solar_panels_timer** (dev)`"]
regen["`**regen_charges_from_solar_panels**`"]

startup -.-> setup_timer
update -.-> resync_timer
update -.-> regen

timer_res@{ shape: doc, label: "SolarPanelsTimer" }

setup_timer ---> |inserts| timer_res
resync_timer ---> |writes duration| timer_res
regen ---> |ticks| timer_res
```

### Read AbilityList and ClaimedTileCount (regen)

Used in the following systems:
- **regen_charges_from_solar_panels**: gates the regen on `AbilityDescriptor::SolarPanels`, scales `gained` by `ClaimedTileCount::current`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
regen["`**regen_charges_from_solar_panels**`"]

update -.-> regen

players_query{{"`players (AbilityList, ClaimedTileCount, BeamCharges)`"}}:::query
regen ---> players_query

player_entity@{ shape: st-rect, label: "Player Entity" }

ability_list>"`**AbilityList**`"] --> |belongs to| player_entity
claimed_tile_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity

players_query ---> |reads| ability_list
players_query ---> |reads current| claimed_tile_count
```

### Write BeamCharges (regen)

Used in the following systems:
- **regen_charges_from_solar_panels**: increments `current`, capped at `max`, for each qualifying player on a finished tick

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
regen["`**regen_charges_from_solar_panels**`"]

update -.-> regen

players_query{{"`players (mutable)`"}}:::query
regen ---> players_query

player_entity@{ shape: st-rect, label: "Player Entity" }

beam_charges>"`**BeamCharges**`"] --> |belongs to| player_entity
bc_current>"`**current**`"] --> |field of| beam_charges

players_query ---> |writes, capped at max| bc_current
```

### Write ChargeRegen messages

Used in the following systems:
- **regen_charges_from_solar_panels**: emits one message per player who gained at least one charge (no consumers yet)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
regen["`**regen_charges_from_solar_panels**`"]

update -.-> regen

charge_regen_message(["`**ChargeRegen**`"])

regen ---> |writes| charge_regen_message
```
