---
id: doc-6
title: '[005] Camera plugin'
type: other
created_date: '2026-02-01 19:27'
updated_date: '2026-03-08 17:04'
---
# Camera Plugin

Contains systems related to camera initialization and runtime updates. The camera smoothly follows the barycenter (centroid) of all player positions and dynamically adjusts its orthographic zoom scale based on the spread between players.

## Plugin workflow

- Startup phase
    - `initialize_camera` spawns the Camera entity with `Camera2d`, `DepthOfField` and an `OrthographicProjection` set to `MIN_ZOOM_SCALE`.
    - `randomize_clear_color` randomizes the background hue each run (fixed saturation and lightness).
- Update phase
    - `update_camera` reads all `Player` transforms, computes the barycenter and max inter-player distance, then smoothly nudges the camera `Transform` toward the barycenter and lerps the orthographic zoom scale.

## Plugin Systems

### Initialize Camera

Spawns the 2D camera entity with `Camera2d`, `DepthOfField` post-process effect, and an `OrthographicProjection` starting at `MIN_ZOOM_SCALE` (0.33 — zoomed out).

### Randomize Clear Color

Runs once at startup. Picks a random hue (0–360°) while keeping a fixed saturation and lightness, then writes it into the `ClearColor` resource to give each game session a unique background tint.

### Update Camera

Runs every frame. Reads the `Transform` of every `Player` entity to compute:
- The **barycenter** (average position) — the camera target.
- The **max inter-player distance** — used to derive the desired zoom scale.

The camera `Transform` is smoothly nudged toward the barycenter using `smooth_nudge` (decay rate `0.5`), and the `OrthographicProjection` scale is lerped toward the target zoom, clamped between `MIN_ZOOM_SCALE` (0.33) and `MAX_ZOOM_SCALE` (2.0).

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

### Write Camera components

Used in the following systems:
- **update_camera**: smoothly updates the camera `Transform` (position) and `OrthographicProjection` (zoom scale) every frame

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

camera_entity@{ shape: st-rect, label: "Camera" }

ce_transform>"`**Transform**`"] --> |belongs to| camera_entity
ce_projection>"`**Projection**`"] --> |belongs to| camera_entity
ce_camera2d>"`**Camera2d**`"] --> |belongs to| camera_entity

camera_query ---> |writes| ce_transform
camera_query ---> |writes| ce_projection
camera_query -..-> |filter With| ce_camera2d
```

### Write commands (Startup)

Used in the following systems:
- **initialize_camera**: spawns the camera entity with its initial components

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
initialize_camera["`**initialize_camera**`"]

startup -.-> initialize_camera

camera_entity@{ shape: st-rect, label: "Camera (spawned)" }

ce_camera2d>"`**Camera2d**`"]
ce_depth_of_field>"`**DepthOfField**`"]
ce_projection>"`**Projection (Orthographic)**`"]

ce_camera2d --> |spawned on| camera_entity
ce_depth_of_field --> |spawned on| camera_entity
ce_projection --> |spawned on| camera_entity

initialize_camera ---> |spawns entity with| ce_camera2d
initialize_camera ---> |spawns entity with| ce_depth_of_field
initialize_camera ---> |spawns entity with| ce_projection
```
