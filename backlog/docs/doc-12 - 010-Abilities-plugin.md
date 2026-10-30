---
id: doc-12
title: '[010] Abilities plugin'
type: other
created_date: '2026-07-12 12:00'
updated_date: '2026-10-30 12:00'
---
# Abilities Plugin

This plugin gives each player a list of abilities. The abilities change how beams and charges work. Other plugins read the list and apply the effects.

## Key ideas

- `AbilityList` is the list of abilities of one player. The Maps plugin adds it to each player.
- `AbilityDescriptor` is one ability: `Lance`, `BorderGrinder` or `SolarPanels`. It is plain data, so a loadout can later be written in a file.
- Straight Shot is the base beam. Every player has it, so it is not in the list. An empty list means "Straight Shot only".
- `PlayerLoadouts` holds the starting abilities of each player. There is no draft screen yet, so the loadouts are set in code.
- Today, player 1 starts with `SolarPanels` and player 2 starts with `BorderGrinder`.
- This plugin has no systems. Other plugins read `AbilityList` and apply each ability.

## Abilities

- **Lance**: lets a player fire from a claimed tile. The beam then uses `BeamBehavior::Lance`. Without Lance, a player on a claimed tile cannot fire, and no charge is spent.
- **Border Grinder**: when a straight beam reaches an enemy tile, it claims that tile and ends there.
- **Solar Panels**: gives back charges on a timer (`charge.solar_panels_tick_secs`, 5 s). The player gains one charge for every `charge.solar_panels_tiles_per_charge` (10) tiles that they own.

See the [Beam plugin](/documentation/7/006-beam-plugin) for Lance and Border Grinder, and the [Charge plugin](/documentation/23/021-charge-plugin) for Solar Panels.

## Plugin workflow

- At plugin build:
    - Registers `AbilityList` and `AbilityDescriptor`, so the world inspector shows each player's abilities.
    - Inserts `PlayerLoadouts`.

## Plugin Systems

None.

## Components, Resources and Messages CRUD

### Who reads the ability data

```mermaid
---
config:
  theme: dark
---

flowchart TD

loadouts[("`**PlayerLoadouts**
resource`")]
list["`**AbilityList**
component on each player`"]

initialize_players["`**initialize_players**
Maps plugin`"]
open_sink["`**open_sink**
Telemetry plugin`"]

loadouts --> |read by| initialize_players
list --> |inserted by| initialize_players
loadouts --> |read by| open_sink

subgraph lance ["Lance"]
  handle_characters_input["`**handle_characters_input**
  Input plugin`"]
  spawn_beam["`**spawn_beam**
  Beam plugin`"]
  bot_think_lance["`**bot_think**
  Bot plugin`"]
end

subgraph grinder ["Border Grinder"]
  beam_step["`**beam_step**
  Beam plugin`"]
  bot_think_grinder["`**bot_think**
  Bot plugin`"]
end

subgraph solar ["Solar Panels"]
  regen["`**regen_charges_from_solar_panels**
  Charge plugin`"]
end

record_outcome["`**record_outcome**
Telemetry plugin`"]

list --> |read by| lance
list --> |read by| grinder
list --> |read by| solar
list --> |read by| record_outcome
```

- `AbilityList`: `Component`, `Reflect`, `Default`, `Clone`.
- `AbilityDescriptor`: `Reflect`, `Clone`, `Debug`, `PartialEq`, `Eq`.
- `PlayerLoadouts`: `Resource`, `Clone`. `for_player(player_id)` returns the list for player 0 or player 1.
