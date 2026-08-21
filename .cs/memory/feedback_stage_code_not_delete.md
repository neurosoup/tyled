---
name: feedback-stage-code-not-delete
description: "When staging a future feature, port existing logic into the future code path instead of deleting it"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3be9a600-98a4-42e0-ad51-d6e057982256
---

When a design stage removes a behavior that a *later* stage will reintroduce in a
reworked form, the user prefers **porting the existing logic into the future
feature's code path (as staged/inert code) rather than deleting it** — even if the
current stage keeps it inactive.

Concrete case (Tyled DECKBUILDING §7 Stage F1): rather than deleting the beam's
`inverted` mode branch, port it verbatim into a `BeamBehavior::Backfill` arm that
is present but never selected in F1; F2 turns it on. Gameplay outcome unchanged
(fizzle), but the code is preserved for the next stage.

**Why:** less rework across the staged rollout; the next stage becomes a "flip the
selection on" change instead of "reimplement." Preserves working, tested logic.

**How to apply:** in staged/deckbuilding work, when a stage deactivates behavior a
later stage revives, relocate it into the target feature's structure with
`#[allow(dead_code)]` + a `// wired in <stage>` comment. Don't delete. Confirm
whether the ported code should be *active* in the current stage or merely staged.
Related: [[project_bevy_019_migration]] staging discipline.
