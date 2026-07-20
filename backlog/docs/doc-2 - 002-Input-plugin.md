---
id: doc-2
title: '[002] Input plugin'
type: other
created_date: '2026-01-27 18:05'
updated_date: '2026-07-20 12:00'
---
# Input Plugin

Contains systems related to player input handling. This plugin registers the `InputManagerPlugin` and the `TweeningPlugin`, sets up an input throttle timer resource, attaches input maps to player entities, and dispatches `EntityMoved` and `BeamFired` messages in reaction to player actions. A facing change while the direction is unlocked starts a transient `IsTurning` state that rotates the character in place through an intermediate 3/4 pose before movement in the new direction resumes.

## Plugin workflow

- Startup phase
    - Setup Input Timer creates the `InputTimer` repeating resource (throttle period `config.timing.input_tick_secs`, default 0.075 s).
- PreUpdate phase
    - Attach Players Actions reacts to newly added `Player` + `Character` entities (without `InputMap`) and inserts the appropriate `InputMap<Action>`.
- Update phase
    - Handle Characters Input ticks the timer and, for each character:
        - Handles `Action::Lock` (toggles look-direction lock)
        - Handles `Action::Shoot` (writes a `BeamFired` message only if the player has a charge **and** the shot is not blocked — firing from an already-claimed tile is refused unless the player has the `Backfill` ability, in which case no message is written, so no beam spawns and no charge is spent) — allowed even mid-turn
        - On a `Action::Move` axis that implies a new facing (while unlocked), inserts an `IsTurning` state and skips movement this frame; a fresh facing change mid-turn restarts the turn toward the new target
        - While an `IsTurning` state is present but the facing has not changed, movement stays suppressed
        - When no turn is active and the timer finishes, reads `Action::Move` axis and writes an `EntityMoved` message
    - Tick Turning advances each `IsTurning` timer through its segments and, only once the target facing is reached, commits `LookDirection` to the target and removes `IsTurning`; `LookDirection` is left untouched mid-turn so it keeps the original heading

## Plugin Systems

### Setup Input Timer

Inserts the `InputTimer` resource, a repeating `Timer` whose period is `config.timing.input_tick_secs` (default 0.075 s) that acts as a throttle on movement inputs.

### Attach Players Actions

Runs in `PreUpdate`. Detects newly spawned `Player` + `Character` entities that do not yet have an `InputMap<Action>` and inserts the appropriate `InputMap<Action>` derived from the player's data.

### Handle Characters Input

Runs in `Update`. Ticks the `InputTimer` and iterates over all `Character` entities (excluding those with `IsKnockedBack`). Immediately handles `Action::Lock` (toggles direction lock) and `Action::Shoot` — emits a `BeamFired` message only when the character has a charge (`BeamCharges::current > 0`) **and** `resolve_fire` (Beam plugin) permits the shot: firing from an already-claimed tile is refused unless the player's `AbilityList` contains `Backfill`, and a refused shot writes no message (no beam, no charge). Both stay active during a turn. For movement it uses `LookDirection::would_look_at` to detect whether the pressed axis implies a new facing: while unlocked, a change from the current heading (or the active turn's target) inserts an `IsTurning` state — starting or restarting the turn immediately, bypassing the throttle — and skips movement that frame. If a turn is already in progress toward the same target, movement stays suppressed. Otherwise, when the timer is finished, it updates `LookDirection` and emits an `EntityMoved` message with the new target `GridCoords`. Locked characters never turn (direction is frozen) and keep strafing along the pressed axis.

### Tick Turning

Runs in `Update` (gated on `RoundPhase::Playing`). Ticks each character's `IsTurning` timer; when a segment elapses it pops the next cardinal waypoint and advances the turn's internal `from` (which the 3/4 pose depends on), then either resets the timer for the next quarter or, once the final target is reached, commits `LookDirection` to the target and removes `IsTurning`. `LookDirection` is deliberately **not** changed mid-turn: it holds the character's original heading for the whole turn, so a shot fired mid-turn fires along the direction the player was facing before the turn began. A 90° turn is one segment; a 180° turn is two, routed through a fixed middle cardinal.

## Components, Resources and Messages CRUD

### Read InputTimer resource

Used in the following systems:
- **handle_characters_input**: ticks and checks the throttle timer each frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update`")):::system-group
setup_input_timer["`**setup_input_timer**`"]
handle_characters_input["`**handle_characters_input**`"]

startup -.-> setup_input_timer
update -.-> handle_characters_input

world@{ shape: st-rect, label: "World" }
input_timer_res@{ shape: doc, label: "InputTimer" }

input_timer_res --> |belongs to| world

setup_input_timer ---> |inserts resource| input_timer_res
handle_characters_input ---> |reads & ticks| input_timer_res
```

### Query Player entities for action attachment

Used in the following systems:
- **attach_players_actions**: detects `Player` + `Character` entities that were just added and do not yet carry an `InputMap<Action>`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

preupdate(("`PreUpdate`")):::system-group
attach_players_actions["`**attach_players_actions**`"]

preupdate -.-> attach_players_actions

players_query{{"`players`"}}:::query
attach_players_actions ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_input_map>"`**InputMap#60;Action#62;**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
pe_character>"`**Character**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query -..-> |filter Added| pe_player
players_query -..-> |filter Without| pe_input_map
players_query -..-> |filter With| pe_character
```

### Write commands — attach InputMap

Used in the following systems:
- **attach_players_actions**: inserts `InputMap<Action>` on each newly added `Player` entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

preupdate(("`PreUpdate`")):::system-group
attach_players_actions["`**attach_players_actions**`"]

preupdate -.-> attach_players_actions

player_entity@{ shape: st-rect, label: "Player" }

pe_input_map>"`**InputMap#60;Action#62;**`"]

pe_input_map --> |inserted on| player_entity

attach_players_actions ---> |inserts component| pe_input_map
```

### Query Character entities for input handling

Used in the following systems:
- **handle_characters_input**: reads action state, grid coords, and the optional `AbilityList` and `IsTurning` state, mutably updates look direction, for all `Character` entities (excluding those with `IsKnockedBack`); it also reads `MapInfo` + `ClaimedTile` to gate firing (see the separate section below)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
handle_characters_input["`**handle_characters_input**`"]

update -.-> handle_characters_input

players_query{{"`players`"}}:::query
handle_characters_input ---> players_query

character_entity@{ shape: st-rect, label: "Character" }

pe_entity>"`**Entity**`"] --> |belongs to| character_entity
pe_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| character_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| character_entity
pe_look_direction>"`**LookDirection**`"] --> |belongs to| character_entity
pe_character>"`**Character**`"] --> |belongs to| character_entity
pe_beam_charges>"`**BeamCharges**`"] --> |belongs to| character_entity
pe_ability_list>"`**AbilityList**`"] --> |belongs to| character_entity
pe_is_turning>"`**IsTurning**`"] --> |belongs to| character_entity
pe_is_knocked_back>"`**IsKnockedBack**`"] --> |belongs to| character_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_action_state
players_query ---> |reads| pe_grid_coords
players_query ---> |writes| pe_look_direction
players_query ---> |"reads (optional)"| pe_beam_charges
players_query ---> |"reads (optional)"| pe_ability_list
players_query ---> |"reads (optional)"| pe_is_turning
players_query -..-> |filter With| pe_character
players_query -..-> |filter Without| pe_is_knocked_back
```

### Read fire-gate inputs (MapInfo + ClaimedTile)

Used in the following systems:
- **handle_characters_input**: to decide whether a Shoot press may fire, reads `MapInfo.claimed_entities` + the `ClaimedTile.owner` at the character's origin (via `resolve_fire`, Beam plugin) — a shot from an already-claimed tile is refused unless the player has `Backfill`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
handle_characters_input["`**handle_characters_input**`"]

update -.-> handle_characters_input

claimed_query{{"Query#60;#38;ClaimedTile#62;"}}:::query
handle_characters_input ---> claimed_query

tile_entity@{ shape: st-rect, label: "Origin tile" }
te_claimed>"`**ClaimedTile**`"] --> |belongs to| tile_entity
claimed_query ---> |reads `owner`| te_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }
map_info_res --> |belongs to| world
handle_characters_input ---> |reads `claimed_entities`| map_info_res
```

### Write EntityMoved messages

Used in the following systems:
- **handle_characters_input**: emits an `EntityMoved` message when the movement axis is non-zero, no turn is in progress, and the input timer has finished


```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
handle_characters_input["`**handle_characters_input**`"]

update -.-> handle_characters_input

entity_moved_message(["`**EntityMoved**`"])

handle_characters_input ---> |writes| entity_moved_message
```

### Write BeamFired messages

Used in the following systems:
- **handle_characters_input**: emits a `BeamFired` message when `Action::Shoot` is just pressed, the player has a charge, and the shot is not blocked (see the fire-gate reads above)


```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
handle_characters_input["`**handle_characters_input**`"]

update -.-> handle_characters_input

beam_fired_message(["`**BeamFired**`"])

handle_characters_input ---> |writes| beam_fired_message
```

### Write commands — insert IsTurning

Used in the following systems:
- **handle_characters_input**: inserts (or replaces) an `IsTurning` state on a character when an unlocked facing change is detected, starting or restarting the turn

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
handle_characters_input["`**handle_characters_input**`"]

update -.-> handle_characters_input

character_entity@{ shape: st-rect, label: "Character" }

pe_is_turning>"`**IsTurning**`"]

pe_is_turning --> |inserted on| character_entity

handle_characters_input ---> |inserts component| pe_is_turning
```

### Query turning characters

Used in the following systems:
- **tick_turning**: advances each `IsTurning` timer and, when the turn completes, commits `LookDirection` to the target facing and removes `IsTurning` (`LookDirection` is left unchanged mid-turn)

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
tick_turning["`**tick_turning**`"]

update -.-> tick_turning

turners_query{{"`turners`"}}:::query
tick_turning ---> turners_query

character_entity@{ shape: st-rect, label: "Character" }

pe_entity>"`**Entity**`"] --> |belongs to| character_entity
pe_look_direction>"`**LookDirection**`"] --> |belongs to| character_entity
pe_is_turning>"`**IsTurning**`"] --> |belongs to| character_entity

turners_query ---> |reads| pe_entity
turners_query ---> |writes| pe_look_direction
turners_query ---> |writes| pe_is_turning

tick_turning ---> |removes component| pe_is_turning
```
