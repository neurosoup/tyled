---
id: doc-11
title: '[009] Damage plugin'
type: other
created_date: '2026-06-02 00:00'
updated_date: '2026-06-02 00:00'
---
# Damage Plugin

Contains systems responsible for dealing damage to entities and emitting `DamageableDied` messages when an entity's HP reaches zero. Three sources of damage are handled: on-enter tile damage the moment a character moves onto an opponent-owned tile *from a non-hostile tile*, periodic tile damage while a character stays on such a tile, and per-step beam damage for any entity sharing a grid position with a moving beam. A private `deal_damage` helper centralises the decrement-and-emit logic used by every source, and a private `is_hostile_tile` helper centralises the "is this coord an opponent-owned claimed tile" check shared by both tile systems.

The on-enter spike fires only on a *fresh incursion* — a step whose destination is hostile but whose origin was not. It reads the origin from a `PreviousGridCoords` component that the entry system updates on every observed move, so crossing a contiguous run of enemy tiles costs a single spike plus the ongoing standing tick, not a spike per tile.

The two tile systems are complementary and chained (entry before poll): on-enter (`Changed<GridCoords>`) cannot phase-miss a tile the way the timer poll can when a character crosses faster than the 500 ms sample, while the poll covers the case on-enter cannot — a tile becoming hostile *beneath* a stationary character, whose `GridCoords` never changes. Damage amounts are named constants: `ON_ENTER_DAMAGE` (5.0) per hostile tile entered and `STANDING_DAMAGE` (1.0) per 500 ms poll tick.

## Plugin workflow

- Startup phase
    - `setup_timer` inserts the `DamageTimer` resource (500 ms repeating).
- Update phase
    - Apply Owned Tile Entry Damage (chained before the poll):
        - Triggers when a `Character`'s `GridCoords` changes (moved onto, or knocked onto, a tile)
            - Reads:
                - `GridCoords`, `PreviousGridCoords`, and `Health` on `Character` entities (filtered by `Changed<GridCoords>`)
                - `ClaimedTile` component on ground tile entities (to identify the owner)
                - `MapInfo` resource (to resolve a `GridCoords` → claimed tile entity)
            - Writes:
                - Updates `PreviousGridCoords` to the current `GridCoords` on every observed move
                - Decrements `Health::current` by `ON_ENTER_DAMAGE` on a character that moved onto an opponent-owned tile *from a non-hostile tile*
                - Resets `DamageTimer` so the poll below cannot double-hit the same frame
                - Writes a `DamageableDied` message if `Health::current` drops to zero
    - Apply Owned Tile Damage:
        - Runs every `DamageTimer` tick (500 ms)
            - Reads:
                - `DamageTimer` resource (for tick gating)
                - `GridCoords` and `Health` on `Character` entities
                - `ClaimedTile` component on ground tile entities (to identify the owner)
                - `MapInfo` resource (to resolve a `GridCoords` → claimed tile entity)
            - Writes:
                - Decrements `Health::current` by `STANDING_DAMAGE` on characters standing on an opponent-owned tile
                - Writes a `DamageableDied` message if `Health::current` drops to zero
    - Apply Beam Damage:
        - Triggers when any `Beam` entity's `GridCoords` changes (i.e. each beam step)
            - Reads:
                - `Beam.owner`, `Beam.direction`, and `GridCoords` on beam entities (filtered by `Changed<GridCoords>`)
                - `Entity`, `GridCoords`, and `Health` on all damageable entities
            - Writes:
                - Decrements `Health::current` on any entity whose `GridCoords` matches the beam head, excluding the beam's owner
                - Writes a `DamageableDied` message if `Health::current` drops to zero
                - Inserts a `KnockbackEffect` (direction = opposite of beam direction) on the hit entity

## Plugin Systems

### Setup Timer

Runs once at startup. Inserts the `DamageTimer` resource — a repeating `Timer` with a 500 ms period — that gates how frequently tile damage is applied.

### Apply Owned Tile Entry Damage

Triggered by Bevy's `Changed<GridCoords>` filter on `Character` entities — runs only for characters whose position changed this frame (moved by the controller, or knocked back by the effects plugin). For each live character it first records the current `GridCoords` into `PreviousGridCoords` (on every move, hostile or not), then applies the spike only on a *fresh incursion*: the destination is hostile (via `is_hostile_tile`) **and** the tile it came from — read from the pre-update `PreviousGridCoords` — was not. On such an entry it calls `deal_damage` with `ON_ENTER_DAMAGE` and resets `DamageTimer` so the poll system chained after it cannot apply standing damage the same frame. Gating on the origin tile means a character walking through a contiguous enemy region is charged once, not per tile, while the standing poll keeps ticking as it traverses. Reading and updating `PreviousGridCoords` in the same system keeps it order-independent of the controller and effects plugins. This is also the fix for a character crossing a claimed tile faster than the 500 ms poll samples — change detection cannot phase-miss the entered tile. Runs *before* `apply_owned_tile_damage` in a `chain()`.

### Apply Owned Tile Damage

Runs every `DamageTimer` tick. Iterates over all `Character` entities with `Health` and checks (via `is_hostile_tile`) whether their current `GridCoords` maps to a `ClaimedTile` owned by a different entity. If so, calls `deal_damage` to decrement `Health::current` by `STANDING_DAMAGE` and emit `DamageableDied` if the entity is now dead. Entities with `Health::current <= 0` are skipped up front so already-dead entities do not generate duplicate death messages. Besides standing damage-over-time, this covers the case the entry system cannot: a tile becoming hostile beneath a stationary character (no `GridCoords` change, so no entry trigger).

### Apply Beam Damage

Triggered by Bevy's `Changed<GridCoords>` filter — runs only for beam entities whose position changed this frame (i.e. each beam step). For every such beam, iterates all entities with `Health`, skipping dead entities and the beam's own owner, and calls `deal_damage` for any whose `GridCoords` matches the beam head position. On a hit, also inserts a `KnockbackEffect` component on the damaged entity (direction = `-beam.direction`) so the effects plugin can push the entity one tile back and play the slide+bounce animation.

### deal_damage (helper)

Private helper, not a system. Decrements `Health::current` by the given `amount` and, if the result is ≤ 0, writes a `DamageableDied` message via the provided `MessageWriter`. All three damage systems delegate to this function to avoid duplicating the decrement-and-emit pattern.

### is_hostile_tile (helper)

Private helper, not a system. Given `MapInfo`, the `ClaimedTile` query, a `GridCoords`, and a character `Entity`, returns whether that coord holds a claimed tile owned by a *different* entity. Resolves the coord via `MapInfo::get_claimed_entity_by_position`, looks up the `ClaimedTile`, and checks `owner.is_some_and(|o| o != character)`. Shared by `apply_owned_tile_entry_damage` and `apply_owned_tile_damage`.

## Components, Resources and Messages CRUD

### Insert DamageTimer resource

Used in the following systems:
- **setup_timer**: inserts the `DamageTimer` resource at startup

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
setup_timer["`**setup_timer**`"]

startup -.-> setup_timer

world@{ shape: st-rect, label: "World" }
damage_timer_res@{ shape: doc, label: "DamageTimer" }

damage_timer_res --> |belongs to| world

setup_timer ---> |inserts| damage_timer_res
```

### Read/Write DamageTimer resource

Used in the following systems:
- **apply_owned_tile_entry_damage**: resets the timer on a hostile-tile entry so the poll cannot double-hit the same frame
- **apply_owned_tile_damage**: ticks the timer and gates damage application to every 500 ms

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
apply_owned_tile_damage["`**apply_owned_tile_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage

world@{ shape: st-rect, label: "World" }
damage_timer_res@{ shape: doc, label: "DamageTimer" }

damage_timer_res --> |belongs to| world

apply_owned_tile_entry_damage ---> |resets| damage_timer_res
apply_owned_tile_damage ---> |ticks| damage_timer_res
```

### Query Character entities (tile damage)

Used in the following systems:
- **apply_owned_tile_entry_damage**: reads `GridCoords` (filtered by `Changed<GridCoords>`) to locate the entered tile and `PreviousGridCoords` for the origin tile, writes `PreviousGridCoords` (to the current position) and `Health::current` to apply on-enter damage
- **apply_owned_tile_damage**: reads `GridCoords` to locate the tile, writes `Health::current` to apply standing damage

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
apply_owned_tile_damage["`**apply_owned_tile_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage

entry_characters_query{{"`characters_query (Changed#60;GridCoords#62;)`"}}:::query
characters_query{{"`characters_query`"}}:::query
apply_owned_tile_entry_damage ---> entry_characters_query
apply_owned_tile_damage ---> characters_query

character_entity@{ shape: st-rect, label: "Character Entity" }

ce_entity>"`**Entity**`"] --> |belongs to| character_entity
ce_coords>"`**GridCoords**`"] --> |belongs to| character_entity
ce_prev>"`**PreviousGridCoords**`"] --> |belongs to| character_entity
ce_health>"`**Health**`"] --> |belongs to| character_entity
ce_character>"`**Character**`"] --> |belongs to| character_entity

entry_characters_query ---> |reads| ce_entity
entry_characters_query ---> |reads| ce_coords
entry_characters_query ---> |reads/writes| ce_prev
entry_characters_query ---> |writes| ce_health
characters_query ---> |reads| ce_entity
characters_query ---> |reads| ce_coords
characters_query ---> |writes| ce_health
```

### Read MapInfo resource

Used in the following systems:
- **apply_owned_tile_entry_damage**: resolves the entered `GridCoords` to the claimed tile entity via `get_claimed_entity_by_position` (through `is_hostile_tile`)
- **apply_owned_tile_damage**: resolves a `GridCoords` to the claimed tile entity via `get_claimed_entity_by_position` (through `is_hostile_tile`)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
apply_owned_tile_damage["`**apply_owned_tile_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

apply_owned_tile_entry_damage ---> |reads `get_claimed_entity_by_position`| map_info_res
apply_owned_tile_damage ---> |reads `get_claimed_entity_by_position`| map_info_res
```

### Query ClaimedTile (tile damage)

Used in the following systems:
- **apply_owned_tile_entry_damage**: reads `ClaimedTile::owner` (via `is_hostile_tile`) to determine whether the entered tile belongs to a different entity
- **apply_owned_tile_damage**: reads `ClaimedTile::owner` (via `is_hostile_tile`) to determine whether the standing tile belongs to a different entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
apply_owned_tile_damage["`**apply_owned_tile_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage

claimed_tiles_query{{"`claimed_tiles_query`"}}:::query
apply_owned_tile_entry_damage ---> claimed_tiles_query
apply_owned_tile_damage ---> claimed_tiles_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_claimed>"`**ClaimedTile**`"] --> |belongs to| claimed_tile_entity
ct_owner>"`**owner**`"] --> |field of| ct_claimed

claimed_tiles_query ---> |reads| ct_owner
```

### Query Beam entities (beam damage)

Used in the following systems:
- **apply_beam_damage**: reads `Beam.owner` and `GridCoords` on beam entities whose position changed this frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_beam_damage["`**apply_beam_damage**`"]

update -.-> apply_beam_damage

beams_query{{"`beams_query (Changed#60;GridCoords#62;)`"}}:::query
apply_beam_damage ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }

be_beam>"`**Beam**`"] --> |belongs to| beam_entity
be_coords>"`**GridCoords**`"] --> |belongs to| beam_entity

beams_query ---> |reads| be_beam
beams_query ---> |reads| be_coords
```

### Query damageable entities (beam damage)

Used in the following systems:
- **apply_beam_damage**: reads `GridCoords` to compare with beam head; writes `Health::current` to apply damage

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_beam_damage["`**apply_beam_damage**`"]

update -.-> apply_beam_damage

damageables_query{{"`damageables_query`"}}:::query
apply_beam_damage ---> damageables_query

damageable_entity@{ shape: st-rect, label: "Damageable Entity" }

de_entity>"`**Entity**`"] --> |belongs to| damageable_entity
de_coords>"`**GridCoords**`"] --> |belongs to| damageable_entity
de_health>"`**Health**`"] --> |belongs to| damageable_entity

damageables_query ---> |reads| de_entity
damageables_query ---> |reads| de_coords
damageables_query ---> |writes| de_health
```

### Write KnockbackEffect (beam damage)

Used in the following systems:
- **apply_beam_damage**: inserts `KnockbackEffect` on the hit entity so the effects plugin slides and bounces it one tile opposite to the beam direction

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_beam_damage["`**apply_beam_damage**`"]

update -.-> apply_beam_damage

damageable_entity@{ shape: st-rect, label: "Damageable Entity" }

de_knockback>"`**KnockbackEffect**`"]

de_knockback --> |inserted on| damageable_entity

apply_beam_damage ---> |inserts component| de_knockback
```

### Write DamageableDied messages

Used in the following systems:
- **apply_owned_tile_entry_damage**: emits `DamageableDied` when a character's HP reaches zero on entering an opponent-owned tile
- **apply_owned_tile_damage**: emits `DamageableDied` when a character's HP reaches zero on an opponent-owned tile
- **apply_beam_damage**: emits `DamageableDied` when an entity's HP reaches zero after a beam hit

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
apply_owned_tile_damage["`**apply_owned_tile_damage**`"]
apply_beam_damage["`**apply_beam_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage
update -.-> apply_beam_damage

damageable_died_message(["`**DamageableDied**`"])

apply_owned_tile_entry_damage ---> |writes| damageable_died_message
apply_owned_tile_damage ---> |writes| damageable_died_message
apply_beam_damage ---> |writes| damageable_died_message
```
