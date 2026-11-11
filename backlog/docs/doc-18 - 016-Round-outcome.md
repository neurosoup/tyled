---
id: doc-18
title: '[016] Round outcome'
type: other
created_date: '2026-07-16 12:00'
updated_date: '2026-11-02 12:00'
---
# Round Outcome

This part of the [Round plugin](/documentation/15/013-round-plugin) shows the winner banner at the end of a round, then starts the next round. The code is in `src/plugins/round/outcome.rs`.

## Key ideas

- The banner shows during `RoundPhase::Outcome`. It says "P1 WINS", "P2 WINS" or "DRAW", from `RoundResult::winner`.
- The `state` submodule chooses the winner and enters `Outcome`. This submodule only shows the result.
- The text is drawn with `spawn_round_label`, on the overlay camera, like the [Round intro](/documentation/16/014-round-intro).
- The banner stays for `round.outcome_linger_secs` (2.5 s). This gives the loser's death bounce time to end.
- Then the phase goes back to `Starting`. On the way out, `reset_round` (in `state`) cleans the board, and the intro countdown starts again.

```mermaid
---
config:
  theme: dark
---

flowchart TD

enter["`**Enter Outcome**
shows the banner`"]
leave["`**Exit Outcome**
banner despawned, board reset`"]
starting["`**Starting**
intro countdown`"]

enter --> |"2.5 s"| leave
leave --> starting
```

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph enter_outcome ["OnEnter(Outcome)"]
  show_outcome_banner["`**show_outcome_banner**`"]
end

subgraph update ["Update, no set"]
  advance_outcome["`**advance_outcome**
  only during Outcome`"]
end

subgraph exit_outcome ["OnExit(Outcome)"]
  despawn_outcome_banner["`**despawn_outcome_banner**`"]
end

enter_outcome ~~~ update
update ~~~ exit_outcome
```

- `advance_outcome` writes `NextState<RoundPhase>`. It is in `RoundPhaseWriter`, like the other systems that set the phase.

## Plugin Systems

### Show Outcome Banner

- Runs on entering `Outcome`.
- Shows the banner text with the `OutcomeBanner` marker.
- Inserts `OutcomeTimer`, a one-shot timer of `round.outcome_linger_secs`.

### Advance Outcome

- Runs only during `Outcome`. Ticks `OutcomeTimer`.
- When the timer ends, sets the next phase to `Starting`.

### Despawn Outcome Banner

- Runs on leaving `Outcome`.
- Despawns the banner with its letters and removes `OutcomeTimer`.

## Components, Resources and Messages CRUD

### Read FontAtlas, GameConfig and Time

Used in the following systems:
- **show_outcome_banner**: reads the font and `round.outcome_linger_secs`.
- **advance_outcome**: reads `Time` to tick the timer.

### Read RoundResult resource

Used in the following systems:
- **show_outcome_banner**: reads `winner` to choose the banner text.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_outcome(("`OnEnter(Outcome)`")):::system-group
show_outcome_banner["`**show_outcome_banner**`"]

enter_outcome -.-> show_outcome_banner

world@{ shape: st-rect, label: "World" }
result_res@{ shape: doc, label: "RoundResult" }

rr_winner>"`**winner**`"]

result_res --> |belongs to| world
rr_winner --> |field of| result_res

show_outcome_banner ---> |reads| rr_winner
```

### OutcomeTimer resource

Used in the following systems:
- **show_outcome_banner**: inserts it.
- **advance_outcome**: ticks it.
- **despawn_outcome_banner**: removes it.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_outcome(("`OnEnter(Outcome)`")):::system-group
update(("`Update`")):::system-group
exit_outcome(("`OnExit(Outcome)`")):::system-group
show_outcome_banner["`**show_outcome_banner**`"]
advance_outcome["`**advance_outcome**`"]
despawn_outcome_banner["`**despawn_outcome_banner**`"]

enter_outcome -.-> show_outcome_banner
update -.-> advance_outcome
exit_outcome -.-> despawn_outcome_banner

world@{ shape: st-rect, label: "World" }
timer_res@{ shape: doc, label: "OutcomeTimer" }

timer_res --> |belongs to| world

show_outcome_banner ---> |inserts resource| timer_res
advance_outcome ---> |ticks| timer_res
despawn_outcome_banner ---> |removes resource| timer_res
```

### Spawn banner label

Used in the following systems:
- **show_outcome_banner**: spawns the banner with `OutcomeBanner`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_outcome(("`OnEnter(Outcome)`")):::system-group
show_outcome_banner["`**show_outcome_banner**`"]

enter_outcome -.-> show_outcome_banner

banner_entity@{ shape: st-rect, label: "Banner Label Entity" }

ob_marker>"`**OutcomeBanner**`"] --> |inserted on| banner_entity

show_outcome_banner ---> |spawns| banner_entity
```

### Query OutcomeBanner entities

Used in the following systems:
- **despawn_outcome_banner**: despawns the banner.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

exit_outcome(("`OnExit(Outcome)`")):::system-group
despawn_outcome_banner["`**despawn_outcome_banner**`"]

exit_outcome -.-> despawn_outcome_banner

banners_query{{"`banners`"}}:::query
despawn_outcome_banner ---> banners_query

banner_entity@{ shape: st-rect, label: "Banner Label Entity" }

ob_entity>"`**Entity**`"] --> |belongs to| banner_entity
ob_marker>"`**OutcomeBanner**`"] --> |belongs to| banner_entity

banners_query ---> |reads| ob_entity
banners_query -..-> |filter With| ob_marker
despawn_outcome_banner ---> |despawns| banner_entity
```

### Set NextState RoundPhase

Used in the following systems:
- **advance_outcome**: sets `Starting` when the timer ends.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
advance_outcome["`**advance_outcome**`"]

update -.-> advance_outcome

next_state_res@{ shape: doc, label: "NextState<RoundPhase>" }

advance_outcome ---> |sets Starting| next_state_res
```

Definitions:
- `OutcomeBanner`: private marker `Component` on the banner label.
- `OutcomeTimer`: private `Resource` with a one-shot `Timer`.
