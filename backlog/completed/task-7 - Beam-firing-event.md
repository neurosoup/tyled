---
id: TASK-7
title: Beam firing event
status: Done
assignee: []
created_date: '2026-01-25 17:17'
updated_date: '2026-03-08 10:45'
labels: []
milestone: m-1
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Capture the action of firing a beam, storing origin, direction, and owner.
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
# Beam firing event

```rust
struct BeamFired {
    origin: (i32, i32),
    direction: (i32, i32),
    owner: PlayerId,
}
```

Listen for player input and send BeamFired event.
<!-- SECTION:PLAN:END -->
