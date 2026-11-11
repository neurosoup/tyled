---
id: doc-20
title: '[018] Bot plugin'
type: other
created_date: '2026-07-21 12:00'
updated_date: '2026-11-11 12:00'
---
# Bot Plugin

This plugin plays a player for the computer. It writes the bot's `ActionState<Action>`, and the Input plugin reads it like a human's input. The code is in `src/plugins/bot.rs`.

## Key ideas

- The bot has no `InputMap`. It presses the same actions as a human, so it follows the same fire and charge rules.
- `controllers.player1_bot` / `player2_bot` choose which players are bots. The Input plugin reads them and adds the `Bot` marker.
- Each bot plays in one of two modes, chosen by `bot.player1_strike` / `player2_strike`:
    - **Striking**: hunts the opponent and shoots at them. It ignores territory.
    - **Territory** (default): claims tiles, dodges beams, and uses its abilities.
- The **parry reflex** runs every frame, before everything else. A parry window is shorter than the time between two decisions.
- All other decisions happen on a **beat**: every `bot.think_interval_ms`, plus a small random jitter. Between beats, the bot stands still and does not shoot.
- `bot.parry_chance` is rolled once per incoming beam. The result is kept, so the reflex and the brace/dodge choice agree on the same beam.
- Each decision is copied into `BotDecision`. The Telemetry plugin logs it when it changes.

## Decision flow

```mermaid
---
config:
  theme: dark
---

flowchart TD

frame["`**Each frame**`"]
reflex{"`Parryable beam and parry roll succeeds?`"}
parry["`**parry**`"]
beat{"`Beat reached?`"}
wait["`Stand still, release Shoot`"]
mode{"`Striking mode?`"}

frame --> reflex
reflex --> |yes| parry
reflex --> |no| beat
beat --> |no| wait
beat --> |yes| mode

subgraph striking ["Striking"]
  line{"`Line of fire to the opponent?`"}
  strike["`**aim** / **strike_lance** / **strike_straight**`"]
  hunt["`**hunt** toward the opponent, or **idle**`"]
  line --> |yes| strike
  line --> |no| hunt
end

subgraph territory ["Territory, in priority order"]
  threat{"`Beam coming on my row or column?`"}
  brace["`**brace** / **dodge**`"]
  grinder{"`Border Grinder flip worth chasing?`"}
  chase["`**border_grinder_chase**`"]
  fire{"`Can fire?`"}
  shoot["`**aim** / **claim** / **aggress** / **lance** / **lance_strike**`"]
  target{"`Reachable unclaimed tile?`"}
  move["`**reposition** / **aggress** (Dijkstra)`"]
  rest["`**hold** on safe ground, **retreat** from enemy ground, or **idle**`"]

  threat --> |yes| brace
  threat --> |no| grinder
  grinder --> |yes| chase
  grinder --> |no| fire
  fire --> |yes| shoot
  fire --> |no| target
  target --> |yes| move
  target --> |no| rest
end

mode --> |yes| line
mode --> |no| threat
```

- With Border Grinder, a shot that flips an enemy tile is chosen before any normal claim.
- The Border Grinder chase is skipped if a flip is possible from the current tile, or if the path costs more than `bot.chase_cost_threshold`.
- A striking bot without Lance never steps onto an enemy tile, because it cannot fire from there.

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph update ["Update"]
  attach_bot_state["`**attach_bot_state**
  every phase`"]
  bot_think["`**bot_think**
  GameplaySet::BotThink, only during Playing`"]
  input["`**GameplaySet::Input**
  handle_characters_input reads the ActionState`"]
end

attach_bot_state --> bot_think
bot_think --> input
```

## Plugin Systems

### Attach Bot State

- Adds `BotBrain` and `BotDecision` to each new `Bot` entity.

### Bot Think

- Runs every frame during `Playing`, for each bot.
- Follows the decision flow above. Writes the move axis and the Shoot button into `ActionState<Action>`.
- Replaces `BotDecision` only when the decision changes.

### Helpers

- `reach`: counts the free tiles in a straight line.
- `border_grinder_target`: true if the next tile belongs to the enemy.
- `lance_landing`, `lance_hits_enemy`: where a Lance shot lands, and whether it hits the opponent.
- `shot_hits_enemy`: whether a Straight or Lance shot hits the opponent.
- `incoming_beam_threat`: finds an enemy beam coming toward the bot, with the facing to parry it and a safe dodge step.
- `dijkstra_first_steps`: cost and first step to every reachable tile. Entering an enemy tile costs `bot.hostile_cost`.
- `astar_first_step`: first step toward one target tile. Used by the striking mode.
- `reposition_score`, `fires_toward_opponent`: scoring for the territory mode, biased toward the opponent when `bot.aggression` ≥ 0.5.

## Components, Resources and Messages CRUD

### Read GameConfig and Time

Used in the following systems:
- **bot_think**: reads the `bot.*` values, `parry.window_fraction` and the elapsed time.

### Query new bots

Used in the following systems:
- **attach_bot_state**: finds `Bot` entities that were just added and have no `BotBrain`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
attach_bot_state["`**attach_bot_state**`"]

update -.-> attach_bot_state

bots_query{{"`bots`"}}:::query
attach_bot_state ---> bots_query

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_entity>"`**Entity**`"] --> |belongs to| bot_entity
be_bot>"`**Bot**`"] --> |belongs to| bot_entity
be_brain>"`**BotBrain**`"] --> |belongs to| bot_entity

bots_query ---> |reads| be_entity
bots_query -..-> |filter Added| be_bot
bots_query -..-> |filter Without| be_brain
```

### Insert BotBrain and BotDecision

Used in the following systems:
- **attach_bot_state**: inserts both with default values.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
attach_bot_state["`**attach_bot_state**`"]

update -.-> attach_bot_state

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_brain>"`**BotBrain**`"]
be_decision>"`**BotDecision**`"]

be_brain --> |inserted on| bot_entity
be_decision --> |inserted on| bot_entity

attach_bot_state ---> |inserts component| be_brain
attach_bot_state ---> |inserts component| be_decision
```

### Query bots

Used in the following systems:
- **bot_think**: reads the bot's position, facing, charges and abilities. Writes its `ActionState`, `BotDecision` and `BotBrain`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

bots_query{{"`bots`"}}:::query
bot_think ---> bots_query

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_entity>"`**Entity**`"] --> |belongs to| bot_entity
be_player>"`**Player**`"] --> |belongs to| bot_entity
be_grid_coords>"`**GridCoords**`"] --> |belongs to| bot_entity
be_look_direction>"`**LookDirection**`"] --> |belongs to| bot_entity
be_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| bot_entity
be_beam_charges>"`**BeamCharges**`"] --> |belongs to| bot_entity
be_ability_list>"`**AbilityList**`"] --> |belongs to| bot_entity
be_decision>"`**BotDecision**`"] --> |belongs to| bot_entity
be_brain>"`**BotBrain**`"] --> |belongs to| bot_entity
be_bot>"`**Bot**`"] --> |belongs to| bot_entity

bots_query ---> |reads| be_entity
bots_query ---> |reads| be_player
bots_query ---> |reads| be_grid_coords
bots_query ---> |reads| be_look_direction
bots_query ---> |writes| be_action_state
bots_query ---> |"reads (optional)"| be_beam_charges
bots_query ---> |"reads (optional)"| be_ability_list
bots_query ---> |writes| be_decision
bots_query ---> |writes| be_brain
bots_query -..-> |filter With| be_bot
```

### Query player positions

Used in the following systems:
- **bot_think**: finds the opponent's tile. `With<Character>` is needed because HUD entities also have `Player` and `GridCoords`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

positions_query{{"`positions`"}}:::query
bot_think ---> positions_query

player_entity@{ shape: st-rect, label: "Character Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_character>"`**Character**`"] --> |belongs to| player_entity

positions_query ---> |reads| pe_entity
positions_query ---> |reads| pe_grid_coords
positions_query -..-> |filter With| pe_player
positions_query -..-> |filter With| pe_character
```

### Query beams

Used in the following systems:
- **bot_think**: finds parryable beams (`find_parryable_beam`, shared with the Input plugin) and incoming beams (`incoming_beam_threat`).

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

beams_query{{"`beams`"}}:::query
bot_think ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }

bm_entity>"`**Entity**`"] --> |belongs to| beam_entity
bm_grid_coords>"`**GridCoords**`"] --> |belongs to| beam_entity
bm_beam>"`**Beam**`"] --> |belongs to| beam_entity
bm_character>"`**Character**`"]

beams_query ---> |reads| bm_entity
beams_query ---> |reads| bm_grid_coords
beams_query ---> |reads| bm_beam
beams_query -..-> |filter Without| bm_character
```

### Read MapInfo and ClaimedTile

Used in the following systems:
- **bot_think**: checks ground, forbidden tiles and tile owners for firing, pathfinding and threats.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

claimed_query{{"`claimed_query`"}}:::query
bot_think ---> claimed_query

tile_entity@{ shape: st-rect, label: "Ground Tile" }
te_claimed>"`**ClaimedTile**`"] --> |belongs to| tile_entity
claimed_query ---> |reads| te_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }
map_info_res --> |belongs to| world

bot_think ---> |reads| map_info_res
```

### ActionState of a bot

Used in the following systems:
- **bot_think**: writes the move axis and presses or releases Shoot.
- **handle_characters_input** (Input plugin): reads it, the same way as for a human.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]
handle_characters_input["`**handle_characters_input**
Input plugin`"]

update -.-> bot_think
update -.-> handle_characters_input

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| bot_entity

bot_think ---> |writes| be_action_state
handle_characters_input ---> |reads| be_action_state
```

### BotDecision

Used in the following systems:
- **bot_think**: replaces it when the decision changes.
- **record_decisions** (Telemetry plugin): logs it.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]
record_decisions["`**record_decisions**
Telemetry plugin`"]

update -.-> bot_think
update -.-> record_decisions

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_decision>"`**BotDecision**`"] --> |belongs to| bot_entity

bot_think ---> |writes| be_decision
record_decisions ---> |reads| be_decision
record_decisions -..-> |filter Changed| be_decision
```

Definitions:
- `Bot`: marker `Component` (`src/components/markers.rs`), added by the Input plugin to a bot player.
- `BotDecision`: public `Component` with `behaviour` (a short tag, in bold in the decision flow), `why`, `move_x`, `move_y` and `shoot`.
- `BotBrain`: private `Component` with the fire cooldown state, the next beat time, a sticky `target` tile and the cached `parry_rolls`.
