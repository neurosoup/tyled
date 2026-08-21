---
name: feedback_concise_doc_comments
description: "keep inline doc comments on functions terse — first sentence only, don't pile on rationale"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 50be53bd-ca48-4057-88d5-6ce6ef4abdd6
---

When writing a doc comment on a new function/system, keep it to a single terse first sentence stating what it does. Don't append multiple sentences of rationale, edge-case explanation, or deferral/priority reasoning inline.

**Why:** The user rejected a 6-line doc comment on a new round-resolution system as "too verbose," asking to keep the first sentence only. The detailed reasoning belongs in `backlog/docs/`, not inline.

**How to apply:** One-sentence `///` doc on the item; put priority/deferral/edge-case detail in the matching `backlog/docs/` doc instead. Distinct from [[feedback_no_added_code_comments]] (which is about not adding comments at all where neighbors lack them) — this is about brevity when a comment is warranted.
