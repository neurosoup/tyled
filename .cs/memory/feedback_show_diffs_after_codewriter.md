---
name: feedback-show-diffs-after-codewriter
description: "After code-writer agent finishes, show the user the actual diff before proceeding, so they can review/request changes"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a9e66867-9ffc-4550-abf3-691265d2947d
  modified: 2026-08-02T16:22:45.828Z
---

After the `code-writer` subagent (or any agent that edits files) completes a change, show the user the diff (`git diff` on touched files) rather than just summarizing what was done in prose.

**Why:** the user wants to review actual changes and request modifications, or push back on the approach, before it's treated as final (e.g. before committing).

**How to apply:** `code-writer`'s own agent definition (`.claude/agents/code-writer.md`) now requires it to end its report with `git diff -- <touched files>`, so this is enforced at the agent level for code-writer specifically — relay that diff to the user rather than re-summarizing it in prose. For other file-editing agents without that built-in requirement (or if the agent definition ever regresses), fall back to running `git diff` yourself after they return, before treating the task as complete.
