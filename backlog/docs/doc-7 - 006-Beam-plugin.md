---
id: doc-7
title: '[006] Beam plugin'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-10-17 12:00'
---
# Beam Plugin

This plugin spawns a beam when a player shoots, moves it one tile at a time, and ends it on the tile it claims. It also handles parries: a parried beam turns around, changes owner and gets faster. The Claim plugin then changes the tile owner.

## Key ideas

- A beam is an invisible logic object. The tiles bounce under it to show where it is.
- `caster` is the player who fired the beam. `owner` is the player it claims for. A parry changes only `owner`.
- A charge is spent when a beam spawns. A blocked shot and a parry cost nothing.
- `InFlightBeamCount` is the number of beams that a player has on the board right now (see the lifecycle below).

## Beam abilities

- `Lance`: the player can shoot from a claimed tile. The beam goes over claimed tiles and claims the first free one.
- `BorderGrinder`: a `Straight` beam that hits an enemy tile claims that enemy tile.

## Ordering

- `spawn_beam`, `resolve_parry` and `beam_step` run in this order, in `GameplaySet::Beam`, only during `Playing`.
- The Claim stage runs after this one, so a tile claimed in this frame is already visible later in the frame.

## Plugin Systems

### Spawn Beam

- Runs for each fire message. `resolve_fire` picks the beam type from the tile the player stands on:
- The player stands on a free tile: the beam is a `Straight` beam.
- The player stands on a claimed tile and has the `Lance` ability: the beam is a `Lance` beam.
- The player stands on a claimed tile without the `Lance` ability: no beam, and no charge is spent.
- A `Straight` beam goes forward until a claimed tile or the map edge. It claims the last free tile before it stops.
- A `Lance` beam goes over claimed tiles. It claims the first free tile it finds, or nothing if it reaches the edge.
- The Input plugin does the same check, but the tile can change before this system runs, so it checks again.
- When a beam spawns: one charge is spent, `InFlightBeamCount` goes up, and `ChargeSpent` is written.
- The beam starts with zero parries and its own step timer. It takes its first step on its spawn frame.
- It gets `WaveSource` and `BounceEffect` (the tile bounce), unless its caster already has a beam in the same lane.
- `WaveSource` makes each tile bounce when the beam steps on it. This bounce is the wave that players see.
- Same lane means the same row for horizontal beams, or the same column for vertical ones. The glow shows for every beam.

### Resolve Parry

- Reads `ParryTriggered`. The Input plugin writes it when a player shoots while facing a beam that is about to hit them.
- "About to hit" means: in the first `window_fraction` of the beam's step, and the beam can claim the player's tile.
- The beam turns around, its `owner` becomes the parrier, and its parry count goes up by one.
- Each parry divides the step time by `speed_multiplier`, but never below `min_step_secs`.
- Its timer restarts, ready to step. Then `BeamParried` is written for the Effects and Hitstop plugins.
- Only the first parry of a beam in a frame counts. A player cannot parry a beam they already own.

### Beam Step

- Ticks the timer of each beam. A beam moves one tile each time its timer ends.
- A beam always ends through `end_beam`: it lowers the caster's `InFlightBeamCount`, then despawns the beam.
- `end_beam` is a small helper function. Every case where a beam stops calls it, so the count always stays right.

**Straight**

- Next tile is off the map: step back out of forbidden tiles, then claim the current tile if it is free.
- Next tile is claimed: same as above. But if the caster has `BorderGrinder` and the tile is an enemy's, claim that tile.
- Otherwise: move one tile.

**Lance**

- Next tile is off the map (not ground, not forbidden): the beam ends and claims nothing.
- Next tile is free ground: claim it, then end.
- Otherwise (claimed or forbidden tile): move one tile.

### Resync Beam Step Timers (dev only)

- Only in dev builds. When the config reloads, it gives every live beam its new step time.
- Runs before `beam_step`, because both change `Beam`.

## Components, Resources and Messages CRUD

### Read BeamFired messages

Used in the following systems:
- **spawn_beam**: one shot for each message.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

message_reader{{"MessageReader#60;BeamFired#62;"}}:::reader
spawn_beam ---> message_reader

beam_fired_message(["`**BeamFired**`"])

message_reader ---> |reads| beam_fired_message
```

### Query Beam entities (spawn)

Used in the following systems:
- **spawn_beam**: checks the caster's live beams for one in the same lane.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

beams_query{{"`beams_query`"}}:::query
spawn_beam ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }

be_beam>"`**Beam**`"] --> |belongs to| beam_entity
be_grid_coords>"`**GridCoords**`"] --> |belongs to| beam_entity

beams_query ---> |reads| be_beam
beams_query ---> |reads| be_grid_coords
```

### Read behavior-selection inputs (spawn)

Used in the following systems:
- **spawn_beam**: checks the tile under the player and the `Lance` ability.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

ability_query{{"Query#60;#38;AbilityList#62;"}}:::query
claimed_query{{"Query#60;#38;ClaimedTile#62;"}}:::query
spawn_beam ---> ability_query
spawn_beam ---> claimed_query

player_entity@{ shape: st-rect, label: "Player (owner)" }
pe_ability>"`**AbilityList**`"] --> |belongs to| player_entity
ability_query ---> |reads| pe_ability

tile_entity@{ shape: st-rect, label: "Origin tile" }
te_claimed>"`**ClaimedTile**`"] --> |belongs to| tile_entity
claimed_query ---> |reads `owner`| te_claimed

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }
map_info_res --> |belongs to| world
spawn_beam ---> |reads `claimed_entities`| map_info_res
```

### Read MapInfo resource (beam step)

Used in the following systems:
- **beam_step**: checks the next tile: ground, forbidden or off the map.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

beam_step ---> |reads `on_ground`| map_info_res
beam_step ---> |reads `claimed_entities`| map_info_res
beam_step ---> |reads `on_forbidden_areas`| map_info_res
```

### Write commands: spawn Beam entity

Used in the following systems:
- **spawn_beam**: the bounce components are left out when the lane is already used.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

beam_entity@{ shape: st-rect, label: "Beam (spawned)" }

be_grid_coords>"`**GridCoords**`"]
be_beam>"`**Beam**`"]
be_wave_source>"`**WaveSource**`"]
be_bounce>"`**BounceEffect**`"]

be_grid_coords --> |spawned on| beam_entity
be_beam --> |spawned on| beam_entity
be_wave_source --> |"spawned on (unless lane-suppressed)"| beam_entity
be_bounce --> |"spawned on (unless lane-suppressed)"| beam_entity

spawn_beam ---> |spawns entity with| be_grid_coords
spawn_beam ---> |spawns entity with| be_beam
spawn_beam ---> |spawns entity with| be_wave_source
spawn_beam ---> |spawns entity with| be_bounce
```

### Query Beam entities

Used in the following systems:
- **beam_step**: all live beams. It ticks their timers and moves them.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

beams_query{{"`beams_query`"}}:::query
beam_step ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }

be_entity>"`**Entity**`"] --> |belongs to| beam_entity
be_beam>"`**Beam**`"] --> |belongs to| beam_entity
be_grid_coords>"`**GridCoords**`"] --> |belongs to| beam_entity

beams_query ---> |reads| be_entity
beams_query ---> |"reads, ticks step_timer"| be_beam
beams_query ---> |writes| be_grid_coords
```

### Query ClaimedTile (beam step)

Used in the following systems:
- **beam_step**: checks if the next tile has an owner.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

claimed_query{{"`claimed_query`"}}:::query
beam_step ---> claimed_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_claimed>"`**ClaimedTile**`"] --> |belongs to| claimed_tile_entity
ct_owner>"`**owner**`"] --> |field of| ct_claimed

claimed_query ---> |reads| ct_claimed
```

### Write BeamResolved messages

Used in the following systems:
- **beam_step**: one message for each claimed tile.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

beam_resolved_message(["`**BeamResolved**`"])

beam_step ---> |writes| beam_resolved_message
```

### Write commands: despawn Beam entity

Used in the following systems:
- **beam_step**: through `end_beam`, which also lowers the caster's in-flight count.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

beam_entity@{ shape: st-rect, label: "Beam" }

beam_step ---> |despawns via end_beam| beam_entity
```

### Query BeamCharges and InFlightBeamCount (spawn_beam)

Used in the following systems:
- **spawn_beam**: one charge less and one in-flight beam more, for each beam spawned.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

owner_state_query{{"`owner_state`"}}:::query
spawn_beam ---> owner_state_query

player_entity@{ shape: st-rect, label: "Player (owner)" }

pe_charges>"`**BeamCharges**`"] --> |belongs to| player_entity
pe_current>"`**current**`"] --> |field of| pe_charges
pe_in_flight>"`**InFlightBeamCount**`"] --> |belongs to| player_entity
pe_in_flight_current>"`**current**`"] --> |field of| pe_in_flight

owner_state_query ---> |"writes (decrements)"| pe_current
owner_state_query ---> |"writes (increments)"| pe_in_flight_current
```

### Write ChargeSpent messages

Used in the following systems:
- **spawn_beam**: one message for each beam spawned. No plugin reads it yet.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
spawn_beam["`**spawn_beam**`"]

update -.-> spawn_beam

charge_spent_message(["`**ChargeSpent**`"])

spawn_beam ---> |writes| charge_spent_message
```

### Read AbilityList (beam step)

Used in the following systems:
- **beam_step**: checks if the caster has `BorderGrinder`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
beam_step["`**beam_step**`"]

update -.-> beam_step

ability_query{{"Query#60;#38;AbilityList#62;"}}:::query
beam_step ---> ability_query

caster_entity@{ shape: st-rect, label: "Player (caster)" }
ce_ability>"`**AbilityList**`"] --> |belongs to| caster_entity
ability_query ---> |reads| ce_ability
```

### Read ParryTriggered, write Beam and BeamParried (resolve_parry)

Used in the following systems:
- **resolve_parry**: turns the parried beam around, gives it to the parrier, and writes `BeamParried`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
resolve_parry["`**resolve_parry**`"]

update -.-> resolve_parry

message_reader{{"MessageReader#60;ParryTriggered#62;"}}:::reader
resolve_parry ---> message_reader
parry_triggered_message(["`**ParryTriggered**`"])
message_reader ---> |reads| parry_triggered_message

beams_query{{"`beams`"}}:::query
resolve_parry ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }
be_beam>"`**Beam**`"] --> |belongs to| beam_entity
beams_query ---> |"writes direction, owner, parries, step_timer"| be_beam

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }
config_res --> |belongs to| world
resolve_parry ---> |reads `parry` + `timing`| config_res

beam_parried_message(["`**BeamParried**`"])
resolve_parry ---> |writes| beam_parried_message
```

### Write Beam step timers (resync_beam_step_timers, dev only)

Used in the following systems:
- **resync_beam_step_timers**: sets the new step time on every live beam when the config changes.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
resync_beam_step_timers["`**resync_beam_step_timers**`"]

update -.-> resync_beam_step_timers

beams_query{{"`beams`"}}:::query
resync_beam_step_timers ---> beams_query

beam_entity@{ shape: st-rect, label: "Beam" }
be_beam>"`**Beam**`"] --> |belongs to| beam_entity
beams_query ---> |"writes step_timer duration"| be_beam

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }
config_res --> |belongs to| world
resync_beam_step_timers ---> |"reads (only when changed)"| config_res
```

### InFlightBeamCount component lifecycle

- The number of beams that a player has on the board right now.
- Bevy applies the spawn or despawn of a `Beam` entity later. So counting `Beam` entities can give an old number.
- This counter changes in the same system that spawns or ends the beam. Every system that reads it after that gets the right number.
- **Inserted** at 0 by `initialize_players` (Maps plugin).
- **Increased** by `spawn_beam` when a beam spawns. **Decreased** by `beam_step` (through `end_beam`) on the caster.
- **Reset** to 0 by `reset_round` (Round plugin).
- **Read** by `animate_charges_bar` (HUD), so a beam still in flight counts in the charges bar.
- **Read** by `resolve_charge_exhaustion` (Round), to wait until no beam is in flight before ending the round.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_players["`**initialize_players** (Maps)`"]
spawn_beam["`**spawn_beam**`"]
beam_step["`**beam_step**`"]
animate_charges_bar["`**animate_charges_bar** (HUD)`"]
resolve_charge_exhaustion["`**resolve_charge_exhaustion** (Round)`"]
reset_round["`**reset_round** (Round)`"]

update -.-> initialize_players
update -.-> spawn_beam
update -.-> beam_step
update -.-> animate_charges_bar
update -.-> resolve_charge_exhaustion

player_entity@{ shape: st-rect, label: "Player Entity" }
in_flight_component@{ shape: doc, label: "InFlightBeamCount" }

initialize_players ---> |inserts, default 0| in_flight_component
spawn_beam ---> |increments| in_flight_component
beam_step ---> |"decrements (via end_beam)"| in_flight_component
reset_round ---> |resets to 0| in_flight_component
in_flight_component --> |belongs to| player_entity
animate_charges_bar -..-> |reads| in_flight_component
resolve_charge_exhaustion -..-> |reads| in_flight_component
```
