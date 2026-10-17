---
id: doc-9
title: '[000] Plugin message relationships'
type: other
created_date: '2026-03-08 17:04'
updated_date: '2026-10-17 12:00'
---
# Plugin Message Relationships

Plugins talk to each other only through messages. A plugin never calls another plugin directly. The diagram below shows which plugin writes each message and which plugins read it.

For the links through game states, see [doc-24 Plugin state relationships](/documentation/24/000-plugin-state-relationships).

## Message kinds

- **Tiled events**: sent by the Tiled library when a map or its objects finish loading.
- **Game messages**: declared in `messages.rs`. They carry gameplay events between plugins.
- **Tween events**: sent by `bevy_tweening` when an animation ends.

## Notes

- The Round plugin has three nodes, one per submodule (`state`, `intro`, `outcome`).
- Maps and Round (outcome) have no edges. They react to state changes, not to messages.
- `EffectRequest` stays inside the Effects plugin, so the diagram does not show it.
- `TileClaimed`, `ChargeSpent` and `ChargeRegen` have no readers yet. They are hooks for future beam abilities.
- If one entity gets several `KnockbackRequested` in the same frame, only the last one applies.

## Reading the diagram

The flow goes from left to right. Map loading sets up the world. Player input drives movement and beams. Beams claim tiles, claimed tiles deal damage, and visual effects react at the end.

```mermaid
---
config:
  theme: dark
---

flowchart LR
classDef system-group stroke-dasharray: 5 5
classDef external stroke-dasharray: 8 2

tiled_plugin(["`**TiledPlugin**`"]):::external
tweening_lib(["`**bevy_tweening**`"]):::external

maps_plugin["`**Maps Plugin**`"]:::system-group
camera_plugin["`**Camera Plugin**`"]:::system-group
round_state_plugin["`**Round Plugin (state)**`"]:::system-group
round_intro_plugin["`**Round Plugin (intro)**`"]:::system-group
round_outcome_plugin["`**Round Plugin (outcome)**`"]:::system-group
input_plugin["`**Input Plugin**`"]:::system-group
controller_plugin["`**Controller Plugin**`"]:::system-group
animations_plugin["`**Animations Plugin**`"]:::system-group
beam_plugin["`**Beam Plugin**`"]:::system-group
claim_plugin["`**Claim Plugin**`"]:::system-group
charge_plugin["`**Charge Plugin**`"]:::system-group
damage_plugin["`**Damage Plugin**`"]:::system-group
effects_plugin["`**Effects Plugin**`"]:::system-group
hud_plugin["`**HUD Plugin**`"]:::system-group

map_created_message(["`**TiledEvent#60;MapCreated#62;**`"])
object_created_message(["`**TiledEvent#60;ObjectCreated#62;**`"])
entity_moved_message(["`**EntityMoved**`"])
beam_fired_message(["`**BeamFired**`"])
beam_resolved_message(["`**BeamResolved**`"])
tile_claimed_message(["`**TileClaimed**`"])
charge_spent_message(["`**ChargeSpent**`"])
charge_regen_message(["`**ChargeRegen**`"])
damageable_died_message(["`**DamageableDied**`"])
movement_stopped_message(["`**MovementStopped**`"])
knockback_requested_message(["`**KnockbackRequested**`"])
entity_knocked_back_message(["`**EntityKnockedBack**`"])
anim_completed_message(["`**AnimCompletedEvent**`"])

tiled_plugin ---> |writes| map_created_message
tiled_plugin ---> |writes| object_created_message

map_created_message ---> |read by| camera_plugin
map_created_message ---> |read by| round_state_plugin

object_created_message ---> |read by| animations_plugin
object_created_message ---> |read by| hud_plugin

input_plugin ---> |writes| entity_moved_message
input_plugin ---> |writes| beam_fired_message
input_plugin ---> |writes| movement_stopped_message

movement_stopped_message ---> |read by| effects_plugin

entity_moved_message ---> |read by| controller_plugin

beam_fired_message ---> |read by| beam_plugin

beam_plugin ---> |writes| beam_resolved_message
beam_plugin ---> |writes| charge_spent_message

beam_resolved_message ---> |read by| claim_plugin
beam_resolved_message ---> |read by| animations_plugin

claim_plugin ---> |writes| tile_claimed_message

charge_plugin ---> |writes| charge_regen_message

damage_plugin ---> |writes| damageable_died_message
damage_plugin ---> |writes| knockback_requested_message
damage_plugin ---> |writes| entity_knocked_back_message

controller_plugin ---> |writes| knockback_requested_message

knockback_requested_message ---> |read by| damage_plugin
entity_knocked_back_message ---> |read by| effects_plugin

damageable_died_message ---> |read by| effects_plugin
damageable_died_message ---> |read by| round_state_plugin

tweening_lib ---> |writes| anim_completed_message

anim_completed_message ---> |read by| effects_plugin
anim_completed_message ---> |read by| round_intro_plugin
