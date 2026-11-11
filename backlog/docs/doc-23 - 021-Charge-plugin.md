---
id: doc-23
title: '[021] Charge plugin'
type: other
created_date: '2026-08-13 12:00'
updated_date: '2026-11-11 12:00'
---
# Charge Plugin

This plugin changes the players' `BeamCharges` outside of firing: it gives charges back and keeps them under a limit. The code is in `src/plugins/charge.rs`.

## Key ideas

- **Solar Panels**: every `charge.solar_panels_tick_secs`, a player with this ability gains one charge per `charge.solar_panels_tiles_per_charge` tiles they own. The gain stops at `BeamCharges::max`.
- **Unclaimed-tile cap**: a charge is only useful to claim a free tile. So every frame, each player's charges are lowered to the number of unclaimed tiles, if they are above it. They are never raised.
- The cap is safe because claimed tiles never become free again during a round.
- A regen writes a `ChargeRegen` message. Nothing reads it yet. The cap writes no message; the HUD already reacts to `Changed<BeamCharges>`.
- This plugin is the place for future abilities that change charges.

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph startup ["Startup"]
  setup_solar_panels_timer["`**setup_solar_panels_timer**`"]
end

subgraph playing ["Update, only during Playing"]
  claim["`**GameplaySet::Claim**
  writes ClaimedTileCount`"]
  subgraph economy ["GameplaySet::Economy"]
    regen["`**regen_charges**`"]
    cap["`**cap_charges_to_unclaimed_tiles**`"]
    regen --> cap
  end
  claim --> economy
end

subgraph every ["Update, every phase"]
  resync["`**resync_solar_panels_timer**
  dev only`"]
end

startup ~~~ playing
playing ~~~ every
```

- Economy runs after Claim in the same frame, so both systems see the tiles claimed earlier in that frame.
- The cap runs after the regen, so a regen can never leave charges above the cap.

## Plugin Systems

### Setup Solar Panels Timer

- Runs once at `Startup`. Inserts `SolarPanelsTimer`, a repeating timer.

### Resync Solar Panels Timer (dev)

- When `GameConfig` changes, sets the new timer duration. The elapsed time is kept.

### Regen Charges

- Ticks the timer. When it ends, gives charges to each player with Solar Panels.
- Writes `ChargeRegen` with the real gain, when the gain is above 0.

### Cap Charges To Unclaimed Tiles

- Runs every frame, for every player.
- Unclaimed tiles = ground tiles minus the claimed tiles of all players.
- Lowers `BeamCharges::current` to that number when it is higher.

## Components, Resources and Messages CRUD

### Read GameConfig and Time

Used in the following systems:
- **setup_solar_panels_timer**, **resync_solar_panels_timer**: read `charge.solar_panels_tick_secs`.
- **regen_charges**: reads `Time` and `charge.solar_panels_tiles_per_charge`.

### SolarPanelsTimer resource

Used in the following systems:
- **setup_solar_panels_timer**: inserts it.
- **resync_solar_panels_timer** (dev): changes its duration.
- **regen_charges**: ticks it.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update`")):::system-group
setup_solar_panels_timer["`**setup_solar_panels_timer**`"]
resync_solar_panels_timer["`**resync_solar_panels_timer**
dev only`"]
regen["`**regen_charges**`"]

startup -.-> setup_solar_panels_timer
update -.-> resync_solar_panels_timer
update -.-> regen

world@{ shape: st-rect, label: "World" }
timer_res@{ shape: doc, label: "SolarPanelsTimer" }

timer_res --> |belongs to| world

setup_solar_panels_timer ---> |inserts resource| timer_res
resync_solar_panels_timer ---> |writes duration| timer_res
regen ---> |ticks| timer_res
```

### Query players (regen)

Used in the following systems:
- **regen_charges**: checks `SolarPanels`, reads the player's tile count and adds charges.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
regen["`**regen_charges**`"]

update -.-> regen

players_query{{"`players`"}}:::query
regen ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_ability_list>"`**AbilityList**`"] --> |belongs to| player_entity
pe_tile_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity
pe_beam_charges>"`**BeamCharges**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_ability_list
players_query ---> |reads| pe_tile_count
players_query ---> |writes| pe_beam_charges
```

### Read MapInfo resource

Used in the following systems:
- **cap_charges_to_unclaimed_tiles**: reads the number of ground tiles.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
cap["`**cap_charges_to_unclaimed_tiles**`"]

update -.-> cap

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

mi_ground>"`**ground_entities**`"]

map_info_res --> |belongs to| world
mi_ground --> |field of| map_info_res

cap ---> |reads| mi_ground
```

### Query claimed tile counts and charges (cap)

Used in the following systems:
- **cap_charges_to_unclaimed_tiles**: adds up the tile counts of all players, then lowers the charges of every player.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
cap["`**cap_charges_to_unclaimed_tiles**`"]

update -.-> cap

counts_query{{"`counts`"}}:::query
charges_query{{"`charges`"}}:::query
cap ---> counts_query
cap ---> charges_query

player_entity@{ shape: st-rect, label: "Player" }

pe_tile_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity
pe_beam_charges>"`**BeamCharges**`"] --> |belongs to| player_entity

counts_query ---> |reads| pe_tile_count
charges_query ---> |writes| pe_beam_charges
```

### Write ChargeRegen messages

Used in the following systems:
- **regen_charges**: writes one message per player whose charges went up.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
regen["`**regen_charges**`"]

update -.-> regen

charge_regen_message(["`**ChargeRegen**`"])

regen ---> |writes| charge_regen_message
```

Definitions:
- `SolarPanelsTimer`: public `Resource` with the repeating Solar Panels timer.
- `ChargeRegen`: `Message` with `owner` and `amount`, declared in `messages.rs`.
