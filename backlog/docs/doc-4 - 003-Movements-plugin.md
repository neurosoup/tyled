---
id: doc-4
title: '[003] Movements plugin'
type: other
created_date: '2026-02-01 16:57'
updated_date: '2026-03-08 17:32'
---
# Movements Plugin

Contains systems responsible for translating game-logic grid positions into actual world-space transforms, and for reacting to movement messages to update player positions. Movement is smoothly interpolated using tweening.

## Plugin workflow

- Update phase
    - On Player Moved:
        - Reacts to `PlayerMoved` message
            - Reads:
                - `MapInfo` resource (for ground validation via `on_ground()`)
                - `PlayerMoved` message fields (`player` entity, target `position`)
            - Writes:
                - Updates `GridCoords` on the player entity if the target tile is walkable ground
    - Translate Objects:
        - Reacts to changed `GridCoords`
            - Reads:
                - Current `Transform` and `GridCoords` of moving entities
                - `MapInfo` resource (for world-space coordinate conversion)
            - Writes:
                - Updates `TweenAnim` with a new `TransformPositionLens` tween toward the destination

## Plugin Systems

### On Player Moved

Reads `PlayerMoved` messages written by the input system. For each message, checks whether the target `GridCoords` position is a valid ground tile via `MapInfo::on_ground()`. If valid, overwrites the player entity's `GridCoords` with the new position.

### Translate Objects

Reacts to any entity whose `GridCoords` component has changed. Computes the world-space destination using `GridCoords::to_translation()` with the `MapInfo` resource, then sets a new `TransformPositionLens` tween on the entity's `TweenAnim` component to smoothly interpolate the transform from its current position to the destination.

## Components, Resources and Messages CRUD

### Read PlayerMoved messages

Used in the following systems:
- **on_player_moved**: used to trigger a player grid position update

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_player_moved["`**on_player_moved**`"]

update -.-> on_player_moved

message_reader{{"MessageReader#60;PlayerMoved#62;"}}:::reader
on_player_moved ---> message_reader

player_moved_message(["`**PlayerMoved**`"])

message_reader ---> |reads| player_moved_message
```

### Read MapInfo resource

Used in the following systems:
- **on_player_moved**: used to validate that the target position is walkable ground via `on_ground()`
- **translate_objects**: used to convert `GridCoords` to world-space translation via `to_translation()`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
on_player_moved["`**on_player_moved**`"]
translate_objects["`**translate_objects**`"]

update -.-> on_player_moved
update -.-> translate_objects

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

on_player_moved ---> |reads `on_ground`| map_info_res
translate_objects ---> |reads `to_translation`| map_info_res
```

### Query moving objects

Used in the following systems:
- **translate_objects**: reads `Transform` and `GridCoords` and writes `TweenAnim` on any entity whose `GridCoords` changed

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
translate_objects["`**translate_objects**`"]

update -.-> translate_objects

moving_objects_query{{"`moving_objects_query`"}}:::query
translate_objects ---> moving_objects_query

moving_entity@{ shape: st-rect, label: "Moving Entity" }

me_transform>"`**Transform**`"] --> |belongs to| moving_entity
me_grid_coords>"`**GridCoords**`"] --> |belongs to| moving_entity
me_tween_anim>"`**TweenAnim**`"] --> |belongs to| moving_entity

moving_objects_query ---> |reads| me_transform
moving_objects_query -..-> |filter Changed| me_grid_coords
moving_objects_query ---> |writes| me_tween_anim
```

### Write GridCoords

Used in the following systems:
- **on_player_moved**: overwrites the player's `GridCoords` with the validated target position (the new value comes entirely from `PlayerMoved::position`, the existing component value is never read)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_player_moved["`**on_player_moved**`"]

update -.-> on_player_moved

players_query{{"`players_query`"}}:::query
on_player_moved ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_player_marker>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |writes| pe_grid_coords
players_query -..-> |filter With| pe_player_marker
```

### Write TweenAnim

Used in the following systems:
- **translate_objects**: sets a new `TransformPositionLens` tween on the entity to smoothly move it toward the grid-aligned world position

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
translate_objects["`**translate_objects**`"]

update -.-> translate_objects

moving_entity@{ shape: st-rect, label: "Moving Entity" }

me_tween_anim>"`**TweenAnim**`"]
me_tween_lens>"`**TransformPositionLens**`"]

me_tween_lens --> |set inside| me_tween_anim
me_tween_anim --> |written on| moving_entity

translate_objects ---> |writes| me_tween_anim
```
