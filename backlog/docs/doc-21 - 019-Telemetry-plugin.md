---
id: doc-21
title: '[019] Telemetry plugin'
type: other
created_date: '2026-07-21 12:00'
updated_date: '2026-11-11 12:00'
---
# Telemetry Plugin

This plugin records human and bot play into `play_trace.jsonl`, one JSON object per line, for balance analysis. The code is in `src/plugins/telemetry.rs`.

## Key ideas

- `telemetry.enabled` turns the whole plugin on or off. Every system checks it.
- If the file cannot be opened, there is no `TelemetrySink`, and all systems do nothing.
- At start, the old trace files are rotated: `play_trace.jsonl` becomes `play_trace.1.jsonl`, and so on. `telemetry.history` is the total number of files kept.
- Each record has a `kind`. Player records also have a `controller` tag: `"human"` or `"bot"`.
- Actions and decisions are written only when they change, so the file stays small.

| `kind` | Written | Content |
|---|---|---|
| `session` | once, at start | which players are bots, and the starting loadouts |
| `action` | when a player's state changes | position, facing, shoot, charges, on an enemy tile |
| `decision` | when a bot's `BotDecision` changes | the bot's behaviour, reason and move |
| `outcome` | once per round | winner, reason (`kill`, `timeout`, `charge_exhaustion`), duration, and tiles, HP, charges spent and loadout per player |

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph startup ["Startup"]
  open_sink["`**open_sink**`"]
end

subgraph update ["Update, no set, only during Playing"]
  record_actions["`**record_actions**`"]
  record_decisions["`**record_decisions**`"]
end

subgraph enter_outcome ["OnEnter(Outcome)"]
  record_outcome["`**record_outcome**`"]
end

startup ~~~ update
update ~~~ enter_outcome
```

- All four systems run only when `telemetry.enabled` is true.

## Plugin Systems

### Open Sink

- Runs once at `Startup`.
- Rotates the old files, opens a new `play_trace.jsonl` and writes the `session` record.
- Inserts `TelemetrySink`. If the file cannot be opened, logs a warning and inserts nothing.

### Record Actions

- Runs every frame during `Playing`.
- Builds a snapshot of each player. If it differs from the last one in `ActionSnapshots`, writes an `action` record.

### Record Decisions

- Runs every frame during `Playing`.
- Writes a `decision` record for each bot whose `BotDecision` changed.

### Record Outcome

- Runs on entering `Outcome`.
- Finds the reason: `kill` if a player has no HP, `timeout` if the countdown is 0, else `charge_exhaustion`.
- Writes the `outcome` record and flushes the file.

## Components, Resources and Messages CRUD

### Read GameConfig and Time

Used in the following systems:
- **open_sink**: reads `telemetry.*` and `controllers.*`.
- **record_actions**, **record_decisions**: read the elapsed time.
- **record_outcome**: reads `round.round_duration_secs`.

### Read PlayerLoadouts resource

Used in the following systems:
- **open_sink**: writes both loadouts in the `session` record.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
open_sink["`**open_sink**`"]

startup -.-> open_sink

world@{ shape: st-rect, label: "World" }
loadouts_res@{ shape: doc, label: "PlayerLoadouts" }

pl_player1>"`**player1**`"]
pl_player2>"`**player2**`"]

loadouts_res --> |belongs to| world
pl_player1 --> |field of| loadouts_res
pl_player2 --> |field of| loadouts_res

open_sink ---> |reads| pl_player1
open_sink ---> |reads| pl_player2
```

### TelemetrySink resource

Used in the following systems:
- **open_sink**: inserts it.
- **record_actions**, **record_decisions**, **record_outcome**: write records into it. They do nothing when it is missing.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
update(("`Update`")):::system-group
enter_outcome(("`OnEnter(Outcome)`")):::system-group
open_sink["`**open_sink**`"]
record_actions["`**record_actions**`"]
record_decisions["`**record_decisions**`"]
record_outcome["`**record_outcome**`"]

startup -.-> open_sink
update -.-> record_actions
update -.-> record_decisions
enter_outcome -.-> record_outcome

world@{ shape: st-rect, label: "World" }
sink_res@{ shape: doc, label: "TelemetrySink" }

ts_writer>"`**writer**`"]

sink_res --> |belongs to| world
ts_writer --> |field of| sink_res

open_sink ---> |inserts resource| sink_res
record_actions ---> |writes| ts_writer
record_decisions ---> |writes| ts_writer
record_outcome ---> |"writes, flushes"| ts_writer
```

### ActionSnapshots resource

Used in the following systems:
- **plugin build**: inserts it empty.
- **record_actions**: compares each player with its last snapshot, and stores the new one.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

build(("`Plugin build`")):::system-group
update(("`Update`")):::system-group
record_actions["`**record_actions**`"]

update -.-> record_actions

world@{ shape: st-rect, label: "World" }
snapshots_res@{ shape: doc, label: "ActionSnapshots" }

snapshots_res --> |belongs to| world

build ---> |inserts resource| snapshots_res
record_actions ---> |reads| snapshots_res
record_actions ---> |writes| snapshots_res
```

### Query players (actions)

Used in the following systems:
- **record_actions**: reads the state of each player to build its snapshot.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
record_actions["`**record_actions**`"]

update -.-> record_actions

players_query{{"`players`"}}:::query
record_actions ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_grid_coords>"`**GridCoords**`"] --> |belongs to| player_entity
pe_look_direction>"`**LookDirection**`"] --> |belongs to| player_entity
pe_action_state>"`**ActionState#60;Action#62;**`"] --> |belongs to| player_entity
pe_beam_charges>"`**BeamCharges**`"] --> |belongs to| player_entity
pe_bot>"`**Bot**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query ---> |reads| pe_grid_coords
players_query ---> |reads| pe_look_direction
players_query ---> |reads| pe_action_state
players_query ---> |"reads (optional)"| pe_beam_charges
players_query ---> |"reads (optional)"| pe_bot
```

### Read MapInfo and ClaimedTile

Used in the following systems:
- **record_actions**: checks if the player stands on an enemy tile (`is_hostile_tile`).
- **record_outcome**: reads the number of ground tiles, for `tile_pct`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
enter_outcome(("`OnEnter(Outcome)`")):::system-group
record_actions["`**record_actions**`"]
record_outcome["`**record_outcome**`"]

update -.-> record_actions
enter_outcome -.-> record_outcome

claimed_query{{"`claimed_query`"}}:::query
record_actions ---> claimed_query

tile_entity@{ shape: st-rect, label: "Ground Tile" }
te_claimed>"`**ClaimedTile**`"] --> |belongs to| tile_entity
claimed_query ---> |reads| te_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

mi_claimed>"`**claimed_entities**`"]
mi_ground>"`**ground_entities**`"]

map_info_res --> |belongs to| world
mi_claimed --> |field of| map_info_res
mi_ground --> |field of| map_info_res

record_actions ---> |reads| mi_claimed
record_outcome ---> |reads| mi_ground
```

### Query bot decisions

Used in the following systems:
- **record_decisions**: reads the bots whose `BotDecision` changed.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
record_decisions["`**record_decisions**`"]

update -.-> record_decisions

decisions_query{{"`decisions`"}}:::query
record_decisions ---> decisions_query

bot_entity@{ shape: st-rect, label: "Bot Player" }

be_player>"`**Player**`"] --> |belongs to| bot_entity
be_decision>"`**BotDecision**`"] --> |belongs to| bot_entity
be_bot>"`**Bot**`"] --> |belongs to| bot_entity

decisions_query ---> |reads| be_player
decisions_query ---> |reads| be_decision
decisions_query -..-> |filter With| be_bot
decisions_query -..-> |filter Changed| be_decision
```

### Read RoundResult and Countdown

Used in the following systems:
- **record_outcome**: reads the winner, and the time left to find the reason and the duration.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_outcome(("`OnEnter(Outcome)`")):::system-group
record_outcome["`**record_outcome**`"]

enter_outcome -.-> record_outcome

world@{ shape: st-rect, label: "World" }
result_res@{ shape: doc, label: "RoundResult" }
countdown_res@{ shape: doc, label: "Countdown" }

rr_winner>"`**winner**`"]
cd_remaining>"`**remaining**`"]

result_res --> |belongs to| world
countdown_res --> |belongs to| world
rr_winner --> |field of| result_res
cd_remaining --> |field of| countdown_res

record_outcome ---> |reads| rr_winner
record_outcome ---> |"reads (optional)"| cd_remaining
```

### Query players (outcome)

Used in the following systems:
- **record_outcome**: reads tiles, HP, charges and abilities of each player.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

enter_outcome(("`OnEnter(Outcome)`")):::system-group
record_outcome["`**record_outcome**`"]

enter_outcome -.-> record_outcome

players_query{{"`players`"}}:::query
record_outcome ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_tile_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity
pe_health>"`**Health**`"] --> |belongs to| player_entity
pe_beam_charges>"`**BeamCharges**`"] --> |belongs to| player_entity
pe_ability_list>"`**AbilityList**`"] --> |belongs to| player_entity
pe_bot>"`**Bot**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
players_query ---> |reads| pe_tile_count
players_query ---> |reads| pe_health
players_query ---> |reads| pe_beam_charges
players_query ---> |reads| pe_ability_list
players_query ---> |"reads (optional)"| pe_bot
```

Definitions:
- `TelemetrySink`: private `Resource` with a buffered writer on `play_trace.jsonl`.
- `ActionSnapshots`: private `Resource` with the last written snapshot of each player.
- `SessionRecord`, `ActionRecord`, `DecisionRecord`, `OutcomeRecord`: the four record types, serialized with `serde`.
