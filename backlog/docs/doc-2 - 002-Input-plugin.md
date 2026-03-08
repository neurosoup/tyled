---
id: doc-2
title: '[002] Input plugin'
type: other
created_date: '2026-01-27 18:05'
updated_date: '2026-03-08 17:04'
---
# Input Plugin

Contains systems related to player input handling. This plugin registers the `InputManagerPlugin` and the `TweeningPlugin`, sets up an input throttle timer resource, attaches input maps to player entities, and dispatches `PlayerMoved` and `BeamFired` messages in reaction to player actions.

## Plugin workflow

- Startup phase
    - Setup Input Timer creates the `InputTimer` repeating resource (16ms throttle).
- PreUpdate phase
    - Attach Players Actions reacts to newly added `Player` entities (without `InputMap`) and inserts the appropriate `InputMap<Action>`.
- Update phase
    - Handle Players Input ticks the timer and, for each player:
        - Handles `Action::Lock` (toggles look-direction lock)
        - Handles `Action::Shoot` (writes a `BeamFired` message)
        - When the timer finishes, reads `Action::Move` axis and writes a `PlayerMoved` message

## Plugin Systems

### Setup Input Timer

Inserts the `InputTimer` resource, a repeating `Timer` with a 62.5ms period that acts as a throttle on movement inputs.

### Attach Players Actions

Runs in `PreUpdate`. Detects newly spawned `Player` entities that do not yet have an `InputMap<Action>` and inserts the appropriate `InputMap<Action>` derived from the player's data.

### Handle Players Input

Runs in `Update`. Ticks the `InputTimer` and iterates over all players. Immediately handles `Action::Lock` (toggles direction lock) and `Action::Shoot` (emits a `BeamFired` message). When the timer is finished, reads the movement axis from `Action::Move`, updates `LookDirection`, and emits a `PlayerMoved` message with the new target `GridCoords`.

## Components, Resources and Messages CRUD

### Read InputTimer resource

Used in the following systems:
- **handle_players_input**: ticks and checks the throttle timer each frame

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
handle_players_input["`**handle_players_input**`"]

startup -.-> setup_input_timer
update -.-> handle_players_input

world@{ shape: st-rect, label: "World" }
input_timer_res@{ shape: doc, label: "InputTimer" }

input_timer_res --> |belongs to| world

setup_input_timer ---> |inserts resource| input_timer_res
handle_players_input ---> |reads & ticks| input_timer_res
```

### Query Player entities for action attachment

Used in the following systems:
- **attach_players_actions**: detects `Player` entities that were just added and do not yet carry an `InputMap<Action>`

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
players_query -..-> |filter Added| pe_player
players_query -..-> |filter Without| pe_input_map
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

### Query Player entities for input handling

Used in the following systems:
- **handle_players_input**: reads action state and grid coords, mutably updates look direction for all `Player` entities

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
handle_players_input["`**handle_players_input**`"]

update -.-> handle_players_input

players_query{{"`players`"}}:::query
handle_players_input ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| player_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_look_direction>"`**LookDirection**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_action_state
players_query ---> |reads| pe_grid_coords
players_query ---> |writes| pe_look_direction
players_query -..-> |filter With| pe_player
```

### Write PlayerMoved messages

Used in the following systems:
- **handle_players_input**: emits a `PlayerMoved` message when the movement axis is non-zero and the input timer has finished

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
handle_players_input["`**handle_players_input**`"]

update -.-> handle_players_input

player_moved_message(["`**PlayerMoved**`"])

handle_players_input ---> |writes| player_moved_message
```

### Write BeamFired messages

Used in the following systems:
- **handle_players_input**: emits a `BeamFired` message when `Action::Shoot` is just pressed

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
handle_players_input["`**handle_players_input**`"]

update -.-> handle_players_input

beam_fired_message(["`**BeamFired**`"])

handle_players_input ---> |writes| beam_fired_message
```
