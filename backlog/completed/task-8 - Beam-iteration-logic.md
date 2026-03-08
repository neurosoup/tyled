---
id: TASK-8
title: Beam iteration logic
status: Done
assignee: []
created_date: '2026-01-25 17:19'
updated_date: '2026-03-08 10:45'
labels: []
milestone: m-1
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
System responsibilities:

- Read BeamFired
- Read tile ownership
- Call beam iteration logic
- Emit TileClaimed when beam us resolved
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
# Tile lookup resource

For fast lookup, build a coordinate → entity HashMap during board initialization
--> We already have it: ```LevelWalkables```

Replace `walkable_locations` field from `HashSet` to `HashMap`.

Example:
```rust
let walkable_locations: HashMap<GridCoords, WalkableBundle> = walkables.iter()
   .map(|entity, grid_coords| (grid_coords), entity.clone()))
   .collect();
```
# Beam resolved event

```rust
struct BeamResolved {
    path: Vec<(i32, i32)>,
    target: (i32, i32),
    owner: PlayerId,
}
```

# Beam resolution system

```rust
fn beam_resolution_system(
    mut fired_events: EventReader<BeamFired>,
    level_walkables: Res<LevelWalkables>,
    mut resolved_events: EventWriter<BeamResolved>,
) {
    for event in fired_events.iter() {
        // 2. Call pure logic
        let path = compute_beam_path(
            event.origin,
            event.direction,
            &level_walkables,
        );

        // 3. Resolve target
        if let Some(&target) = path.last() {
            resolved_events.send(BeamResolved {
                path,
                target,
                owner: event.owner,
            });
        }
    }
}
```

# Beam iteration function

```rust
fn compute_beam_path(
    origin: (i32, i32),
    direction: (i32, i32),
    tiles: &HashMap<(i32, i32), Option<PlayerId>>,
) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let mut x = origin.0 + direction.0;
    let mut y = origin.1 + direction.1;

    loop {
        if !is_inside_board(x, y) {
            break;
        }

        path.push((x, y));

        if let Some(Some(_)) = tiles.get(&(x, y)) {
            break; // stop before or on colored tile (per your rules)
        }

        x += direction.0;
        y += direction.1;
    }

    path
}
```
<!-- SECTION:PLAN:END -->
