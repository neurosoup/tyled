# HUD Redesign — branch setup + commit hygiene

## Context

The HUD redesign is already underway by hand: the working tree on `main` has
uncommitted edits to `assets/hud2.tmx`/`.png`/`.aseprite` (new `hud-bars.tsx`
tileset wired in — segmented vertical HP-bar tiles replacing the old
scale-tweened bar object, digit groups repositioned, some layers locked) plus
two orphan assets not yet referenced anywhere (`hud32.aseprite`,
`palette10.aseprite`). None of this has a home on a branch, and it's tangled up
in the working tree with unrelated `.cs/` session-bookkeeping changes
(`.cs/README.md`, `.cs/timeline.jsonl`). Goal: give the redesign a proper
branch and a clean commit history, then wire the Rust side to actually consume
the new bar tileset instead of leaving it inert art.

Per your answers: branch off `main` as-is (no stash dance), and scope includes
code wiring — not just an asset-only commit.

## Part A — branch + commits (ready to execute now)

1. `git checkout -b hud-redesign` from current `main` HEAD (working tree carries
   over uncommitted).
2. Commit the HUD asset WIP as one commit, staged explicitly (not `git add -A`):
   - `assets/hud2.aseprite`, `assets/hud2.png`, `assets/hud2.tmx`
   - `assets/hud-bars.aseprite`, `assets/hud-bars.png`, `assets/hud-bars.tsx`
   - `assets/hud32.aseprite`, `assets/palette10.aseprite`
   - Message: describes the WIP HP-bar tileset swap + digit reposition, no
     `Co-Authored-By` line (per standing preference).
3. Commit `.cs/README.md` and `.cs/timeline.jsonl` separately (session
   bookkeeping, unrelated to the redesign) — its own commit, its own message.
4. `git status` after both commits to confirm a clean tree.

## Part B — wire hud-bars into the Rust side

Research is in. Findings:

- **Today**: `animate_hp` in `src/plugins/hud.rs:29` is the entire HP-bar
  mechanism — a single sprite marked `HPBar`, continuously lerping
  `Transform.scale.x` toward `Health.ratio()`. Not tile-based at all.
- **The new assets are inert**: `hud-bars.tsx`, the `BarMiddle` tile layer, and
  the `Digital-effect` objectgroup in `hud2.tmx` are referenced nowhere in
  `src/` (grepped, zero hits). `bevy_ecs_tiled` auto-renders any populated tile
  layer with no opt-in, so `BarMiddle` would already show a *static* fill if
  populated — but nothing in this codebase does **runtime per-tile
  toggling/swapping** of a Tiled tile-layer's tiles (no code touches
  `TileTextureIndex`/`TileVisible` or listens for `TiledEvent<TileCreated>`
  anywhere today). That would be new machinery, full stop.
- **Closest working precedent**: `ClaimedTile` in `src/plugins/maps.rs` +
  `animate_claimed_tile`/`animate_unclaimed_tile` in `animations.rs` — N
  discrete cells, each a **hand-spawned sprite entity** (not a Tiled tile-layer
  tile), switched via `SpritesheetAnimation::switch(handle)` off a
  `Resource`-held handle table, reacting to `Changed<ClaimedTile>` /
  `BeamResolved`. This is the pattern to mirror for a segmented HP bar: spawn
  N segment sprites per player (analogous to claimed-tile cells), each
  switched filled/empty off `Health.ratio()`.
- **Digit-counter machinery (`DigitAnimations`/`animate_digit`) does NOT reuse
  as-is** — its `[[Handle;10];10]` table and rolling-odometer math are
  digit-specific (0–9 spritesheet positions). The *shape* (marker component +
  resource lookup table + idempotent apply-if-changed fn + thin per-domain
  wrapper) is worth mirroring, but the concrete table/logic is new.

**Recommended approach**: build the segmented bar the same way `ClaimedTile`
cells work — a new marker (e.g. `HPBarSegment { index: u8 }`, alongside
existing `HPBar` in `src/components/markers.rs`) on hand-spawned sprite
entities using the `hud-bars` tileset frames, a small `Resource` (filled/empty
handles, no 10×10 table needed), and a driver system in `hud.rs` that computes
`filled_segments = (ratio * N).round()` and idempotently switches each
segment — replacing (or sitting alongside, TBD) `animate_hp`'s scale tween.
Exact segment count `N` and whether `BarMiddle`/`Digital-effect` objects in the
`.tmx` are meant to be those segments or are placeholder/scratch content is
**still open — confirm with you before writing the spawn logic**, since you
authored the Tiled layout and know the intended visual.

## Verification

- `cargo run --features dev` after the commits to confirm the build is
  unaffected by the asset-only commit, and again after Part B to confirm the
  new bar actually reflects `Health` changes in-game (dev bot-vs-bot menu or
  manual 2-player HP damage works per README controls).
- `git log --oneline -3` and `git status` after each commit to confirm history
  and a clean tree.
