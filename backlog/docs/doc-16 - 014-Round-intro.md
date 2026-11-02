---
id: doc-16
title: '[014] Round intro'
type: other
created_date: '2026-07-15 12:00'
updated_date: '2026-10-30 12:00'
---
# Round Intro

This part of the [Round plugin](/documentation/15/013-round-plugin) shows the "3 · 2 · 1 · GO!" banner at the start of each round. The code is in `src/plugins/round/intro.rs`.

## Key ideas

- The banner shows during `RoundPhase::Starting`. Gameplay is frozen during this phase.
- The text is drawn with `spawn_round_label`, on the overlay camera (render layer 3). The Text plugin draws the letters and the Camera plugin owns the camera.
- Each number shows for one second. The intro lasts `round.intro_count` seconds (3).
- "GO!" grows fast to `round.go_end_scale` (24) over `round.go_linger_ms` (500 ms), so it looks like it flies past the players. The game is already in `Playing` while it grows.

```mermaid
---
config:
  theme: dark
---

flowchart TD

enter["`**Enter Starting**
shows 3`"]
two["`shows 2`"]
one["`shows 1`"]
go["`shows GO!
sets Playing`"]
gone["`GO! despawned`"]

enter --> |1 s| two
two --> |1 s| one
one --> |1 s| go
go --> |"500 ms tween"| gone
```

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph enter_starting ["OnEnter(Starting)"]
  begin_intro_countdown["`**begin_intro_countdown**`"]
end

subgraph update ["Update, no set"]
  advance_intro_countdown["`**advance_intro_countdown**
  only during Starting`"]
  despawn_go_banner["`**despawn_go_banner**
  every phase`"]
end

enter_starting ~~~ update
```

- `advance_intro_countdown` writes `NextState<RoundPhase>`. It is in `RoundPhaseWriter`, like the other systems that set the phase.
- `despawn_go_banner` runs in every phase. "GO!" ends during `Playing`, so a `Starting` gate would leave it on screen.

## Plugin Systems

### Begin Intro Countdown

- Runs on entering `Starting`.
- Shows the first number (`round.intro_count`) with the `CountdownNumber` marker.
- Inserts `IntroCountdown`: a one-second repeating timer and the current number.

### Advance Intro Countdown

- Runs only during `Starting`. Each second, despawns the current number and removes 1 from the count.
- While the count is above 0, shows the next number.
- At 0, shows "GO!" with `GoBanner` and a scale `TweenAnim` (`ExponentialIn`). Then removes `IntroCountdown` and sets the next phase to `Playing`.

### Despawn Go Banner

- Reads `AnimCompletedEvent`. When the tween of a `GoBanner` ends, despawns it with its letters.

## Components, Resources and Messages CRUD

### Read FontAtlas and GameConfig

Used in the following systems:
- **begin_intro_countdown**: reads the font and `round.intro_count`.
- **advance_intro_countdown**: reads the font, `round.go_linger_ms` and `round.go_end_scale`.

### IntroCountdown resource

Used in the following systems:
- **begin_intro_countdown**: inserts it.
- **advance_intro_countdown**: ticks the timer, changes `remaining`, and removes the resource at the end.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_starting(("`OnEnter(Starting)`")):::system-group
update(("`Update`")):::system-group
begin_intro_countdown["`**begin_intro_countdown**`"]
advance_intro_countdown["`**advance_intro_countdown**`"]

enter_starting -.-> begin_intro_countdown
update -.-> advance_intro_countdown

world@{ shape: st-rect, label: "World" }
intro_res@{ shape: doc, label: "IntroCountdown" }

ic_timer>"`**timer**`"]
ic_remaining>"`**remaining**`"]

intro_res --> |belongs to| world
ic_timer --> |field of| intro_res
ic_remaining --> |field of| intro_res

begin_intro_countdown ---> |inserts resource| intro_res
advance_intro_countdown ---> |ticks| ic_timer
advance_intro_countdown ---> |writes| ic_remaining
advance_intro_countdown ---> |removes resource| intro_res
```

### Spawn banner labels

Used in the following systems:
- **begin_intro_countdown**: spawns the first number with `CountdownNumber`.
- **advance_intro_countdown**: spawns the next numbers with `CountdownNumber`, then "GO!" with `GoBanner` and `TweenAnim`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_starting(("`OnEnter(Starting)`")):::system-group
update(("`Update`")):::system-group
begin_intro_countdown["`**begin_intro_countdown**`"]
advance_intro_countdown["`**advance_intro_countdown**`"]

enter_starting -.-> begin_intro_countdown
update -.-> advance_intro_countdown

number_entity@{ shape: st-rect, label: "Number Label Entity" }
go_entity@{ shape: st-rect, label: "GO! Label Entity" }

nl_marker>"`**CountdownNumber**`"] --> |inserted on| number_entity
go_marker>"`**GoBanner**`"] --> |inserted on| go_entity
go_tween>"`**TweenAnim**`"] --> |inserted on| go_entity

begin_intro_countdown ---> |spawns| number_entity
advance_intro_countdown ---> |spawns| number_entity
advance_intro_countdown ---> |spawns| go_entity
```

### Query CountdownNumber entities

Used in the following systems:
- **advance_intro_countdown**: despawns the number that is on screen.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
advance_intro_countdown["`**advance_intro_countdown**`"]

update -.-> advance_intro_countdown

numbers_query{{"`numbers`"}}:::query
advance_intro_countdown ---> numbers_query

number_entity@{ shape: st-rect, label: "Number Label Entity" }

nl_entity>"`**Entity**`"] --> |belongs to| number_entity
nl_marker>"`**CountdownNumber**`"] --> |belongs to| number_entity

numbers_query ---> |reads| nl_entity
numbers_query -..-> |filter With| nl_marker
advance_intro_countdown ---> |despawns| number_entity
```

### Read AnimCompletedEvent messages

Used in the following systems:
- **despawn_go_banner**: finds the end of the "GO!" tween.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
despawn_go_banner["`**despawn_go_banner**`"]

update -.-> despawn_go_banner

message_reader{{"MessageReader#60;AnimCompletedEvent#62;"}}:::reader
despawn_go_banner ---> message_reader

anim_completed_message(["`**AnimCompletedEvent**`"])

message_reader ---> |reads| anim_completed_message
```

### Query GoBanner entities

Used in the following systems:
- **despawn_go_banner**: despawns the "GO!" label whose tween ended.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
despawn_go_banner["`**despawn_go_banner**`"]

update -.-> despawn_go_banner

go_banners_query{{"`go_banners`"}}:::query
despawn_go_banner ---> go_banners_query

go_entity@{ shape: st-rect, label: "GO! Label Entity" }

go_id>"`**Entity**`"] --> |belongs to| go_entity
go_marker>"`**GoBanner**`"] --> |belongs to| go_entity

go_banners_query ---> |reads| go_id
go_banners_query -..-> |filter With| go_marker
despawn_go_banner ---> |despawns| go_entity
```

### Set NextState RoundPhase

Used in the following systems:
- **advance_intro_countdown**: sets `Playing` when "GO!" shows.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
advance_intro_countdown["`**advance_intro_countdown**`"]

update -.-> advance_intro_countdown

next_state_res@{ shape: doc, label: "NextState<RoundPhase>" }

advance_intro_countdown ---> |sets Playing| next_state_res
```

Definitions:
- `CountdownNumber`: private marker `Component` on the number label on screen.
- `GoBanner`: private marker `Component` on the "GO!" label.
- `IntroCountdown`: private `Resource` with a one-second `timer` and `remaining`, the number on screen. 0 means "show GO!".
