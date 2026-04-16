---
id: doc-3
title: '[001] Maps plugin'
type: other
created_date: '2026-02-01 16:02'
updated_date: '2026-06-15 12:01'
---
# Maps Plugin

Contains systems related to map loading and entity-related initializations. This plugin also initializes the `MapInfo` resource to give world-wide access to specific tile lookups and map-related information.

## Plugin workflow

- Startup phase
    - `load_maps` spawns two `TiledMap` entities: one for `level0.tmx` (tagged `CurrentLevel`) and one for `hud.tmx` (tagged `HudMap`).
    - The `TiledPlugin` later emits `TiledEvent<MapCreated>` for each loaded map.
- Update phase (chained)
    - `initialize_map_info`:
        - Reacts to `TiledEvent<MapCreated>` for `CurrentLevel` maps only
            - Reads tilemap metadata components, all `Ground` tiles (`Entity`, `TilePos`), and all `ForbiddenArea` tiles
            - Writes the `MapInfo` resource, including `ground_entities`, `claimed_entities`, `forbidden_areas` HashMaps and all map geometry fields
    - Then in parallel (after `initialize_map_info`):
        - `initialize_players`:
            - Reacts to `TiledEvent<MapCreated>` for `CurrentLevel` maps only
            - For each `Player` TiledObject: computes `GridCoords` from `Transform`, inserts `GridCoords`, `LookDirection`, `TranslateEffectTarget`, `DamageEffectTarget`, `Health{current:100,max:100}`
            - Inserts `Anchor` on the first child sprite entity of each player
        - `initialize_claimed_tiles`:
            - Reacts to `TiledEvent<MapCreated>` for `CurrentLevel` maps only
            - For each ground tile, spawns a `ClaimedTile{owner:None}` entity with `WaveEffectTarget`, `GridCoords`, `Transform`, `Anchor`
            - Stores each spawned entity in `MapInfo::claimed_entities`
        - `initialize_hp_bars`:
            - Reacts to `TiledEvent<MapCreated>` for `HudMap` maps only
            - Initializes `HPBar` entities with `GridCoords` and `Transform`
            - Sets `Anchor` and `custom_size` on the child sprite entity of each HP bar

## Plugin Systems

### Load Maps

Spawns two `TiledMap` entities at startup:
- `level0.tmx` — the current game level, tagged with the `CurrentLevel` marker component.
- `hud.tmx` — the heads-up display overlay map, tagged with the `HudMap` marker component.

### Initialize Map Info

Reacts to `TiledEvent<MapCreated>` filtered to `CurrentLevel` maps only. Reads tilemap metadata components, iterates all `Ground`-marked tile entities (storing them in `ground_entities`) and all `ForbiddenArea`-marked tile entities (storing them in `forbidden_areas`). Also allocates the `claimed_entities` HashMap keyed by `GridCoords`. Writes all collected data into the `MapInfo` resource so it is available world-wide.

### Initialize Players

Reacts to `TiledEvent<MapCreated>` filtered to `CurrentLevel` maps only. For each `Player`-marked `TiledObject` entity it:
1. Computes the initial `GridCoords` from the entity world-space `Transform` using the `MapInfo` resource.
2. Derives the starting `LookDirection` from the player id.
3. Inserts `GridCoords`, `LookDirection`, `TranslateEffectTarget`, `DamageEffectTarget`, and `Health{current:20, max:100}` on the player entity.
4. Inserts an `Anchor` component on the first child entity (the sprite entity) to properly anchor the sprite.

### Initialize Claimed Tiles

Reacts to `TiledEvent<MapCreated>` filtered to `CurrentLevel` maps only. For each ground tile in `MapInfo::ground_entities` it spawns a new entity with `ClaimedTile{owner:None}`, `WaveEffectTarget`, `GridCoords`, `Transform`, and `Anchor`. Each spawned entity is stored in `MapInfo::claimed_entities` keyed by its `GridCoords`, making it available for later lookup by the beam and animation systems.

### Initialize HP Bars

Reacts to `TiledEvent<MapCreated>` filtered to `HudMap` maps only. For each `HPBar` entity already spawned by the Tiled loader, computes its `GridCoords` from its world-space `Transform`, and inserts `GridCoords` and `Transform` on the entity. Also sets the `Anchor` and `custom_size` on the child sprite entity of each HP bar so the bar scales correctly from the correct pivot point.

## Components, Resources and Messages CRUD

### Read TiledEvent MapCreated messages

Used in the following systems:
- **initialize_map_info**: used to trigger map metadata initialization
- **initialize_players**: used to trigger player entity initialization
- **initialize_claimed_tiles**: used to trigger claimed tile entity spawning
- **initialize_hp_bars**: used to trigger HP bar initialization — filtered to `HudMap` maps only

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

Used in the following systems:
- **initialize_map_info**: used to get various map informations (e.g. map size, tile size, etc.)

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
- **initialize_map_info**: used to get all `Entity` and `TilePos` of `Ground`-marked tile entities spawned after loading the map

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

### Query TilePos of ForbiddenArea tiles

Used in the following systems:
- **initialize_map_info**: used to get all `Entity` and `TilePos` of `ForbiddenArea`-marked tile entities spawned after loading the map

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

forbidden_tiles_query{{"`forbidden_tiles_query`"}}:::query
on_map_created ---> forbidden_tiles_query

forbidden_entity@{ shape: st-rect, label: "ForbiddenArea Tile" }

ft_entity>"`**Entity**`"] --> |belongs to| forbidden_entity
ft_tile_pos>"`**TilePos**`"] --> |belongs to| forbidden_entity
ft_forbidden>"`**ForbiddenArea**`"] --> |belongs to| forbidden_entity

forbidden_tiles_query ---> |reads| ft_entity
forbidden_tiles_query ---> |reads| ft_tile_pos
forbidden_tiles_query -..-> |filter With| ft_forbidden
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
- **initialize_map_info**: writes the `MapInfo` resource, including `ground_entities`, `claimed_entities`, `forbidden_areas` HashMaps and all map geometry fields

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
map_info_claimed>"`**claimed_entities**`"]
map_info_forbidden>"`**forbidden_areas**`"]
map_info_map_size>"`**map_size**`"]
map_info_grid_size>"`**grid_size**`"]
map_info_tile_size>"`**tile_size**`"]
map_info_map_type>"`**map_type**`"]
map_info_map_anchor>"`**map_anchor**`"]

map_info_res --> |belongs to| world
map_info_ground --> |field of| map_info_res
map_info_claimed --> |field of| map_info_res
map_info_forbidden --> |field of| map_info_res
map_info_map_size --> |field of| map_info_res
map_info_grid_size --> |field of| map_info_res
map_info_tile_size --> |field of| map_info_res
map_info_map_type --> |field of| map_info_res
map_info_map_anchor --> |field of| map_info_res

initialize_map_info ---> |writes| map_info_res
```

### Write commands — initialize_players

Used in systems:
- **initialize_players**: inserts `GridCoords`, `LookDirection`, `TranslateEffectTarget`, `DamageEffectTarget`, `Health{current:20,max:100}` on each `Player` entity, and inserts `Anchor` on the first child sprite entity

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
pe_translate_effect>"`**TranslateEffectTarget**`"]
pe_damage_effect>"`**DamageEffectTarget**`"]
pe_health>"`**Health**`"]
ce_anchor>"`**Anchor**`"]

pe_grid_coords --> |inserted on| player_entity
pe_look_direction --> |inserted on| player_entity
pe_translate_effect --> |inserted on| player_entity
pe_damage_effect --> |inserted on| player_entity
pe_health --> |inserted on| player_entity
ce_anchor --> |inserted on| child_entity

initialize_players ---> |inserts component| pe_grid_coords
initialize_players ---> |inserts component| pe_look_direction
initialize_players ---> |inserts component| pe_translate_effect
initialize_players ---> |inserts component| pe_damage_effect
initialize_players ---> |inserts component| pe_health
initialize_players ---> |inserts component| ce_anchor
```

### Write commands — initialize_claimed_tiles

Used in systems:
- **initialize_claimed_tiles**: spawns one `ClaimedTile` entity per ground tile and stores each entity in `MapInfo::claimed_entities`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_claimed_tiles["`**initialize_claimed_tiles**`"]

update -.-> initialize_claimed_tiles

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile (spawned)" }
map_info_res@{ shape: doc, label: "MapInfo" }

ct_claimed_tile>"`**ClaimedTile**`"]
ct_wave_effect>"`**WaveEffectTarget**`"]
ct_grid_coords>"`**GridCoords**`"]
ct_transform>"`**Transform**`"]
ct_anchor>"`**Anchor**`"]

ct_claimed_tile --> |spawned on| claimed_tile_entity
ct_wave_effect --> |spawned on| claimed_tile_entity
ct_grid_coords --> |spawned on| claimed_tile_entity
ct_transform --> |spawned on| claimed_tile_entity
ct_anchor --> |spawned on| claimed_tile_entity

initialize_claimed_tiles ---> |spawns entity with| ct_claimed_tile
initialize_claimed_tiles ---> |spawns entity with| ct_wave_effect
initialize_claimed_tiles ---> |spawns entity with| ct_grid_coords
initialize_claimed_tiles ---> |spawns entity with| ct_transform
initialize_claimed_tiles ---> |spawns entity with| ct_anchor
initialize_claimed_tiles ---> |stores entity in claimed_entities| map_info_res
```

### Write commands — initialize_hp_bars

Used in systems:
- **initialize_hp_bars**: initializes existing `HPBar` entities (spawned by the Tiled loader from `hud.tmx`) with `GridCoords` and `Transform`, and sets `Anchor` and `custom_size` on the child sprite entity; triggered by `TiledEvent<MapCreated>` for the `HudMap`

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
initialize_hp_bars["`**initialize_hp_bars**`"]

update -.-> initialize_hp_bars

message_reader{{"MessageReader#60;TiledEvent#60;MapCreated#62;#62;"}}:::reader
initialize_hp_bars ---> message_reader

hud_map_query{{"`hud_map_query`"}}:::query
initialize_hp_bars ---> hud_map_query

hud_map_entity@{ shape: st-rect, label: "TiledMap (HudMap)" }
hm_tiled_map>"`**TiledMap**`"] --> |belongs to| hud_map_entity
hm_hud_map>"`**HudMap**`"] --> |belongs to| hud_map_entity
hud_map_query -..-> |filter With| hm_tiled_map
hud_map_query -..-> |filter With| hm_hud_map

hp_bar_entity@{ shape: st-rect, label: "HPBar Entity (from hud.tmx)" }
hp_bar_child@{ shape: st-rect, label: "HPBar Child (Sprite)" }

hb_grid_coords>"`**GridCoords**`"]
hb_transform>"`**Transform**`"]
hbc_anchor>"`**Anchor**`"]
hbc_custom_size>"`**custom_size**`"]

hb_grid_coords --> |inserted on| hp_bar_entity
hb_transform --> |inserted on| hp_bar_entity
hbc_anchor --> |set on| hp_bar_child
hbc_custom_size --> |set on| hp_bar_child

initialize_hp_bars ---> |inserts component| hb_grid_coords
initialize_hp_bars ---> |inserts component| hb_transform
initialize_hp_bars ---> |sets on child| hbc_anchor
initialize_hp_bars ---> |sets on child| hbc_custom_size
```
