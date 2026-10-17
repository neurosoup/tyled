---
id: doc-3
title: '[001] Maps plugin'
type: other
created_date: '2026-02-01 16:02'
updated_date: '2026-10-17 12:00'
---
# Maps Plugin

This plugin loads the level map and the HUD map when a round begins. After loading, it fills the shared map lookup and sets up the players, the claimable tiles and the HUD bars.

The setup reads no message. When it runs, the maps and all their entities already exist, so it reads the world directly.

## Plugin Systems

### Load Maps

- Runs on `OnEnter(AppState::InRound)`, after the matchup is confirmed in the menu.
- Spawns `level2.tmx` (tagged `CurrentLevel`) and `hud2.tmx` (tagged `HudMap`).

### Setup chain

- The four systems below run on `OnExit(RoundPhase::Loading)`. This happens only once: no round goes back to `Loading`.
- `initialize_map_info` runs first. The other three run after it, in any order.

### Initialize Map Info

- Fills `MapInfo`: ground tiles, forbidden tiles, an empty claimed-tile map, and the map geometry.
- Both maps have a tilemap named `"ground"`. It reads only the one of the level map.

### Initialize Players

- Sets up each player with its grid position, facing, health, charges, abilities and counters.
- `SpawnPoint` and `SpawnLookDirection` keep the start position and facing for the round reset.
- `PreviousGridCoords` starts on the spawn tile, so tile-entry damage works from the first move.
- `RestingTranslation` starts at the spawn position. The Effects plugin keeps it up to date.
- Health is `starting_health`. Charges are the ground-tile count divided by `beam_charges_divisor`.
- Abilities come from `PlayerLoadouts`. The tile and beam counters start at zero.

### Initialize Claimed Tiles

- Spawns one `ClaimedTile` with no owner for each ground tile, and stores it in `MapInfo`.
- The Effects plugin adds the lit overlay to each tile later. This plugin does not.

### Initialize HUD Bars

- Places the HP, damage, territory and charges bars from `hud2.tmx`. Player 1's bars move 16px to the right.
- Territory and charges bars start empty and grow in. HP and damage bars start at their real value.
- Player 1's bars grow from the left, player 2's from the right.
- HP and damage bars are 176x32. Territory and charges bars are 432x32 and grow toward each other.

## Components, Resources and Messages CRUD

### Query Tilemap metadata

Used in the following systems:
- **initialize_map_info**: reads the map geometry (size, tile size, etc.).

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_map_info["`**initialize_map_info**`"]

on_exit_loading -.-> initialize_map_info

tilemap_query{{"`tilemap_query`"}}:::query
initialize_map_info ---> tilemap_query

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
- **initialize_map_info**: all ground tiles.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_map_info["`**initialize_map_info**`"]

on_exit_loading -.-> initialize_map_info

ground_tiles_query{{"`ground_tiles_query`"}}:::query
initialize_map_info ---> ground_tiles_query

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
- **initialize_map_info**: all forbidden tiles.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_map_info["`**initialize_map_info**`"]

on_exit_loading -.-> initialize_map_info

forbidden_tiles_query{{"`forbidden_tiles_query`"}}:::query
initialize_map_info ---> forbidden_tiles_query

forbidden_entity@{ shape: st-rect, label: "ForbiddenArea Tile" }

ft_entity>"`**Entity**`"] --> |belongs to| forbidden_entity
ft_tile_pos>"`**TilePos**`"] --> |belongs to| forbidden_entity
ft_forbidden>"`**ForbiddenArea**`"] --> |belongs to| forbidden_entity

forbidden_tiles_query ---> |reads| ft_entity
forbidden_tiles_query ---> |reads| ft_tile_pos
forbidden_tiles_query -..-> |filter With| ft_forbidden
```

### Query Character entities (initialize_players)

Used in the following systems:
- **initialize_players**: all player characters.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_players["`**initialize_players**`"]

on_exit_loading -.-> initialize_players

players_query{{"`players_query`"}}:::query
initialize_players ---> players_query

player_entity@{ shape: st-rect, label: "Player (Character)" }

pe_entity>"`**Entity**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity
pe_transform>"`**Transform**`"] --> |belongs to| player_entity
pe_character>"`**Character**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_entity
players_query ---> |reads| pe_player
players_query ---> |reads| pe_transform
players_query -..-> |filter With| pe_character
```

### Write MapInfo resource

Used in systems:
- **initialize_map_info**: fills every field of `MapInfo`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_map_info["`**initialize_map_info**`"]

on_exit_loading -.-> initialize_map_info

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
- **initialize_players**: the player setup listed above. The diagram does not show `SpawnPoint`, `SpawnLookDirection` and `PreviousGridCoords`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_players["`**initialize_players**`"]

on_exit_loading -.-> initialize_players

player_entity@{ shape: st-rect, label: "Player (Character)" }
child_entity@{ shape: st-rect, label: "Player Child (Sprite)" }

pe_grid_coords>"`**GridCoords**`"]
pe_look_direction>"`**LookDirection**`"]
pe_translate_effect>"`**TranslateEffectTarget**`"]
pe_resting_translation>"`**RestingTranslation**`"]
pe_damage_effect>"`**DamageEffectTarget**`"]
pe_health>"`**Health**`"]
pe_beam_charges>"`**BeamCharges**`"]
pe_ability_list>"`**AbilityList**`"]
pe_claimed_tile_count>"`**ClaimedTileCount**`"]
pe_in_flight_count>"`**InFlightBeamCount**`"]
ce_anchor>"`**Anchor**`"]

world@{ shape: st-rect, label: "World" }
player_loadouts_res@{ shape: doc, label: "PlayerLoadouts" }
player_loadouts_res --> |belongs to| world

pe_grid_coords --> |inserted on| player_entity
pe_look_direction --> |inserted on| player_entity
pe_translate_effect --> |inserted on| player_entity
pe_resting_translation --> |inserted on| player_entity
pe_damage_effect --> |inserted on| player_entity
pe_health --> |inserted on| player_entity
pe_beam_charges --> |inserted on| player_entity
pe_ability_list --> |inserted on| player_entity
pe_claimed_tile_count --> |inserted on| player_entity
pe_in_flight_count --> |inserted on| player_entity
ce_anchor --> |inserted on| child_entity

initialize_players ---> |reads| player_loadouts_res
initialize_players ---> |inserts component| pe_grid_coords
initialize_players ---> |inserts component| pe_look_direction
initialize_players ---> |inserts component| pe_translate_effect
initialize_players ---> |inserts component| pe_resting_translation
initialize_players ---> |inserts component| pe_damage_effect
initialize_players ---> |inserts component| pe_health
initialize_players ---> |inserts component| pe_beam_charges
initialize_players ---> |inserts component| pe_ability_list
initialize_players ---> |inserts component| pe_claimed_tile_count
initialize_players ---> |inserts component| pe_in_flight_count
initialize_players ---> |inserts component| ce_anchor
```

### Write commands — initialize_claimed_tiles

Used in systems:
- **initialize_claimed_tiles**: one tile entity for each ground tile.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_claimed_tiles["`**initialize_claimed_tiles**`"]

on_exit_loading -.-> initialize_claimed_tiles

claimed_tile_entity@{ shape: st-rect, label: "ClaimedTile (spawned)" }
map_info_res@{ shape: doc, label: "MapInfo" }

ct_claimed_tile>"`**ClaimedTile**`"]
ct_wave_effect>"`**WaveEffectTarget**`"]
ct_glow_effect>"`**GlowEffectTarget**`"]
ct_grid_coords>"`**GridCoords**`"]
ct_transform>"`**Transform**`"]

ct_claimed_tile --> |spawned on| claimed_tile_entity
ct_wave_effect --> |spawned on| claimed_tile_entity
ct_glow_effect --> |spawned on| claimed_tile_entity
ct_grid_coords --> |spawned on| claimed_tile_entity
ct_transform --> |spawned on| claimed_tile_entity

initialize_claimed_tiles ---> |spawns entity with| ct_claimed_tile
initialize_claimed_tiles ---> |spawns entity with| ct_wave_effect
initialize_claimed_tiles ---> |spawns entity with| ct_glow_effect
initialize_claimed_tiles ---> |spawns entity with| ct_grid_coords
initialize_claimed_tiles ---> |spawns entity with| ct_transform
initialize_claimed_tiles ---> |stores entity in claimed_entities| map_info_res
```

### Query HPBar/DamageBar/TerritoryBar/ChargesBar entities (initialize_hud_bars)

Used in the following systems:
- **initialize_hud_bars**: all four bar kinds. Only territory and charges bars are handled differently.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_hud_bars["`**initialize_hud_bars**`"]

on_exit_loading -.-> initialize_hud_bars

bars_query{{"`bars_query`"}}:::query
initialize_hud_bars ---> bars_query

bar_entity@{ shape: st-rect, label: "HPBar/DamageBar/TerritoryBar/ChargesBar Entity" }

be_player>"`**Player**`"] --> |belongs to| bar_entity
be_transform>"`**Transform**`"] --> |belongs to| bar_entity
be_is_territory>"`**Has#60;TerritoryBar#62;**`"] --> |belongs to| bar_entity
be_is_charges>"`**Has#60;ChargesBar#62;**`"] --> |belongs to| bar_entity

bars_query ---> |reads| be_player
bars_query ---> |reads| be_transform
bars_query ---> |reads| be_is_territory
bars_query ---> |reads| be_is_charges
bars_query -..-> |filter Or With HPBar/DamageBar/TerritoryBar/ChargesBar| bar_entity
```

### Query Sprite entities (initialize_hud_bars)

Used in the following systems:
- **initialize_hud_bars**: sets the size of each bar's child sprite. Matches any sprite.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_hud_bars["`**initialize_hud_bars**`"]

on_exit_loading -.-> initialize_hud_bars

sprite_query{{"`sprite_query`"}}:::query
initialize_hud_bars ---> sprite_query

sprite_entity@{ shape: st-rect, label: "Any Sprite Entity" }

se_sprite>"`**Sprite**`"] --> |belongs to| sprite_entity

sprite_query ---> |writes custom_size| se_sprite
```

### Write commands — initialize_hud_bars

Used in systems:
- **initialize_hud_bars**: places the bars spawned from `hud2.tmx`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

on_exit_loading(("`OnExit(Loading)`")):::system-group
initialize_hud_bars["`**initialize_hud_bars**`"]

on_exit_loading -.-> initialize_hud_bars

hp_bar_entity@{ shape: st-rect, label: "HPBar/DamageBar/TerritoryBar/ChargesBar Entity (from hud2.tmx)" }
hp_bar_child@{ shape: st-rect, label: "Bar Child (Sprite)" }

hb_grid_coords>"`**GridCoords**`"]
hb_transform>"`**Transform**`"]
hb_scale_zero>"`**scale.x = 0 (TerritoryBar/ChargesBar only)**`"]
hbc_anchor>"`**Anchor**`"]
hbc_custom_size>"`**custom_size**`"]

hb_grid_coords --> |inserted on| hp_bar_entity
hb_transform --> |inserted on| hp_bar_entity
hb_scale_zero --> |set on| hp_bar_entity
hbc_anchor --> |set on| hp_bar_child
hbc_custom_size --> |set on| hp_bar_child

initialize_hud_bars ---> |inserts component| hb_grid_coords
initialize_hud_bars ---> |inserts component| hb_transform
initialize_hud_bars ---> |zeroes scale.x| hb_scale_zero
initialize_hud_bars ---> |sets on child| hbc_anchor
initialize_hud_bars ---> |sets on child| hbc_custom_size
```
