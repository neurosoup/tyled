---
model: claude-sonnet-5
---

**Usage**: `/manual-code-review [path | glob | diff]`

Interactive, step-by-step walkthrough of a diff with the user — a conversation, not a batch report. Unlike `/code-review` (which produces a findings list) or `/review-comments` (which is specifically about comment/doc text), this walks the user through the actual changes themselves, one logical step at a time, so they can follow along, dig deeper, or steer changes as we go.

## Scope

- If an argument is given, treat it as a path, glob, or `git diff` target (e.g. `HEAD~3`, a branch name) and scan only that.
- If no argument is given, default to the current uncommitted diff (`git status` / `git diff`) — if that's ambiguous or empty, ask the user what to review instead of guessing.

## Prepare

Break the diff into logical steps — not necessarily one per file. Group by what the change is doing (a bug fix, a refactor, a new system, a config addition) rather than mechanically by file, so each step is something explainable in one coherent chunk. Order steps in a sensible reading order (e.g. data/types before the systems that use them, root cause before its fix). Tell the user the step list up front (short, one line each) before starting step 1, so they know the shape of the walkthrough.

## Review loop

For each step, in order:

1. Show the user the relevant hunk(s): paste the actual diff (a fenced ```diff block, file path as a heading) directly in your response — a prose paraphrase is not a substitute, and a tool call whose output isn't echoed into the message doesn't count as "shown". Then explain what changed and why — the reasoning behind the change, not just a restatement of the diff.
2. Ask what they want to do next, structured as three lanes rather than one flat question:
   - **Move on** — proceed to the next step.
   - **Go deeper** — they ask questions about this step; answer, then re-offer the same choice for this same step (it isn't resolved until they say move on).
   - **Discuss / change** — they want to talk through or request a change to this step. Discuss it with them like any other design conversation (weigh tradeoffs, don't just comply reflexively) before applying anything. Apply the agreed change immediately, don't batch it for later, then re-show the (now-updated) step before moving on.
3. Don't skip ahead on your own — every step gets its own pause, even if it looks trivial.

## Wrap-up

Once every step has been moved past (or the user stops early): if this review added or modified any code comments — either ones that already existed in the diff going in, or new ones introduced during the discussion — run `/review-comments` scoped to the same target, so those comments get their own one-at-a-time pass. Skip that step entirely if no comments are in play.

Give a short tally at the end: how many steps, how many resulted in a change, which files were touched.
