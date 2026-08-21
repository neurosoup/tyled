---
name: project_bevy_019_migration
description: Bevy 0.19 migration completed 2026-08-05 — main is now on Bevy 0.19
metadata: 
  node_type: memory
  type: project
  originSessionId: 515d38f8-8d0c-4339-bea9-792ab02405b6
  modified: 2026-08-05T12:17:38.885Z
---

Tyled's Bevy 0.18 → 0.19 upgrade is **done** — merged to `main` 2026-08-05 (commit
`70a1374`, fast-forwarded from a rebased `bevy-0.19` branch).

- `bevy_ecs_tiled` bumped to `0.13` (targets `bevy_ecs = "^0.19"`).
- `bevy_smooth_pixel_camera` has no 0.19-compatible crates.io release yet — pinned to a
  git rev on the crate's GitHub `main` (commit `01c14805676a9542e0b8386f5e98ba9925223177`,
  PR doonv/bevy_smooth_pixel_camera#16). **Revisit periodically**: swap to a normal
  version pin once a crates.io release ships.
- Compiled clean with zero source-code changes required (only `Cargo.toml`); manually
  verified via `cargo run --features dev` (GPU adapter selected, window created, no
  panics). `MIGRATION_0.19.md` (the flip checklist) was deleted after landing.

**How to apply:** if a new blocker crate shows up stale again in the future, this file's
old approach (stub branch + `# BLOCKED` comments + a periodic watch check) is the
pattern to reuse. Right now there's nothing pending except eventually un-pinning
`bevy_smooth_pixel_camera` from git. Unrelated to [[project_strategic_mechanics_rework]].
