---
name: Documentation scope preference
description: When asked to document a struct, only add a doc comment to the struct itself — not its fields or impl methods
type: feedback
originSessionId: 031c8309-63e4-429c-97ac-4d71eb39da6d
---
When asked to "add documentation for X struct", only doc-comment the struct definition itself.

**Why:** User rejected an edit that also added field-level and impl method docs — they only wanted the struct-level comment.

**How to apply:** Unless the user explicitly asks for field docs, method docs, or "full documentation", limit struct docs to the struct item only.
