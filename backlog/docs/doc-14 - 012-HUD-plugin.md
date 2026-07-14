---
id: doc-14
title: '[012] HUD plugin'
type: other
created_date: '2026-07-14 12:00'
updated_date: '2026-07-14 12:00'
---
# HUD Plugin

Owns all HUD animations rendered on the HUD camera (render layer 1): the HP-bar fill (`animate_hp`) and the numeric counters rendered as rolling-odometer digit sprites. For the counters it holds the generic digit-animation machinery (the `DigitAnimations` resource and `initialize_digit_animations` system) plus one `animate_*` system per counter. Every value the HUD displays is maintained by its own domain plugin — player health by the Damage plugin, beam charges by the Beam plugin, claimed-tile count by the Claim plugin — so this plugin never computes or mutates those values; it only reads them and drives the HUD sprites, lerping the HP bar's `Transform` toward the current health ratio and switching each `Digit` entity's `SpritesheetAnimation` to the correct from→to transition clip when the underlying value changes.

It is registered immediately after the Animations plugin in `AppPlugin`.

## Plugin workflow

- Update phase
    - Animate HP:
        - Runs every frame
            - Reads:
                - All `Player`-marked `DamageEffectTarget` entities with their `Health` and `Player` components
                - All `HPBar` entities with their `Player` and `Transform` components
            - Writes:
                - Lerps `Transform::scale.x` on each matching `HPBar` entity toward `Health::ratio()` for the corresponding player (snapping to `0.0` once it drops below `0.001`)
    - Initialize Digit Animations:
        - Reacts to `TiledEvent<ObjectCreated>` message
            - Reads:
                - All `Digit`-marked `TiledObject` entities and their `Entity` components
                - The `Sprite` component on each digit's child sprite entity (to get the image handle)
            - Writes:
                - Builds all 90 from→to transition animation handles (for all `from != to` in `0..10`) using a single `make_anim` closure
                - Inserts `DigitAnimations` resource into the world
                - Inserts `SpritesheetAnimation` on the child sprite entity
    - Animate Beam Charges:
        - Reacts to `Changed<BeamCharges>` on player entities
            - Reads:
                - `Player` and `BeamCharges` components on changed player entities
                - All `BeamChargesDigit`-marked entities with `Player` and `Digit` components
                - `DigitAnimations` resource (optional/`If`)
            - Writes:
                - Computes per-digit target value from `BeamCharges::current` by position (`(current / 10^position) % 10`)
                - Switches `SpritesheetAnimation` on the child sprite entity to the matching from→to transition clip
                - Updates `Digit::value` to the new digit
    - Animate Claimed Tiles:
        - Reacts to `Changed<ClaimedTileCount>` on player entities
            - Reads:
                - `Player` and `ClaimedTileCount` components on changed player entities
                - All `ClaimedTilesDigit`-marked entities with `Player` and `Digit` components
                - `MapInfo` resource (to read `ground_entities` for the total tile count)
                - `DigitAnimations` resource (optional/`If`)
            - Writes:
                - Computes the owned-tile count as a rounded percentage of the whole board (`(current * 100 + total/2) / total`, guarded when `total == 0`)
                - Computes per-digit target value from that percentage by position (`(percent / 10^position) % 10`)
                - Switches `SpritesheetAnimation` on the child sprite entity to the matching from→to transition clip
                - Updates `Digit::value` to the new digit

## Plugin Systems

### Animate HP

Runs every frame. Queries all player entities that carry `DamageEffectTarget`, reading their `Health` and `Player` components. For each player, it finds the matching `HPBar` entity by `player_id` and lerps the bar's `Transform::scale.x` toward `Health::ratio()` (a value in `[0.0, 1.0]`, snapped to `0.0` once it drops below `0.001`), giving the bar a smooth animated transition rather than an instant snap.

### Initialize Digit Animations

Reacts to the `TiledEvent<ObjectCreated>` message for entities carrying a `Digit` component. For each matching entity, walks the hierarchy to find the child sprite entity, reads its image handle to build a `Spritesheet`, then creates all 90 directional transition animation handles (every `from != to` combination in `0..10`) via a single `make_anim` closure. The special 9→0 and 0→9 wrap transitions use non-contiguous frame sequences (`add_cell(39, 2)` + `add_partial_row(2, 0..=3)` played forwards or backwards). All handles are stored in the `DigitAnimations` resource. A `SpritesheetAnimation` is inserted on the child sprite entity.

### Animate Beam Charges

Runs every frame, filtered by `Changed<BeamCharges>`. For each player entity whose `BeamCharges` component changed, iterates all `BeamChargesDigit`-marked entities whose `Player::player_id` matches. For each digit, computes the target value as `(BeamCharges::current / 10^digit.position) % 10`, looks up the from→to transition handle in `DigitAnimations`, then walks the entity's children to find the `SpritesheetAnimation` and switches it to the new clip. Updates `Digit::value` to the new digit value.

### Animate Claimed Tiles

Runs every frame, filtered by `Changed<ClaimedTileCount>`. Reads `MapInfo::ground_entities` to obtain the total number of ground tiles (returning early if that total is zero). For each player entity whose `ClaimedTileCount` changed, computes the owned-tile count as a rounded percentage of the whole board (`(count.current * 100 + total / 2) / total`, giving a value in `0..=100`). It then iterates all `ClaimedTilesDigit`-marked entities whose `Player::player_id` matches, computes each digit's target value as `(percent / 10^digit.position) % 10`, looks up the from→to transition handle in `DigitAnimations`, walks the entity's children to find the `SpritesheetAnimation` and switches it to the new clip, and updates `Digit::value`. It is the digit-display counterpart of `animate_beam_charges`, but renders each player's owned-tile count as a rounded percentage rather than a raw charge count.

## Components, Resources and Messages CRUD

### Read TiledEvent ObjectCreated messages (digits)

Used in the following systems:
- **initialize_digit_animations**: used to trigger animation setup when a digit Tiled object is created

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
initialize_digit_animations["`**initialize_digit_animations**`"]

update -.-> initialize_digit_animations

message_reader{{"MessageReader#60;TiledEvent#60;ObjectCreated#62;#62;"}}:::reader
initialize_digit_animations ---> message_reader

object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])

message_reader ---> |reads| object_created_message
```

### Query Digit entities (attach)

Used in the following systems:
- **initialize_digit_animations**: detects newly created `Digit`-marked `TiledObject` entities and initializes their animation components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
initialize_digit_animations["`**initialize_digit_animations**`"]

update -.-> initialize_digit_animations

digits_query{{"`digits_query`"}}:::query
initialize_digit_animations ---> digits_query

digit_entity@{ shape: st-rect, label: "Digit (TiledObject)" }

de_entity>"`**Entity**`"] --> |belongs to| digit_entity
de_digit>"`**Digit**`"] --> |belongs to| digit_entity

digits_query ---> |reads| de_entity
digits_query -..-> |filter With| de_digit
```

### Query Children hierarchy

Used in the following systems:
- **initialize_digit_animations**: walks descendants via `iter_descendants` to find the child sprite entity
- **animate_beam_charges**: walks descendants via `iter_descendants` to find the child `SpritesheetAnimation`
- **animate_claimed_tiles**: walks descendants via `iter_descendants` to find the child `SpritesheetAnimation`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
initialize_digit_animations["`**initialize_digit_animations**`"]
animate_beam_charges["`**animate_beam_charges**`"]
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> initialize_digit_animations
update -.-> animate_beam_charges
update -.-> animate_claimed_tiles

children_query{{"`children_query`"}}:::query
initialize_digit_animations ---> children_query
animate_beam_charges ---> children_query
animate_claimed_tiles ---> children_query

child_entity@{ shape: st-rect, label: "Any Child Entity" }

ch_children>"`**Children**`"] --> |belongs to| child_entity

children_query ---> |reads| ch_children
```

### Query Player entities with Changed\<BeamCharges\>

Used in the following systems:
- **animate_beam_charges**: detects players whose `BeamCharges` component changed this frame to drive digit flip-counter animations

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_beam_charges["`**animate_beam_charges**`"]

update -.-> animate_beam_charges

players_query{{"`players_query`"}}:::query
animate_beam_charges ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_charges>"`**BeamCharges**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
players_query ---> |reads| pe_charges
players_query -..-> |filter Changed| pe_charges
```

### Query BeamChargesDigit entities (update)

Used in the following systems:
- **animate_beam_charges**: reads `Player::player_id`, `Digit::position`, and mutably updates `Digit::value` for all `BeamChargesDigit`-marked entities matching the changed player

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_beam_charges["`**animate_beam_charges**`"]

update -.-> animate_beam_charges

digits_query{{"`digits_query`"}}:::query
animate_beam_charges ---> digits_query

digit_entity@{ shape: st-rect, label: "Digit Entity" }

de_entity>"`**Entity**`"] --> |belongs to| digit_entity
de_player>"`**Player**`"] --> |belongs to| digit_entity
de_digit>"`**Digit**`"] --> |belongs to| digit_entity
de_marker>"`**BeamChargesDigit**`"] --> |belongs to| digit_entity

digits_query ---> |reads| de_entity
digits_query ---> |reads| de_player
digits_query ---> |writes| de_digit
digits_query -..-> |filter With| de_marker
```

### Query Player entities with Changed\<ClaimedTileCount\>

Used in the following systems:
- **animate_claimed_tiles**: detects players whose `ClaimedTileCount` component changed this frame to drive claimed-tile percentage digit animations

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> animate_claimed_tiles

players_query{{"`players_query`"}}:::query
animate_claimed_tiles ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
players_query ---> |reads| pe_count
players_query -..-> |filter Changed| pe_count
```

### Query ClaimedTilesDigit entities (update)

Used in the following systems:
- **animate_claimed_tiles**: reads `Player::player_id`, `Digit::position`, and mutably updates `Digit::value` for all `ClaimedTilesDigit`-marked entities matching the changed player

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> animate_claimed_tiles

digits_query{{"`digits_query`"}}:::query
animate_claimed_tiles ---> digits_query

digit_entity@{ shape: st-rect, label: "Digit Entity" }

de_entity>"`**Entity**`"] --> |belongs to| digit_entity
de_player>"`**Player**`"] --> |belongs to| digit_entity
de_digit>"`**Digit**`"] --> |belongs to| digit_entity
de_marker>"`**ClaimedTilesDigit**`"] --> |belongs to| digit_entity

digits_query ---> |reads| de_entity
digits_query ---> |reads| de_player
digits_query ---> |writes| de_digit
digits_query -..-> |filter With| de_marker
```

### Read MapInfo resource (claimed tiles)

Used in the following systems:
- **animate_claimed_tiles**: reads `MapInfo::ground_entities` to obtain the total number of ground tiles used to compute the owned-tile percentage

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> animate_claimed_tiles

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

animate_claimed_tiles ---> |reads `ground_entities`| map_info_res
```

### Read DigitAnimations resource

Used in the following systems:
- **animate_beam_charges**: used to retrieve the from→to transition animation handle for each digit; accessed via `If<Res<...>>` (optional — skipped if not yet inserted)
- **animate_claimed_tiles**: used to retrieve the from→to transition animation handle for each digit; accessed via `If<Res<...>>` (optional — skipped if not yet inserted)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
animate_beam_charges["`**animate_beam_charges**`"]
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> animate_beam_charges
update -.-> animate_claimed_tiles

world@{ shape: st-rect, label: "World" }
digit_anims_res@{ shape: doc, label: "DigitAnimations" }

da_handles>"`**handles**`"]

digit_anims_res --> |belongs to| world
da_handles --> |field of| digit_anims_res

animate_beam_charges ---> |"reads get(from, to)"| digit_anims_res
animate_claimed_tiles ---> |"reads get(from, to)"| digit_anims_res
```

### Write DigitAnimations resource

Used in the following systems:
- **initialize_digit_animations**: builds all 90 from→to transition animation handles and inserts the `DigitAnimations` resource into the world

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_digit_animations["`**initialize_digit_animations**`"]

update -.-> initialize_digit_animations

world@{ shape: st-rect, label: "World" }
digit_anims_res@{ shape: doc, label: "DigitAnimations" }

da_handles>"`**handles**`"]

digit_anims_res --> |belongs to| world
da_handles --> |field of| digit_anims_res

initialize_digit_animations ---> |inserts resource| digit_anims_res
```

### Write commands (attach digit animations)

Used in the following systems:
- **initialize_digit_animations**: inserts `SpritesheetAnimation` on the child sprite entity of each `Digit` entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_digit_animations["`**initialize_digit_animations**`"]

update -.-> initialize_digit_animations

digit_child_entity@{ shape: st-rect, label: "Digit Child (Sprite)" }

dc_anim>"`**SpritesheetAnimation**`"]

dc_anim --> |inserted on| digit_child_entity

initialize_digit_animations ---> |inserts component| dc_anim
```

### Write SpritesheetAnimation (update digit)

Used in the following systems:
- **animate_beam_charges**: switches the `SpritesheetAnimation` clip on the digit child sprite entity to the from→to transition clip
- **animate_claimed_tiles**: switches the `SpritesheetAnimation` clip on the digit child sprite entity to the from→to transition clip

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_beam_charges["`**animate_beam_charges**`"]
animate_claimed_tiles["`**animate_claimed_tiles**`"]

update -.-> animate_beam_charges
update -.-> animate_claimed_tiles

sprite_animations_query{{"`sprite_animations_query (mutable)`"}}:::query
animate_beam_charges ---> sprite_animations_query
animate_claimed_tiles ---> sprite_animations_query

digit_child_entity@{ shape: st-rect, label: "Digit Child (Sprite)" }

dc_anim>"`**SpritesheetAnimation**`"] --> |belongs to| digit_child_entity

sprite_animations_query ---> |"writes (switches clip)"| dc_anim
```

### Query Player entities (health)

Used in the following systems:
- **animate_hp**: reads `Health` and `Player` components on `DamageEffectTarget`-marked entities to determine the current health ratio for each player

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_hp["`**animate_hp**`"]

update -.-> animate_hp

players_query{{"`players_query`"}}:::query
animate_hp ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_health>"`**Health**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_marker>"`**DamageEffectTarget**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_health
players_query ---> |reads| pe_player
players_query -..-> |filter With| pe_marker
```

### Query HPBar entities

Used in the following systems:
- **animate_hp**: reads the `Player` component (to match against player id) and writes `Transform::scale.x` to reflect the current health ratio

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
animate_hp["`**animate_hp**`"]

update -.-> animate_hp

hp_bars_query{{"`hp_bars_query`"}}:::query
animate_hp ---> hp_bars_query

hp_bar_entity@{ shape: st-rect, label: "HPBar Entity" }

hb_hp_bar>"`**HPBar**`"] --> |belongs to| hp_bar_entity
hb_transform>"`**Transform**`"] --> |belongs to| hp_bar_entity

hp_bars_query ---> |reads| hb_hp_bar
hp_bars_query ---> |writes| hb_transform
```
