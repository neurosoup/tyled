---
id: doc-7
title: '[006] Beam plugin'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-03-08 17:04'
---
# Beam Plugin

Contains systems responsible for spawning and stepping beam projectiles fired by players. When a player shoots, a `Beam` entity is created at the player's current grid position and advances one tile per frame in the firing direction until it either leaves the map bounds or hits an already-claimed tile, at which point the current tile is claimed and the beam is despawned.

## Plugin workflow

- Update phase
    - Spawn Beam:
        - Reacts to `BeamFired` message
            - Reads:
                - `BeamFired` message fields (`owner`, `origin`, `direction`)
                - `Player` component of the firing player (for sprite atlas index)
                - `MapInfo` resource (for world-space translation of origin)
                - `AssetServer` and `Assets<TextureAtlasLayout>` (for sprite setup)
            - Writes:
                - Spawns a new `Beam` entity with `GridCoords`, `Transform`, `Beam`, `Sprite` and `TweenAnim`
    - Beam Step:
        - Runs every frame on all existing `Beam` entities
            - Reads:
                - `Beam` component (`owner`, `direction`)
                - `MapInfo` resource (for bounds check and tile entity lookup)
                - `ClaimedTile` component on ground tile entities (for claimed-tile check)
            - Writes:
                - Advances `GridCoords` of the beam if the next tile is valid and unclaimed
                - Writes a `TileClaimed` message and despawns the beam when it must stop

## Plugin Systems

### Spawn Beam

Reacts to `BeamFired` messages emitted by the input system. For each message, it looks up the firing `Player` to determine the correct sprite atlas index, converts the origin `GridCoords` to a world-space `Transform` via `MapInfo`, and spawns a `Beam` entity carrying `GridCoords`, `Transform`, `Beam` (owner + direction + speed), `Sprite` (atlas tile from `grid_tiles2-Sheet.png`), and a `TweenAnim` initialised at rest (same start and end position) with `destroy_on_completed` disabled.

### Beam Step

Runs every frame. For each `Beam` entity it computes the next grid position (`current + direction`) and applies two stopping rules in order:

1. **Out of bounds** — if the next position is not on ground, emit `TileClaimed` for the *current* position and despawn.
2. **Already claimed** — if the next tile's ground entity already carries a `ClaimedTile` component, emit `TileClaimed` for the *current* position and despawn.

If neither rule fires, the beam advances: `GridCoords` is overwritten with the next position (which triggers `translate_objects` in the Movements plugin to tween the sprite).

## Components, Resources and Messages CRUD

### Read BeamFired messages

Used in the following systems:
- **spawn_beam**: used to trigger beam entity creation

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

message_reader{{"MessageReader#60;BeamFired#62;"}}:::reader
spawn_beam ---> message_reader

beam_fired_message(["`**BeamFired**`"])

message_reader ---> |reads| beam_fired_message
```

### Query Player (spawn)

Used in the following systems:
- **spawn_beam**: reads the `Player` component of the firing entity to select the correct sprite atlas index

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

players_query{{"`players_query`"}}:::query
spawn_beam ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_player>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
```

### Read MapInfo resource (spawn)

Used in the following systems:
- **spawn_beam**: converts the beam origin `GridCoords` to a world-space `Vec3` via `to_translation()`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

spawn_beam ---> |reads `to_translation`| map_info_res
```

### Write commands — spawn Beam entity

Used in the following systems:
- **spawn_beam**: spawns a new `Beam` entity with all required components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

beam_entity@{ shape: st-rect, label: "Beam (spawned)" }

be_grid_coords>"`**GridCoords**`"]
be_transform>"`**Transform**`"]
be_beam>"`**Beam**`"]
be_sprite>"`**Sprite**`"]
be_tween_anim>"`**TweenAnim**`"]

be_grid_coords --> |spawned on| beam_entity
be_transform --> |spawned on| beam_entity
be_beam --> |spawned on| beam_entity
be_sprite --> |spawned on| beam_entity
be_tween_anim --> |spawned on| beam_entity

spawn_beam ---> |spawns entity with| be_grid_coords
spawn_beam ---> |spawns entity with| be_transform
spawn_beam ---> |spawns entity with| be_beam
spawn_beam ---> |spawns entity with| be_sprite
spawn_beam ---> |spawns entity with| be_tween_anim
```

### Query Beam entities

Used in the following systems:
- **beam_step**: reads `Beam` (owner + direction) and writes `GridCoords` on all active beam entities each frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

beams_query{{"`beams_query`"}}:::query
beam_step ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }

be_entity>"`**Entity**`"] --> |belongs to| beam_entity
be_beam>"`**Beam**`"] --> |belongs to| beam_entity
be_grid_coords>"`**GridCoords**`"] --> |belongs to| beam_entity

beams_query ---> |reads| be_entity
beams_query ---> |reads| be_beam
beams_query ---> |writes| be_grid_coords
```

### Query ClaimedTile (beam step)

Used in the following systems:
- **beam_step**: checks whether the next ground tile entity already carries a `ClaimedTile` component to decide if the beam must stop

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

claimed_query{{"`claimed_query`"}}:::query
beam_step ---> claimed_query

ground_entity@{ shape: st-rect, label: "Ground Tile" }

gt_claimed>"`**ClaimedTile**`"] --> |belongs to| ground_entity

claimed_query -..-> |filter With| gt_claimed
```

### Read MapInfo resource (beam step)

Used in the following systems:
- **beam_step**: used to check `on_ground()` for the next position and to resolve the ground tile entity via `ground_entities` HashMap

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

beam_step ---> |reads `on_ground`| map_info_res
beam_step ---> |reads `ground_entities`| map_info_res
```

### Write TileClaimed messages

Used in the following systems:
- **beam_step**: emits a `TileClaimed` message with the beam's current position and owner when the beam stops (out of bounds or claimed tile hit)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

tile_claimed_message(["`**TileClaimed**`"])

beam_step ---> |writes| tile_claimed_message
```

### Write commands — despawn Beam entity

Used in the following systems:
- **beam_step**: despawns the beam entity after emitting `TileClaimed` when a stopping condition is met

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

beam_entity@{ shape: st-rect, label: "Beam" }

beam_step ---> |despawns| beam_entity
```
