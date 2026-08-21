---
name: feedback-no-added-code-comments
description: "Don't add doc-comments/inline comments to Rust code unless the surrounding code already has them"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a27e27cc-f5ee-4383-85bc-f7f2a1569d0d
---

In this project's plugin/component Rust source, do **not** add `///` doc-comments
or explanatory inline comments to new items (systems, struct fields, etc.) unless
the surrounding code in that same file already carries them. Match the local
comment density exactly.

**Why:** the user twice rejected edits that added doc-comments to `animations.rs`
(a new system fn and new `ClaimedTileAnimations` reverse-clip fields) — that file's
existing systems and struct fields have no comments, so added ones read as noise.
Note some other files (e.g. `round/state.rs`, `messages.rs`) *do* comment
liberally — so this is per-file, judged by neighbors, not a blanket ban.

**How to apply:** before adding a comment to code, glance at the sibling
items in the same file. Comment only to match them. When a comment *is*
warranted, keep it terse — one line where possible; the user rejected a
3-line `///` on a new component (`PreviousGridCoords`) as too verbose and
asked for less. This is code only — the `backlog/docs/*` doc-sync updates
are still expected and thorough. Related:
[[feedback_docs_scope]], [[feedback_no_claudemd_refs_in_code]].
