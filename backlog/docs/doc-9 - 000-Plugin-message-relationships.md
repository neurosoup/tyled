---
id: doc-9
title: '[000] Plugin message relationships'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-06-15 12:00'
---
# Plugin Message Relationships

This document summarises how the game's plugins are connected to each other through the message-passing system. Messages are the only coupling point between plugins — a plugin never calls into another plugin directly. Each message is written by one system and consumed by one or more systems in other plugins.

There are two categories of messages in this codebase:

- **Tiled events** (`TiledEvent<MapCreated>`, `TiledEvent<ObjectCreated>`) — emitted by the external `TiledPlugin` and consumed by the Maps, Camera, and Animations plugins respectively to react to map and object loading completion.
- **Game messages** (`EntityMoved`, `BeamFired`, `BeamResolved`, `DamageableDied`) — defined in the Messages plugin and exchanged between plugins to drive gameplay logic.

The diagram below shows every plugin as a node, every message type as a distinct node, and the write/read relationships as directed edges. The flow generally moves from left to right: external events bootstrap the world, player input drives movement and combat, beam collisions trigger tile ownership changes, damage accumulates on claimed tiles, and visual effects react to the resulting state changes.

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
camera_plugin["`**Camera Plugin**`"]:::system-group
input_plugin["`**Input Plugin**`"]:::system-group
controller_plugin["`**Controller Plugin**`"]:::system-group
animations_plugin["`**Animations Plugin**`"]:::system-group
beam_plugin["`**Beam Plugin**`"]:::system-group
damage_plugin["`**Damage Plugin**`"]:::system-group
effects_plugin["`**Effects Plugin**`"]:::system-group
hud_plugin["`**HUD Plugin**`"]:::system-group

map_created_message(["`**TiledEvent#60;MapCreated#62;**`"])
object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])
entity_moved_message(["`**EntityMoved**`"])
beam_fired_message(["`**BeamFired**`"])
beam_resolved_message(["`**BeamResolved**`"])
damageable_died_message(["`**DamageableDied**`"])

tiled_plugin ---> |writes| map_created_message
tiled_plugin ---> |writes| object_created_message

map_created_message ---> |reads| maps_plugin
map_created_message ---> |reads| camera_plugin

object_created_message ---> |reads| animations_plugin

input_plugin ---> |writes| entity_moved_message
input_plugin ---> |writes| beam_fired_message

entity_moved_message ---> |reads| controller_plugin

beam_fired_message ---> |reads| beam_plugin

beam_plugin ---> |writes| beam_resolved_message

beam_resolved_message ---> |reads claim_tile| beam_plugin
beam_resolved_message ---> |reads| animations_plugin

damage_plugin ---> |writes| damageable_died_message

damageable_died_message ---> |reads| effects_plugin