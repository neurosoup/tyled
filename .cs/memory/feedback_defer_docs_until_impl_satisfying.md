---
name: feedback_defer_docs_until_impl_satisfying
description: Defer backlog/docs updates until the implementation is validated and satisfactory
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a32db30b-7c5b-4c23-a48b-dde26eecd788
---

Do not update `backlog/docs/` (or other documentation) as part of the main implementation steps.
Defer all doc syncing to a final pass, done only once the code (and any art) is working and the user
is satisfied with the result.

**Why:** Docs written against unvalidated code get rewritten; the user wants documentation to reflect
the settled implementation, not an in-progress design.

**How to apply:** In plans and execution, put doc updates as the LAST step, gated on the
implementation being validated. Still keep docs in sync eventually (CLAUDE.md requires it) — just not
prematurely. Relates to [[feedback_concise_doc_comments]] and [[feedback_no_deckbuilding_ref_in_docs]].
