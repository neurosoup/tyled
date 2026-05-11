---
id: doc-5
title: '[004] Animations plugin'
type: other
created_date: '2026-02-01 18:59'
updated_date: '2026-06-15 12:00'
---
# Animations Plugin

Contains systems responsible for attaching and updating spritesheet animations on player entities, claimed tile entities, and HUD digit entities. This plugin reacts to the `ObjectCreated` Tiled event to initialize per-player and per-digit animation handle resources, drives the active player animation each frame based on the player's current `LookDirection`, manages animations for claimed tiles in reaction to `BeamResolved` messages, and drives flip-counter animations on `Digit` entities in reaction to `BeamChargesChanged` messages.

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
    - Attach Digit Animations:
        - Reacts to `TiledEvent<ObjectCreated>` message
            - Reads:
                - All `Digit`-marked `TiledObject` entities and their `Entity` components
                - The `Sprite` component on each digit's child sprite entity (to get the image handle)
            - Writes:
                - Builds all 90 from→to transition animation handles (for all `from != to` in `0..10`) using a single `make_anim` closure
                - Inserts `DigitAnimations` resource into the world
                - Inserts `SpritesheetAnimation` on the child sprite entity
    - Update Digit Animation:
        - Reacts to `BeamChargesChanged` messages
            - Reads:
                - `BeamChargesChanged` message fields (`player_id`, `current`)
                - All `Digit`-marked entities with `Player` and `Digit` components
                - `DigitAnimations` resource (optional/`If`)
            - Writes:
                - Computes per-digit target value from `current` by position (`(current / 10^position) % 10`)
                - Switches `SpritesheetAnimation` on the child sprite entity to the matching from→to transition clip
                - Updates `Digit::value` to the new digit

## Plugin Systems

### Attach Player Animations

Reacts to the `TiledEvent<ObjectCreated>` message emitted by the Tiled loader when a player object is created. For each matching `Player`-marked `TiledObject` entity, it walks the entity hierarchy to find the child entity carrying a `Sprite`, reads its image handle to build a `Spritesheet`, then creates idle animation handles for the three directional variants (`idle_x`, `idle_down`, `idle_up`). It stores these handles in a per-player resource (`PlayerOneAnimations` or `PlayerTwoAnimations`) and inserts a `SpritesheetAnimation` on the child sprite entity with the initial idle clip.

### Update Players Animation

Runs every frame. For each `Player`-marked `TiledObject` entity, it walks the entity hierarchy to find the descendant with a `Sprite` and `SpritesheetAnimation`, then switches the active animation clip based on the player's current `LookDirection`. It also sets `Sprite::flip_x` to mirror the horizontal idle sprite when the player faces left.

### Attach Claimed Tile Animations

Reacts to `Added<ClaimedTile>` — fires once for each newly spawned claimed tile entity. Builds the `ClaimedTileAnimations` resource (if not already present) containing animation clip handles for each player color variant. Inserts `SpritesheetAnimation` (with the neutral/default clip), `Sprite`, and `BounceEffect` on the claimed tile entity so it is ready to display ownership animations.

### Update Claimed Tile Animation

Reads `BeamResolved` messages. For each message, resolves the claimed tile entity from `MapInfo::claimed_entities` using the message's `GridCoords` position. Reads the `ClaimedTileAnimations` resource to select the correct player-color clip based on the owning player id, then switches the `SpritesheetAnimation` clip on the claimed tile entity. Also inserts `BounceEffectTarget` on the claimed tile entity to trigger the bounce visual effect.

### Attach Digit Animations

Reacts to the `TiledEvent<ObjectCreated>` message for entities carrying a `Digit` component. For each matching entity, walks the hierarchy to find the child sprite entity, reads its image handle to build a `Spritesheet`, then creates all 90 directional transition animation handles (every `from != to` combination in `0..10`) via a single `make_anim` closure. The special 9→0 and 0→9 wrap transitions use non-contiguous frame sequences (`add_cell(39, 8)` + `add_partial_row(8, 0..=3)` played forwards or backwards). All handles are stored in the `DigitAnimations` resource. A `SpritesheetAnimation` is inserted on the child sprite entity.

### Update Digit Animation

Reads `BeamChargesChanged` messages. For each message, iterates all `Digit`-marked entities whose `Player::player_id` matches `message.player_id`. For each digit, computes the target value as `(message.current / 10^digit.position) % 10`, looks up the from→to transition handle in `DigitAnimations`, then walks the entity's children to find the `SpritesheetAnimation` and switches it to the new clip. Updates `Digit::value` to the new digit value.

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
- **attach_digit_animations**: used to walk descendants via `iter_descendants` to find the child sprite entity
- **update_digit_animation**: used to walk descendants via `iter_descendants` to find the child `SpritesheetAnimation`

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
attach_digit_animations["`**attach_digit_animations**`"]
update_digit_animation["`**update_digit_animation**`"]

update -.-> attach_player_animations
update -.-> update_players_animation
update -.-> attach_digit_animations
update -.-> update_digit_animation

children_query{{"`children_query`"}}:::query
attach_player_animations ---> children_query
update_players_animation ---> children_query
attach_digit_animations ---> children_query
update_digit_animation ---> children_query

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

### Read TiledEvent ObjectCreated messages (digits)

Used in the following systems:
- **attach_digit_animations**: used to trigger animation setup when a digit Tiled object is created

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_digit_animations["`**attach_digit_animations**`"]

update -.-> attach_digit_animations

message_reader{{"MessageReader#60;TiledEvent#60;ObjectCreated#62;#62;"}}:::reader
attach_digit_animations ---> message_reader

object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])

message_reader ---> |reads| object_created_message
```

### Read BeamChargesChanged messages

Used in the following systems:
- **update_digit_animation**: used to trigger digit flip-counter animation when a player's beam charges change

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_digit_animation["`**update_digit_animation**`"]

update -.-> update_digit_animation

message_reader{{"MessageReader#60;BeamChargesChanged#62;"}}:::reader
update_digit_animation ---> message_reader

beam_charges_changed_message(["`**BeamChargesChanged**`"])

message_reader ---> |reads| beam_charges_changed_message
```

### Query Digit entities (attach)

Used in the following systems:
- **attach_digit_animations**: detects newly created `Digit`-marked `TiledObject` entities and initializes their animation components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_digit_animations["`**attach_digit_animations**`"]

update -.-> attach_digit_animations

digits_query{{"`digits_query`"}}:::query
attach_digit_animations ---> digits_query

digit_entity@{ shape: st-rect, label: "Digit (TiledObject)" }

de_entity>"`**Entity**`"] --> |belongs to| digit_entity
de_digit>"`**Digit**`"] --> |belongs to| digit_entity

digits_query ---> |reads| de_entity
digits_query -..-> |filter With| de_digit
```

### Query Digit entities (update)

Used in the following systems:
- **update_digit_animation**: reads `Player::player_id`, `Digit::position`, and mutably updates `Digit::value` for all digit entities matching the message's player

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_digit_animation["`**update_digit_animation**`"]

update -.-> update_digit_animation

digits_query{{"`digits_query`"}}:::query
update_digit_animation ---> digits_query

digit_entity@{ shape: st-rect, label: "Digit Entity" }

de_entity>"`**Entity**`"] --> |belongs to| digit_entity
de_player>"`**Player**`"] --> |belongs to| digit_entity
de_digit>"`**Digit**`"] --> |belongs to| digit_entity

digits_query ---> |reads| de_entity
digits_query ---> |reads| de_player
digits_query ---> |writes| de_digit
```

### Read DigitAnimations resource

Used in the following systems:
- **update_digit_animation**: used to retrieve the from→to transition animation handle for each digit; accessed via `If<Res<...>>` (optional — skipped if not yet inserted)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
update_digit_animation["`**update_digit_animation**`"]

update -.-> update_digit_animation

world@{ shape: st-rect, label: "World" }
digit_anims_res@{ shape: doc, label: "DigitAnimations" }

da_handles>"`**handles #91;#91;Handle#60;Animation#62;; 10#93;; 10#93;**`"]

digit_anims_res --> |belongs to| world
da_handles --> |field of| digit_anims_res

update_digit_animation ---> |reads `get(from, to)`| digit_anims_res
```

### Write DigitAnimations resource

Used in the following systems:
- **attach_digit_animations**: builds all 90 from→to transition animation handles and inserts the `DigitAnimations` resource into the world

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_digit_animations["`**attach_digit_animations**`"]

update -.-> attach_digit_animations

world@{ shape: st-rect, label: "World" }
digit_anims_res@{ shape: doc, label: "DigitAnimations" }

da_handles>"`**handles #91;#91;Handle#60;Animation#62;; 10#93;; 10#93;**`"]

digit_anims_res --> |belongs to| world
da_handles --> |field of| digit_anims_res

attach_digit_animations ---> |inserts resource| digit_anims_res
```

### Write commands (attach digit animations)

Used in the following systems:
- **attach_digit_animations**: inserts `SpritesheetAnimation` on the child sprite entity of each `Digit` entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_digit_animations["`**attach_digit_animations**`"]

update -.-> attach_digit_animations

digit_child_entity@{ shape: st-rect, label: "Digit Child (Sprite)" }

dc_anim>"`**SpritesheetAnimation**`"]

dc_anim --> |inserted on| digit_child_entity

attach_digit_animations ---> |inserts component| dc_anim
```

### Write SpritesheetAnimation (update digit)

Used in the following systems:
- **update_digit_animation**: switches the `SpritesheetAnimation` clip on the digit child sprite entity to the from→to transition clip

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_digit_animation["`**update_digit_animation**`"]

update -.-> update_digit_animation

sprite_animations_query{{"`sprite_animations_query (mutable)`"}}:::query
update_digit_animation ---> sprite_animations_query

digit_child_entity@{ shape: st-rect, label: "Digit Child (Sprite)" }

dc_anim>"`**SpritesheetAnimation**`"] --> |belongs to| digit_child_entity

sprite_animations_query ---> |writes (switches clip)| dc_anim
```
