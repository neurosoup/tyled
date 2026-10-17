---
id: doc-2
title: '[002] Input plugin'
type: other
created_date: '2026-01-27 18:05'
updated_date: '2026-10-17 12:00'
---
# Input Plugin

This plugin reads the player keys and turns them into move, shoot and stop messages. It also sets up the input on each new player and turns characters in place when they face a new direction. A bot player has no key map: the Bot plugin fills the same action state, earlier in the same frame.

The plugin also registers the `InputManagerPlugin` and the `TweeningPlugin`.

## Plugin Systems

### Attach Players Actions

- Runs in `PreUpdate`, once for each new player.
- A human seat gets an `InputMap`. A bot seat gets a bare `ActionState` and the `Bot` marker instead.
- Both kinds of seat also get `MoveRepeat` and `MovementSlide`.

### Handle Characters Input

- Runs in `Update`, only during `Playing`. Skips knocked-back and dead characters.
- **Lock** switches the direction lock on or off. A locked character never turns.
- **Shoot** fires only if a charge is left. From a claimed tile it also needs `Lance`, else nothing is spent.
- Lock and Shoot also work during a turn.
- **Release** after moving asks for a settle effect and resets the repeat, so the next press steps at once.
- **New facing** (unlocked): the facing changes at once and a turn starts.
- A turn from rest holds the step back, so a quick tap only turns. A turn while running keeps stepping.
- **Step**: the first step is immediate. The next ones wait `move_repeat_delay_ms`, then `move_repeat_rate_ms`.
- Straight (non-diagonal) steps wait `1/√2` less, so they look as fast as diagonal steps.

### Tick Turning

- Runs in `Update`, only during `Playing`. Moves each turn through its steps, then removes `IsTurning`.
- A 90° turn has one step. A 180° turn has two, through a fixed middle direction.
- It never changes the facing, so a shot during a turn goes in the new direction.

## Components, Resources and Messages CRUD

### Query Player entities for action attachment

Used in the following systems:
- **attach_players_actions**: new players with no `InputMap` yet. The config says if the seat is a bot.

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

players_query -..-> |filter Added| pe_player
players_query -..-> |filter Without| pe_input_map
players_query -..-> |filter With| pe_character

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }
config_res --> |belongs to| world

attach_players_actions ---> |reads `controllers.is_bot`| config_res
```

### Write commands — attach InputMap or bot ActionState

Used in the following systems:
- **attach_players_actions**: human and bot seats get different components.

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
bot_player_entity@{ shape: st-rect, label: "Player (bot seat)" }

pe_input_map>"`**InputMap#60;Action#62;**`"]
pe_action_state>"`**ActionState#60;Action#62;**`"]
pe_bot>"`**Bot**`"]
pe_move_repeat>"`**MoveRepeat**`"]
pe_slide>"`**MovementSlide**`"]

pe_input_map --> |inserted on| player_entity
pe_move_repeat --> |inserted on| player_entity
pe_slide --> |inserted on| player_entity

pe_action_state --> |inserted on| bot_player_entity
pe_bot --> |inserted on| bot_player_entity
pe_move_repeat --> |inserted on| bot_player_entity
pe_slide --> |inserted on| bot_player_entity

attach_players_actions ---> |"inserts component (human)"| pe_input_map
attach_players_actions ---> |"inserts component (bot)"| pe_action_state
attach_players_actions ---> |"inserts component (bot)"| pe_bot
attach_players_actions ---> |inserts component| pe_move_repeat
attach_players_actions ---> |inserts component| pe_slide
```

### Query Character entities for input handling

Used in the following systems:
- **handle_characters_input**: all characters, except knocked-back or dead ones.

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
pe_move_repeat>"`**MoveRepeat**`"] --> |belongs to| character_entity
pe_slide>"`**MovementSlide**`"] --> |belongs to| character_entity
pe_is_knocked_back>"`**IsKnockedBack**`"] --> |belongs to| character_entity
pe_is_dead>"`**IsDead**`"] --> |belongs to| character_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_action_state
players_query ---> |reads| pe_grid_coords
players_query ---> |writes| pe_look_direction
players_query ---> |writes| pe_move_repeat
players_query ---> |writes| pe_slide
players_query ---> |"reads (optional)"| pe_beam_charges
players_query ---> |"reads (optional)"| pe_ability_list
players_query ---> |"reads (optional)"| pe_is_turning
players_query -..-> |filter With| pe_character
players_query -..-> |filter Without| pe_is_knocked_back
players_query -..-> |filter Without| pe_is_dead
```

### Read fire-gate inputs (MapInfo + ClaimedTile)

Used in the following systems:
- **handle_characters_input**: checks who owns the tile under the player before a shot.

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
- **handle_characters_input**: one message for each step. A turn and a step can happen in the same frame.

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
- **handle_characters_input**: one message for each allowed shot.

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

### Write MovementStopped messages

Used in the following systems:
- **handle_characters_input**: when the move key is released after moving.

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

movement_stopped_message(["`**MovementStopped**`"])

handle_characters_input ---> |writes| movement_stopped_message
```

### Write commands — insert IsTurning

Used in the following systems:
- **handle_characters_input**: when an unlocked character faces a new direction. A new turn replaces the old one.

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
- **tick_turning**: all characters that are turning.

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
pe_is_turning>"`**IsTurning**`"] --> |belongs to| character_entity

turners_query ---> |reads| pe_entity
turners_query ---> |writes| pe_is_turning

tick_turning ---> |removes component| pe_is_turning
```
