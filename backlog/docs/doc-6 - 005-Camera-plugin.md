---
id: doc-6
title: '[005] Camera plugin'
type: other
created_date: '2026-02-01 19:27'
updated_date: '2026-06-15 12:00'
---
# Camera Plugin

Contains systems related to camera initialization and runtime updates. The plugin spawns two cameras: a main gameplay camera that smoothly follows the barycenter of all player positions and dynamically adjusts its orthographic zoom, and a HUD camera that renders the HUD map in a fixed viewport strip at the top of the window.

## Plugin workflow

- Startup phase
    - `initialize_cameras` spawns two camera entities:
        - A main `Camera2d` with `DepthOfField` and an `OrthographicProjection` set to `MIN_ZOOM_SCALE`.
        - A HUD `Camera2d` assigned to `RenderLayers` layer 1, with a fixed viewport occupying the top strip of the window.
    - `randomize_clear_color` randomizes the background hue each run (fixed saturation and lightness).
- Update phase
    - `initialize_hud_rendering`:
        - Reacts to `TiledEvent<MapCreated>` for the `HudMap` entity only
        - Inserts `Propagate(RenderLayers)` on the HUD map root entity so all its children are rendered exclusively in the HUD camera layer
    - `update_camera`:
        - Reads all `Player` transforms, computes the barycenter and max inter-player distance
        - Smoothly nudges the main camera `Transform` toward the barycenter and lerps the orthographic zoom scale
    - `update_hud_viewport`:
        - Reacts to `WindowResized` events
        - Recalculates the HUD camera viewport rect and orthographic scale to keep the HUD correctly sized regardless of window dimensions

## Plugin Systems

### Initialize Cameras

Spawns two camera entities at startup:

1. **Main camera** — carries `Camera2d`, `DepthOfField` post-process effect, and an `OrthographicProjection` starting at `MIN_ZOOM_SCALE` (0.33 — zoomed out). This camera renders the game world on the default render layer.
2. **HUD camera** — carries `Camera2d` and is assigned to `RenderLayers` layer 1. Its viewport is set to a fixed top strip of the window. `order` is set higher than the main camera so it composites on top.

### Randomize Clear Color

Runs once at startup. Picks a random hue (0–360°) while keeping a fixed saturation and lightness, then writes it into the `ClearColor` resource to give each game session a unique background tint.

### Initialize HUD Rendering

Reacts to `TiledEvent<MapCreated>` filtered to the `HudMap` entity only. Inserts a `Propagate(RenderLayers)` component on the HUD map root entity, propagating `RenderLayers` layer 1 down the entire entity hierarchy via `HierarchyPropagatePlugin`. This ensures the HUD tilemap and all its child sprites are rendered only in the HUD camera and never appear in the main game camera.

### Update Camera

Runs every frame. Reads the `Transform` of every `Player` entity to compute:
- The **barycenter** (average position) — the camera target.
- The **max inter-player distance** — used to derive the desired zoom scale.

The camera `Transform` is smoothly nudged toward the barycenter using `smooth_nudge` (decay rate `0.5`), and the `OrthographicProjection` scale is lerped toward the target zoom, clamped between `MIN_ZOOM_SCALE` (0.33) and `MAX_ZOOM_SCALE` (2.0).

### Update HUD Viewport

Runs whenever a `WindowResized` event is received. Recomputes the HUD camera's `Viewport` rect — position and size — to keep the HUD strip anchored to the top of the window at the correct pixel dimensions. Also updates the HUD camera's `OrthographicProjection` scale so the HUD tiles remain at their intended size regardless of the window resolution.

## Components, Resources and Messages CRUD

### Write ClearColor resource

Used in the following systems:
- **randomize_clear_color**: writes a randomized hue into the global background color at startup

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
randomize_clear_color["`**randomize_clear_color**`"]

startup -.-> randomize_clear_color

world@{ shape: st-rect, label: "World" }
clear_color_res@{ shape: doc, label: "ClearColor" }

clear_color_res --> |belongs to| world

randomize_clear_color ---> |writes| clear_color_res
```

### Read TiledEvent MapCreated messages (HUD)

Used in the following systems:
- **initialize_hud_rendering**: used to detect when the HudMap has finished loading so `Propagate(RenderLayers)` can be inserted

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
initialize_hud_rendering["`**initialize_hud_rendering**`"]

update -.-> initialize_hud_rendering

message_reader{{"MessageReader#60;TiledEvent#60;MapCreated#62;#62;"}}:::reader
initialize_hud_rendering ---> message_reader

map_created_message(["`**TiledEvent#60;MapCreated#62;**`"])

message_reader ---> |reads| map_created_message
```

### Read WindowResized events

Used in the following systems:
- **update_hud_viewport**: reacts to window resize events to recalculate the HUD camera viewport and orthographic scale

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_hud_viewport["`**update_hud_viewport**`"]

update -.-> update_hud_viewport

event_reader{{"EventReader#60;WindowResized#62;"}}:::reader
update_hud_viewport ---> event_reader

window_resized_event(["`**WindowResized**`"])

event_reader ---> |reads| window_resized_event
```

### Query Player transforms

Used in the following systems:
- **update_camera**: reads all `Transform` components on `Player`-marked entities to compute the camera target position and zoom level

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_camera["`**update_camera**`"]

update -.-> update_camera

players_query{{"`players_query (ParamSet)`"}}:::query
update_camera ---> players_query

player_entity@{ shape: st-rect, label: "Player" }

pe_transform>"`**Transform**`"] --> |belongs to| player_entity
pe_player>"`**Player**`"] --> |belongs to| player_entity

players_query ---> |reads| pe_transform
players_query -..-> |filter With| pe_player
```

### Write Camera components (main)

Used in the following systems:
- **update_camera**: smoothly updates the main camera `Transform` (position) and `OrthographicProjection` (zoom scale) every frame

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_camera["`**update_camera**`"]

update -.-> update_camera

camera_query{{"`camera_query (ParamSet)`"}}:::query
update_camera ---> camera_query

camera_entity@{ shape: st-rect, label: "Main Camera" }

ce_transform>"`**Transform**`"] --> |belongs to| camera_entity
ce_projection>"`**Projection**`"] --> |belongs to| camera_entity
ce_camera2d>"`**Camera2d**`"] --> |belongs to| camera_entity

camera_query ---> |writes| ce_transform
camera_query ---> |writes| ce_projection
camera_query -..-> |filter With| ce_camera2d
```

### Write HUD Camera components (viewport)

Used in the following systems:
- **update_hud_viewport**: updates the HUD camera `Camera::viewport` rect and `OrthographicProjection` scale on window resize

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
update_hud_viewport["`**update_hud_viewport**`"]

update -.-> update_hud_viewport

hud_camera_query{{"`hud_camera_query`"}}:::query
update_hud_viewport ---> hud_camera_query

hud_camera_entity@{ shape: st-rect, label: "HUD Camera" }

hc_camera>"`**Camera**`"] --> |belongs to| hud_camera_entity
hc_projection>"`**OrthographicProjection**`"] --> |belongs to| hud_camera_entity
hc_render_layers>"`**RenderLayers**`"] --> |belongs to| hud_camera_entity

hud_camera_query ---> |writes viewport on| hc_camera
hud_camera_query ---> |writes scale on| hc_projection
hud_camera_query -..-> |filter With| hc_render_layers
```

### Write commands — initialize_hud_rendering

Used in the following systems:
- **initialize_hud_rendering**: inserts `Propagate(RenderLayers)` on the HUD map root entity so the render layer propagates to all children

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
initialize_hud_rendering["`**initialize_hud_rendering**`"]

update -.-> initialize_hud_rendering

hud_map_entity@{ shape: st-rect, label: "HUD Map Root Entity" }

hm_propagate>"`**Propagate#60;RenderLayers#62;**`"]

hm_propagate --> |inserted on| hud_map_entity

initialize_hud_rendering ---> |inserts component| hm_propagate
```

### Write commands — initialize_cameras (Startup)

Used in the following systems:
- **initialize_cameras**: spawns the main camera and the HUD camera entities with their initial components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
initialize_cameras["`**initialize_cameras**`"]

startup -.-> initialize_cameras

main_camera_entity@{ shape: st-rect, label: "Main Camera (spawned)" }
hud_camera_entity@{ shape: st-rect, label: "HUD Camera (spawned)" }

mc_camera2d>"`**Camera2d**`"]
mc_depth_of_field>"`**DepthOfField**`"]
mc_projection>"`**Projection (Orthographic)**`"]

hc_camera2d>"`**Camera2d**`"]
hc_render_layers>"`**RenderLayers (layer 1)**`"]
hc_viewport>"`**Camera::viewport**`"]

mc_camera2d --> |spawned on| main_camera_entity
mc_depth_of_field --> |spawned on| main_camera_entity
mc_projection --> |spawned on| main_camera_entity

hc_camera2d --> |spawned on| hud_camera_entity
hc_render_layers --> |spawned on| hud_camera_entity
hc_viewport --> |spawned on| hud_camera_entity

initialize_cameras ---> |spawns entity with| mc_camera2d
initialize_cameras ---> |spawns entity with| mc_depth_of_field
initialize_cameras ---> |spawns entity with| mc_projection
initialize_cameras ---> |spawns entity with| hc_camera2d
initialize_cameras ---> |spawns entity with| hc_render_layers
initialize_cameras ---> |spawns entity with| hc_viewport
```
