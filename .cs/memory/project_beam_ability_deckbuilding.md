---
name: project-beam-ability-deckbuilding
description: "Plan to turn Tyled's beam behaviors + charge economy into a Balatro-style deckbuilding system (triggers/enablers/stack/payoff/archetypes)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 576aed90-1186-4cd7-9d9c-2a0445115aa7
---

Agreed direction (decided 2026-07-05) to bring beam behaviors and the
beam-charges economy into a Balatro-style ability/deckbuilding system.
**Moved into the repo (2026-07-09)** — full plan now lives at
`/home/law/dev/tyled/DECKBUILDING.md` (repo root, alongside CLAUDE.md, which
now references it; renamed from BEAM_ABILITY_DECKBUILDING.md 2026-07-12), not
the old personal plans directory.
It's a real, git-tracked project artifact now — edit it directly rather
than treating this memory as the source of truth. Supersedes
[[project_strategic_mechanics_rework]].

**Why:** The beam plugin already has two mutually-exclusive resolution modes
(normal/"Straight" and inverted/"Pierce", the latter added in `7cb49aa`).
Rather than treating one as legacy, both are kept and generalized into a
system where beam behaviors, charge regen, body-blocking, and contested-tile
mechanics all become draftable "abilities" using Balatro's design vocabulary:
triggers (game events like `BeamFired`/`BeamResolved`/tile-claim/charge-spend),
enablers (set up a resource/condition), payoffs (cash in on it), and stacks
(same-family effects compounding).

**Key decisions:**
- Straight Shot is a universal, non-drafted baseline every player always has.
- Pierce Shot (today's "inverted" mode) is reframed as the *first draftable
  ability* — a fallback appended to Straight, not a rival starting pick
  (Pierce would strictly dominate Straight if both were offered pre-match).
- Internally, `Beam.inverted: bool` becomes an ordered list of behavior
  descriptors — the substrate every future beam-behavior ability appends to.
- Acquisition: symmetric round-draft (both players pick from the same
  pick-1-of-3 offer between rounds) is the initial build target. Asymmetric
  personal shops are an explicitly deferred *future game mode*, not part of
  the first implementation.
- Architecture: data-driven ability descriptors + trigger-keyed resolver
  systems, not one component-bundle-with-systems per ability (avoids system
  sprawl, keeps abilities introspectable for a draft UI, hot-reloadable via
  the existing `file_watcher`).
- **Long-press activation input (decided 2026-07-08)**: Overcharge and Wide
  Shot are "pay more for a bigger effect, per shot" abilities, which the
  existing "no separate input, everything auto-selected by context" rule
  (§2) doesn't cover. Resolution: reuse the existing shoot button with hold-
  duration detection (`leafwing-input-manager`) — tap fires a normal 1-charge
  shot, holding past a threshold activates the drafted cost-scaling
  ability(-ies). Requires moving `BeamFired` from firing-on-press to firing-
  on-release with a charge-level/width parameter. Checked the full roster:
  only these two abilities need this; everything else is either passive or
  auto-triggered by context. **Overcharge redesigned** around this (was a
  flat 3-charge upfront cost, now graduated): holding accrues 1 tile of claim
  budget per beam-step tick at 1 charge/tick (reducible via new ability
  **Capacitor**, down to 0.25/tick at max stacks), deducted in real time so
  you can watch the lane and release early instead of blindly committing.
  Flagged a major, less-bounded-than-Ricochet+Splitter combo: Overcharge's
  claims are real `on_claim` events, so Splitter/Chain Reaction multiply
  every tile it claims — needs a session 6 balance pass.
- **Parry (2026-07-08) — upcoming core mechanic, not yet built, but the
  roster now accounts for it.** Same Shoot button doubles as parry: if an
  incoming enemy beam is within its parry window when you press Shoot, that
  press reflects it (reversed direction, increased speed, ownership flips to
  you) instead of firing a new shot — no new input. **Decided: the reflected
  beam keeps the ORIGINAL shooter's effects** (mode, bounce budget, whatever
  abilities were baked in), not the parrier's own loadout — deliberate
  trade-off, means a 0-ability player can temporarily wield an opponent's
  kit, and an aggressive player's own Breach/Overpenetration can be turned
  against their own territory if parried. **Decided: no cap on chained
  parries** (rally) — left to self-limit via increasing speed rather than a
  hard rule, revisit if playtesting shows otherwise. **Body Blocker** is now
  reframed as the fallback for any non-parry outcome (missed timing or no
  attempt), not an alternative to parrying. Added 5 provisional Parry
  abilities (Riposte, Extended Window, Perfect Parry, Parry Refund,
  **Unparryable** — the last one close to required tech for Breach
  Aggression/Chain Cannon, since those are the archetypes most exposed to
  having their own kit reflected back). Full reasoning and all roster
  cross-references in the plan file's §2 "Parry" subsection.
- **Overcharge claim-direction fixed (2026-07-08)**: the original "fill in
  from origin outward" model had a real flaw — a normal 1-charge shot always
  resolves at the *far* end of its lane, so a lightly-charged Overcharge
  could claim only near tiles and never even reach that point, making it
  worse than not charging at all. Fixed: Overcharge always claims the normal
  resolve tile first (matching a plain shot's value as a floor), then spends
  extra budget backward toward the origin for Straight (pushing the frontier
  out toward the obstruction) or forward past the resolve point for Pierce
  (deeper into freshly-opened territory) — monotonically better with more
  charge, direction matches what's actually useful per mode.
- **Long-press cancel-on-hit (2026-07-08)**: getting hit while charging
  Overcharge/Wide Shot cancels the hold — default is total loss (accrued
  charges/progress gone, nothing fires), standard charge-shot risk/reward.
  New ability **Composure** softens this to a forced early release
  (salvages whatever was accrued) instead of total loss. Body Blocker does
  NOT prevent the cancel — it only softens the physical hit consequence.
  Also resolves the earlier open mid-hold-parry question: release inside
  the parry window → parry (charge discarded); get hit without parrying →
  cancel rule applies. Also confirmed (2026-07-08): **parry is per-lane on
  Wide Shot** — it only ever touches the single beam entity at the
  defender's tile (Wide Shot spawns 3 independent entities, not one
  volley), so a parry never re-widens on the way back; falls out of the
  existing architecture, no special-casing needed.
- 28-ability candidate roster (cut **Long Fuse** 2026-07-08 — its
  travel-distance-cap justification assumed a mechanic that doesn't exist;
  Ricochet/Bank Shot are already bounce-capped, Straight/Pierce are
  map-bounded, Breach is charge-bounded, so nothing needed the cap it
  extended): added **Bank Shot** (2026-07-07, stacks +1
  Ricochet bounce per copy; Ricochet itself now claims at each turn, making
  Ricochet+Splitter a hard combo instead of a soft/map-dependent one), and
  redesigned **Twin Barrel → Wide Shot** (2026-07-07, symmetric 3-beam spread
  centered on the player instead of an off-center pair with no natural
  center; stackable wider per copy, same pattern as Bank Shot). 4 archetypes
  (Solar Economy / Breach
  Aggression / Iron Wall / Chain Cannon) are drafted in the plan file; treat
  them as a starting point to argue with, not a locked spec. Includes
  **Reckoning** (viewer-suggested, 2026-07-06): a match-end payoff that cashes
  in owned-tile count as damage when the board saturates — an opt-in
  "painting" win path meant to sit alongside direct-damage aggression.
- 7-session staged rollout, starting with the behavior-list refactor (no new
  content) before any ability is draftable.
- **Confirmed real game mechanics (2026-07-06), corrected several ability
  designs**: (1) beams hitting a player body don't stop by default — they
  deal 1 HP and knock the victim 1 tile in the beam's direction, and since
  the victim's movement input is locked while knocked back (`inputs.rs`),
  they get dragged tile-by-tile and re-hit every 62.5ms tick with no way to
  escape until the beam resolves or knockback fails. (2) A kill ends the
  round immediately, and a round boundary is a **full reset** of board +
  charges — only round wins/kills and drafted abilities persist. Together
  these forced reworks: **Body Blocker** is now a defensive ability held by
  the potential *target* (stops an incoming beam on contact instead of the
  default drag), not an attacker's own beam modifier; **Impaler** rewards a
  guaranteed Rare in the next draft offer instead of a charge/tile refund
  (worthless under full reset); **Scorched Earth** now leaves Scorched
  (impassable) tiles that deliberately survive one round's reset, instead of
  a radius burst with no target in a 2-player match. **Overpenetration** was
  later resolved (2026-07-06): instant, unconditional enemy-tile flip (no
  countdown/adjacency), the blunt counterpart to Contested Ground+Breach's
  slower conditional one — kept both since they're differentiated by risk/
  scale, not redundant. See the plan file for full reasoning on each.
- **Ricochet/Wide Shot turn-direction resolved (2026-07-07)**: Ricochet now
  claims the tile at each turn instead of a free pass-through, and its turn
  direction is no longer ambiguous — a solo beam prefers whichever
  perpendicular side is open ground (clockwise default if tied); a beam
  spawned by Wide Shot carries a lane-offset and turns outer beams away from
  center deterministically, so a widened spread can't collapse back into
  itself. Not gated behind Wide Shot — still a standalone reach tool, just
  gets a cleaner turn rule when both are drafted together. Closed a
  follow-up corner case same day: center's turn (and any 2nd+ bounce via
  Bank Shot) now prefers *unclaimed* ground specifically, not just valid
  ground — otherwise center could turn straight into a tile an outer sibling
  already claimed the same volley, wasting the turn. Also flagged a narrow
  same-tick race if claim mutations are deferred across the 3 Wide Shot
  beams; not a crash, just a rare missed avoidance.

- **Contested Ground clarified + combo-checked (2026-07-09)**: fixed
  ambiguous "hold an adjacent tile" wording (in both Contested Ground and
  Breach) to explicit ownership — it's a `ClaimedTile` check at the 2s mark,
  not a requirement to stand nearby; the attacker can walk away and fire
  other shots freely, no interference with normal Straight/Pierce behavior.
  Checked interactions: Splitter claims the pending tile's 2 neighbors on
  the contest's *start* (a guaranteed floor even if the contest later
  fails); Chain Reaction triggers on the eventual flip like any claim;
  Ricochet just passes through a pending tile. **Overpenetration + Contested
  Ground is an anti-synergy, not a combo** — Overpenetration's instant flip
  takes priority, so the contest never triggers; drafting both wastes a
  slot. **Overcharge + Pierce + Contested Ground**: the forward budget can
  chain through multiple enemy tiles at once (contesting each
  simultaneously) *only if Breach is also drafted* — Breach is what makes a
  pending tile passable at all, and extending that to the same beam's own
  travel (not just a later beam) turns Overcharge into a one-button
  execution of a full Breach chain rather than a way to skip drafting
  Breach. Without Breach, the forward budget is simply wasted in that case.
  Deeper links in such a chain are naturally riskier with no extra rule
  needed — a still-pending neighbor doesn't count as owned for the next
  tile's adjacency check.

- **Chain Reaction fixed to actually chain (2026-07-09)**: was specified as
  a single hop (claim → one bonus claim), contradicting its own name and the
  Chain Cannon archetype's "explosive cascades" flavor text — same
  flattening bug Breach had before that got caught and fixed. Now the
  auto-claimed neighbor's claim is itself a real trigger, so it cascades
  again if it also qualifies (≥3 owned neighbors). Left uncapped, same
  reasoning as the parry rally — naturally self-limiting since each hop
  consumes a finite, depleting neutral tile; flagged for the balance pass
  regardless. Also added a visible-armed-state note to **Landmine**: since
  Tyled is shared-screen local multiplayer (no per-player render target),
  "owner-only" visibility isn't possible, so armed tiles are visible to both
  players via the existing `*EffectTarget` pattern.

- **Scorched Earth → Beachhead, reworked again (2026-07-09)**: was "on your
  own death, your tiles become impassable" (a denial capstone for the
  loser). Now "on enemy death, burst-claim every tile in a radius around
  where they died (forced claim, any status except forbidden ground), and
  these claims survive the round's full reset" — a reward for the killer,
  giving them a territorial head start next round. Reuses the exact same
  round-reset exception hook the old design needed, just with a different
  payload (claimed territory instead of a hazard state) — generalized that
  hook's description since the payload already changed once. Bypasses
  "claimed tiles never change owner" as an area effect, joining
  Overpenetration and Contested Ground as the third way to flip enemy
  tiles. No longer aggression/self-death-specific — reads as a universal
  kill-reward capstone for any archetype that can secure kills. Renamed
  because "Scorched Earth" (destroy what you can't keep) no longer fit a
  seize-and-hold mechanic.
- **Beachhead now has TWO possible victim counters (2026-07-09)**: rather
  than replacing "Salted Earth" with the mirrored burst-claim idea, both
  were kept as distinct options. **Last Stand** (#20) burst-claims the same
  radius as Beachhead but for the *victim*, also surviving the reset —
  symmetric with Beachhead, so when both are present the conflicting claims
  just cancel (one uniform rule: opposing claims over the same ground
  cancel). **Salted Earth** (#21) instead makes the radius hazardous/
  unclaimable for everyone — this one needs no bespoke cancellation logic
  at all, since processing the hazard-marking before any claim attempt
  means Beachhead's claim just fails against forbidden-like ground the same
  way it always would. Last Stand and Salted Earth are an anti-synergy with
  *each other* if the same player drafts both (contradictory responses to
  their own death). All three have standalone value independent of each
  other. Roster renumbered again to fit both (now 30 abilities total).
  Needs a cross-player ability check at death resolution for the
  Beachhead/Last-Stand case specifically, same shape as Body Blocker's
  target-side lookup — Salted Earth avoids needing this by construction.
- **Salted Earth redesigned again — anti-synergy removed (2026-07-09)**:
  instead of being a second independent `on_death` trigger that conflicted
  with Last Stand if both were drafted, Salted Earth is now a **transform
  enabler**: requires Last Stand, does nothing alone, and when present
  flips Last Stand's resolution from claim-for-self to deny-to-everyone
  (same radius, same reset-survival). Since it no longer fires on its own,
  there's no scenario where both effects exist to conflict — it's the
  mechanism that changes which effect Last Stand has, not a competing
  ability. Intentional cost asymmetry: claim-mode costs 1 slot, denial-mode
  costs 2 (Last Stand + Salted Earth) — situational counter-tech should
  cost more than the always-good default. Beachhead's interaction check
  extended to also ask "is Last Stand transformed by Salted Earth" as part
  of the same existing lookup, not new architecture. Roster count unchanged
  (31) — reuses Salted Earth's existing slot rather than adding a new one.
- **Fallout (2026-07-09)**: shared stacking enabler, +1 tile to the death-
  burst radius per copy, benefits whichever of Beachhead/Last Stand (#19/
  #20) the drafter has — Last Stand's radius stacks the same whether or not
  Salted Earth has transformed it, since that only changes claim-vs-deny,
  not scope. One shared parameter across the family rather than Bank Shot/
  Capacitor's one-ability-each pattern. Breaks the "always-equal radius"
  assumption the cancellation rule relied on, so it now resolves
  **per-tile in both modes**: within the overlap, the smaller radius's
  rule wins (opposing claims cancel in claim-mode; denial blocks in
  denial-mode, processed first); any tile inside Beachhead's radius but
  outside the other radius was never touched by it, so Beachhead's claim
  succeeds there regardless of mode. **Corrected 2026-07-09** (caught by
  the user): the denial-mode case was originally, wrongly, described as
  size-independent ("forbidden ground blocks a claim at any radius") — it
  only wins within its own radius, same per-tile shape as claim-mode, not
  an exception to it. Same combo-risk category as other stacking abilities
  (compounds with Splitter/Chain Reaction) — balance-pass flag. Roster is
  31 abilities.

- **Round-ending model formalized — two win vectors (2026-07-11)**: round
  ending is now framed as **HP and tiles being the only two win vectors**;
  every ending condition (kill / first-to-N-tiles / timeout) is a function of
  one of them. Key insight driving it: a **kill-only** round rewards beam-spam
  and plays flat (matches the user's real playtest finding that Straight- and
  Backfill-mirror games felt strategy-less) — making *tiles* a first-class win
  vector is what gives the baseline territorial strategy. Decided model: kill
  (HP→0, instant, top priority) + first-to-N-tiles (optional explicit finish
  line) + timeout (backstop, resolves by tile-count-then-HP). Ship kill +
  most-tiles-timeout to cover both vectors. Balance crux: the two vectors must
  be *comparably fast* or the faster dominates and one archetype dies. Updated
  plan file §1, §5 (new "Round-ending model" block), §7 (testing-protocol
  layer 1 + Stage F3a now specify both-vectors-live, not kill-only), and the
  "Direct-HP payoffs" open thread (**elevated from nice-to-have to required**:
  if HP is a win vector, the roster is lopsided — the tile vector has a whole
  economy family, the HP vector has only Riposte; needs HP enablers/payoffs for
  vector parity). **Recommended next test**: build F3a with both vectors, then
  re-run the Straight mirror — a territory race may inject strategy with zero
  abilities, isolating goal-vs-abilities before spending effort on kit configs.
  Then the first asymmetric content test is `{Solar Panels}` (not +Tithe —
  Tithe just accelerates shooting) vs `{Overpen}` (±Backfill; Overpen is the
  breaching variable that matters), seat-swapped.

**Open thread, deliberately deferred:** a consumables/enhancers layer
(Balatro's Tarot/Planet/Spectral-equivalent — limited-use, one-off effects,
separate from the persistent drafted abilities above). Not designed yet;
revisit once sessions 1-3 of the rollout are playable and there's a feel for
whether the ability layer alone carries enough moment-to-moment texture. See
"Open threads for later" in the plan file.

**How to apply:** when a session touches beam behavior, beam charges, tile
claiming/contesting, or ability/draft systems, load the full plan file first.
Update the plan file (not just this memory) as sessions land or design
forks get resolved differently than currently written.
