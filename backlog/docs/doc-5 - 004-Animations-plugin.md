---
id: doc-5
title: '[004] Animations plugin'
type: other
created_date: '2026-02-01 18:59'
updated_date: '2026-06-15 12:00'
---
# Animations Plugin

Contains systems responsible for attaching and updating spritesheet animations on player entities and claimed tile entities. This plugin reacts to the `ObjectCreated` Tiled event to initialize per-player animation handle resources, drives the active player animation each frame based on the player's current `LookDirection`, and manages animations for claimed tiles in reaction to `BeamResolved` messages.

## Plugin workflow

- Update phase
    - Attach Player Animations:
        - Reacts to `TiledEvent<ObjectCreated>` message
            - Reads:
                - All `Player`-marked `TiledObject` entities and their `Entity` + `Player` components
                - The `Sprite` component on each player's child sprite entity (to get the image handle)
            - Writes:
                - Inserts `PlayerOneAnimations` or `PlayerTwoAnimations` resource into the world
                - Inserts `SpritesheetAnimation` on the child sprite entity
    - Update Players Animation:
        - Runs every frame
            - Reads:
                - All `Player`-marked entities with `LookDirection`
                - `PlayerOneAnimations` and `PlayerTwoAnimations` resources (optional/`If`)
                - `Sprite` and `SpritesheetAnimation` on descendant sprite entities
            - Writes:
                - Updates `SpritesheetAnimation` (switches active clip)
                - Updates `Sprite::flip_x` for left/right facing directions
    - Attach Claimed Tile Animations:
        - Reacts to `Added<ClaimedTile>` on newly spawned claimed tile entities
            - Reads:
                - `ClaimedTile` entity and its components
            - Writes:
                - Builds `ClaimedTileAnimations` resource
                - Inserts `SpritesheetAnimation`, `Sprite`, and `BounceEffect` on the claimed tile entity
    - Update Claimed Tile Animation:
        - Reacts to `BeamResolved` messages
            - Reads:
                - `BeamResolved` message fields (`position`, `owner`)
                - `MapInfo` resource (to resolve `GridCoords` → claimed tile `Entity` via `claimed_entities`)
                - `ClaimedTileAnimations` resource
            - Writes:
                - Switches `SpritesheetAnimation` clip on the claimed tile entity to the player-color animation
                - Inserts `BounceEffectTarget` on the claimed tile entity

## Plugin Systems

### Attach Player Animations

Reacts to the `TiledEvent<ObjectCreated>` message emitted by the Tiled loader when a player object is created. For each matching `Player`-marked `TiledObject` entity, it walks the entity hierarchy to find the child entity carrying a `Sprite`, reads its image handle to build a `Spritesheet`, then creates idle animation handles for the three directional variants (`idle_x`, `idle_down`, `idle_up`). It stores these handles in a per-player resource (`PlayerOneAnimations` or `PlayerTwoAnimations`) and inserts a `SpritesheetAnimation` on the child sprite entity with the initial idle clip.

### Update Players Animation

Runs every frame. For each `Player`-marked `TiledObject` entity, it walks the entity hierarchy to find the descendant with a `Sprite` and `SpritesheetAnimation`, then switches the active animation clip based on the player's current `LookDirection`. It also sets `Sprite::flip_x` to mirror the horizontal idle sprite when the player faces left.

### Attach Claimed Tile Animations

Reacts to `Added<ClaimedTile>` — fires once for each newly spawned claimed tile entity. Builds the `ClaimedTileAnimations` resource (if not already present) containing animation clip handles for each player color variant. Inserts `SpritesheetAnimation` (with the neutral/default clip), `Sprite`, and `BounceEffect` on the claimed tile entity so it is ready to display ownership animations.

### Update Claimed Tile Animation

Reads `BeamResolved` messages. For each message, resolves the claimed tile entity from `MapInfo::claimed_entities` using the message's `GridCoords` position. Reads the `ClaimedTileAnimations` resource to select the correct player-color clip based on the owning player id, then switches the `SpritesheetAnimation` clip on the claimed tile entity. Also inserts `BounceEffectTarget` on the claimed tile entity to trigger the bounce visual effect.

## Components, Resources and Messages CRUD

### Read TiledEvent ObjectCreated messages

Used in the following systems:
- **attach_player_animations**: used to trigger animation setup when a player Tiled object is created

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]

update -.-> attach_player_animations

message_reader{{"MessageReader#60;TiledEvent#60;ObjectCreated#62;#62;"}}:::reader
attach_player_animations ---> message_reader

object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])

message_reader ---> |reads| object_created_message
```

### Read BeamResolved messages

Used in the following systems:
- **update_claimed_tile_animation**: used to trigger claimed tile animation switching when a beam resolves

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_claimed_tile_animation["`**update_claimed_tile_animation**`"]

update -.-> update_claimed_tile_animation

message_reader{{"MessageReader#60;BeamResolved#62;"}}:::reader
update_claimed_tile_animation ---> message_reader

beam_resolved_message(["`**BeamResolved**`"])

message_reader ---> |reads| beam_resolved_message
```

### Query Player entities (attach)

Used in the following systems:
- **attach_player_animations**: used to get the `Entity` and `Player` of each `TiledObject`-marked player entity to look up the correct sprite child and build per-player animation handles

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]

update -.-> attach_player_animations

players_query{{"`players_query`"}}:::query
attach_player_animations ---> players_query

player_entity@{ shape: st-rect, label: "Player (TiledObject)" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_tiled_object>"`**TiledObject**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query -..-> |filter With| pe_tiled_object
```

### Query Player entities (update)

Used in the following systems:
- **update_players_animation**: used to get `Entity`, `Player::player_id` and `LookDirection` of each `TiledObject`-marked player entity to decide which animation clip to activate

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_players_animation["`**update_players_animation**`"]

update -.-> update_players_animation

players_query{{"`players_query`"}}:::query
update_players_animation ---> players_query

player_entity@{ shape: st-rect, label: "Player (TiledObject)" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_look_direction>"`**LookDirection**`"] --> |belongs to| player_entity
pe_tiled_object>"`**TiledObject**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query ---> |reads| pe_look_direction
players_query -..-> |filter With| pe_tiled_object
```

### Query ClaimedTile entities (attach)

Used in the following systems:
- **attach_claimed_tile_animations**: detects newly added `ClaimedTile` entities and initializes their animation components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_claimed_tile_animations["`**attach_claimed_tile_animations**`"]

update -.-> attach_claimed_tile_animations

claimed_tiles_query{{"`claimed_tiles_query`"}}:::query
attach_claimed_tile_animations ---> claimed_tiles_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_entity>"`**Entity**`"] --> |belongs to| claimed_tile_entity
ct_claimed>"`**ClaimedTile**`"] --> |belongs to| claimed_tile_entity

claimed_tiles_query ---> |reads| ct_entity
claimed_tiles_query -..-> |filter Added| ct_claimed
```

### Query Children hierarchy

Used in the following systems:
- **attach_player_animations**: used to walk descendants via `iter_descendants` to find the child sprite entity
- **update_players_animation**: used to walk descendants via `iter_descendants` to find the child sprite entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]
update_players_animation["`**update_players_animation**`"]

update -.-> attach_player_animations
update -.-> update_players_animation

children_query{{"`children_query`"}}:::query
attach_player_animations ---> children_query
update_players_animation ---> children_query

child_entity@{ shape: st-rect, label: "Any Child Entity" }

ch_children>"`**Children**`"] --> |belongs to| child_entity

children_query ---> |reads| ch_children
```

### Query child Sprite (attach)

Used in the following systems:
- **attach_player_animations**: used to find the descendant entity carrying a `Sprite` and retrieve its `image` handle to build the `Spritesheet`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]

update -.-> attach_player_animations

sprites_query{{"`sprites_query (read-only)`"}}:::query
attach_player_animations ---> sprites_query

child_entity@{ shape: st-rect, label: "Player Child (Sprite)" }

ce_sprite>"`**Sprite**`"] --> |belongs to| child_entity

sprites_query ---> |reads| ce_sprite
```

### Query child Sprite and SpritesheetAnimation (update)

Used in the following systems:
- **update_players_animation**: used to mutably access `Sprite::flip_x` and switch the active `SpritesheetAnimation` clip on the child sprite entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_players_animation["`**update_players_animation**`"]

update -.-> update_players_animation

sprites_query{{"`sprites_query (mutable)`"}}:::query
update_players_animation ---> sprites_query

child_entity@{ shape: st-rect, label: "Player Child (Sprite)" }

ce_sprite>"`**Sprite**`"] --> |belongs to| child_entity
ce_anim>"`**SpritesheetAnimation**`"] --> |belongs to| child_entity

sprites_query ---> |reads/writes| ce_sprite
sprites_query ---> |reads/writes| ce_anim
```

### Read MapInfo resource

Used in the following systems:
- **update_claimed_tile_animation**: used to resolve the `GridCoords` in the `BeamResolved` message to a claimed tile entity via `MapInfo::claimed_entities`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
update_claimed_tile_animation["`**update_claimed_tile_animation**`"]

update -.-> update_claimed_tile_animation

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

update_claimed_tile_animation ---> |reads `claimed_entities`| map_info_res
```

### Read PlayerOneAnimations and PlayerTwoAnimations resources

Used in the following systems:
- **update_players_animation**: used to retrieve the animation clip handles for each player; accessed via `If<Res<...>>` (optional — skipped if not yet inserted)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
update_players_animation["`**update_players_animation**`"]

update -.-> update_players_animation

world@{ shape: st-rect, label: "World" }

p1_res@{ shape: doc, label: "PlayerOneAnimations" }
p2_res@{ shape: doc, label: "PlayerTwoAnimations" }

p1_idle_x>"`**idle_x**`"]
p1_idle_up>"`**idle_up**`"]
p1_idle_down>"`**idle_down**`"]

p2_idle_x>"`**idle_x**`"]
p2_idle_up>"`**idle_up**`"]
p2_idle_down>"`**idle_down**`"]

p1_res --> |belongs to| world
p2_res --> |belongs to| world
p1_idle_x --> |field of| p1_res
p1_idle_up --> |field of| p1_res
p1_idle_down --> |field of| p1_res
p2_idle_x --> |field of| p2_res
p2_idle_up --> |field of| p2_res
p2_idle_down --> |field of| p2_res

update_players_animation ---> |reads| p1_res
update_players_animation ---> |reads| p2_res
```

### Read ClaimedTileAnimations resource

Used in the following systems:
- **update_claimed_tile_animation**: used to retrieve the correct player-color animation clip handle when switching a claimed tile's animation

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
update_claimed_tile_animation["`**update_claimed_tile_animation**`"]

update -.-> update_claimed_tile_animation

world@{ shape: st-rect, label: "World" }
ct_anims_res@{ shape: doc, label: "ClaimedTileAnimations" }

ct_anims_player1>"`**player1_clip**`"]
ct_anims_player2>"`**player2_clip**`"]

ct_anims_res --> |belongs to| world
ct_anims_player1 --> |field of| ct_anims_res
ct_anims_player2 --> |field of| ct_anims_res

update_claimed_tile_animation ---> |reads| ct_anims_res
```

### Write PlayerOneAnimations and PlayerTwoAnimations resources

Used in the following systems:
- **attach_player_animations**: inserts `PlayerOneAnimations` or `PlayerTwoAnimations` resource into the world after building animation handles from the player's spritesheet

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]

update -.-> attach_player_animations

world@{ shape: st-rect, label: "World" }

p1_res@{ shape: doc, label: "PlayerOneAnimations" }
p2_res@{ shape: doc, label: "PlayerTwoAnimations" }

p1_idle_x>"`**idle_x**`"]
p1_idle_up>"`**idle_up**`"]
p1_idle_down>"`**idle_down**`"]

p2_idle_x>"`**idle_x**`"]
p2_idle_up>"`**idle_up**`"]
p2_idle_down>"`**idle_down**`"]

p1_res --> |belongs to| world
p2_res --> |belongs to| world
p1_idle_x --> |field of| p1_res
p1_idle_up --> |field of| p1_res
p1_idle_down --> |field of| p1_res
p2_idle_x --> |field of| p2_res
p2_idle_up --> |field of| p2_res
p2_idle_down --> |field of| p2_res

attach_player_animations ---> |inserts resource| p1_res
attach_player_animations ---> |inserts resource| p2_res
```

### Write ClaimedTileAnimations resource

Used in the following systems:
- **attach_claimed_tile_animations**: builds and inserts the `ClaimedTileAnimations` resource into the world with clip handles for each player color

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_claimed_tile_animations["`**attach_claimed_tile_animations**`"]

update -.-> attach_claimed_tile_animations

world@{ shape: st-rect, label: "World" }
ct_anims_res@{ shape: doc, label: "ClaimedTileAnimations" }

ct_anims_player1>"`**player1_clip**`"]
ct_anims_player2>"`**player2_clip**`"]

ct_anims_res --> |belongs to| world
ct_anims_player1 --> |field of| ct_anims_res
ct_anims_player2 --> |field of| ct_anims_res

attach_claimed_tile_animations ---> |inserts resource| ct_anims_res
```

### Write commands (attach player animations)

Used in the following systems:
- **attach_player_animations**: inserts `SpritesheetAnimation` on the child sprite entity after building the animation handles

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_player_animations["`**attach_player_animations**`"]

update -.-> attach_player_animations

child_entity@{ shape: st-rect, label: "Player Child (Sprite)" }

ce_anim>"`**SpritesheetAnimation**`"]

ce_anim --> |inserted on| child_entity

attach_player_animations ---> |inserts component| ce_anim
```

### Write commands (attach claimed tile animations)

Used in the following systems:
- **attach_claimed_tile_animations**: inserts `SpritesheetAnimation`, `Sprite`, and `BounceEffect` on each newly added `ClaimedTile` entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_claimed_tile_animations["`**attach_claimed_tile_animations**`"]

update -.-> attach_claimed_tile_animations

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_anim>"`**SpritesheetAnimation**`"]
ct_sprite>"`**Sprite**`"]
ct_bounce>"`**BounceEffect**`"]

ct_anim --> |inserted on| claimed_tile_entity
ct_sprite --> |inserted on| claimed_tile_entity
ct_bounce --> |inserted on| claimed_tile_entity

attach_claimed_tile_animations ---> |inserts component| ct_anim
attach_claimed_tile_animations ---> |inserts component| ct_sprite
attach_claimed_tile_animations ---> |inserts component| ct_bounce
```

### Write commands (update claimed tile animation)

Used in the following systems:
- **update_claimed_tile_animation**: switches the `SpritesheetAnimation` clip and inserts `BounceEffectTarget` on the resolved claimed tile entity when a `BeamResolved` message is received

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_claimed_tile_animation["`**update_claimed_tile_animation**`"]

update -.-> update_claimed_tile_animation

claimed_tiles_query{{"`claimed_tiles_query (mutable)`"}}:::query
update_claimed_tile_animation ---> claimed_tiles_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_anim>"`**SpritesheetAnimation**`"]
ct_bounce_target>"`**BounceEffectTarget**`"]

ct_anim --> |belongs to| claimed_tile_entity
ct_bounce_target --> |inserted on| claimed_tile_entity

claimed_tiles_query ---> |writes| ct_anim
update_claimed_tile_animation ---> |inserts component| ct_bounce_target
```
