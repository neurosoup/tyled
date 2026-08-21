---
name: multipart-plan-execution
description: "How to execute an approved multi-part plan — autonomously per part, but pause between parts"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f7a55d23-884f-479a-a392-31d7b587300d
  modified: 2026-08-05T10:37:24.501Z
---

When a plan has multiple distinct parts/phases (e.g. "migrate deps", "build feature",
"cut release"), and the user approves it with "auto mode but stop after each Part":
execute each part autonomously (don't ask for step-by-step confirmation within a part,
delegate/iterate as needed to get it done), but stop and report back to the user once a
part is complete rather than auto-continuing to the next part.

**Why:** User approved the [[project_bevy_019_migration]]-adjacent 3-part plan (0.19
migration → matchup menu → 0.2.0 release) this way instead of via straight ExitPlanMode
approval — wants checkpoints between major chunks of work, not a fully unsupervised run
through all of them, but also doesn't want to be asked permission for every sub-step
inside a part.

**How to apply:** After finishing a part, send a concise summary of what changed/landed
and explicitly wait for the go-ahead before starting the next part. Still surface any
genuinely risky sub-step (e.g. pushing a git tag, force-pushing) for explicit
confirmation even within an "auto mode" part.
