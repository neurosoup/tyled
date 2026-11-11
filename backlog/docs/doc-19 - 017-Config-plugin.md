---
id: doc-19
title: '[017] Config plugin'
type: other
created_date: '2026-07-19 12:00'
updated_date: '2026-11-11 12:00'
---
# Config Plugin

This plugin loads the tuning values from `assets/game_config.ron` into the `GameConfig` resource. Other plugins read them with `Res<GameConfig>`. The code is in `src/plugins/config.rs`.

## Key ideas

- To rebalance the game, edit `assets/game_config.ron`. No code change is needed.
- Each `.ron` value has a tag that says when an edit takes effect: `[live]`, `[next-round]`, `[restart]` or `[menu]`. The legend is at the top of the `.ron` file.
- The plugin is registered right after `defaults`. It inserts `GameConfig` at plugin build, so the resource exists before any `Startup` system.
- Fixed layout values stay as consts in their own modules, for example render layers, z-indices and atlas grids.
- The test `embedded_config_parses` checks that the `.ron` file still parses. A broken edit fails the tests, and the game does not start with bad values.

## Dev and release builds

```mermaid
---
config:
  theme: dark
---

flowchart TD

ron["`**assets/game_config.ron**`"]
release["`**Release**
embedded with include_str!
fixed values`"]
dev["`**Dev**
read from disk at start
falls back to the embedded copy`"]
reload["`**Dev hot reload**
saving the file updates GameConfig`"]

ron --> release
ron --> dev
dev --> reload
```

- `[live]` values are read every frame or every step, so they change at once in dev.
- The damage and beam step timers are re-synced by dev-only systems in their own plugins.

## Plugin Systems

### Load Config Asset (dev)

- Runs once at `Startup`.
- Loads `game_config.ron` as an asset and keeps its handle, so the file watcher tracks it.

### Apply Config Reload (dev)

- Runs every frame in `Update`.
- When the asset changes, copies it into the `GameConfig` resource.

## Components, Resources and Messages CRUD

### Read GameConfig

Used in the following systems:
- Almost every plugin reads it. Each plugin doc lists its own readers.

### Insert GameConfig resource

Used in the following systems:
- **plugin build**: parses the embedded copy (release) or the file on disk (dev), and inserts the resource.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

build(("`Plugin build`")):::system-group

world@{ shape: st-rect, label: "World" }
config_res@{ shape: doc, label: "GameConfig" }

config_res --> |belongs to| world

build ---> |inserts resource| config_res
```

### GameConfigHandle resource (dev)

Used in the following systems:
- **load_config_asset**: loads the asset and inserts the handle.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
load_config_asset["`**load_config_asset**`"]

startup -.-> load_config_asset

world@{ shape: st-rect, label: "World" }
asset_server@{ shape: doc, label: "AssetServer" }
handle_res@{ shape: doc, label: "GameConfigHandle" }

asset_server --> |belongs to| world
handle_res --> |belongs to| world

load_config_asset ---> |"loads game_config.ron"| asset_server
load_config_asset ---> |inserts resource| handle_res
```

### Read AssetEvent messages and update GameConfig (dev)

Used in the following systems:
- **apply_config_reload**: on `Modified` or `LoadedWithDependencies`, copies the new asset into `GameConfig`.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5
classDef reader stroke-dasharray: 3 3

update(("`Update`")):::system-group
apply_config_reload["`**apply_config_reload**`"]

update -.-> apply_config_reload

message_reader{{"MessageReader#60;AssetEvent#60;GameConfig#62;#62;"}}:::reader
apply_config_reload ---> message_reader

asset_event_message(["`**AssetEvent**`"])
message_reader ---> |reads| asset_event_message

world@{ shape: st-rect, label: "World" }
assets_res@{ shape: doc, label: "Assets<GameConfig>" }
config_res@{ shape: doc, label: "GameConfig" }

assets_res --> |belongs to| world
config_res --> |belongs to| world

apply_config_reload ---> |reads| assets_res
apply_config_reload ---> |writes| config_res
```

Definitions:
- `GameConfig`: public `Resource` and `Asset`, with one group of values per domain.
- `GameConfigHandle`: private dev-only `Resource` that keeps the asset handle alive.
- `GameConfigLoader`: dev-only `AssetLoader` that parses the `.ron` file.
