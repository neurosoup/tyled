---
id: TASK-14
title: 'Camera: players near the top are hidden by the HUD'
status: To Do
assignee: []
created_date: '2026-11-11 17:16'
labels:
  - camera
  - bug
dependencies: []
priority: medium
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A player on the top rows of the arena is drawn under the HUD strip. In the 2026-11-11 screenshot, the red player stands on the top row and only its legs show below the HUD, while the bottom of the arena is also cut off.

`update_camera` (`src/plugins/camera.rs`) moves the camera target up by `HUD_VIEWPORT_H / 2 * ortho.scale` (see doc-6 Camera plugin, 'The target is moved up by half the HUD height'). This only puts the midpoint between the players in the center of the area below the HUD. Nothing makes sure that each player stays inside that area.

Possible causes to check:
- The zoom level depends only on the distance between the players. It does not fit all players in the area below the HUD.
- The main camera renders to an off-screen texture (`PixelCamera`), so `ortho.scale` may not equal world units per window pixel. Then the offset is too small.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A player on the top row of the arena is fully visible below the HUD, at every zoom level
- [ ] #2 Both players stay fully visible when they stand at opposite ends of the arena
- [ ] #3 doc-6 describes the new framing rule
<!-- AC:END -->
