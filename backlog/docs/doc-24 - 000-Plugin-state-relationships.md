---
id: doc-24
title: '[000] Plugin state relationships'
type: other
created_date: '2026-09-15 12:00'
updated_date: '2026-09-16 12:00'
---
# Plugin State Relationships

This document summarises how the game's plugins are coupled through Bevy's state-machine gating, as distinct from the message-passing coupling described in `doc-9 - 000-Plugin-message-relationships.md`. Where messages are an explicit write/read contract between plugins, state gating is implicit: a system's `run_if(in_state(...))` or `OnEnter`/`OnExit` schedule attaches it to a state machine owned by a different plugin, and only a `NextState<T>::set` call actually drives a transition.

There are two state machines in this codebase, and three distinct relationship types to either of them:

- **Gated by** (`run_if(in_state(...))`) — the system is skipped entirely while the game is not in that state.
- **Hooked to enter/exit** (`OnEnter`/`OnExit`) — the system runs exactly once, at the instant of the transition.
- **Writes** (`NextState<T>::set`) — the system decides when the whole app moves to a different state. For either machine, only a small number of systems, spread across few files, are allowed to do this.

Each state machine also has exactly one **default state** — the variant Bevy sets it to at `init_state::<T>()`, before any system runs. The diagrams mark this with a small filled-circle node (a UML-style initial pseudostate) pointing at that state, distinct from the `Gated by`/`Hooked to`/`Writes` edges above since nothing "writes" the default — it's just where the machine starts.

## AppState

`AppState` (`src/plugins/menu.rs`) is the app's top-level lifecycle: `MainMenu` (default) and `InRound`. It has exactly one directed edge, `MainMenu → InRound`, and nothing ever transitions back.

- **Gated by**: the Menu plugin's three chained systems (`ensure_menu_spawned`, `update_selection`, `confirm_selection`) all run `run_if(in_state(AppState::MainMenu))`.
- **Hooked to enter/exit**: the Maps plugin's `load_maps` runs `OnEnter(AppState::InRound)`.
- **Writes**: `confirm_selection` (Menu plugin) is the only system that calls `NextState<AppState>::set(AppState::InRound)`, when a matchup is confirmed with Q or `/`.

## RoundPhase

`RoundPhase` (`src/plugins/round/state.rs`) is the round lifecycle: `Loading` (default) → `Starting` → `Playing` → `Outcome` → back to `Starting`. Unlike `AppState`, its writers are spread across three files rather than one.

### The causal handoff between the two machines

The two machines are not independent: `RoundPhase`'s first transition is downstream of `AppState`'s only transition, relayed through a Tiled message rather than a direct state read.

1. `confirm_selection` (Menu plugin) sets `AppState::InRound`.
2. `OnEnter(AppState::InRound)` runs `load_maps` (Maps plugin), which spawns the level map (`CurrentLevel`) and the HUD map (`HudMap`).
3. Each map fires a `TiledEvent<MapCreated>` once loaded. `start_round_on_map_created` (Round plugin, gated `run_if(in_state(RoundPhase::Loading))`) reads both, latching `level_ready`/`hud_ready` in two `Local<bool>`s, and only calls `next_phase.set(RoundPhase::Starting)` once both maps have reported in.
4. `OnExit(RoundPhase::Loading)` then runs the Maps plugin's four bootstrap systems (`initialize_map_info`, then `initialize_players`/`initialize_claimed_tiles`/`initialize_hud_bars`, chained), which need `MapInfo` populated and both maps' entities present.

So `AppState` gates map loading; `RoundPhase::Loading`'s exit gates world bootstrap on top of that, and the dependency between the two machines is a message, not a shared state read.

### Writers (three files)

- `round/state.rs`: `start_round_on_map_created` writes `Loading → Starting` (see above). The shared `conclude_round` helper writes `→ Outcome`, called from three systems in the same file — `resolve_kill`, `resolve_timeout`, `resolve_charge_exhaustion` — each of which independently detects a round-ending condition (a death, the countdown reaching zero, or every player exhausting their charges with no beam in flight) and defers to `conclude_round` to record the result and transition.
- `round/intro.rs`: `advance_intro_countdown` writes `Starting → Playing`, once the "3·2·1·GO!" intro countdown reaches zero.
- `round/outcome.rs`: `advance_outcome` writes `Outcome → Starting`, once the win banner's linger timer elapses.

`start_round_on_map_created` is not in any `GameplaySet`; only the `(tick_countdown, resolve_kill, resolve_timeout, resolve_charge_exhaustion)` tuple in `round/state.rs` is tagged `GameplaySet::RoundResolution`. So the Round plugin's state-writing systems sit partly inside and partly outside the `schedule.rs` pipeline chain — the ones that fire before or between rounds (map-created latch, countdown start) are outside it, the ones that fire during live play (round resolution) are inside it.

### Gated by `run_if(in_state(RoundPhase::Playing))`

Every stage of the gameplay pipeline declared in `schedule.rs`'s `GameplaySet` chain gates on `Playing`, with two exceptions (see Asymmetries below):

- `GameplaySet::BotThink` — `bot.rs`'s `bot_think`
- `GameplaySet::Input` — `inputs.rs`'s `handle_characters_input` (`tick_turning` shares the same `Playing` gate but is not tagged `GameplaySet::Input`)
- `GameplaySet::Movement` — `controller.rs`'s `move_characters`
- `GameplaySet::Beam` — `beam.rs`'s `spawn_beam`, `beam_step`
- `GameplaySet::Economy` — `charge.rs`'s `regen_charges_from_solar_panels`, `cap_charges_to_unclaimed_tiles`
- `GameplaySet::Damage` — `damage.rs`'s `apply_owned_tile_entry_damage`, `apply_owned_tile_damage`, `apply_beam_damage`, `apply_collision_damage`
- `GameplaySet::RoundResolution` — `round/state.rs`'s `tick_countdown`, `resolve_kill`, `resolve_timeout`, `resolve_charge_exhaustion`

Outside the `GameplaySet` chain, `telemetry.rs`'s `record_actions` and `record_decisions` also gate on `run_if(in_state(RoundPhase::Playing))`, stacked with a second `run_if(telemetry_enabled)` config-resource gate — the only place in the codebase a state gate and a config gate are stacked on the same systems, so these can be inactive even while `Playing`.

Three other systems gate on phases other than `Playing`:

- `round/state.rs`'s `start_round_on_map_created` gates on `run_if(in_state(RoundPhase::Loading))`.
- `round/intro.rs`'s `advance_intro_countdown` gates on `run_if(in_state(RoundPhase::Starting))`.
- `round/outcome.rs`'s `advance_outcome` gates on `run_if(in_state(RoundPhase::Outcome))`.

### Hooked to enter/exit

- `OnEnter(RoundPhase::Starting)`: `round/intro.rs`'s `begin_intro_countdown` spawns the first countdown number.
- `OnEnter(RoundPhase::Outcome)`: `round/outcome.rs`'s `show_outcome_banner` spawns the win banner; `telemetry.rs`'s `record_outcome` (also gated `telemetry_enabled`) writes the round's outcome record.
- `OnExit(RoundPhase::Outcome)`: `round/state.rs`'s `reset_round` wipes tile ownership, health, charges, and positions back to spawn; `round/outcome.rs`'s `despawn_outcome_banner` removes the win banner.
- `OnExit(RoundPhase::Loading)`: the Maps plugin's bootstrap chain (see above).
- `OnExit(RoundPhase::Playing)`: `effects.rs`'s `clear_illumination_drivers` clears any beam-illumination tint still applied to a tile.

## Asymmetries

- `claim_tile` (`claim.rs`) is gated on `Playing`, matching the rest of the `GameplaySet` pipeline: a `BeamResolved` message is always written and drained within the same frame, while `Playing`.
- `hud.rs`'s `HudSync` stage and `GameplaySet::Presentation` (`animations.rs`, plus `effects.rs`'s `apply_death_effect`) are both deliberately ungated, not oversights. Neither writes anything round-resolution logic reads back, and both need to keep running past `Playing`: to let a tween or `smooth_nudge` (HP bar, damage flash, death bounce) finish playing out into `Outcome`, or to let a `Starting`-phase system pick up `reset_round`'s `OnExit(Outcome)` mass write — the tile-unclaim cascade, the HUD digit resets — as a visible transition during the intro instead of a snap at "GO!".
- `round/state.rs`'s `start_countdown` and `round/intro.rs`'s `despawn_go_banner` are similarly ungated by design, for the same reason: continuous state (a resource re-insert, a tween) that must keep evolving right across a phase boundary rather than freezing at it.
- `maps.rs` is the only plugin coupled to both state machines from opposite layers: `load_maps` hooks `OnEnter(AppState::InRound)`, its bootstrap chain hooks `OnExit(RoundPhase::Loading)`.
- `RoundPhase` ownership is split three ways: `round/state.rs`, `round/intro.rs`, and `round/outcome.rs` each gate their own systems on the phase *and* write one of its transitions, so no single file in the `round` feature is purely a reader or purely a writer of it.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef default-marker fill:#fff,stroke:#fff

menu_plugin["`**Menu Plugin**`"]:::system-group
maps_plugin["`**Maps Plugin**`"]:::system-group
round_state_plugin["`**Round Plugin (state)**`"]:::system-group
round_intro_plugin["`**Round Plugin (intro)**`"]:::system-group
round_outcome_plugin["`**Round Plugin (outcome)**`"]:::system-group
bot_plugin["`**Bot Plugin**`"]:::system-group
inputs_plugin["`**Input Plugin**`"]:::system-group
controller_plugin["`**Controller Plugin**`"]:::system-group
beam_plugin["`**Beam Plugin**`"]:::system-group
charge_plugin["`**Charge Plugin**`"]:::system-group
damage_plugin["`**Damage Plugin**`"]:::system-group
telemetry_plugin["`**Telemetry Plugin**`"]:::system-group
effects_plugin["`**Effects Plugin**`"]:::system-group
claim_plugin["`**Claim Plugin**`"]:::system-group
hud_plugin["`**HUD Plugin — HudSync**`"]:::system-group
animations_plugin["`**Animations Plugin — Presentation**`"]:::system-group

app_main_menu(["`**AppState::MainMenu**`"])
app_in_round(["`**AppState::InRound**`"])

phase_loading(["`**RoundPhase::Loading**`"])
phase_starting(["`**RoundPhase::Starting**`"])
phase_playing(["`**RoundPhase::Playing**`"])
phase_outcome(["`**RoundPhase::Outcome**`"])

default_app((" ")):::default-marker
default_round((" ")):::default-marker

default_app ---> |default| app_main_menu
default_round ---> |default| phase_loading

app_main_menu ---> |gates| menu_plugin
menu_plugin ---> |writes| app_in_round
app_in_round ---> |on enter| maps_plugin

phase_loading ---> |gates| round_state_plugin
round_state_plugin ---> |writes| phase_starting
phase_loading ---> |on exit| maps_plugin

phase_starting ---> |on enter| round_intro_plugin
phase_starting ---> |gates| round_intro_plugin
round_intro_plugin ---> |writes| phase_playing

subgraph pipeline ["Playing-gated stages of the GameplaySet chain"]
  bot_plugin
  inputs_plugin
  controller_plugin
  beam_plugin
  charge_plugin
  damage_plugin
end

phase_playing ---> |gates| pipeline
phase_playing ---> |gates| round_state_plugin
round_state_plugin ---> |writes| phase_outcome
phase_playing ---> |gates| telemetry_plugin
phase_playing ---> |gates| claim_plugin
phase_playing ---> |on exit| effects_plugin

phase_outcome ---> |on enter| round_outcome_plugin
phase_outcome ---> |gates| round_outcome_plugin
phase_outcome ---> |on enter| telemetry_plugin
phase_outcome ---> |on exit| round_state_plugin
phase_outcome ---> |on exit| round_outcome_plugin
round_outcome_plugin ---> |writes| phase_starting
```
