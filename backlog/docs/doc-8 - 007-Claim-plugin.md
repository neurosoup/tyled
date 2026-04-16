---
id: doc-8
title: '[008] Damage plugin'
type: other
created_date: '2026-06-15 12:00'
updated_date: '2026-06-15 12:00'
---
# Damage Plugin

Contains systems responsible for applying damage to players standing on opponent-owned tiles, and for emitting `DamageableDied` messages when a player's health reaches zero. Damage is applied on a fixed timer cadence rather than every frame, giving players a brief window to move off a dangerous tile.

## Plugin workflow

- Startup phase
    - `setup_timer` inserts the `DamageTimer` resource — a repeating `Timer` with a 500 ms period.
- Update phase
    - `apply_damage`:
        - Ticks the `DamageTimer` each frame
        - When the timer fires, iterates all `Player` entities and checks whether their current `GridCoords` sits on a tile owned by an opposing player
            - Reads:
                - `DamageTimer` resource
                - `Player` entity `GridCoords` and `Entity`
                - `ClaimedTile` components on claimed tile entities (via `MapInfo::claimed_entities`)
                - `MapInfo` resource (to resolve `GridCoords` → claimed tile entity)
            - Writes:
                - Applies 1.0 damage to the player's `Health` component each tick while on an opponent tile
                - Emits a `DamageableDied` message when the player's `Health` reaches zero

## Plugin Systems

### Setup Timer

Runs once at startup. Inserts the `DamageTimer` resource — a repeating `Timer` configured to fire every 500 ms — which gates how often the `apply_damage` system deals damage to players.

### Apply Damage

Runs every frame. Ticks the `DamageTimer` and, when the timer finishes:

1. Iterates all `Player` entities, reading their `Entity` and `GridCoords`.
2. Resolves the `GridCoords` to a claimed tile entity via `MapInfo::claimed_entities`.
3. Reads the `ClaimedTile` component on that entity to determine the current owner.
4. If the owner is a different player than the one standing on the tile, decrements that player's `Health` by 1.0.
5. If the player's `Health` has reached zero, emits a `DamageableDied` message carrying the player's `Entity`.

## Components, Resources and Messages CRUD

### Read DamageTimer resource

Used in the following systems:
- **apply_damage**: ticks and checks the damage cadence timer each frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update`")):::system-group
setup_timer["`**setup_timer**`"]
apply_damage["`**apply_damage**`"]

startup -.-> setup_timer
update -.-> apply_damage

world@{ shape: st-rect, label: "World" }
damage_timer_res@{ shape: doc, label: "DamageTimer" }

damage_timer_res --> |belongs to| world

setup_timer ---> |inserts resource| damage_timer_res
apply_damage ---> |reads & ticks| damage_timer_res
```

### Read MapInfo resource

Used in the following systems:
- **apply_damage**: used to resolve player `GridCoords` to a claimed tile entity via `MapInfo::claimed_entities`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_damage["`**apply_damage**`"]

update -.-> apply_damage

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

apply_damage ---> |reads `claimed_entities`| map_info_res
```

### Query Player entities

Used in the following systems:
- **apply_damage**: reads the `Entity` and `GridCoords` of every `Player`-marked entity to determine their position each damage tick

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_damage["`**apply_damage**`"]

update -.-> apply_damage

player_entities_query{{"`player_entities`"}}:::query
apply_damage ---> player_entities_query

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity

player_entities_query ---> |reads| pe_entity
player_entities_query ---> |reads| pe_grid_coords
player_entities_query -..-> |filter With| pe_player
```

### Query ClaimedTile entities

Used in the following systems:
- **apply_damage**: reads `ClaimedTile::owner` on claimed tile entities to determine if a player is standing on an opponent-owned tile

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_damage["`**apply_damage**`"]

update -.-> apply_damage

claimed_entities_query{{"`claimed_entities`"}}:::query
apply_damage ---> claimed_entities_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_claimed>"`**ClaimedTile**`"] --> |belongs to| claimed_tile_entity
ct_owner>"`**owner**`"] --> |field of| ct_claimed

claimed_entities_query ---> |reads| ct_claimed
```

### Write Health component

Used in the following systems:
- **apply_damage**: decrements the `Health` component of the player standing on an opponent tile by 1.0 on each damage tick

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_damage["`**apply_damage**`"]

update -.-> apply_damage

damageable_query{{"`damageable_entities`"}}:::query
apply_damage ---> damageable_query

player_entity@{ shape: st-rect, label: "Player" }

pe_health>"`**Health**`"] --> |belongs to| player_entity

damageable_query ---> |writes| pe_health
```

### Write DamageableDied messages

Used in the following systems:
- **apply_damage**: emits a `DamageableDied` message carrying the player `Entity` when their `Health` reaches zero

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_damage["`**apply_damage**`"]

update -.-> apply_damage

damageable_died_message(["`**DamageableDied**`"])

apply_damage ---> |writes| damageable_died_message
```
