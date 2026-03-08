---
id: doc-9
title: '[000] Plugin message relationships'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-03-08 17:04'
---
# Plugin Message Relationships

This document summarises how the game's plugins are connected to each other through the message-passing system. Messages are the only coupling point between plugins — a plugin never calls into another plugin directly. Each message is written by one system and consumed by one or more systems in other plugins.

There are two categories of messages in this codebase:

- **Tiled events** (`TiledEvent<MapCreated>`, `TiledEvent<ObjectCreated>`) — emitted by the external `TiledPlugin` and consumed by the Maps and Animations plugins respectively to react to map and object loading completion.
- **Game messages** (`PlayerMoved`, `BeamFired`, `BeamResolved`) — defined in the Messages plugin and exchanged between the Input, Movements, Beam and Claim plugins to drive gameplay logic.

The diagram below shows every plugin as a node, every message type as a distinct node, and the write/read relationships as directed edges. The flow generally moves from left to right: external events bootstrap the world, player input drives movement and combat, and beam collisions trigger tile ownership changes.

```mermaid
---
config:
  theme: dark
---

flowchart LR
classDef system-group stroke-dasharray: 5 5
classDef external stroke-dasharray: 8 2

tiled_plugin(["`**TiledPlugin**`"]):::external

maps_plugin["`**Maps Plugin**`"]:::system-group
input_plugin["`**Input Plugin**`"]:::system-group
movements_plugin["`**Movements Plugin**`"]:::system-group
animations_plugin["`**Animations Plugin**`"]:::system-group
beam_plugin["`**Beam Plugin**`"]:::system-group
claim_plugin["`**Claim Plugin**`"]:::system-group

map_created_message(["`**TiledEvent#60;MapCreated#62;**`"])
object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])
player_moved_message(["`**PlayerMoved**`"])
beam_fired_message(["`**BeamFired**`"])
beam_resolved_message(["`**BeamResolved**`"])

tiled_plugin ---> |writes| map_created_message
tiled_plugin ---> |writes| object_created_message

map_created_message ---> |reads| maps_plugin
object_created_message ---> |reads| animations_plugin

input_plugin ---> |writes| player_moved_message
input_plugin ---> |writes| beam_fired_message

player_moved_message ---> |reads| movements_plugin
beam_fired_message ---> |reads| beam_plugin

beam_plugin ---> |writes| beam_resolved_message
beam_resolved_message ---> |reads| claim_plugin