# Tyled — Beam Abilities as a Balatro-Style Deckbuilding System

Design plan for reworking beam behaviors and the beam-charges economy into a
Balatro-style deckbuilding system.

## 0. Framing

Tyled already emits a stream of cheap, typed events (`BeamFired`, `BeamResolved`,
`DamageableDied`, plus implicit tile-claim / charge-spend / move / damage-tick
moments) — that stream *is* the scoring engine, the same way Balatro jokers hook
scoring events. The design job is mostly plumbing existing messages into
hookable slots, not inventing new simulation. Abilities should modify beams,
not replace them — beams at 62.5ms/step are the analog of "playing a hand."

## 1. Vocabulary mapping

| Trigger | Source today | Notes |
|---|---|---|
| `on_fire` | `BeamFired{owner, origin, direction}` | Every shot, before resolution |
| `on_resolve` | `BeamResolved{position, owner}` | Core "scoring" moment |
| `on_claim` | tile owner flips in `MapInfo.claimed_entities` | Distinct from resolve — a resolve on an already-owned tile is a no-op; needs its own message (`TileClaimed`) |
| `on_death` | `DamageableDied{entity}` | Rare, big-payoff |
| `on_charge_spent` | silent decrement today | Needs to emit a message |
| `on_charge_regen` | doesn't exist yet | The economy archetype's backbone |
| `on_move` | `EntityMoved` | Frequent, positional |
| `on_damage_tick` | damage plugin's 500ms tick | Attrition trigger |
| `on_step` | beam advances one tile (62.5ms) | Very hot — emit only if an ability needs it |
| `on_board_saturated` | doesn't exist yet | Fires once when claimed tiles / total ground tiles crosses ~95%. Rare, match-defining, like `on_death`. |
| `on_body_hit` | `apply_beam_damage` in `src/plugins/damage.rs` | Fires when a beam's `GridCoords` matches an enemy player's. Today: 1 HP + a 1-tile `KnockbackEffect` in the beam's direction, independent of the beam's own resolve/claim logic. Because the beam also advances 1 tile/step in the same direction, and the hit player's movement input is locked (`inputs.rs` excludes `IsKnockedBack` entities), a caught player is dragged tile-by-tile and re-hit every 62.5ms tick with **no way to escape**, until knockback fails (blocked destination) or the beam itself resolves. Verified against the actual `inputs.rs`/`damage.rs`/`effects.rs` code, not assumed. |
| `on_parry_window` | doesn't exist yet — upcoming core mechanic, not yet built | Opens when an incoming enemy beam is close enough to the player that pressing Shoot would parry it instead of firing. See "Parry" in §2. |
| `on_parried` | doesn't exist yet | Fires when a Shoot press lands inside the window: beam ownership flips to the parrier, direction reverses, speed increases. The beam's cast-time behavior/effects are **not** re-evaluated — see §2 for the ownership-vs-effects distinction. |

- **Enabler**: changes the shape of a trigger stream or a resource, doesn't itself win (regen, marks tiles, opens windows).
- **Payoff**: cashes a built-up condition into board control/damage.
- **Stack**: homogeneous (same-family, additive) vs. heterogeneous (enabler→payoff combo).
- **Archetype**: a cluster of enablers + payoff sharing a resource/trigger.
- Rule of thumb: aim for a roster ~60% enabler / 30% payoff / 10% hybrid "engine" pieces.
- **Roles are relative, not fixed.** Enabler/payoff is a role an ability plays
  *toward a specific other ability's trigger*, not a permanent label. An
  ability can be a payoff for its own trigger (Splitter cashes in `on_resolve`
  for an immediate extra claim) while simultaneously being an enabler for a
  different payoff that consumes a resource it produces as a side effect (see
  **Reckoning**, §3, which turns any tile-claiming payoff into a de facto
  enabler for itself). Don't try to hand-assign a single fixed role per
  ability — classify per combo.
- **A kill ends the round, and a round boundary is a full reset**: both the
  board (tile ownership) and charges wipe completely; only round wins/kills
  accumulate toward match score, and only drafted abilities persist.
  Consequence: **`on_death` can only ever pay off into the persistent layer**
  (next-round draft offers/starting conditions, match score) — never in-round
  board state or charges, since those are gone the instant the trigger fires.

## 2. Tier-0 model (decided)

There is no pre-match Straight-vs-Backfill choice.

- **Straight Shot is a universal baseline** — every player always has it, like
  Balatro's "you always get to play a hand." Fired from unclaimed ground: stops
  at the first blocked tile, claims the last unclaimed tile before it. Fired
  from a tile the player already owns with nothing new to claim ahead: fizzles
  silently, charge spent for nothing.
- **Backfill Shot is the first draftable ability**, not a starting pick. It adds
  a fallback on top of Straight: when Straight would otherwise fizzle (fired
  from the player's own claimed territory), the beam instead pierces through
  claimed/forbidden tiles and resolves on the first unclaimed tile it finds —
  despawning silently if none exists before the map edge. This is close to
  today's auto-selected "inverted mode," reframed as something earned via the
  draft rather than always-on.

**Substrate implication**: internally, beam resolution becomes an *ordered
list* of behavior entries tried in priority order, not a single `bool`. Every
future beam-behavior ability appends an entry to that list. Straight is always
entry 0; Backfill (once drafted) appends as a fallback entry.

### Long-press activation

The "no separate input" principle above covers always-on passive modifiers
(Straight/Backfill's mode is decided by context, not a button). It doesn't fit
abilities that let a player **pay more for a bigger effect on a specific
shot** — Full Draw and Wide Shot (§3) are both like this. Without an
explicit way to declare "this shot is the big one," either the ability is
mandatory every time (removing the flexibility to fire a normal cheap shot)
or it can never be used at all. Resolution: reuse the existing shoot button
(Tab / `/`) with hold-duration detection via `leafwing-input-manager`
(already this project's input crate) — no new key.

- **Tap** (release before a hold threshold, ~150-200ms TBD): a normal single
  Straight/Backfill shot at 1 charge, unchanged from today.
- **Hold past the threshold**: activates whichever cost-scaling ability the
  player has drafted. For **Wide Shot**, this is binary — release after the
  threshold fires the full drafted width at its flat total cost. For
  **Full Draw**, this is graduated — see its entry (#12) for the accrual
  model.
- If both are drafted, holding feeds both independently; releasing fires a
  wide volley where each beam also carries whatever Full Draw budget was
  accrued during the hold. Not specially coded — an emergent composition of
  two independent hold-consumers, not a combo that needs its own logic.

**Which future abilities need this**: only ones structured as "pay X more
for Y bigger effect, player's choice per shot." Everything else in the
current roster is either a passive always-on modifier (Overpenetration,
Splitter, Ricochet, Body Blocker, Impaler, the whole economy family) or
auto-triggered by context with no cost choice (Straight/Backfill's mode,
Contested Ground/Breach's mandatory-when-aimed trigger) — none of those need
explicit activation. Full Draw and Wide Shot are the only two that qualify
today.

**Technical note**: `BeamFired` currently fires on button *press* (a single
discrete input event). This needs to move to firing on *release*, carrying a
charge-level/width parameter determined by hold duration — a real change to
the `inputs` plugin, not just an additive message. The existing
`BeamCharges`-exhaustion gate extends naturally: holding force-releases
(fires whatever's accrued so far) if the charge pool drains mid-hold.

**Getting hit cancels the hold.** If the charging player is hit by an enemy
beam (`on_body_hit`) while mid-hold on Full Draw or Wide Shot, the hold is
cancelled: default is **total loss** — whatever charges were spent building
the budget, or however far into the threshold the hold had gotten, are
simply gone, nothing fires. Standard charge-shot convention (Mega
Man/Zelda-style) — holding for a bigger payoff means real exposure, not a
free action. Softened by **Composure** (#25, §3): forces an early release
instead of total loss, salvaging whatever had been accrued. Body Blocker
does **not** grant immunity to this — it softens the physical consequence
of being hit (no drag, just 1 HP), but the player was still hit, so the
hold still cancels regardless.

Mid-hold parry interaction follows the same tree: if a beam enters the
parry window while charging and the player releases inside that window,
it's a parry (the charge is discarded, since that press is now doing
something else); if they get hit without landing a parry first, the cancel
rule above applies. One coherent decision tree — react in time and convert
to a counter-attack, or fail to react and lose the investment (mitigated
only by Composure).

### Parry (upcoming core mechanic)

**Not yet built** — a planned base-game feature independent of this ability
system, but the roster needs to account for it once it lands. Mechanic: the
**same Shoot button** doubles as parry, no new input. If an incoming enemy
beam is within its parry window relative to the player when they press
Shoot, that press becomes a parry instead of firing a new shot — same
context-decides-the-outcome philosophy as Straight/Backfill (§2), not a
separate control. On a landed parry: the beam's direction reverses, its
speed increases, and it heads back toward the original shooter.

**Three outcomes when a beam approaches a player**: (1) Shoot pressed inside
the window → parry. (2) Shoot pressed outside the window (too early/late) →
fires the player's own normal shot, the incoming beam still hits them with
default consequences. (3) No Shoot press during the approach → same default
consequences. Body Blocker (#7) is the fallback across *both* (2) and (3) —
its job is "insurance for whenever you don't land a parry," not an
alternative to attempting one.

**Ownership vs. effects**: a parried beam's ownership flips to the parrier
(they get the damage/claims it produces), but its cast-time behavior —
mode, remaining bounce budget, whatever abilities were baked into it when it
was fired — is **not** re-evaluated under the parrier's own loadout. It keeps
the original shooter's effects. Accepted trade-off, chosen deliberately: a
player with zero drafted abilities can temporarily wield an opponent's whole
kit for one counter-shot (e.g. parry an Overpenetration-owner's shot and it
still instantly flips an enemy tile). This is the sharpest with aggressive
kits — **a Breach Aggression or Overpenetration player's own beam, if
parried, chains into or flips *their own* territory**, since the beam's
behavior came from their loadout, but the outcome now benefits whoever
parried it. Makes **Unparryable** (§3) close to a necessary tech piece for
those archetypes, not just a nice-to-have, counter-teching parry the same
way Impaler counter-techs Body Blocker.

**Rally**: no hard cap. If the original shooter also parries the return
(and so on), each bounce increases speed further — left to self-limit via
increasing difficulty rather than an explicit rule. Revisit with a cap if
playtesting shows it doesn't actually self-limit (e.g. speed increments
turn out too small to matter).

**Interaction notes** (full detail lives here rather than scattered across
roster entries):
- **Body Blocker/Impaler (#7/#8)**: see "three outcomes" above — Body
  Blocker downgrades any non-parry outcome to its guaranteed 1 HP/no-drag
  stop. Impaler's punish-a-turtle logic is unaffected by parry directly.
- **Overpenetration (#3)**: a parried beam that keeps this effect can
  instantly flip a tile belonging to whoever it's now heading toward —
  including the original shooter's own territory.
- **Contested Ground/Breach (#15/#16)**: a parried beam that keeps these
  effects can open/chain contests into the original shooter's territory for
  free (no charge cost to the parrier) — flag as a likely balance hotspot,
  same weight as the Ricochet+Splitter and Full Draw+Splitter combos.
- **Ricochet/Bank Shot (#5/#21)**: whatever bounce budget remained at the
  moment of the parry carries over unchanged (not refreshed) — consistent
  with "same effect," but worth confirming during implementation rather than
  assuming.
- **Wide Shot (#6) — parry is per-lane, never re-widens.** Wide Shot spawns
  3 independent beam entities at `on_fire` time (§3); it isn't one volley
  object, and a parry only ever touches the single entity actually at the
  defender's tile — the other lanes are on different rows, untouched, still
  the original shooter's. The reflected beam is just that one entity,
  reversed and sped up; it keeps whatever *else* it was carrying (a Ricochet
  bounce budget, a Full Draw claim budget) but does not re-trigger Wide
  Shot's spawn-3-beams effect, since that's a one-time `on_fire` action and a
  parry never goes through `on_fire` — it mutates an existing entity rather
  than firing a new shot.
- **Economy family (Solar Panels/Tithe/Salvage/Full Draw/Capacitor),
  Landmine, Beachhead, Reckoning**: no special interaction — any claims a
  parried beam produces simply accrue to the parrier like any other claim,
  naturally feeding these systems with no redesign needed.

**Rollout note**: since the base parry mechanic ships independently of this
plan, insert a follow-up session once it lands: redefine Body Blocker's
fallback role (above) and build the Parry ability cluster (#28-32, §3).
Don't block the existing 7-session rollout (§7) on it.

## 3. Candidate ability roster (32)

Format: **Name** — effect · trigger · [enabler/payoff/hybrid] · stacks with.

### Beam-behavior modifiers
1. **Straight Shot** (baseline, not drafted) — frontier-extend claim. · on_fire/on_resolve · enabler · everything.
2. **Backfill Shot** (first draftable ability) — fallback infill claim when Straight fizzles from own territory. · on_fire/on_resolve · enabler · Breach, chain pieces.
3. **Overpenetration** — your beam no longer stops or skips at enemy-claimed tiles: it can resolve directly onto one, **instantly** overwriting the claim to you. No countdown, no adjacency requirement, no defender reaction window — breaks the "claimed tiles never change owner" default (§1) outright rather than through a contest. The blunt, immediate answer to a permanent enemy wall; see Contested Ground (#15) for the slower, conditional alternative and why both are worth keeping despite the overlap. **Anti-synergy, not combo**: drafting both is likely wasted value — this ability's instant flip takes priority, so Contested Ground's contest never triggers for the same player. If parried (§2, upcoming mechanic), this effect travels back with the beam — an Overpenetration-owner's shot, parried, can instantly flip *their own* tile. See Juggernaut (#27) to extend the flip past a single tile, instantly resolving multiple consecutive enemy-owned tiles in one shot. · on_resolve (target tile is enemy-owned) · payoff (denial/aggression) · strongest against wide-claimed Solar Economy boards.
4. **Splitter** — on resolve, also claim the two tiles orthogonally adjacent to the landing tile. · on_resolve · payoff · anything that generates resolves; area builds.
5. **Ricochet** — a Straight beam that hits a blocked tile, instead of stopping, first claims its current tile (same "claim if unclaimed" check as a normal stop) then turns 90° and continues. Each bounce is a real `on_claim`/`on_resolve` event, not a free pass-through — Splitter, Tithe, Chain Reaction, and Reckoning's stack all fire per bounce, not just at the final landing tile. Base grants 1 bounce; see Bank Shot (#23) to stack more. Soft cap at 3-4 total bounces recommended (TBD, session 6 balance pass) — uncapped bounces stacked with Splitter is a real degenerate-combo risk (each bounce = landing tile + 2 Splitter neighbors, multiplying per bounce). **Turn direction**: exactly one case has a deterministic forced direction — the *first* turn of an outer Wide Shot beam, which always turns *away* from center using its lane-offset tag, so a widened spread can't collapse back into itself. Every other turn (a solo beam's turn, the center beam's turn, or an outer beam's second-or-later bounce via Bank Shot) uses the same generic rule: **prefer whichever perpendicular side is unclaimed ground**, tie-break clockwise if both sides qualify, fall back to valid-but-claimed ground if neither side is unclaimed, fall back to a normal stop (no turn) if neither side is even valid ground. Not gated behind Wide Shot — remains a fully standalone reach tool, Wide Shot just gives its beams a cleaner, guaranteed-divergent first turn when both are drafted. **Implementation note**: all beams from one Wide Shot volley tick in the same step — if claim mutations are deferred (`Commands`), a sibling's claim made in that same tick may not be visible yet to another sibling's same-tick turn decision, a narrow race, not a crash. Resolve claims synchronously within a tick if implementation cost allows; otherwise accept as a low-impact edge case. · on_step, claims on each turn · hybrid (guarantees its own value per bounce, and multiplies every other claim-triggered payoff) · Splitter (hard combo), Bank Shot, Wide Shot (clean divergent turns, all 3 beams keep full value), reach builds.
6. **Wide Shot** — 3 parallel beams: one at your own tile, one on each side perpendicular to the firing direction — at 3x charge cost (linear per-beam pricing). Each beam is tagged with a lane-offset (its side relative to center) at spawn — this is what lets Ricochet (#5) give outer beams a deterministic outward turn instead of an arbitrary/ambiguous one, guaranteeing the spread can't converge back on itself. Stackable, same pattern as Bank Shot/Ricochet: each additional copy widens the spread by one more beam per side (5, 7, 9...), cost scaling linearly with total beam count. Skip a side beam if its origin tile isn't valid ground (e.g. player against a map edge). Activated via long-press (§2): tap fires a normal single shot as usual, holding past the threshold fires the full drafted width at its flat total cost. · on_fire (long-press, binary) · hybrid · Splitter, Chain Reaction (more simultaneous claims across a wider front), Ricochet (clean divergent bounces), Full Draw.
7. **Body Blocker** — defensive, held by the potential target, not the shooter. Default behavior (see `on_body_hit`, §1) is that any beam hitting a body deals 1 HP and drags the victim tile-by-tile, re-hitting every tick with input locked, until the beam resolves or knockback fails — a potentially long, inescapable HP drain. Body Blocker overrides this for its holder: an enemy beam that hits you stops immediately on a single 1 HP hit, no knockback/drag, and despawns without resolving/claiming anything beyond your tile. Trades a guaranteed 1 HP tax for planting your feet and denying the attacker's claim. Once parry (§2, upcoming mechanic) ships, this becomes the fallback for any non-parry outcome — a missed timing or no attempt — rather than a standalone alternative to parrying. · on_body_hit (as the target, overriding the attacker's default resolution) · payoff (denial) · Iron Wall, Landmine.
8. **Impaler** — attacker-side punish for Body Blocker turtling: if your beam's hit is the killing blow against a Body-Blocking defender, you don't get a charge/tile reward (worthless — the round resets on that same kill) — instead your **next draft offer is guaranteed to include a Rare**. Converts Body Blocker into a real bluff at low HP instead of a free wall. · on_death · payoff, persistent-layer reward · Body Blocker (counter-combo, not same-owner).

### Charge-economy modifiers
9. **Solar Panels** — regenerate 1 charge every T seconds per K owned tiles. · on_charge_regen (new tick) · enabler · all payoffs; the economy backbone.
10. **Salvage** — refund 1 charge whenever a beam despawns claiming nothing (Backfill misses). · on_resolve(null) · enabler · Backfill.
11. **Tithe** — each real claim (enemy/neutral flip) refunds 0.5 charge (rounds/accumulates). · on_claim · enabler · Straight expansion, Splitter.
12. **Full Draw** — activated via long-press (§2). While held, the beam is "charging": each beam-step tick (62.5ms) held costs 1 charge by default (reducible — see Capacitor, #24) and adds 1 tile to a claim budget. Release to fire: the beam first resolves at its **normal** Straight/Backfill point — the same tile a plain 1-charge shot would claim — guaranteeing Full Draw is never worse than not charging at all, then spends the remaining budget extending the claim in whichever direction is actually useful for that mode: **backward toward the origin for Straight** (claiming the N tiles leading up to the resolve point — pushing your frontier out toward the obstruction, the contested/valuable ground, rather than padding tiles near where you're already standing), or **forward past the resolve point for Backfill** (claiming N tiles deeper into the freshly-opened territory it just found, rather than backward into the claimed/forbidden ground it pierced through, which it can't claim anyway). Value scales monotonically with hold duration — every extra tick is strictly more claimed ground, never less. Stops early if it runs out of room in that direction (open ground ends) before exhausting the budget — remaining budget is simply unused, same as any over-commitment risk. Charges deduct in real time as you hold, so you can't accrue more budget than you can afford, and you can watch the lane ahead and release accordingly. Soft cap recommended on max hold duration/budget regardless of banked charges (TBD, session 6 balance pass) — same category of risk as the combo below. **Splitter/Chain Reaction combo**: each tile Full Draw claims is a real `on_claim`/`on_resolve` event, same as Ricochet's bounces — Splitter triggers on every one of them. A well-charged shot down a long open lane claiming 6 tiles directly, with Splitter claiming 2 neighbors per landing, is 18 tiles from one shot; Chain Reaction compounds further. Bigger and less bounded than the Ricochet+Splitter+Bank Shot combo (#5) — no bounce cap, ceiling is just lane length and charges spent — flag hard for the session 6 balance pass. **Contested Ground interaction** (#15/#16): on Straight, works cleanly — the enemy tile still becomes a pending contest exactly as a plain Contested Ground shot would, and the extra budget extends backward into ordinary unclaimed ground, claimed instantly. On Backfill, the beam stops/contests at the first enemy tile per Contested Ground's rule, and the forward budget would need to pass through that same pending tile — **this requires Breach also being drafted**: without Breach, the same beam can't exploit its own fresh contest (forward budget wasted, same as any over-commitment); with Breach, the pending-tile-is-passable permission extends to this beam too, so releasing chains through *every* enemy tile within budget, contesting each one simultaneously in one action instead of needing separate manual shots per hop — Full Draw becomes the one-button execution of a Breach chain, not a shortcut around drafting it. Each contested tile deeper in that chain checks its own adjacency independently at its own 2s mark, and a still-pending neighbor doesn't count as owned — so the deeper links are naturally riskier with no extra rule needed for that. · on_fire (long-press, graduated) · payoff, scales with hold duration/lane length · Solar Panels, Tithe (fund the pool this spends), Capacitor (#24, reduces per-tile cost), Splitter, Chain Reaction (major combo, flag for balance), Contested Ground + Breach (chain-execution capstone, requires both).
13. **Frugal Frontier** — first shot each 5s costs 0 charges. · on_fire (cooldown) · enabler · low-charge/control builds.
14. **Battery Cap** — +50% max charges, regen sources -25%. · passive · hybrid · burst/Full Draw builds; anti-synergy with Solar Panels.

### Tile-state/contest modifiers
15. **Contested Ground** — by default, flipping an enemy-claimed tile is impossible in any form (§1); this ability introduces that capability, but conditionally. Trigger is the *first enemy-owned tile* a beam's path would otherwise stop at (Straight) or skip past (Backfill) — mandatory when it fires, decided entirely by where you aim, no separate input. Instead of blocking/skipping, the beam resolves there and starts a 2s countdown; if you still **own** a tile adjacent to it when the countdown ends, it flips to you, otherwise nothing happens and the charge is spent for no gain. **Ownership check, not player position**: this is a `ClaimedTile` lookup at the 2s mark, not a requirement to stand nearby — the attacker can walk away and fire other Straight/Backfill shots freely the moment the contest starts, and it resolves independently in the background based on board state. No interference with normal shot behavior. While pending, the tile is neutral for *other* beams' travel. **Intentionally weak/risky as a standalone pick** — Overpenetration (#3) does the same flip instantly and unconditionally, so nobody should draft this for the flip itself. Its real job is the pending-neutral travel window, which is the only thing Breach can hook into (Overpenetration's flip is instant and solid, creating no window) — this is a pure enabler that's a trap pick without Breach, by design. **Interactions**: Splitter triggers on the contest's start (claims its 2 orthogonal neighbors if unclaimed, giving a small guaranteed floor even if the contest later fails); Chain Reaction triggers on the eventual flip like any other claim; Ricochet just passes through a pending tile rather than turning at it; Full Draw chains through multiple enemy tiles at once if Breach is also drafted (see #12). **Anti-synergy with Overpenetration (#3)**: drafting both is likely wasted value, not a combo — Overpenetration's instant unconditional flip would take priority the moment a beam reaches an enemy tile, so Contested Ground's contest never gets a chance to trigger for that player. They're alternative answers to the same problem for different playstyles, not meant to be stacked. · on_resolve (first enemy-owned tile encountered) · enabler (Breach only; weak alone) · Breach.
16. **Breach** — a beam passing through a contested tile can fire a follow-up beam straight through it, opening a *new* contest deeper in enemy territory, and chain further with additional beams — each one costs a charge and is its own live 2s window that needs an owned adjacent tile at expiry (ownership check, not player position — see Contested Ground, #15). This is a deep-strike payoff, not a single-tile flip: a full chain (3-4 charges) plants contests across a whole lane of enemy territory in one push, reshaping the board far beyond what one Overpenetration flip reaches. **Not redundant with Overpenetration — differentiated by risk and scale, not gated by cost.** Overpenetration is the safe, cheap, incremental border-grinder (one tile, one charge, no risk); Breach is the risky, expensive, explosive incursion (multiple tiles, multiple charges, each one a live gamble). Both are legitimate answers to the permanent-wall problem for different playstyles. If parried (§2, upcoming mechanic), a Breach-owner's shot can chain into their *own* territory for free — flag for balance alongside the other major combos. · on_step (chained via Contested Ground's neutral-travel state) · payoff, scales with chain length · Contested Ground (defines the archetype; without it, Breach has nothing to hook).
17. **Landmine** — owned tiles not stood on (by their owner) in 10s "arm," dealing double damage on the next enemy damage-tick they trigger, then reset (spent, needs another 10s of neglect to re-arm — not a sustained multiplier for a stationary victim). **Visible to both players** — Tyled is shared-screen local multiplayer (one Main/Viewport/HUD camera setup, no per-player render target per `CLAUDE.md`), so "owner-only" visibility isn't technically possible; hiding it from both would be strictly worse since even the owner needs to know which tiles are armed to zone/bait intentionally. Armed tiles get a subtle marker (`LandmineArmedEffectTarget`, same `*EffectTarget` pattern as contest countdowns/regen pulses, §6) — a faint pulse/tint rather than a blaring indicator, so avoiding one rewards map-awareness rather than being automatic. An enemy can route around a visible armed tile deliberately, but not when dragged onto one via `on_body_hit` knockback (§1) — that combo is untouched by visibility, since the victim doesn't control their path during a drag. Cashes in a stack from ordinary wide-claim play (time-since-visited, per tile) rather than a dedicated enabler, same shape as Reckoning — Solar Economy pieces are de facto enablers here since more territory means more tiles nobody's standing on. · on_damage_tick · payoff · Solar Panels (wide board), zoning builds.
18. **Chain Reaction** — a claim with ≥3 owned neighbors auto-claims one random neutral neighbor; that auto-claim is itself a real `on_claim` event, so if *it* also ends up with ≥3 owned neighbors, it cascades again, and so on. No hard cap: naturally self-limiting, since each hop consumes a neutral tile (a finite, depleting local resource) — the cascade can only run as far as there's still-neutral ground embedded in an increasingly dense cluster, then terminates on its own. Flag for the session 6 balance pass regardless (a large checkerboard-style neutral pocket inside dense territory could still cascade a long way), same category as the other combo risks, but not an obvious hard-cap candidate the way Ricochet's bounces were. · on_claim, recursive · payoff · Splitter, Solar Panels, wide boards.
19. **Beachhead** — on enemy death, burst-claim every tile in a radius around where they died — forcing the claim regardless of current status (enemy-owned, neutral, forbidden excepted) — and these claims survive the round's full reset, giving you a head start of already-owned territory at the start of the next round. Bypasses "claimed tiles never change owner" (§1) as an area effect, the third way to flip an enemy tile alongside Overpenetration (single-tile, per-shot) and Contested Ground (single-tile, conditional) — this one triggered by a kill instead of a beam. Reuses the round-reset exception hook (§6) with a claimed-territory payload. Radius size TBD, session 6 balance pass — the base radius is a natural cap on its own, though Fallout (#26) can stack it further (see that entry for the resulting per-tile cancellation resolution once radii can differ in size). Reads as a universal kill-reward capstone for any archetype that can secure kills, not an aggression-specific denial tool. **One victim ability, two possible modes**: see Last Stand (#20) — undrafted-modifier claim-mode cancels via conflicting ownership; with Salted Earth (#21) also drafted, Last Stand transforms to denial-mode and naturally overrides this claim instead of needing a bespoke cancellation check. No anti-synergy between #20/#21 since they're base-ability-plus-modifier, not two independent competing picks. · on_death (enemy — you're the killer) · payoff (conquest capstone, persistent-layer via the reset exception) · any build that can reliably secure kills (Iron Wall's chip damage, Breach Aggression's pressure, Chain Cannon's burst combos).
20. **Last Stand** — on your **own** death, burst-claim the same radius around your death position (identical scope to Beachhead) for *yourself* instead of denying it to everyone — forcing the claim regardless of current status, forbidden ground excepted, and these claims also survive the round's full reset. A consolation foothold: even losing the exchange leaves you with real territory heading into the next round. Standalone value independent of the pairing — works the same whether or not the killer has Beachhead. **The cancellation rule (claim-mode only — see #21 for the alternative)**: if the killer has Beachhead *and* the victim has Last Stand (not transformed), both attempt to burst-claim the *same* tiles for *different* owners at the same moment — the conflicting claims cancel, and the radius simply reverts to a normal wipe, as if neither ability were drafted. Requires the death-resolution system to check **both** players' ability descriptors (killer's Beachhead, victim's Last Stand, and whether Salted Earth also modifies it) — same cross-entity lookup shape as Body Blocker's target-side check (§6). · on_death (self) · payoff (consolation capstone, persistent-layer via the reset exception), doubles as counter-tech against Beachhead when both are present · any build wary of losing a risky exchange, but wants the head-start upside for themselves rather than pure denial.
21. **Salted Earth** — requires Last Stand already drafted; does nothing alone. When present, **it transforms Last Stand's resolution from claim-for-self into deny-to-everyone** — same radius, same reset-survival, opposite intent, "make sure nobody profits" instead of "I profit from this myself." Since Salted Earth doesn't fire on its own trigger, there is no scenario where both effects exist simultaneously to conflict — it *is* the mechanism that flips which effect Last Stand has, not a second competing ability. Cost asymmetry is intentional: the claim version costs 1 slot (Last Stand alone), the denial version costs 2 (Last Stand + this) — reasonable, since "always good regardless of opponent" should be cheaper than "specifically counters an opponent's Beachhead." **Interaction with Beachhead when transformed (corrected 2026-07-09 — this was wrongly described as size-independent)**: no bespoke check needed for *whether* denial blocks a claim — process the hazard-marking before any claim attempt, and Beachhead's claim fails against the now-forbidden-like tiles the same way it fails against ordinary forbidden ground. But this only holds *within Salted Earth's own radius*. If Beachhead's radius is larger (e.g. the killer stacked Fallout, #26, and the victim didn't), the ring of tiles inside Beachhead's radius but outside Salted Earth's was never marked forbidden at all, so Beachhead's claim succeeds there completely normally. Same per-tile shape as Last Stand's claim-mode cancellation, not an exception to it. · on_death (modifies Last Stand's resolution, no trigger of its own) · enabler (qualitative mode-switch rather than Bank Shot/Capacitor's numeric stacking, but same "does nothing alone" shape) · Last Stand (required), counter-tech specifically against Beachhead-drafting opponents.

### Match-end payoffs
22. **Reckoning** — when the board saturates (~95% of tiles claimed), deal damage to the opponent proportional to your owned-tile count. A second, non-aggressive win path ("paint the board") alongside direct-damage aggression; drafting it opt-in keeps the two playstyles balanced against each other instead of requiring a hand-tuned global formula. · on_board_saturated · payoff, cashes in a **stack** (owned-tile count from ordinary play), not a dedicated enabler — see the "roles are relative" note in §1 · any ability that claims more tiles becomes a de facto enabler for it (Splitter, Chain Reaction, Wide Shot, the whole Solar Economy archetype).

### Stack modifiers
23. **Bank Shot** — +1 max Ricochet bounce per copy drafted. Pure homogeneous-stack enabler in the same family shape as Solar Panels/Tithe/Salvage/Frugal Frontier (multiple same-family abilities additively building one resource) — here the resource is "max bounces per beam" instead of charge income. Does nothing without Ricochet already drafted; exists purely to extend its chain. Soft cap recommended at 3-4 total bounces (see Ricochet, #5) — tune during the session 6 balance pass rather than leaving unbounded. · on_step (extends Ricochet's bounce budget) · enabler · Ricochet, Splitter (the actual payoff multiplier), reach/Chain Cannon builds.
24. **Capacitor** — reduces Full Draw's per-tile hold cost by 0.25 per copy drafted (1.0 → 0.75 → 0.5 → 0.25 charge per tile claimed while charging). Pure enabler, does nothing without Full Draw drafted — same "does nothing alone, extends one specific payoff" shape as Bank Shot does for Ricochet, except this stack reduces a payoff's *cost* instead of increasing its budget/reach. First cost-reduction enabler in the roster — everything else in the economy family generates more charges rather than making a specific payoff cheaper to run, so this is a genuinely new lever, not a reskin. · on_fire (modifies Full Draw's per-tick hold cost) · enabler · Full Draw only.
25. **Composure** — softens the default long-press cancel rule (§2: getting hit while charging Full Draw/Wide Shot is a total loss). With Composure, getting hit forces an early release instead — you still get whatever you'd built (a smaller Full Draw shot at current claim budget, or a normal single shot for Wide Shot if you hadn't cleared the widen threshold, the full width if you had). Pure enabler, binary (no stacking — there's no natural "more" of this, unlike Bank Shot/Capacitor's numeric scaling), does nothing without an active long-press ability to protect. Distinct from Body Blocker: Body Blocker protects against the attacker's claim/knockback, Composure protects your own charge investment — different axis, a player can want one without the other. · on_body_hit (while mid-hold) · enabler · Full Draw, Wide Shot.
26. **Fallout** — +1 tile to the shared death-burst radius per copy drafted, benefiting whichever of Beachhead or Last Stand (#19/#20) the drafting player has — Last Stand's radius stacks the same whether or not it's transformed by Salted Earth (#21), since that only changes claim-vs-deny, not scope. Single shared radius parameter across the family rather than being tied to one specific ability like Bank Shot/Capacitor are. Does nothing without at least one of Beachhead/Last Stand drafted. **Differing radii change both modes the same way (corrected 2026-07-09 — the denial-mode case was wrongly described as size-independent)**: if only the killer or only the victim has stacked Fallout, the two radii are no longer the same size, so Beachhead's interaction with Last Stand resolves **per-tile** rather than as a single switch, in *both* claim-mode and denial-mode: within the **overlap** of the two radii, the smaller-radius side's rule wins (opposing claims cancel in claim-mode; denial always blocks a claim attempt in denial-mode, since it's processed first) — but any tile inside Beachhead's radius that falls **outside** the other radius was never touched by the other effect at all, so Beachhead's claim succeeds there completely normally, regardless of mode. Salted Earth's "denial wins" rule only holds *within its own radius* — it was wrongly described as unconditional; a killer with a bigger Fallout-stacked radius than the victim's Salted-Earth-modified Last Stand still claims the ring beyond it. This makes Fallout genuinely valuable on either side against either victim mode: the killer keeps whatever ring the victim's radius doesn't reach, and the victim (in either mode) keeps or denies whatever ring the killer's radius doesn't reach. Soft cap recommended (TBD, balance pass) — burst area grows roughly quadratically with radius, and it compounds with Splitter/Chain Reaction the same way the base radius already does, same combo-risk category as the roster's other stacking abilities. · on_death (whichever of #19/#20 triggers) · enabler · Beachhead, Last Stand (shared support, not tied to just one).
27. **Juggernaut** — +1 additional consecutive enemy-owned tile instantly overwritten per copy drafted, extending Overpenetration (#3). Does nothing without Overpenetration drafted. Instead of resolving on the first enemy-owned tile and stopping, the beam keeps traveling and instantly overwrites each of the next N consecutive enemy-owned tiles in the line (N = 1 + copies drafted), reverting to normal resolve/stop behavior the instant it reaches a tile that isn't enemy-owned (neutral, self-owned, forbidden) or the budget runs out. Each flip in the run is a real `on_resolve` event — Splitter, Tithe, Chain Reaction all fire per tile in the run, same principle as Ricochet's bounces (#5), not a free pass-through. **Deliberately scoped to Overpenetration only, not Contested Ground** — the equivalent "chain through multiple consecutive enemy tiles in one action" outcome for Contested Ground already exists via Full Draw (#12) + Breach (#16): Full Draw's linear resolve-then-extend budget is the same *shape* of mechanic (extend along the beam's own travel line), so a second, cheaper, always-on way to do the identical thing would make that combo's "one-button execution of a Breach chain" pointless. Bank Shot (#23) isn't folded in here either, for the opposite reason: Ricochet's bounces are 90° turns onto new segments, a genuinely different shape than "more of the same line," so it doesn't overlap with Full Draw's model the way Contested Ground's chaining did. Soft cap TBD, session 6 balance pass — same combo-risk category as Ricochet+Splitter (every additional tile in the run is a fresh claim event, multiplying per copy) — though naturally self-limiting the way Chain Reaction is: a run only extends as far as contiguous enemy territory actually exists in that line, a finite, board-state-dependent resource, not an unconditionally growing one. · on_resolve (extends how many consecutive enemy tiles one shot instantly flips) · enabler · Overpenetration (required), Splitter/Chain Reaction (combo-risk multiplier), Breach Aggression archetype.

### Parry modifiers (provisional — depend on the base parry mechanic shipping first; see §2)
28. **Riposte** — a landed parry deals a small bonus effect (e.g. +1 direct HP to the original shooter) separate from whatever the reflected beam itself does on top. · on_parried · payoff · any parry attempt; the direct cash-in for landing one.
29. **Extended Window** — lengthens the parry timing window. Pure accessibility/reliability enabler, same shape as Bank Shot/Capacitor (stacks a specific mechanic's margin rather than a shared resource). · on_parry_window · enabler · all parry abilities.
30. **Perfect Parry** — a narrower nested timing window inside the normal one; landing it grants something beyond a normal parry (bigger speed multiplier on the reflected beam, or a guaranteed claim regardless of tile state). Skill-expression tier on top of the base mechanic. · on_parried (within the tighter sub-window) · hybrid · Riposte, Extended Window (a longer window makes the nested perfect-timing sub-window easier to find).
31. **Parry Refund** — a landed parry refunds a charge to the parrying player. Ties parry into the charge economy. · on_parried · enabler · Solar Economy family.
32. **Unparryable** — your outgoing beams can't be parried. Close to a necessary tech piece once parry ships (§2) rather than a nice-to-have — without it, an aggressive kit (Breach, Overpenetration) risks being turned against its own owner by a parry. Same counter-tech shape as Impaler countering Body Blocker: a defensive universal mechanic gets an attacker-side answer. · on_fire · enabler (denies the opponent's parry option entirely) · Breach Aggression, Chain Cannon, any archetype whose own beam effects would be dangerous if turned around.

## 4. Archetypes (4, deliberate rock-paper-scissors — B beats A, C beats B, A beats C; D is a high-variance wildcard)

- **A — Solar Economy** (Solar Panels + Tithe + Frugal Frontier → Full Draw/Chain Reaction, or → Reckoning as an alternate win-condition capstone): wide-claim engine, snowballs regen, dumps into board-flooding or cashes the board out directly via Reckoning. Full Draw is a hold-to-charge payoff (§3) that reads the lane in real time rather than a blind flat-cost gamble, and Capacitor cheapens it further — with Splitter also drafted, this is a genuine cross-archetype bridge into Chain Cannon (D), not just an economy payoff. Weak to early aggression.
- **B — Breach Aggression** (Backfill + Contested Ground + Breach): don't grind the border one tile at a time — commit several charges in one push to chain contests deep into enemy territory, reshaping their whole flank at once. High commitment, high risk (each chained contest can whiff independently if adjacency slips); denies opponent's economy outright when it lands. Loses if ignored while the opponent races board control elsewhere — the payoff is concentrated, not attritional. Once parry ships (§2), this archetype is the one most exposed to having its own kit turned against it — Unparryable (§3) is close to a required tech slot here, not optional.
- **C — Iron Wall** (Body Blocker + Landmine + Straight Shot, with Impaler as the counter-punish an *attacker* takes against it): contiguous frontier, plant your feet to deny beams reaching past you, zone damage. Beats Breach (bodies wall the lanes); loses to Solar Economy's flood. Body Blocker/Impaler is a cross-role interaction, not a same-owner combo (see §3) — Iron Wall is defined by the defender's Body Blocker, and Impaler belongs to whoever's attacking an Iron Wall player, not to Iron Wall itself. Once parry ships (§2), this is also the archetype's natural home for Riposte/Extended Window/Perfect Parry — Body Blocker is the fallback for a missed parry, so a build leaning into reliable defense wants both the fallback and better odds of not needing it.
- **D — Chain Cannon** (Splitter + Ricochet + Bank Shot + Chain Reaction + Wide Shot, funded by Solar Economy pieces): Ricochet+Splitter is a hard combo (§3) — every bounce claims and triggers Splitter, and Bank Shot stacks more bounces onto the same beam, so a single well-aimed shot can cascade across a whole dense cluster. Wide Shot adds a second axis of scale, and the two compose cleanly rather than fighting each other: outer beams always turn away from center (§3), so a widened, bouncing spread guaranteedly diverges instead of risking beams converging back into each other's claimed territory — full Ricochet value on all 3+ beams at once. Explosive when the board is cluttered enough to bounce/spread through, brick-y when sparse. Beachhead (§3) fits loosely here too — Chain Cannon's cascading claims are exactly the kind of build that can convert a kill into a big burst-claim radius — but it's a universal kill-reward capstone, not exclusive to this archetype. Also exposed once parry ships, same reasoning as Breach Aggression — a heavily-loaded beam (Ricochet+Bank Shot+Splitter) is exactly what you don't want reflected back at you.

## 5. Acquisition

**Phase 1 (build now): symmetric round-draft.** No pre-match ability choice
(Straight is baseline for everyone, see §2). Segment a match into rounds,
ending on first-to-N-tiles, a timer, or a kill — a kill ends the round
outright. A round boundary is a **full reset**: board and charges both wipe;
only round wins/kills accumulate toward match score and drafted abilities
persist. Between rounds, both players draft from the **same** pick-1-of-3
offer — fair for versus play, doesn't interrupt real-time pace mid-round.

**Phase 2 (future game mode, not in the initial build): asymmetric personal
shops.** Each player sees their own distinct offer between rounds — more
build diversity and variance, less strictly fair. Keep the ability-descriptor
data model (§6) generic enough that swapping the offer-generation strategy
(symmetric vs. per-player) doesn't require touching ability logic — it's a
change to the *offer generator*, not to how abilities are represented or
resolved.

## 6. Technical integration sketch (prose, no code)

**The pinch point, named first:** `Beam.inverted: bool` must become an ordered
list/enum-set of beam-behavior descriptors resolved in priority order (see
§2). This is the true first slice — it unblocks every beam-behavior ability
in the roster.

**Two architectural approaches for "an ability":**
- **Component bundle + own systems** (idiomatic Bevy, but N abilities = N
  systems = scheduling sprawl, and "what does this player have" isn't easily
  introspectable/serializable for a draft UI).
- **Data descriptor + generic resolver** (`Vec<AbilityDescriptor>` per player,
  a handful of resolver systems keyed by trigger type; data-driven,
  hot-reloadable via the existing `file_watcher`, ordering is explicit list
  order; risk of the effect enum becoming an unreadable god-match).

**Recommendation: hybrid, biased to the data-descriptor approach.** Beam
behaviors and economy modifiers are data descriptors read by resolver systems
in the beam/damage plugins. Reserve component-bundles for abilities needing
bespoke query shapes (Landmine's per-tile timers, Beachhead's radius
burst).

**New messages needed** (additive, fit the existing message-driven pattern):
- `ChargeSpent{owner, amount}` / `ChargeRegen{owner, amount}` — currently a
  silent decrement inside the beam plugin.
- `TileClaimed{position, old_owner, new_owner}` — distinguishes a real flip
  from a no-op resolve; `BeamResolved` alone can't tell them apart.
- `on_body_hit`-adjacent: today `apply_beam_damage` (damage plugin) already
  owns this collision check and unconditionally applies knockback. Body
  Blocker requires that system to consult the *target's* ability descriptors
  (not the shooter's) before applying knockback — a cross-entity resolver
  lookup that doesn't exist yet, distinct from the shooter-side resolvers
  everything else in the roster uses. Worth its own message
  (`BodyHit{beam, target}`) so the override can live in a resolver rather
  than being inlined into `apply_beam_damage` directly.
- A throttled on-step/collision message for Ricochet/Breach — emit only when
  at least one such ability is active, to avoid per-tick spam.

**Heaviest single item**: Contested tiles need a third `ClaimedTile.owner`
state (contested + timer + pending owner) — touches `MapInfo`, the beam
travel check, and the damage plugin's "standing on enemy tile" query.

**Round-reset exception for Beachhead**: the round-transition system (built
in session 3) resets board + charges unconditionally. Beachhead (§3) needs
that system to support one explicit carve-out — a set of tiles marked as
surviving the wipe, owned by whoever last claimed them via the burst, rather
than reverting to unclaimed. Design this as a deliberate exception hook in
the reset system from session 3 onward, even though Beachhead itself isn't
built until session 6, so the reset code doesn't need retrofitting later.
Generalize the hook to "these tiles keep their post-burst state across the
reset" rather than hardcoding "impassable" — the payload may evolve again.

**Visuals**: `BounceEffect`/`WaveEffectTarget` is already decoupled from beam
logic via messages, so new behaviors mostly reuse it. New ability feedback
(regen pulse, contest countdown) follows the same `*EffectTarget` pattern used
today.

## 7. Staged rollout (session by session, smallest viable slice first)

1. **Behavior-list refactor (no new content).** Replace `Beam.inverted: bool`
   with an ordered behavior list. Straight Shot is entry 0, always present,
   for every player — no pre-match pick. Emit `TileClaimed`, `ChargeSpent`,
   `ChargeRegen`. Outcome: identical gameplay to today, ability-ready
   substrate. Update `backlog/docs/doc-7 - 006-Beam-plugin.md`.

2. **Minimal ability container + Backfill as the first real ability.** Add the
   player-side ability descriptor list and the `on_resolve`/`on_claim`
   resolver. Ship Backfill Shot as a data descriptor (today's "inverted"
   behavior, reframed as an appended fallback entry) — hardcoded onto players
   for now, no draft yet, to prove the descriptor/resolver plumbing end to
   end.

3. **Draft loop.** Round segmentation (first-to-N-tiles, timer, or a kill) +
   symmetric between-round pick-1-of-3 UI reading the ability list as data.
   Round boundary does a full reset of board + charges (§1/§5); include the
   Beachhead exception hook in the reset system now (§6) even though the
   ability itself lands in session 6. Backfill Shot becomes acquired, not
   hardcoded. Add Solar Panels + Full Draw to prove the enabler→payoff loop
   is draftable too.

4. **Economy cluster.** Add Tithe, Salvage, Frugal Frontier, Battery Cap —
   Solar Economy becomes a real, playtestable archetype.

5. **Contested tiles + Breach cluster.** The heavy data-model change: third
   tile state in `MapInfo`, contest timer, Breach/Contested Ground. Isolated
   to its own session because it touches beam travel + damage queries.
   Delivers the Breach Aggression archetype.

6. **Combat/area clusters + capstones.** Body Blocker (needs the
   cross-entity `BodyHit` resolver, §6 — checks the *target's* abilities, not
   the shooter's) + Impaler (guaranteed-Rare-offer reward, persistent-layer
   only), Splitter/Ricochet/Chain Reaction, Landmine, Beachhead (uses
   the reset-exception hook built in session 3), and **Reckoning** (needs the
   new `on_board_saturated` trigger — a cheap tick comparing claimed vs.
   total ground tiles). Mostly additive resolver effects or small
   component-bundle abilities. Balance pass across all four archetypes, with
   particular attention to Reckoning vs. direct-damage aggression parity
   (the original design worry the ability came from), and to Body
   Blocker/Impaler as a cross-role counter-pick rather than a same-owner
   combo.

7. **Ongoing.** After session 6 the machinery is done; new abilities become
   descriptor data + occasional resolver arms. Keep `backlog/docs/` in sync
   per plugin as each session lands. Asymmetric personal-shop game mode
   (§5 Phase 2) can be picked up independently once the descriptor model is
   proven.

## Open threads for later

- **Consumables/enhancers layer.** Balatro separates permanent build pieces
  (Jokers) from one-time-use resources (Tarot/Planet/Spectral cards,
  22+12+18). Everything in §3 so far is Joker-equivalent (persistent,
  passive). Not yet explored: whether Tyled needs an analogous consumable
  layer — a limited-use pickup/item spent for a one-off effect (e.g. an
  instant extra charge, a one-shot forced tile flip, a temporary behavior
  swap) — as a separate resource track from drafted abilities. Revisit after
  the ability substrate (sessions 1-3) is playable, so there's a feel for
  whether the ability layer alone provides enough moment-to-moment texture or
  whether a consumable layer is needed on top.

- **Direct-HP payoffs are almost entirely unexplored.** Riposte (#28) is
  currently the *only* ability in the roster whose payoff touches HP
  directly (+1 HP on a landed parry) — everything else pays off into tile
  claims, charges, or board state, with HP damage otherwise only occurring
  through base mechanics (`on_body_hit` knockback, standing on an
  enemy-owned tile). Worth a dedicated brainstorm later: a whole design
  space of direct-damage enablers/payoffs (e.g. abilities that build toward
  a burst of HP damage, HP-cost resources, execute-style payoffs at low
  enemy HP) hasn't been touched at all. Revisit once the roster's
  tile/charge economy is playtested, so any HP-focused additions are
  weighed against a working baseline instead of guessed at cold.

- **§4's archetype balance is implicitly tuned for one unstated board
  size.** The rock-paper-scissors triangle (B beats A, C beats B, A beats C)
  and Chain Cannon's (D) "explosive when cluttered, brick-y when sparse"
  variance are both board-density arguments, and density is a function of
  board size, which the doc never pins down — the live map
  (`assets/level2.tmx`) is 30×30 (900 tiles), but nothing here was reasoned
  against that number specifically. Concretely: Chain Cannon's cold-start
  "brick" window (time for territory to build up enough density to
  bounce/cascade through) scales with board size directly. Iron Wall (C) is
  a single body defending one lane — its coverage is a much smaller fraction
  of a large board's perimeter than a small one's, which also reweights the
  A-beats-C leg (Solar Economy's flood matters more, relatively, the bigger
  the board). Reckoning's ~95% saturation threshold is a fraction, so a
  bigger board means more absolute tiles to claim — more time for both A's
  economy and D's density to develop, pulling in opposite directions on
  whether that helps or hurts either. Breach's push-depth cost also scales
  with distance to the target lane. None of this is testable in the abstract
  — revisit once sessions 1-5 are playable on the actual 30×30 map, and
  treat any of §4's stated matchups/variance claims as unverified until then
  rather than settled.

- **Wide Shot and Full Draw already bind to the same long-press input with
  no stated resolution if both are drafted.** Wide Shot (#6) reads a
  hold-past-threshold as binary (release after the threshold fires the full
  drafted width at flat cost); Full Draw (#12) reads the same hold as
  graduated (charge accrues every tick held, released budget extends the
  claim). Both are on the shoot button (§2, Long-press activation) and
  nothing in either entry says what a single hold means once a player has
  both — does it do one, the other, both at once, or does drafting the
  second ability change what holding means for the first? This surfaced
  while auditing Full Draw's overlap with other reach-extending abilities
  (see Juggernaut, #27, and its note on why Contested Ground's chaining was
  folded into Full Draw+Breach instead of staying a separate stack
  modifier) — that audit is exactly the kind of check this conflict needs
  too, and it's a higher-priority gap than any further stack-modifier
  consolidation since it's a live ambiguity in an already-committed input
  scheme, not a hypothetical future one. Resolve before session 3
  (Full Draw/Wide Shot's landing session, §7).
