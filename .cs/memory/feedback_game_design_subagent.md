---
name: feedback-game-design-subagent
description: "Delegate Tyled game-design/balance questions to the project's game-designer subagent instead of reasoning inline"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5a538e8e-f232-4abb-bca3-4422bd2a1809
  modified: 2026-07-31T15:48:49.184Z
---

For any Tyled game-design question (ability design, balance, archetype identity, draft/economy tradeoffs, whether a new mechanic is worth its implementation cost), use the Agent tool with `subagent_type: game-designer` — defined at `.claude/agents/game-designer.md` in the repo — instead of reasoning about it inline in the main session.

**Why:** the user asked to have an Opus subagent dedicated to this, grounded in `DECKBUILDING.md` as the authoritative design doc. Keeping design reasoning in a dedicated agent keeps it consistently grounded (reads the full doc + relevant source before opining) and separates "thinking about design" from "implementing/editing" — the agent is read-only and explicitly does not edit DECKBUILDING.md or code itself; the main session applies any resulting doc/code changes after the user agrees.

**How to apply:** trigger on requests like "what do you think about ability X", "do we need Y for this slice", "is this balanced", "how should archetype Z work" — for Tyled specifically. Not for mechanical/implementation-only requests (those stay in the main session or use Explore/Plan as usual).
