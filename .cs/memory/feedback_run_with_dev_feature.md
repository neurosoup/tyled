---
name: feedback-run-with-dev-feature
description: "Always launch Tyled with `cargo run --features dev`, never plain `cargo run`"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4faa3256-bb1c-433a-bdc2-c387bf62b2a1
  modified: 2026-08-04T07:19:06.750Z
---

Always use `cargo run --features dev` (never plain `cargo run`) when launching the Tyled game, whether for manual verification, bot-vs-bot trace runs, or any other "run the app and see" check.

**Why:** The `dev` feature (`src/plugins/config.rs`) enables hot-reload of `assets/game_config.ron` from disk and dev-only systems (e.g. `resync_damage_timer`). It's the mode the user actually plays/tests in, so running without it doesn't match their real usage even though a fresh `cargo build` without it will still embed the current RON content at compile time via `include_str!`.

**How to apply:** Any `Bash` invocation that starts the game (`cargo run`, `cargo build` immediately followed by running the binary, etc.) should include `--features dev`.
