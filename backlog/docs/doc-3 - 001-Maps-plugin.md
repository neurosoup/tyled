---
id: doc-3
title: '[001] Maps plugin'
type: other
created_date: '2026-02-01 16:02'
updated_date: '2026-02-28 11:29'
---
- Startup phase
    - Load Map spawns the Tilemap entity with TiledMap + TilemapAnchor.
    - The TiledPlugin later emits TiledEvent<MapCreated>.
- Update phase
    - On Map Created System:
        - Reacts to MapCreated message
            - Reads:
                - Tilemap metadata components
                - All Ground tiles (TilePos)
                - All Player tiled objects
            - Writes:
                - Initializes MapLookup resource
                - Inserts GridCoords, LookDirection, TweenAnim on player entities



---



```mermaid
---
title: Read TiledEvent MapCreated messages
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_map_created["`**On Map Created**`"]

update -.-> on_map_created

message_reader{{"MessageReader#60;TiledEvent#60;MapCreated#62;#62;"}}:::reader
on_map_created ---> message_reader

map_created_message(["`**TiledEvent#60;MapCreated**#62;`"])

message_reader ---> |reads| map_created_message
```
---

```mermaid
---
title: Query Tilemap metadata  --> MapLookup
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_map_created["`**On Map Created**`"]

update -.-> on_map_created

tilemap_query{{"`tilemap_query`"}}:::query
on_map_created ---> tilemap_query

tilemap_entity@{ shape: st-rect, label: "Tilemap (MapTiles)" }

tm_name>"`**TiledName**`"] --> |belongs to| tilemap_entity
tm_tile_size>"`**TilemapTileSize**`"] --> |belongs to| tilemap_entity
tm_grid_size>"`**TilemapGridSize**`"] --> |belongs to| tilemap_entity
tm_map_size>"`**TilemapSize**`"] --> |belongs to| tilemap_entity
tm_map_type>"`**TilemapType**`"] --> |belongs to| tilemap_entity
tm_anchor>"`**TilemapAnchor**`"] --> |belongs to| tilemap_entity
tm_marker>"`**TiledTilemap**`"] --> |belongs to| tilemap_entity

tilemap_query ---> |reads| tm_name
tilemap_query ---> |reads| tm_tile_size
tilemap_query ---> |reads| tm_grid_size
tilemap_query ---> |reads| tm_map_size
tilemap_query ---> |reads| tm_map_type
tilemap_query ---> |reads| tm_anchor
tilemap_query -..-> |filter With| tm_marker
```
---

```mermaid
---
title: Query TilePos of Ground tiles --> Ground locations lookup in MapLookup
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_map_created["`**On Map Created**`"]

update -.-> on_map_created

ground_tiles_query{{"`ground_tiles_query`"}}:::query
on_map_created ---> ground_tiles_query

ground_entity@{ shape: st-rect, label: "Ground Tile" }

gt_tile_pos>"`**TilePos**`"] --> |belongs to| ground_entity
gt_ground>"`**Ground**`"] --> |belongs to| ground_entity

ground_tiles_query ---> |reads| gt_tile_pos
ground_tiles_query -..-> |filter With| gt_ground
```
---

```mermaid
---
title: Query All Player tiled objects --> Set Player entities GridCoords
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
on_map_created["`**On Map Created**`"]

update -.-> on_map_created

players_query{{"`players_query`"}}:::query
on_map_created ---> players_query

player_entity@{ shape: st-rect, label: "Player (TiledObject)" }

pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_transform>"`**Transform**`"] --> |belongs to| player_entity
pe_tiled_object>"`**TiledObject**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_player
players_query ---> |reads| pe_transform
players_query -..-> |filter With| pe_tiled_object
```
---

```mermaid
---
title: Initializes MapLookup resource and Inserts GridCoords, LookDirection, TweenAnim on player entities
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
on_map_created["`**On Map Created**`"]

update -.-> on_map_created

world@{ shape: st-rect, label: "World" }
map_lookup_res@{ shape: doc, label: "Map Lookup" }

player_entity@{ shape: st-rect, label: "Player (TiledObject)" }

pe_grid_coords>"`**GridCoords**`"]
pe_look_direction>"`**LookDirection**`"]
pe_tween>"`**TweenAnim**`"]

map_lookup_res --> |belongs to| world
pe_grid_coords --> |belongs to| player_entity
pe_look_direction --> |belongs to| player_entity
pe_tween --> |belongs to| player_entity

on_map_created ---> |writes resource| map_lookup_res
on_map_created ---> |inserts component| pe_grid_coords
on_map_created ---> |inserts component| pe_look_direction
on_map_created ---> |inserts component| pe_tween
```
