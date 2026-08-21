---
name: feedback-no-deckbuilding-ref-in-docs
description: "backlog/docs must describe the present state only — no DECKBUILDING.md names, no history, no 'inverted'"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3be9a600-98a4-42e0-ad51-d6e057982256
---

The `backlog/docs/*` plugin/component docs must reflect **the Now** — the current
state of the code, described directly and in present tense. When updating them:

- **No `DECKBUILDING.md` names or citations** — no "Stage F1/F2", "Slice N", bare
  "F2", "deckbuilding", or "(see `DECKBUILDING.md`)". Describe the behavior itself.
- **No history / "how it was before"** — drop "replaced the former X", "reproduces
  the pre-F1 behavior", "ported/preserved verbatim", "reframed", etc.
- **Never mention "inverted mode"** — the `Backfill` behavior is described on its
  own terms, not as ex-inverted logic.
- Present-state "not yet" facts are fine without roadmap names: e.g. `TileClaimed`
  is emitted but "(no consumers yet)" — not "consumed starting Stage F2".
- **Cross-reference other docs by TITLE, not internal id** — write "see the Effects
  plugin doc" / "the Round plugin doc", never "(`doc-10`)" / "see `doc-15`". Same
  rule in source comments. (Confirmed 2026-07-16; older docs still carry `doc-N`
  ids from before this rule — fix them when you touch that section, don't sweep.)
- **CRUD message-read mermaids use the Claim-plugin pattern** — a dedicated
  `MessageReader#60;Type#62;` node (`classDef reader stroke-dasharray: 3 3`,
  `{{...}}` shape) from the system to a stadium message node
  (`msg(["`**Type**`"])`), one reader node per reading system. Do **not** use the
  old `@{ shape: das }` message node with a `|read by|` edge. Full CRUD sections
  also want an entry per new message/resource/component + query and a Definitions
  bullet — see the Round plugin doc for a complete example.

**Why:** the code docs should stand on their own as a description of what the game
does right now; `DECKBUILDING.md` is the separate, roadmap-flavored design plan and
is the only place stage/slice framing and "IMPLEMENTED" markers belong.

**How to apply:** editing `DECKBUILDING.md` itself with Stage/Slice framing is
expected; editing `backlog/docs` (or source comments) is Now-only per the bullets
above. After doc edits, grep for `stage|slice|\bf[0-9]|inverted|former|deckbuilding`
to catch leftovers. Related: [[feedback_stage_code_not_delete]].
