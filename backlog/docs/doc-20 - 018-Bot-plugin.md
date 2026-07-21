---
id: doc-20
title: '[018] Bot plugin'
type: other
created_date: '2026-07-21 12:00'
updated_date: '2026-07-21 12:00'
---
# Bot Plugin

Drives a bot-controlled player's synthetic `ActionState<Action>` through the same `handle_characters_input` system a human's `InputMap` drives, so the bot is bound by the identical fire and charge gates — it can never bypass `resolve_fire`'s claimed-tile refusal or fire without a `BeamCharges` charge. Each beat it deliberates in priority order — fire, aim, or path toward a target tile — and mirrors the chosen behaviour into `BotDecision` so the Telemetry plugin can log bot reasoning alongside human play.

## Concepts

- `Bot` — zero-sized marker component (`src/components/markers.rs`). Attached by the Input plugin's `attach_players_actions` to a bot-controlled player's entity in place of an `InputMap<Action>`; a bare `ActionState<Action>` is inserted alongside it instead. No standalone marker doc exists for it — it is documented here since this plugin is its primary consumer.
- `BotDecision` — public component mirroring the bot's most recently chosen behaviour: `behaviour` (a short tag such as `"claim"`, `"aggress"`, `"aim"`, `"reposition"`, `"idle"`), `why` (a human-readable reason string), `move_x`/`move_y` (the chosen move axis), and `shoot` (whether it fired this beat). Overwritten only when the value differs from the previous beat, so a cross-module reader — the Telemetry plugin's `record_decisions` — can log strictly on change via `Changed<BotDecision>`.
- `BotBrain` — private per-bot scratch state: `last_fire_secs` and `shooting` (fire-cooldown bookkeeping), `next_beat_secs` (the paced-deliberation gate), and `target` (a sticky `GridCoords` destination that persists across beats until it is claimed or claimed by the opponent).
- `config.controllers.player1_bot` / `player2_bot` — which seats are bot-driven; read by the Input plugin, not here.
- `config.bot.fire_cooldown_ms`, `aggression`, `think_interval_ms`, `hostile_cost` — the four tunables `bot_think` reads every beat (see the Config plugin doc for edit-timing tags).

## Plugin workflow

- Update phase
    - Attach Bot State reacts to newly added `Bot` entities that don't yet carry `BotBrain` and inserts default `BotBrain`/`BotDecision`.
    - Bot Think (`.after(attach_bot_state)`, `.before(handle_characters_input)`, gated `in_state(RoundPhase::Playing)`) paces itself to `think_interval_ms` and, once its beat elapses, decides whether to fire, turn to aim, or path toward a target tile, writing the result into the bot's `ActionState<Action>` and mirroring it into `BotDecision`.

## Plugin Systems

### Attach Bot State

Runs in `Update`. Query filters `Added<Bot>, Without<BotBrain>` — for every bot entity that was just added and doesn't yet carry brain state, inserts `BotBrain::default()` and `BotDecision::default()`.

### Bot Think

Runs in `Update`, ordered `.after(attach_bot_state)` and `.before(handle_characters_input)` (so its synthesized `ActionState` is in place before the input handler reads it that same frame), gated `in_state(RoundPhase::Playing)`.

**Beat gate**: if `time.elapsed_secs()` is still short of `brain.next_beat_secs`, the system zeroes the move axis, releases `Action::Shoot`, and continues to the next bot without deliberating — this paces movement/aim/fire choices to `config.bot.think_interval_ms` rather than re-deciding every frame. Otherwise it schedules the next beat and deliberates:

1. **Fire check** — computes whether firing from the current tile is legal at all (`fireable`, via `resolve_fire` with the bot's own `AbilityList`/`Backfill`), and `best_fire`: the current facing's `reach` if it is at least 1 tile, otherwise whichever of the four `CARDINALS` has the greatest `reach` (≥ 1). Committing to the current facing until its line is exhausted avoids swivelling between two directions of equal reach.
2. **Has charges and a shot is available** —
   - If not yet facing `best_fire`'s direction: releases `Action::Shoot`, turns to face it (moves the axis toward that direction) — behaviour `"aim"`.
   - Otherwise, once facing the runway: if `fire_cooldown_ms` has elapsed since the last shot and it isn't already mid-shot, presses `Action::Shoot` — behaviour `"aggress"` if the shot's line also crosses the opponent's tile (`fires_toward_opponent`), else `"claim"`. When no runway exists at all, firing into the blocked neighbour still claims the bot's own tile, so this same branch also covers that "claim current tile" case.
3. **No charges, or no shot available** — releases `Action::Shoot` and pathfinds via `dijkstra_first_steps` (4-connected from the bot's tile, cost 1 per normal tile and `config.bot.hostile_cost` to enter an opponent-owned tile). Keeps the sticky `brain.target` if it is still reachable and unclaimed; otherwise picks the reachable unclaimed tile that minimizes `reposition_score` (Dijkstra cost, biased toward the opponent's tile once `aggression ≥ 0.5`), breaking ties by Manhattan distance; if the whole board is already claimed, falls back to targeting the opponent's own tile when it is reachable, so the bot pressures rather than idles. Steps one tile toward the target — behaviour `"aggress"` if `aggression ≥ 0.5` else `"reposition"` — or reports `"idle"` if nothing is reachable.

The resulting move axis is written to `ActionState::<Action>::Move`, and a `BotDecision` built from the chosen behaviour/reason/axis/shoot flag replaces the component only if it differs from the previous beat's.

### reach (helper)

Private helper, not a system. Counts consecutive unclaimed, non-forbidden, on-ground tiles starting one step past `from` in direction `dir`, stopping at the first claimed tile, forbidden area, or off-ground position. Used by `bot_think` to score candidate firing directions.

### dijkstra_first_steps (helper)

Private helper, not a system. Runs Dijkstra's algorithm over walkable ground tiles from the bot's position, 4-connected via `CARDINALS`. Entering a tile owned by an opponent (`is_hostile_tile`) costs `hostile_cost`; any other ground tile costs 1. Returns, per reachable coordinate, the total cost and the first step taken from the start to reach it — the latter is what lets `bot_think` move one tile per beat toward a multi-tile-distant target without recomputing the full path.

## Components, Resources and Messages CRUD

### Query bots awaiting brain state

Used in the following systems:
- **attach_bot_state**: detects `Bot` entities that were just added and don't yet carry `BotBrain`, and inserts `BotBrain`/`BotDecision` on them

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

bot_entity@{ shape: st-rect, label: "Bot" }

be_entity>"`**Entity**`"] --> |belongs to| bot_entity
be_bot>"`**Bot**`"] --> |belongs to| bot_entity
be_brain>"`**BotBrain**`"] --> |belongs to| bot_entity

bots_query ---> |reads| be_entity
bots_query -..-> |filter Added| be_bot
bots_query -..-> |filter Without| be_brain
```

### Write commands — insert BotBrain and BotDecision

Used in the following systems:
- **attach_bot_state**: inserts default `BotBrain` and `BotDecision` on each newly attached bot entity

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

bot_entity@{ shape: st-rect, label: "Bot" }

be_brain>"`**BotBrain**`"]
be_decision>"`**BotDecision**`"]

be_brain --> |inserted on| bot_entity
be_decision --> |inserted on| bot_entity

attach_bot_state ---> |inserts component| be_brain
attach_bot_state ---> |inserts component| be_decision
```

### Query bots for decision-making

Used in the following systems:
- **bot_think**: reads position and facing, mutably drives `ActionState#60;Action#62;`, and reads/writes its own `BotDecision`/`BotBrain` scratch state, for all `Bot` entities

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

bot_entity@{ shape: st-rect, label: "Bot (Player)" }

be_entity>"`**Entity**`"] --> |belongs to| bot_entity
be_grid_coords>"`**GridCoords**`"] --> |belongs to| bot_entity
be_look_direction>"`**LookDirection**`"] --> |belongs to| bot_entity
be_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| bot_entity
be_beam_charges>"`**BeamCharges**`"] --> |belongs to| bot_entity
be_ability_list>"`**AbilityList**`"] --> |belongs to| bot_entity
be_decision>"`**BotDecision**`"] --> |belongs to| bot_entity
be_brain>"`**BotBrain**`"] --> |belongs to| bot_entity
be_bot>"`**Bot**`"] --> |belongs to| bot_entity

bots_query ---> |reads| be_entity
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
- **bot_think**: collects every `Player` entity's `Entity` + `GridCoords` up front, to locate the opponent's current tile for aggression scoring and the opponent-pressure fallback

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

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity

positions_query ---> |reads| pe_entity
positions_query ---> |reads| pe_grid_coords
positions_query -..-> |filter With| pe_player
```

### Read MapInfo and ClaimedTile (bot pathfinding and fire check)

Used in the following systems:
- **bot_think**: reads `MapInfo.on_ground`/`on_forbidden_areas`/`claimed_entities` and `ClaimedTile.owner` (via `resolve_fire`, `reach`, `is_position_claimed`, `is_hostile_tile`, and `dijkstra_first_steps`) to decide whether a shot is legal, score candidate firing directions, and pathfind toward a reachable unclaimed tile

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

claimed_query{{"Query#60;#38;ClaimedTile#62;"}}:::query
bot_think ---> claimed_query

tile_entity@{ shape: st-rect, label: "Ground tile" }
te_claimed>"`**ClaimedTile**`"] --> |belongs to| tile_entity
claimed_query ---> |reads `owner`| te_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }
map_info_res --> |belongs to| world

bot_think ---> |reads `on_ground`| map_info_res
bot_think ---> |reads `on_forbidden_areas`| map_info_res
bot_think ---> |reads `claimed_entities`| map_info_res
```

### Read GameConfig (bot tuning)

Used in the following systems:
- **bot_think**: reads `config.bot.fire_cooldown_ms`, `config.bot.aggression`, `config.bot.think_interval_ms`, and `config.bot.hostile_cost` every beat

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }
config_res --> |belongs to| world

bot_think ---> |reads `bot.*`| config_res
```

### Write ActionState (bot)

Used in the following systems:
- **bot_think**: sets `Action::Move`'s axis pair each beat, and presses or releases `Action::Shoot` — the same component `handle_characters_input` reads for a human, so the bot obeys identical fire/charge gates

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]

update -.-> bot_think

bot_entity@{ shape: st-rect, label: "Bot" }

be_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| bot_entity

bot_think ---> |writes `Action::Move` axis| be_action_state
bot_think ---> |"writes (press/release) `Action::Shoot`"| be_action_state
```

### Write BotDecision (cross-module read)

Used in the following systems:
- **bot_think**: replaces `BotDecision` with the beat's chosen behaviour, reason, move axis, and shoot flag, only when it differs from the previous value

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
bot_think["`**bot_think**`"]
record_decisions["`**record_decisions** (Telemetry plugin)`"]

update -.-> bot_think
update -.-> record_decisions

bot_entity@{ shape: st-rect, label: "Bot" }

be_decision>"`**BotDecision**`"] --> |belongs to| bot_entity

bot_think ---> |"writes (on change)"| be_decision
record_decisions ---> |"reads (filter Changed)"| be_decision
```
