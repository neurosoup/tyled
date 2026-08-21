---
name: feedback-deckbuilding-implemented-tag-scope
description: "In DECKBUILDING.md, only §7 stage/slice headers get \"— IMPLEMENTED\"; don't inline-tag individual mechanics elsewhere"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ad978ff0-df19-4940-9698-3817f951dba9
  modified: 2026-08-02T12:52:06.905Z
---

Don't add "**implemented**" callouts to individual mechanics described in DECKBUILDING.md's design-model sections (§1-§5, the roster in §3) — that tag is reserved for §7's stage/slice headers (e.g. "Stage F3a — ... — **IMPLEMENTED**").

**Why:** rejected when proposed for the round-ending model's charge-exhaustion condition in §5, even though the mechanic is genuinely implemented (`resolve_charge_exhaustion` in `src/plugins/round/state.rs`, part of the already-`IMPLEMENTED`-tagged Stage F3a). The user's correction: don't duplicate the implementation-status signal at the mechanic level when the owning stage already carries it. See [[project_bevy_019_migration]]-style precedent of keeping status markers at one canonical location rather than scattered.

**How to apply:** when a design-model section describes a mechanic that happens to be built, describe it plainly like its unimplemented siblings in the same list (no "implemented"/"built" qualifier inline). Implementation status lives exclusively in §7's per-stage headers and prose — that's the single source of truth for "is this built yet," not §1-§5's design vocabulary. Applies broadly to [[feedback_no_deckbuilding_ref_in_docs]]'s sibling concern about keeping status markers scoped to their proper home rather than duplicated across the doc.
