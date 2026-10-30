---
id: doc-11
title: '[009] Damage plugin'
type: other
created_date: '2026-06-02 00:00'
updated_date: '2026-10-29 12:00'
---
# Damage Plugin

This plugin removes health from players and writes `DamageableDied` when a player reaches zero health. It also moves players that are knocked back.

## Key ideas

- **Hostile tile**: a tile claimed by another player (`is_hostile_tile`). The Bot plugin uses this helper too.
- **Entry damage** (`damage.on_enter`, 5): a player steps onto a hostile tile from a tile that is not hostile.
- **Standing damage** (`damage.standing`, 1): a player is on a hostile tile when `DamageTimer` ends (every `damage.tick_secs`, 0.5 s).
- **Beam damage** (`damage.beam_contact`, 1): a beam steps onto the tile of a player who does not own the beam. The player is also knocked back.
- **Collision damage** (`damage.collision`, 5): the loser of a collision between two players.
- `deal_damage` removes the health and writes `DamageableDied` when the health reaches zero. All four damage systems use it.
- **Knockback**: a push of one tile. Only `apply_knockback` moves a player because of a knockback.

## Ordering

An arrow means "runs before". Systems with no arrow between them have no fixed order.

```mermaid
---
config:
  theme: dark
---

flowchart TD

setup_timer["`**setup_timer**
Startup`"]

subgraph playing ["Update, only during Playing"]
  subgraph damage ["GameplaySet::Damage"]
    apply_owned_tile_entry_damage["`**apply_owned_tile_entry_damage**`"]
    apply_owned_tile_damage["`**apply_owned_tile_damage**`"]
    apply_beam_damage["`**apply_beam_damage**`"]
    apply_collision_damage["`**apply_collision_damage**`"]
    apply_owned_tile_entry_damage --> apply_owned_tile_damage
  end
  subgraph displacement ["GameplaySet::Displacement"]
    apply_knockback["`**apply_knockback**`"]
  end
  damage --> displacement
end

subgraph always ["Update, every phase"]
  tick_knockback_lock["`**tick_knockback_lock**`"]
  resync_damage_timer["`**resync_damage_timer**
  dev only`"]
end

setup_timer ~~~ playing
playing ~~~ always
tick_knockback_lock ~~~ resync_damage_timer
```

- `tick_knockback_lock` runs in every phase. So a lock still ends after the phase changes.

## Plugin Systems

### Setup Timer

- Runs at `Startup`. Inserts `DamageTimer`, a repeating timer of `damage.tick_secs`.

### Apply Owned Tile Entry Damage

- Runs for players whose `GridCoords` changed this frame: after a move or a knockback.
- Saves the new position in `PreviousGridCoords`. It does this on every move.
- Deals entry damage only when the new tile is hostile and the old tile was not.
- So a player who crosses many enemy tiles in a row takes entry damage once. The standing damage still applies.
- After entry damage, it resets `DamageTimer`. So the standing damage cannot hit in the same frame.
- It sees every tile that a player enters. The 0.5 s timer alone can miss a tile that the player crosses fast.

### Apply Owned Tile Damage

- Ticks `DamageTimer`. When the timer ends, deals standing damage to each player on a hostile tile.
- It also catches a tile that becomes hostile under a player who does not move. That player's `GridCoords` does not change, so the entry system does not run for them.

### Apply Beam Damage

- Runs for beams that moved this frame. Any player on the beam's tile takes beam damage, except the beam's owner.
- It also writes `KnockbackRequested` in the beam's direction.

### Apply Collision Damage

- Reads `CharacterCollision` from the Controller plugin. The loser takes collision damage.

### Apply Knockback

- Reads all `KnockbackRequested` of the frame, from the Controller plugin and from `apply_beam_damage`.
- For each player, only the last request counts. The Controller writes in `Movement`, and this plugin writes later in `Damage`.
- Skips a dead player, a player with zero health, or a target tile that is not ground.
- Else it moves the player one tile, inserts a new `IsKnockedBack` lock (`effects.knockback_tween_ms`) and writes `EntityKnockedBack`.
- The Effects plugin reads `EntityKnockedBack` and plays the slide. The entry damage sees the move on the next frame.

### Tick Knockback Lock

- Ticks each `IsKnockedBack` timer. Removes the lock when the timer ends.

### Resync Damage Timer (dev only)

- When the config changes, sets the `DamageTimer` duration to the new `damage.tick_secs`.

## Components, Resources and Messages CRUD

### Insert DamageTimer resource

Used in the following systems:
- **setup_timer**: inserts `DamageTimer` at startup.

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
- **apply_owned_tile_entry_damage**: resets the timer after entry damage.
- **apply_owned_tile_damage**: ticks the timer. Standing damage applies when it ends.

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
- **apply_owned_tile_entry_damage**: reads the new and old tiles, saves the new one in `PreviousGridCoords`, and lowers `Health`.
- **apply_owned_tile_damage**: reads the tile of each player and lowers `Health`.

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
- **apply_owned_tile_entry_damage**: finds the claimed tile at a position (through `is_hostile_tile`).
- **apply_owned_tile_damage**: same.

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
- **apply_owned_tile_entry_damage**: reads the tile owner (through `is_hostile_tile`).
- **apply_owned_tile_damage**: same.

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
- **apply_beam_damage**: reads the beams that moved this frame.

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
- **apply_beam_damage**: finds the players on the beam's tile and lowers `Health`.

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

### Read CharacterCollision messages and query Health (collision damage)

Used in the following systems:
- **apply_collision_damage**: reads `CharacterCollision` and lowers the `Health` of the loser.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_collision_damage["`**apply_collision_damage**`"]

update -.-> apply_collision_damage

message_reader{{"MessageReader#60;CharacterCollision#62;"}}:::reader
apply_collision_damage ---> message_reader
collision_message(["`**CharacterCollision**`"])
message_reader ---> |reads| collision_message

characters_query{{"`characters_query`"}}:::query
apply_collision_damage ---> characters_query

character_entity@{ shape: st-rect, label: "Character Entity (loser)" }
ce_health>"`**Health**`"] --> |belongs to| character_entity
ce_character>"`**Character**`"] --> |belongs to| character_entity

characters_query ---> |writes| ce_health
characters_query -..-> |filter With| ce_character
```

### Knockback messages

Used in the following systems:
- **apply_beam_damage**: writes `KnockbackRequested` in the beam's direction.
- **move_characters** (Controller plugin): writes `KnockbackRequested` for walk-ins and collisions.
- **apply_knockback**: reads `KnockbackRequested` and writes `EntityKnockedBack` for each player that moved.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_beam_damage["`**apply_beam_damage**`"]
apply_knockback["`**apply_knockback**`"]
move_characters["`**move_characters** (Controller)`"]

update -.-> apply_beam_damage
update -.-> apply_knockback

knockback_requested_message(["`**KnockbackRequested**`"])
entity_knocked_back_message(["`**EntityKnockedBack**`"])

apply_beam_damage ---> |writes| knockback_requested_message
move_characters ---> |writes| knockback_requested_message
knockback_requested_message ---> |read by| apply_knockback
apply_knockback ---> |writes| entity_knocked_back_message
```

### Query knocked-back entities (apply_knockback)

Used in the following systems:
- **apply_knockback**: checks the player is alive and the target is ground, then moves the player and inserts `IsKnockedBack`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_knockback["`**apply_knockback**`"]

update -.-> apply_knockback

knockback_query{{"`query`"}}:::query
apply_knockback ---> knockback_query

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }
map_info_res --> |belongs to| world

target_entity@{ shape: st-rect, label: "Knocked-back Entity" }

ke_coords>"`**GridCoords**`"] --> |belongs to| target_entity
ke_is_dead>"`**IsDead**`"] --> |belongs to| target_entity
ke_health>"`**Health**`"] --> |belongs to| target_entity
ke_is_knocked>"`**IsKnockedBack**`"] --> |belongs to| target_entity

knockback_query ---> |writes| ke_coords
knockback_query ---> |reads Has| ke_is_dead
knockback_query ---> |"reads (optional)"| ke_health
apply_knockback ---> |reads `on_ground`| map_info_res
apply_knockback ---> |"inserts, timer seeded from knockback_tween_ms"| ke_is_knocked
```

### IsKnockedBack component lifecycle

- `IsKnockedBack(Timer)` is the input lock of a knocked-back player. The Input plugin reads it.
- **Inserted** by `apply_knockback`. A new knockback replaces a running lock.
- **Removed** by `tick_knockback_lock` when the timer ends, or by `reset_round` (Round plugin).

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
apply_knockback["`**apply_knockback**`"]
tick_knockback_lock["`**tick_knockback_lock**`"]

update -.-> apply_knockback
update -.-> tick_knockback_lock
reset_round["`**reset_round** (Round)`"]

is_knocked_back@{ shape: doc, label: "IsKnockedBack" }
reset_round ---> |always removes| is_knocked_back

apply_knockback ---> |"inserts, timer seeded from knockback_tween_ms"| is_knocked_back
tick_knockback_lock ---> |ticks timer every frame| is_knocked_back
tick_knockback_lock ---> |"removes once timer finishes"| is_knocked_back
```

### Write DamageableDied messages

Used in the following systems:
- **apply_owned_tile_entry_damage**, **apply_owned_tile_damage**, **apply_beam_damage**, **apply_collision_damage**: write `DamageableDied` when the health reaches zero (through `deal_damage`).

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
apply_collision_damage["`**apply_collision_damage**`"]

update -.-> apply_owned_tile_entry_damage
update -.-> apply_owned_tile_damage
update -.-> apply_beam_damage
update -.-> apply_collision_damage

damageable_died_message(["`**DamageableDied**`"])

apply_owned_tile_entry_damage ---> |writes| damageable_died_message
apply_owned_tile_damage ---> |writes| damageable_died_message
apply_beam_damage ---> |writes| damageable_died_message
apply_collision_damage ---> |writes| damageable_died_message
```

### Write DamageTimer duration (dev only)

Used in the following systems:
- **resync_damage_timer**: sets the timer duration when the config changes.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
resync_damage_timer["`**resync_damage_timer**`"]

update -.-> resync_damage_timer

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }
damage_timer_res@{ shape: doc, label: "DamageTimer" }

config_res --> |belongs to| world
damage_timer_res --> |belongs to| world

resync_damage_timer ---> |reads damage.tick_secs| config_res
resync_damage_timer ---> |writes duration| damage_timer_res
```
