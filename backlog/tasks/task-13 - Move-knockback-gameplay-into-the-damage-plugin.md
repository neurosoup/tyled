---
id: TASK-13
title: Move knockback gameplay into the damage plugin
status: To Do
assignee: []
created_date: '2026-10-06 17:14'
labels: []
milestone: m-5
dependencies:
  - TASK-11
references:
  - src/plugins/effects.rs
  - src/plugins/damage.rs
  - src/plugins/controller.rs
  - src/plugins/messages.rs
  - backlog/tasks/task-11 - Effect-drivers-and-resolver-refactor.md
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Stage 4 of task-11, split out. Today the effects plugin owns both halves of knockback: apply_knockback moves GridCoords and sets the IsKnockedBack input lock (gameplay), then writes a Knockback effect request (visual). Move the gameplay half into the damage plugin, which DECKBUILDING.md already names as the knockback owner, and replace the KnockbackEffect marker with a message. Needed before Barrier (#33) and Body Blocker, which add new knockback sources and need one clear ordering rule (which knockback wins in a tick).
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add KnockbackRequested { entity, direction } to messages.rs. controller.rs (walk into a beam, collision bump) and damage.rs (beam hit) write it instead of inserting KnockbackEffect.
2. In damage.rs, add a system in GameplaySet::Displacement that reads KnockbackRequested and does the gameplay half now in effects.rs apply_knockback: skip when IsDead or Health <= 0, check map_info.on_ground(target), set GridCoords, insert or refresh IsKnockedBack(Timer). Move tick_knockback_lock with it.
3. After a successful knockback, damage writes EntityKnockedBack { entity }. Effects reads it in EffectsSet::Request and writes EffectRequest { kind: Knockback { ms: knockback_tween_ms } }.
4. Delete KnockbackEffect and apply_knockback. Remove KnockbackEffect from the reset_round removal tuple (messages need no cleanup).
5. If several knockbacks arrive in one tick for one entity, decide the rule (latest wins, or a fixed priority) and write it down for Barrier.
6. Docs: doc-11 Damage, doc-4 Controller, doc-10 Effects, doc-9 messages (two new messages), doc-15 reset list, DECKBUILDING.md knockback mentions.
Verify: cargo check, cargo test --features dev. Manual (user): drag across several tiles, death during a drag, collision bump, input locked until the drag ends.
Decided (2026-10-07): the stun (IsKnockedBack lock) and the slide always last the same time. Keep the single knob knockback_tween_ms for both; do not add knockback_lock_ms.
<!-- SECTION:PLAN:END -->
