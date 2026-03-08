---
id: TASK-10
title: Claim and  visualize claimed tile
status: Done
assignee: []
created_date: '2026-01-25 17:53'
updated_date: '2026-03-08 10:45'
labels: []
milestone: m-1
dependencies: []
---

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
# Claim system

```rust
fn tile_claim_system(
    mut events: EventReader<BeamResolved>,
    mut tiles: Query<&mut Tile>,
    lookup: Res<TileLookup>,
) {
    for event in events.iter() {
        if let Some(&entity) = lookup.map.get(&event.target) {
            if let Ok(mut tile) = tiles.get_mut(entity) {
                // Claim only neutral tiles
                if tile.owner.is_none() {
                    tile.owner = Some(event.owner);
                }
            }
        }
    }
}
```
<!-- SECTION:PLAN:END -->
