---
id: doc-6
title: '[005] Camera plugin'
type: other
created_date: '2026-02-01 19:27'
updated_date: '2026-05-31 00:00'
---
# Camera Plugin

Contains systems related to camera initialization and runtime updates. The plugin uses `bevy_smooth_pixel_camera` for pixel-perfect sub-pixel smoothing on the main camera. Four cameras are active at runtime:

| Camera | Render layer | Order | Purpose |
|--------|-------------|-------|---------|
| Main (`PixelCamera`) | default (no `RenderLayers`) | 0 | Renders the game world to an off-screen texture |
| ViewportCamera | layer 2 | 2 | Spawned automatically by `PixelCamera`; composites the off-screen texture onto the window |
| HUD camera | layer 1 | 3 | Renders the HUD map in a fixed viewport strip at the top of the window |
| Overlay camera | layer 3 | 4 | Renders screen-space round-phase banners (intro countdown, etc.) full-window, on top of everything; content is drawn by the round `intro` submodule |

The `PixelCamera` component (from `bevy_smooth_pixel_camera`) also spawns a `ViewportImage` sprite child. Its `snap_camera_position` system (PostUpdate) snaps the main camera's `GlobalTransform` to the nearest integer world unit and offsets the `ViewportImage` by the subpixel remainder to produce smooth motion without pixel crawl.

## Plugin workflow

- Startup phase
    - `initialize_cameras` spawns three camera entities:
        - A main `Camera2d` with `PixelCamera` (`ViewportScalingMode::PixelSize(1.0)`) and an `OrthographicProjection` starting at `ZOOM_LEVELS[0]` (1/4). `PixelCamera` automatically spawns a `ViewportCamera` child on layer 2 with render order 2.
        - A HUD `Camera2d` assigned to `RenderLayers` layer 1, render order 3, with a fixed viewport occupying the top strip of the window.
        - An overlay `Camera2d` assigned to `RenderLayers` layer 3, render order 4, `ClearColorConfig::None`, with a `ScalingMode::FixedVertical` orthographic projection (constant fraction of screen height at any window size, centred on the origin — no resize handling needed).
    - `randomize_clear_color` randomizes the background hue each run (fixed saturation and lightness).
- Update phase
    - `initialize_hud_rendering`:
        - Reacts to `TiledEvent<MapCreated>` for the `HudMap` entity only
        - Inserts `Propagate(RenderLayers)` on the HUD map root entity so all its children are rendered exclusively in the HUD camera layer
    - `update_camera`:
        - Reads all `Character` transforms, computes the barycenter and max inter-player distance
        - Smoothly nudges the main camera `Transform` toward the barycenter (`CAMERA_DECAY_RATE`), shifted up in world space by `(HUD_VIEWPORT_H / 2) * scale` so the framing is centred in the region below the HUD strip and top-of-arena players aren't occluded by the HUD overlay
        - Lerps the orthographic scale toward the nearest pixel-perfect zoom level (`ZOOM_DECAY_RATE`), then snaps exactly once within 0.001 to guarantee a clean 1/n value
    - `update_hud_viewport`:
        - Reacts to `WindowResized` events
        - Recalculates the HUD camera viewport rect and re-applies the pixel-perfect fixed 2× orthographic scale on window resize

## Plugin Systems

### Initialize Cameras

Spawns three camera entities at startup, resulting in four active cameras at runtime:

1. **Main camera** — carries `Camera2d` and `PixelCamera` (`PixelSize(1.0)`, render layer 2, order 2). `PixelCamera::on_add` automatically spawns a `ViewportCamera` (layer 2, order 2) and a `ViewportImage` sprite child. The `OrthographicProjection` starts at `ZOOM_LEVELS[0]` (0.25 — most zoomed out). The main camera has no `RenderLayers` component; the `update_camera` query uses `Without<RenderLayers>` to isolate it from the ViewportCamera, HUD camera, and overlay camera. **Invariant:** every non-main camera must carry an explicit `RenderLayers`, or that `Single` matches more than one entity and breaks.
2. **HUD camera** — carries `Camera2d`, `RenderLayers` layer 1, and `Camera { order: 3 }`. Its viewport is set to a fixed top strip of the window, and its `OrthographicProjection` scale is initialized to the pixel-perfect fixed 2× (`scale_factor / HUD_SCALE`) at spawn so the HUD is correct from the first frame. Render order 3 ensures it composites on top of the ViewportCamera (order 2).
3. **Overlay camera** — carries `Camera2d`, `RenderLayers` layer 3, `Msaa::Off` (matching the HUD camera to avoid a sample-count mismatch), and `Camera { order: 4, clear_color: ClearColorConfig::None }`. Its `OrthographicProjection` uses `ScalingMode::FixedVertical { viewport_height: OVERLAY_VIEWPORT_HEIGHT }`, keeping banner glyphs a constant fraction of screen height at any resolution and centred on the origin — so no `WindowResized` handling is needed. Order 4 composites it above the HUD. The banners it renders are spawned by the round `intro` submodule.

### Randomize Clear Color

Runs once at startup. Picks a random hue (0–360°) while keeping a fixed saturation and lightness, then writes it into the `ClearColor` resource to give each game session a unique background tint.

### Initialize HUD Rendering

Reacts to `TiledEvent<MapCreated>` filtered to the `HudMap` entity only. Inserts a `Propagate(RenderLayers)` component on the HUD map root entity, propagating `RenderLayers` layer 1 down the entire entity hierarchy via `HierarchyPropagatePlugin`. This ensures the HUD tilemap and all its child sprites are rendered only in the HUD camera and never appear in the main game camera.

### Update Camera

Runs every frame. Reads the `Transform` of every `Character` entity to compute:
- The **barycenter** (average position) — the camera target.
- The **max inter-player distance** — used to derive the desired zoom scale.

The camera `Transform` is smoothly nudged toward the barycenter using `smooth_nudge` (decay rate `CAMERA_DECAY_RATE`). The target is offset upward in world space by `(HUD_VIEWPORT_H / 2) * scale` (world-per-physical-pixel equals the orthographic scale, so the offset tracks the current zoom). This re-centres the framing in the visible region *below* the HUD strip instead of the whole window, so players near the top of the arena stay clear of the HUD overlay. Only `scale` affects pixel-perfection — translation is snapped/subpixel-blitted by the pixel camera — so the offset is safe at any zoom.

Zoom is pixel-perfect: only the four levels in `ZOOM_LEVELS` (`[1/4, 1/3, 1/2, 1]`) are valid target values, since with `PixelSize(1.0)` these are the only scales where 1 world unit = an integer number of screen pixels. The nearest level is selected by mapping `max_distance` through `BASE_ZOOM_DISTANCE` to a continuous scale, then picking the closest entry in `ZOOM_LEVELS`. The `OrthographicProjection` scale lerps toward the target at `ZOOM_DECAY_RATE` and snaps exactly once within 0.001 to guarantee the final value is a clean 1/n fraction.

### Update HUD Viewport

Runs whenever a `WindowResized` event is received. Recomputes the HUD camera's `Viewport` rect — a full-window-width strip of fixed height anchored to the top of the window. The `OrthographicProjection` scale is set to `window.scale_factor() / HUD_SCALE`, a pixel-perfect fixed 2×: since physical pixels per world unit = `scale_factor / scale`, every 16px HUD tile maps to exactly `16 * HUD_SCALE` physical pixels at any DPI. The HUD is therefore a fixed-size, horizontally-centered band (the camera sits at the origin and the map uses `TilemapAnchor::Center`) rather than stretched to fill the window width — the transparent side margins show the game world beneath.

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

### Query Character transforms

Used in the following systems:
- **update_camera**: reads all `Transform` components on `Character`-marked entities to compute the camera target position and zoom level

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

characters_query{{"`characters_query (ParamSet)`"}}:::query
update_camera ---> characters_query

character_entity@{ shape: st-rect, label: "Character" }

ce_transform>"`**Transform**`"] --> |belongs to| character_entity
ce_character>"`**Character**`"] --> |belongs to| character_entity

characters_query ---> |reads| ce_transform
characters_query -..-> |filter With| ce_character
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
camera_query -..-> |filter Without RenderLayers - excludes ViewportCamera and HUD camera| ce_camera2d
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
- **initialize_cameras**: spawns the main camera, the HUD camera, and the overlay camera entities with their initial components. `PixelCamera::on_add` automatically spawns additional child entities (`ViewportCamera` on layer 2 / order 2, and `ViewportImage`).

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
viewport_camera_entity@{ shape: st-rect, label: "ViewportCamera + ViewportImage (auto-spawned by PixelCamera)" }
hud_camera_entity@{ shape: st-rect, label: "HUD Camera (spawned)" }
overlay_camera_entity@{ shape: st-rect, label: "Overlay Camera (spawned)" }

mc_camera2d>"`**Camera2d**`"]
mc_pixel_camera>"`**PixelCamera (PixelSize 1.0, layer 2, order 2)**`"]
mc_projection>"`**Projection (Orthographic, scale ZOOM_LEVELS[0])**`"]

hc_camera2d>"`**Camera2d**`"]
hc_render_layers>"`**RenderLayers (layer 1)**`"]
hc_camera>"`**Camera (order 3, viewport top strip)**`"]

oc_camera2d>"`**Camera2d + Msaa::Off**`"]
oc_render_layers>"`**RenderLayers (layer 3)**`"]
oc_camera>"`**Camera (order 4, ClearColorConfig::None)**`"]
oc_projection>"`**Projection (Orthographic, FixedVertical)**`"]

mc_camera2d --> |spawned on| main_camera_entity
mc_pixel_camera --> |spawned on| main_camera_entity
mc_projection --> |spawned on| main_camera_entity
mc_pixel_camera -.-> |triggers on_add → spawns| viewport_camera_entity

hc_camera2d --> |spawned on| hud_camera_entity
hc_render_layers --> |spawned on| hud_camera_entity
hc_camera --> |spawned on| hud_camera_entity

initialize_cameras ---> |spawns entity with| mc_camera2d
initialize_cameras ---> |spawns entity with| mc_pixel_camera
initialize_cameras ---> |spawns entity with| mc_projection
initialize_cameras ---> |spawns entity with| hc_camera2d
initialize_cameras ---> |spawns entity with| hc_render_layers
initialize_cameras ---> |spawns entity with| hc_camera

oc_camera2d --> |spawned on| overlay_camera_entity
oc_render_layers --> |spawned on| overlay_camera_entity
oc_camera --> |spawned on| overlay_camera_entity
oc_projection --> |spawned on| overlay_camera_entity

initialize_cameras ---> |spawns entity with| oc_camera2d
initialize_cameras ---> |spawns entity with| oc_render_layers
initialize_cameras ---> |spawns entity with| oc_camera
initialize_cameras ---> |spawns entity with| oc_projection
```
