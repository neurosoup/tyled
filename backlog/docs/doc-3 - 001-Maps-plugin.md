---
id: doc-3
title: '[001] Maps plugin'
type: other
created_date: '2026-02-01 16:02'
updated_date: '2026-03-08 17:04'
---
# Maps Plugin

Contains system related to map loading and entity-related initializations. This plugin also initialize the MapInfo resource to give worldwide access to specific tiles lookup or map related information.

## Plugin workflow

- Startup phase
    - Load Map spawns the Tilemap entity with TiledMap + TilemapAnchor.
    - The TiledPlugin later emits TiledEvent<MapCreated>.
- Update phase
    - On Map Created System:
        - Reacts to MapCreated message
            - Reads:
                - Tilemap metadata components
                - All Ground tiles (Entity, TilePos)
                - All Player tiled objects
            - Writes:
                - Initializes MapInfo resource
                - Inserts GridCoords, LookDirection, TweenAnim on player entities
                - Inserts Anchor on player child sprite entities

## Plugin Systems

### Load Map

Loads the `level0.tmx` Tile Project File that represents a temporary test level.

### Initialize Map Info

Store map informations to be used accros all systems. Initialize several tile lookups, in particular the `ground_entities` HashMap that links `TilePos` to `Ground` tile entities, as well as map geometry fields (`map_size`, `grid_size`, `tile_size`, `map_type`, `map_anchor`) read from the Tilemap entity.

### Initialize Players

Reacts to the `MapCreated` event and initializes all player entities spawned by the Tiled loader. For each `Player`-marked entity, it computes the initial `GridCoords` from the entity world-space `Transform` using the `MapInfo` resource, derives the starting `LookDirection` from the player id, and inserts a `TweenAnim` for movement interpolation. It also inserts an `Anchor` component on the first child entity of each player (the sprite entity) to properly anchor the sprite.

## Components, Resources and Messages CRUD

### Read TiledEvent MapCreated messages

Used in the folowing systems:
- **initialize_map_info**: used to trigger the system
- **initialize_players**: used to trigger the system

```mermaid
---
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

### Query Tilemap metadata

Used in the folowing systems:
- **initialize_map_info** : used to get various map informations (e.g. map size, tile size, etc.)

```mermaid
---
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

### Query TilePos of Ground tiles

Used in the following systems:
- **initialize_map_info** : used to get all `Entity` and `TilePos` of `Ground`-marked tile entities spawned after loading the map

```mermaid
---
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

gt_entity>"`**Entity**`"] --> |belongs to| ground_entity
gt_tile_pos>"`**TilePos**`"] --> |belongs to| ground_entity
gt_ground>"`**Ground**`"] --> |belongs to| ground_entity

ground_tiles_query ---> |reads| gt_entity
ground_tiles_query ---> |reads| gt_tile_pos
ground_tiles_query -..-> |filter With| gt_ground
```

### Query All Player tiled objects

Used in the following systems:
- **initialize_players**: used to get all `Entity`, `Player::player_id` and `Transform` of `Player`-marked entities spawned after loading the map

```mermaid
---
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

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_transform>"`**Transform**`"] --> |belongs to| player_entity
pe_tiled_object>"`**TiledObject**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query ---> |reads| pe_transform
players_query -..-> |filter With| pe_tiled_object
```

### Write MapInfo resource

Used in systems:
- **initialize_map_info**: writes the `MapInfo` resource, including the `ground_entities` HashMap that links `TilePos` to `Ground` tile entities, and all map geometry fields

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_map_info["`**initialize_map_info**`"]

update -.-> initialize_map_info

world@{ shape: st-rect, label: "World" }
map_info_res@{ shape: doc, label: "MapInfo" }

map_info_ground>"`**ground_entities**`"]
map_info_map_size>"`**map_size**`"]
map_info_grid_size>"`**grid_size**`"]
map_info_tile_size>"`**tile_size**`"]
map_info_map_type>"`**map_type**`"]
map_info_map_anchor>"`**map_anchor**`"]

map_info_res --> |belongs to| world
map_info_ground --> |field of| map_info_res
map_info_map_size --> |field of| map_info_res
map_info_grid_size --> |field of| map_info_res
map_info_tile_size --> |field of| map_info_res
map_info_map_type --> |field of| map_info_res
map_info_map_anchor --> |field of| map_info_res

initialize_map_info ---> |writes| map_info_res
```

### Write commands

Used in systems:
- **initialize_players**: inserts `GridCoords`, `LookDirection` and `TweenAnim` on each `Player` entity, and inserts `Anchor` on the first child sprite entity

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_players["`**initialize_players**`"]

update -.-> initialize_players

player_entity@{ shape: st-rect, label: "Player (TiledObject)" }
child_entity@{ shape: st-rect, label: "Player Child (Sprite)" }

pe_grid_coords>"`**GridCoords**`"]
pe_look_direction>"`**LookDirection**`"]
pe_tween>"`**TweenAnim**`"]
ce_anchor>"`**Anchor**`"]

pe_grid_coords --> |inserted on| player_entity
pe_look_direction --> |inserted on| player_entity
pe_tween --> |inserted on| player_entity
ce_anchor --> |inserted on| child_entity

initialize_players ---> |inserts component| pe_grid_coords
initialize_players ---> |inserts component| pe_look_direction
initialize_players ---> |inserts component| pe_tween
initialize_players ---> |inserts component| ce_anchor
```
