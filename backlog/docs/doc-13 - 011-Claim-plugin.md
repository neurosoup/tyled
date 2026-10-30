---
id: doc-13
title: '[011] Claim plugin'
type: other
created_date: '2026-07-12 12:00'
updated_date: '2026-10-30 12:00'
---
# Claim Plugin

This plugin changes the owner of a tile when a beam ends on it. It is the only place in the code that writes `ClaimedTile::owner`.

## Key ideas

- The Beam plugin writes `BeamResolved` when a beam ends on a tile. This plugin reads it. It never reads `Beam` entities.
- A **flip** is a claim that changes the owner. Claiming a tile that the player already owns is not a flip.
- Each player has a `ClaimedTileCount`: the number of tiles they own. This plugin keeps it correct after each flip.
- `TileClaimed` is written for every claim, flip or not. No system reads it yet. Future abilities will use it.

## Ordering

- `claim_tile` runs in `GameplaySet::Claim`, only during `Playing`.
- `GameplaySet::Claim` runs after `GameplaySet::Beam`. So a beam that ends in a frame claims its tile in the same frame.
- A new system that writes `BeamResolved` must also run only during `Playing`, before `GameplaySet::Claim`. Otherwise its message can be lost.

## Plugin Systems

### Claim Tile

- Reads `BeamResolved`. Finds the tile entity at the message position with `MapInfo::claimed_entities`.
- Sets `ClaimedTile::owner` to the player of the message.
- On a flip, adds 1 to the new owner's `ClaimedTileCount` and removes 1 from the old owner's count, if there was an old owner.
- Writes `TileClaimed` with the position, the old owner and the new owner.
- The Animations plugin sees the changed `ClaimedTile` and plays the tile flip.

## Components, Resources and Messages CRUD

### Read BeamResolved messages

Used in the following systems:
- **claim_tile**: each message is one claim.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
claim_tile["`**claim_tile**`"]

update -.-> claim_tile

message_reader{{"MessageReader#60;BeamResolved#62;"}}:::reader
claim_tile ---> message_reader

beam_resolved_message(["`**BeamResolved**`"])

message_reader ---> |reads| beam_resolved_message
```

### Read MapInfo resource (claim tile)

Used in the following systems:
- **claim_tile**: finds the tile entity at a position.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
claim_tile["`**claim_tile**`"]

update -.-> claim_tile

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_res --> |belongs to| world

claim_tile ---> |reads `claimed_entities`| map_info_res
```

### Write ClaimedTile (claim tile)

Used in the following systems:
- **claim_tile**: sets the new owner.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
claim_tile["`**claim_tile**`"]

update -.-> claim_tile

claimed_tiles_query{{"`claimed_tiles_query (mutable)`"}}:::query
claim_tile ---> claimed_tiles_query

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile Entity" }

ct_claimed>"`**ClaimedTile**`"] --> |belongs to| claimed_tile_entity
ct_owner>"`**owner**`"] --> |field of| ct_claimed

claimed_tiles_query ---> |writes| ct_owner
```

### Write TileClaimed messages

Used in the following systems:
- **claim_tile**: writes one message per claim. No system reads it yet.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
claim_tile["`**claim_tile**`"]

update -.-> claim_tile

tile_claimed_message(["`**TileClaimed**`"])

claim_tile ---> |writes| tile_claimed_message
```

### Write ClaimedTileCount (claim tile)

Used in the following systems:
- **claim_tile**: on a flip, adds 1 for the new owner and removes 1 for the old owner.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
claim_tile["`**claim_tile**`"]

update -.-> claim_tile

counts_query{{"`counts (mutable)`"}}:::query
claim_tile ---> counts_query

player_entity@{ shape: st-rect, label: "Player Entity" }

ctc_count>"`**ClaimedTileCount**`"] --> |belongs to| player_entity
ctc_current>"`**current**`"] --> |field of| ctc_count

counts_query ---> |writes| ctc_current
```
