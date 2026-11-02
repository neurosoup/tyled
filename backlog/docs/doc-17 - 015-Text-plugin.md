---
id: doc-17
title: '[015] Text plugin'
type: other
created_date: '2026-07-16 12:00'
updated_date: '2026-11-02 12:00'
---
# Text Plugin

This plugin draws text with a bitmap font. It loads the font once, and other plugins call `spawn_label` to show a string. The code is in `src/plugins/text.rs`.

## Key ideas

- `FontAtlas` holds the font image (`assets/font.png`) and its grid layout. It is inserted at `Startup`.
- `spawn_label` turns a string into one parent entity with one sprite child per letter. The letters are centered on the parent's `Transform`.
- The caller chooses the camera: it passes the `RenderLayers`, and `spawn_label` puts them on each letter. Render layers do not pass from parent to child here.
- Despawn the parent to remove the whole label.
- Users today: the [Round intro](/documentation/16/014-round-intro) ("3 · 2 · 1 · GO!"), the [Round outcome](/documentation/18/016-round-outcome) banner and the [Menu](/documentation/22/020-menu-plugin) rows.
- A system that runs on `OnEnter` of a state's default value can run before `Startup`. There, `FontAtlas` may not exist yet, so read it as `Option<Res<FontAtlas>>` (see `ensure_menu_spawned` in the Menu plugin).

## Font atlas

- `assets/font.png` is a grid of 16 px cells. It has uppercase letters, digits and `! ? . : >`.
- `glyph_index` returns the number of the cell that holds a character (for example `'C'` → 2). `spawn_label` uses this number to show that cell. Other characters draw nothing.

## Plugin Systems

### Setup Font Atlas

- Runs once at `Startup`.
- Loads `font.png`, builds the grid layout and inserts `FontAtlas`.

`spawn_label` and `glyph_index` are helper functions. Other plugins call them from their own systems.

## Components, Resources and Messages CRUD

### Insert FontAtlas resource

Used in the following systems:
- **setup_font_atlas**: loads the image, adds the layout and inserts the resource.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

startup(("`Startup`")):::system-group
setup_font_atlas["`**setup_font_atlas**`"]

startup -.-> setup_font_atlas

world@{ shape: st-rect, label: "World" }
asset_server@{ shape: doc, label: "AssetServer" }
layouts@{ shape: doc, label: "Assets<TextureAtlasLayout>" }
font_res@{ shape: doc, label: "FontAtlas" }

fa_image>"`**image**`"]
fa_layout>"`**layout**`"]

asset_server --> |belongs to| world
layouts --> |belongs to| world
font_res --> |belongs to| world
fa_image --> |field of| font_res
fa_layout --> |field of| font_res

setup_font_atlas ---> |"loads font.png"| asset_server
setup_font_atlas ---> |adds grid layout| layouts
setup_font_atlas ---> |inserts resource| font_res
```

### Read FontAtlas resource

Used in the following systems:
- **begin_intro_countdown**, **advance_intro_countdown** (Round intro): draw the numbers and "GO!".
- **show_outcome_banner** (Round outcome): draws the winner banner.
- **ensure_menu_spawned** (Menu): draws the first rows. Reads `Option<Res<FontAtlas>>` and waits until it exists.
- **update_selection** (Menu): draws the rows again when the selection changes.

```mermaid
---
config:
  theme: dark
---

flowchart TD
classDef system-group stroke-dasharray: 5 5

enter_starting(("`OnEnter(Starting)`")):::system-group
enter_outcome(("`OnEnter(Outcome)`")):::system-group
update(("`Update`")):::system-group
begin_intro_countdown["`**begin_intro_countdown**
Round intro`"]
advance_intro_countdown["`**advance_intro_countdown**
Round intro`"]
show_outcome_banner["`**show_outcome_banner**
Round outcome`"]
ensure_menu_spawned["`**ensure_menu_spawned**
Menu`"]
update_selection["`**update_selection**
Menu`"]

enter_starting -.-> begin_intro_countdown
update -.-> advance_intro_countdown
enter_outcome -.-> show_outcome_banner
update -.-> ensure_menu_spawned
update -.-> update_selection

world@{ shape: st-rect, label: "World" }
font_res@{ shape: doc, label: "FontAtlas" }

font_res --> |belongs to| world

begin_intro_countdown ---> |reads| font_res
advance_intro_countdown ---> |reads| font_res
show_outcome_banner ---> |reads| font_res
ensure_menu_spawned ---> |"reads (optional)"| font_res
update_selection ---> |reads| font_res
```

### Spawn label entities

Used in the following systems (through `spawn_label`):
- **begin_intro_countdown**, **advance_intro_countdown** (Round intro).
- **show_outcome_banner** (Round outcome).
- **ensure_menu_spawned**, **update_selection** (Menu).

```mermaid
---
config:
  theme: dark
---

flowchart TD

spawn_label["`**spawn_label**
called by the systems above`"]

label_entity@{ shape: st-rect, label: "Label Entity" }
glyph_entity@{ shape: st-rect, label: "Glyph Entity (one per letter)" }

lb_name>"`**Name**`"] --> |inserted on| label_entity
lb_transform>"`**Transform**`"] --> |inserted on| label_entity
lb_visibility>"`**Visibility**`"] --> |inserted on| label_entity

gl_sprite>"`**Sprite**`"] --> |inserted on| glyph_entity
gl_transform>"`**Transform**`"] --> |inserted on| glyph_entity
gl_layers>"`**RenderLayers**`"] --> |inserted on| glyph_entity

glyph_entity --> |child of| label_entity

spawn_label ---> |spawns| label_entity
spawn_label ---> |spawns| glyph_entity
```

Definitions:
- `FontAtlas`: public `Resource` with the font `image` and the grid `layout` handles.
- `FONT_COLS` (16), `FONT_ROWS` (3), `FONT_CELL` (16 x 16 px), `FONT_ADVANCE` (12 px): the atlas grid and the letter spacing.
