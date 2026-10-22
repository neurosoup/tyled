---
id: doc-4
title: '[003] Controller plugin'
type: other
created_date: '2026-02-01 16:57'
updated_date: '2026-10-17 12:00'
---
# Controller Plugin

This plugin moves a character to the tile it asked for, if the move is legal. A move into an enemy beam or into another character asks for a knockback, and the Damage plugin applies it.

## Plugin Systems

### Move Characters

- Runs in `GameplaySet::Movement`, only during `Playing`. It handles each move request in this order:
- **Enemy beam on the target tile**: knockback for the mover, in the beam direction. Nothing else happens.
- **Target is not ground**: the move is ignored.
- **Target is free**: the character's `GridCoords` becomes the target. The old value is never read.
- **Another character is on the target**: a collision (see below).

### Collisions

- Each side's distance is how far its sprite still is from its destination tile.
- **Tie**: both sides are knocked back one tile, away from their destination.
- **No tie**: the side with the larger distance wins. It is knocked back one tile, away from its destination.
- The loser is pushed in the winner's move direction, and a `CharacterCollision` is written for it.
- Both characters are then done for this frame.

### Knockback order

- The Damage plugin moves knocked-back characters, later in the frame.
- If one entity gets several knockbacks in a frame, only the last one applies.
- Beam damage writes its knockback after this plugin, so a beam hit wins over a walk-in or a bump.

## Components, Resources and Messages CRUD

### Read EntityMoved messages

Used in the following systems:
- **move_characters**: one move request for each message.

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
- **move_characters**: checks that the target is ground.

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
- **move_characters**: finds enemy beams, then writes the knockback and collision messages.

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
- **move_characters**: sets the new position of the character.

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
