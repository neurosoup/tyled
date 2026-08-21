---
name: feedback-no-claudemd-refs-in-code
description: Never reference CLAUDE.md from inline code comments/doc-comments
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6eb22acf-6b15-4d9e-891b-d5b1b94c10fa
---

Do not mention or point to `CLAUDE.md` from inline code documentation (`//` comments or `///` doc-comments). Keep code comments self-contained.

**Why:** When centralizing the camera-adding convention into CLAUDE.md and trimming `camera.rs` comments, I wrote a module header that said "See CLAUDE.md (Cameras) for the convention." The user rejected it: "Do not specify CLAUDE.md in inline documentation."

**How to apply:** When moving conceptual/convention docs to CLAUDE.md (or `backlog/docs/`), just remove the inline version — don't replace it with a "see CLAUDE.md" pointer. Convention/architecture lives in CLAUDE.md and `backlog/docs/`; code comments explain only the local code. Relates to [[feedback-no-deckbuilding-ref-in-docs]] (keep cross-doc references out of code/docs).
