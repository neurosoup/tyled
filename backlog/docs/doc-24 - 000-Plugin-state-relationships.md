---
id: doc-24
title: '[000] Plugin state relationships'
type: other
created_date: '2026-09-15 12:00'
updated_date: '2026-10-17 12:00'
---
# Plugin State Relationships

Some plugins are also linked through game states. A plugin can run only in one state, run once when a state starts or ends, or move the game to a new state. The diagram below shows each of these links.

For the links through messages, see [doc-9 Plugin message relationships](/documentation/9/000-plugin-message-relationships).

## Link kinds

- **gates**: the systems run only while the game is in that state.
- **on enter / on exit**: the systems run once, when the state starts or ends.
- **writes**: the systems move the game to a new state. Only a few systems do this.
- **default**: the state the machine starts in (the small white circle).

## State machines

- `AppState` (`menu.rs`): `MainMenu` → `InRound`. It never goes back.
- `RoundPhase` (`round/state.rs`): `Loading` → `Starting` → `Playing` → `Outcome` → `Starting` again.

## States and GameplaySet

The two state machines decide *when* systems run. The `GameplaySet` chain (`schedule.rs`) decides *in which order* they run inside one frame.

- `AppState` changes only once: from the menu to the round. `InRound` loads the maps, and that starts `RoundPhase`.
- `RoundPhase` loops for each round: `Starting` (intro) → `Playing` → `Outcome` (result) → `Starting`.
- In Bevy, a set (`SystemSet`) is a named group of systems. You can order a whole group with one rule.
- `GameplaySet` is a list of stages in `Update`, from `BotThink` to `HudSync`. Each plugin puts its systems in a stage.
- All stages run only during `Playing`, except `Presentation` and `HudSync`, which always run.
- `RoundResolution` is the stage that ends a round. It moves the game to `Outcome`.
- The systems that change `RoundPhase` are in `RoundPhaseWriter`. Each runs in a different phase, so their order does not matter.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef always stroke-dasharray: 5 5

subgraph app ["AppState"]
  main_menu(["MainMenu"]) --> in_round(["InRound"])
end

subgraph round ["RoundPhase"]
  loading(["Loading"]) --> starting(["Starting"]) --> playing(["Playing"]) --> outcome(["Outcome"]) --> starting
end

in_round ---> |loads the maps| loading

subgraph chain ["GameplaySet chain (Update, top to bottom)"]
  bot_think["`**BotThink**
  bots pick their keys`"]
  input["`**Input**
  keys become moves and shots`"]
  movement["`**Movement**
  characters move`"]
  beam["`**Beam**
  beams spawn, get parried, move`"]
  claim["`**Claim**
  tiles get an owner`"]
  economy["`**Economy**
  charges grow or shrink`"]
  damage["`**Damage**
  players lose health`"]
  displacement["`**Displacement**
  knockback pushes players`"]
  presentation["`**Presentation**
  animations and effects`"]:::always
  round_resolution["`**RoundResolution**
  checks if the round ends`"]
  hud_sync["`**HudSync**
  HUD bars and numbers`"]:::always
  bot_think --> input --> movement --> beam --> claim --> economy --> damage --> displacement --> presentation --> round_resolution --> hud_sync
end

playing -.-> |"gates every stage except the dashed ones"| chain
round_resolution ---> |"ends the round (Playing → Outcome)"| outcome
```

## Notes

- `Loading` ends only when both maps (level and HUD) have finished loading.
- A round reaches `Outcome` on a kill, at the end of the countdown, or when all players are out of charges.
- Telemetry also needs the `telemetry_enabled` config, so it can be off during `Playing`.
- HUD, animations and effects are not gated, on purpose: their tweens must finish across phase changes.
- The knockback lock timer is not gated, so a lock still ends after the phase changes.
- On exit of `Outcome`, effects remove the drivers that move players, with no fixed order against `reset_round`. An effect driver is a small entity that plays one effect tween (for example a slide or a knockback) for one player.

## Reading the diagram

The game starts in the menu. Picking a matchup loads the maps, and the end of loading starts the round loop. Each round goes from the intro to play, then to the result banner, and back to the intro.

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
phase_outcome ---> |on exit| effects_plugin
round_outcome_plugin ---> |writes| phase_starting
```
