---
id: doc-8
title: '[007] Claim plugin'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-03-08 17:04'
---
# Claim Plugin

Contains the system responsible for processing tile claim events. When a `BeamResolved` message is received, this plugin marks the corresponding ground tile entity with a `ClaimedTile` component and spawns a colored sprite overlay on that tile to visually indicate which player owns it.

## Plugin workflow

- Update phase
    - On Tile Claimed:
        - Reacts to `BeamResolved` message
            - Reads:
                - `MapInfo` resource (to resolve `GridCoords` → `TilePos` → tile `Entity` and compute world-space tile transform)
                - `Player` component on the owning player entity (to determine the sprite atlas index)
            - Writes:
                - Inserts `ClaimedTile` component on the ground tile entity
                - Spawns a colored sprite overlay entity at the tile's world position

## Plugin Systems

### On Tile Claimed

Reacts to `BeamResolved` messages. For each message it:
1. Resolves the `GridCoords` position to a `TilePos` and looks up the corresponding ground tile entity via `MapInfo::ground_entities`.
2. Looks up the owning `Player` to determine the correct sprite atlas index (index 6 for player 0, index 7 for player 1).
3. Inserts a `ClaimedTile { owner }` component on the ground tile entity.
4. Spawns a new overlay sprite entity at the tile's world-space center (z = −0.1) using the `grid_tiles2-Sheet.png` atlas.

## Components, Resources and Messages CRUD

### Read BeamResolved messages

Used in the following systems:
- **on_tile_claimed**: used to trigger tile ownership processing

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_tile_claimed["`**on_tile_claimed**`"]

update -.-> on_tile_claimed

message_reader{{"MessageReader#60;BeamResolved#62;"}}:::reader
on_tile_claimed ---> message_reader

tile_claimed_message(["`**BeamResolved**`"])

message_reader ---> |reads| tile_claimed_message
```

### Read MapInfo resource

Used in the following systems:
- **on_tile_claimed**: used to resolve `GridCoords` → `TilePos`, look up the ground tile entity via `ground_entities`, and compute the tile world-space transform via `center_in_world()`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
on_tile_claimed["`**on_tile_claimed**`"]

update -.-> on_tile_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

on_tile_claimed ---> |reads `to_tile_pos` + `ground_entities` + `center_in_world`| map_info_res
```

### Query Player

Used in the following systems:
- **on_tile_claimed**: reads the `Player` component on the owning entity to determine the correct sprite atlas index

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_tile_claimed["`**on_tile_claimed**`"]

update -.-> on_tile_claimed

players_query{{"`players_query`"}}:::query
on_tile_claimed ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_player>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
```

### Write ClaimedTile component

Used in the following systems:
- **on_tile_claimed**: inserts `ClaimedTile { owner }` on the ground tile entity to mark it as owned

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
on_tile_claimed["`**on_tile_claimed**`"]

update -.-> on_tile_claimed

ground_tile_entity@{ shape: st-rect, label: "Ground Tile" }

claimed_tile>"`**ClaimedTile**`"]
ct_owner>"`**owner**`"]

ct_owner --> |field of| claimed_tile
claimed_tile --> |inserted on| ground_tile_entity

on_tile_claimed ---> |inserts component| claimed_tile
```

### Write commands (spawn overlay sprite)

Used in the following systems:
- **on_tile_claimed**: spawns a colored sprite overlay entity at the claimed tile's world-space center

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
on_tile_claimed["`**on_tile_claimed**`"]

update -.-> on_tile_claimed

overlay_entity@{ shape: st-rect, label: "Claimed Tile Overlay (spawned)" }

ov_name>"`**Name**`"]
ov_transform>"`**Transform**`"]
ov_sprite>"`**Sprite (TextureAtlas)**`"]

ov_name --> |spawned on| overlay_entity
ov_transform --> |spawned on| overlay_entity
ov_sprite --> |spawned on| overlay_entity

on_tile_claimed ---> |spawns entity with| ov_name
on_tile_claimed ---> |spawns entity with| ov_transform
on_tile_claimed ---> |spawns entity with| ov_sprite
```
