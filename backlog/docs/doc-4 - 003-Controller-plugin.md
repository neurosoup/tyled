---
id: doc-4
title: '[003] Controller plugin'
type: other
created_date: '2026-02-01 16:57'
updated_date: '2026-10-16 12:00'
---
# Controller Plugin

Contains the system responsible for reacting to movement messages and updating player grid positions. When an `EntityMoved` message is received, the controller validates the target position against walkable ground tiles and overwrites the player's `GridCoords` if the move is legal. A move into an enemy beam, or into another character, does not move the character. It writes `KnockbackRequested` messages instead. The damage plugin applies them.

## Plugin workflow

- Update phase (`GameplaySet::Movement`, `in_state(RoundPhase::Playing)`)
    - Move Characters:
        - Reacts to `EntityMoved` message
            - Reads:
                - `MapInfo` resource (for ground validation via `on_ground()`)
                - `EntityMoved` message fields (`entity`, target `position`)
                - `Beam` and `GridCoords` on beam entities, and a snapshot of every `Character` position and translation
            - Writes:
                - Updates `GridCoords` on the `Character` entity if the target tile is walkable ground and free
                - Writes `KnockbackRequested { entity, direction }` when the target tile holds an enemy beam (direction = the beam direction), and for both sides of a character collision
                - Writes `CharacterCollision { loser }` when a collision has a loser

## Plugin Systems

### Move Characters

Reads `EntityMoved` messages written by the input system. For each message, in order:
- If a beam of another owner is on the target tile, writes `KnockbackRequested` for the mover with the beam direction and stops.
- If the target is not ground (`MapInfo::on_ground()`), ignores the move.
- If no other character is on the target tile, overwrites the `Character` entity's `GridCoords` with the new position.
- Otherwise it resolves a collision (`resolve_collision`) between the mover and the occupant. Each side's distance is how far its `Transform` still is from its destination. On a tie, both sides get a `KnockbackRequested` one tile back, away from their destination. Otherwise the side with the larger distance wins. The winner gets a `KnockbackRequested` one tile back, away from its destination. The loser gets one in the winner's move direction, and a `CharacterCollision { loser }` message is written. Both characters are then marked resolved for this frame.

The controller never moves an entity because of a knockback. `apply_knockback` in the damage plugin does that in `GameplaySet::Displacement`. When one entity gets several `KnockbackRequested` messages in a frame, the damage plugin applies only the last one. The controller writes in `Movement`, which runs before the damage plugin's `apply_beam_damage` in `Damage`, so a beam hit on the same frame beats a walk-in or a collision bump.

## Components, Resources and Messages CRUD

### Read EntityMoved messages

Used in the following systems:
- **move_characters**: used to trigger a character grid position update

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
move_characters["`**move_characters**`"]

update -.-> move_characters

message_reader{{"MessageReader#60;EntityMoved#62;"}}:::reader
move_characters ---> message_reader

entity_moved_message(["`**EntityMoved**`"])

message_reader ---> |reads| entity_moved_message
```

### Read MapInfo resource

Used in the following systems:
- **move_characters**: used to validate that the target position is walkable ground via `on_ground()`


```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
move_characters["`**move_characters**`"]

update -.-> move_characters

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

move_characters ---> |reads `on_ground`| map_info_res
```

### Write KnockbackRequested and CharacterCollision messages

Used in the following systems:
- **move_characters**: writes `KnockbackRequested` when a mover steps into an enemy beam (direction = beam direction) and for both sides of a character collision. Writes `CharacterCollision` for the loser of an uneven collision. Reads `Beam` and `GridCoords` on beam entities to find the enemy beam.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
move_characters["`**move_characters**`"]

update -.-> move_characters

beams_query{{"`beams_query`"}}:::query
move_characters ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }
be_beam>"`**Beam**`"] --> |belongs to| beam_entity
be_coords>"`**GridCoords**`"] --> |belongs to| beam_entity
beams_query ---> |reads| be_beam
beams_query ---> |reads| be_coords

knockback_requested_message(["`**KnockbackRequested**`"])
character_collision_message(["`**CharacterCollision**`"])

move_characters ---> |writes| knockback_requested_message
move_characters ---> |writes| character_collision_message
```

### Write GridCoords

Used in the following systems:
- **move_characters**: overwrites the player's `GridCoords` with the validated target position (the new value comes entirely from `EntityMoved::position`, the existing component value is never read)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
move_characters["`**move_characters**`"]

update -.-> move_characters

players_query{{"`players_query`"}}:::query
move_characters ---> players_query

character_entity@{ shape: st-rect, label: "Character" }

pe_grid_coords>"`**GridCoords**`"] --> |belongs to| character_entity
pe_character_marker>"`**Character**`"] --> |belongs to| character_entity

players_query ---> |writes| pe_grid_coords
players_query -..-> |filter With| pe_character_marker
```
