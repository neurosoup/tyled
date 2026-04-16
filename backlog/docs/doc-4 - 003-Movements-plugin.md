---
id: doc-4
title: '[003] Controller plugin'
type: other
created_date: '2026-02-01 16:57'
updated_date: '2026-06-15 12:00'
---
# Controller Plugin

Contains the system responsible for reacting to movement messages and updating player grid positions. When an `EntityMoved` message is received, the controller validates the target position against walkable ground tiles and overwrites the player's `GridCoords` if the move is legal.

## Plugin workflow

- Update phase
    - Move Players:
        - Reacts to `EntityMoved` message
            - Reads:
                - `MapInfo` resource (for ground validation via `on_ground()`)
                - `EntityMoved` message fields (`entity`, target `position`)
            - Writes:
                - Updates `GridCoords` on the player entity if the target tile is walkable ground

## Plugin Systems

### Move Players

Reads `EntityMoved` messages written by the input system. For each message, checks whether the target `GridCoords` position is a valid ground tile via `MapInfo::on_ground()`. If valid, overwrites the player entity's `GridCoords` with the new position.

## Components, Resources and Messages CRUD

### Read EntityMoved messages

Used in the following systems:
- **move_players**: used to trigger a player grid position update

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
move_players["`**move_players**`"]

update -.-> move_players

message_reader{{"MessageReader#60;EntityMoved#62;"}}:::reader
move_players ---> message_reader

entity_moved_message(["`**EntityMoved**`"])

message_reader ---> |reads| entity_moved_message
```

### Read MapInfo resource

Used in the following systems:
- **move_players**: used to validate that the target position is walkable ground via `on_ground()`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
move_players["`**move_players**`"]

update -.-> move_players

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

move_players ---> |reads `on_ground`| map_info_res
```

### Write GridCoords

Used in the following systems:
- **move_players**: overwrites the player's `GridCoords` with the validated target position (the new value comes entirely from `EntityMoved::position`, the existing component value is never read)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
move_players["`**move_players**`"]

update -.-> move_players

players_query{{"`players_query`"}}:::query
move_players ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_player_marker>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |writes| pe_grid_coords
players_query -..-> |filter With| pe_player_marker
```
