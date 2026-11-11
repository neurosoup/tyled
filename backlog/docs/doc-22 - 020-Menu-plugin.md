---
id: doc-22
title: '[020] Menu plugin'
type: other
created_date: '2026-08-05 12:00'
updated_date: '2026-11-11 12:00'
---
# Menu Plugin

This plugin shows the main menu, where players choose who is a bot. No map loads before a choice is confirmed. The code is in `src/plugins/menu.rs`.

## Key ideas

- `AppState` is the top-level state, above `RoundPhase`. It starts in `MainMenu` and goes to `InRound` once. It never goes back.
- The Maps plugin runs `load_maps` on `OnEnter(AppState::InRound)`. So the maps, the players and the round all wait for the menu.
- The confirmed choice is written into `GameConfig.controllers`. The Input plugin reads it when the players spawn.
- The entries are drawn with the [Text plugin](/documentation/17/015-text-plugin) on the overlay camera. The selected entry starts with `>`.
- Controls: W / S or the arrow keys move the cursor. Q or `/` confirms.

| Entry | Player 1 | Player 2 |
|---|---|---|
| `1P VS 2P` | human | human |
| `1P VS BOT` | human | bot |
| `BOT VS 2P` | bot | human |
| `BOT VS BOT` (dev builds only) | bot | bot |

## Ordering

```mermaid
---
config:
  theme: dark
---

flowchart TD

subgraph update ["Update, only during MainMenu"]
  ensure_menu_spawned["`**ensure_menu_spawned**`"]
  update_selection["`**update_selection**`"]
  confirm_selection["`**confirm_selection**`"]
  ensure_menu_spawned --> update_selection
  update_selection --> confirm_selection
end

subgraph enter_in_round ["OnEnter(AppState::InRound)"]
  load_maps["`**load_maps**
  Maps plugin`"]
end

update ~~~ enter_in_round
```

- `ensure_menu_spawned` runs in `Update`, because `OnEnter` of the default state can run before `Startup`, when `FontAtlas` does not exist yet.

## Plugin Systems

### Ensure Menu Spawned

- Waits for `FontAtlas`, then draws the entries once, if none exist.

### Update Selection

- Moves the cursor up or down. It stops at the first and last entries.
- On a change, despawns the entries and draws them again.

### Confirm Selection

- On Q or `/`, writes the chosen matchup into `GameConfig.controllers`.
- Despawns the entries and sets the next state to `InRound`.

## Components, Resources and Messages CRUD

### Read FontAtlas and ButtonInput

Used in the following systems:
- **ensure_menu_spawned**: reads `FontAtlas` as an `Option`.
- **update_selection**: reads `FontAtlas` and the up and down keys.
- **confirm_selection**: reads the confirm keys.

### MenuSelection resource

Used in the following systems:
- **plugin build**: inserts it with entry 0.
- **ensure_menu_spawned**: reads it to mark the selected entry.
- **update_selection**: changes it.
- **confirm_selection**: reads it to find the chosen matchup.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

build(("`Plugin build`")):::system-group
update(("`Update`")):::system-group
ensure_menu_spawned["`**ensure_menu_spawned**`"]
update_selection["`**update_selection**`"]
confirm_selection["`**confirm_selection**`"]

update -.-> ensure_menu_spawned
update -.-> update_selection
update -.-> confirm_selection

world@{ shape: st-rect, label: "World" }
selection_res@{ shape: doc, label: "MenuSelection" }

selection_res --> |belongs to| world

build ---> |inserts resource| selection_res
ensure_menu_spawned ---> |reads| selection_res
update_selection ---> |writes| selection_res
confirm_selection ---> |reads| selection_res
```

### Spawn menu entries

Used in the following systems:
- **ensure_menu_spawned**: draws the first entries.
- **update_selection**: draws the entries again after a change.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
ensure_menu_spawned["`**ensure_menu_spawned**`"]
update_selection["`**update_selection**`"]

update -.-> ensure_menu_spawned
update -.-> update_selection

entry_entity@{ shape: st-rect, label: "Menu Entry Label Entity" }

ml_marker>"`**MenuLabel**`"] --> |inserted on| entry_entity

ensure_menu_spawned ---> |spawns| entry_entity
update_selection ---> |spawns| entry_entity
```

### Query MenuLabel entities

Used in the following systems:
- **ensure_menu_spawned**: checks if entries already exist.
- **update_selection**, **confirm_selection**: despawn the entries.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef query stroke-dasharray: 3 3

update(("`Update`")):::system-group
ensure_menu_spawned["`**ensure_menu_spawned**`"]
update_selection["`**update_selection**`"]
confirm_selection["`**confirm_selection**`"]

update -.-> ensure_menu_spawned
update -.-> update_selection
update -.-> confirm_selection

labels_query{{"`labels`"}}:::query
ensure_menu_spawned ---> labels_query
update_selection ---> labels_query
confirm_selection ---> labels_query

entry_entity@{ shape: st-rect, label: "Menu Entry Label Entity" }

ml_entity>"`**Entity**`"] --> |belongs to| entry_entity
ml_marker>"`**MenuLabel**`"] --> |belongs to| entry_entity

labels_query ---> |reads| ml_entity
labels_query -..-> |filter With| ml_marker
update_selection ---> |despawns| entry_entity
confirm_selection ---> |despawns| entry_entity
```

### Write GameConfig controllers

Used in the following systems:
- **confirm_selection**: writes `player1_bot` and `player2_bot`.
- **attach_players_actions** (Input plugin): reads them when the players spawn.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

update(("`Update`")):::system-group
pre_update(("`PreUpdate`")):::system-group
confirm_selection["`**confirm_selection**`"]
attach_players_actions["`**attach_players_actions**
Input plugin`"]

update -.-> confirm_selection
pre_update -.-> attach_players_actions

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }

gc_controllers>"`**controllers**`"]

config_res --> |belongs to| world
gc_controllers --> |field of| config_res

confirm_selection ---> |writes| gc_controllers
attach_players_actions ---> |reads| gc_controllers
```

### AppState

Used in the following systems:
- **plugin build**: registers the state, starting in `MainMenu`.
- **confirm_selection**: sets `NextState<AppState>` to `InRound`.
- **load_maps** (Maps plugin): runs on `OnEnter(AppState::InRound)`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

build(("`Plugin build`")):::system-group
update(("`Update`")):::system-group
enter_in_round(("`OnEnter(InRound)`")):::system-group
confirm_selection["`**confirm_selection**`"]
load_maps["`**load_maps**
Maps plugin`"]

update -.-> confirm_selection
enter_in_round -.-> load_maps

world@{ shape: st-rect, label: "World" }
state_res@{ shape: doc, label: "State#60;AppState#62;" }
next_state_res@{ shape: doc, label: "NextState#60;AppState#62;" }

state_res --> |belongs to| world
next_state_res --> |belongs to| world

build ---> |inserts resource| state_res
confirm_selection ---> |sets InRound| next_state_res
next_state_res -.-> |"triggers OnEnter(InRound)"| load_maps
```

Definitions:
- `AppState`: public `States` enum: `MainMenu` (default) and `InRound`.
- `MatchupChoice`: one menu choice. `controllers()` returns `(player1_bot, player2_bot)`.
- `MENU_ENTRIES`: the ordered list of entries and their text.
- `MenuSelection`: public `Resource` with the index of the selected entry.
- `MenuLabel`: public marker `Component` on each entry label.
