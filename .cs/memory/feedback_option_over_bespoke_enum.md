---
name: feedback_option_over_bespoke_enum
description: "Prefer Option<Existing> over a new two-variant enum when the only extra state is \"absent\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7fc7afc2-8be8-4a9b-95ef-52c68d3fbfdd
---

When a decision has an existing enum plus one extra "nothing / blocked / don't"
outcome, use `Option<ExistingEnum>` (`None` = the extra outcome) rather than
introducing a bespoke wrapper enum. The user rejected a planned
`enum FireResolution { Blocked, Fire(BeamBehavior) }` in favor of
`Option<BeamBehavior>` for `resolve_fire`.

**Why:** fewer types, more idiomatic Rust, same single-source-of-truth.

**How to apply:** before adding a small enum, check whether `Option<T>` (or
`Result`) of an existing type already expresses it. Relates to
[[feedback_concise_doc_comments]].
