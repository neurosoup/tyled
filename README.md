# Tyled

> Tyled is in early development — game design is built day after day. Follow the journey through devlog videos on the [DEVLOGS YouTube playlist](https://www.youtube.com/@TeamChipsStudio/playlists).

Fast-paced real-time strategy game played on a top-down, tile-based board. Each player controls a character and spreads their color by shooting a beam in a straight line. The beam does not paint every tile it passes through; instead, it targets a single tile per shot:
- If the beam does not encounter any colored tile, it paints the tile at the edge of the board.
- If the beam encounters a colored tile, it paints the tile immediately before the first colored tile in its path.

Once a tile is claimed, it becomes permanently locked in that player’s color and cannot be converted by the opponent. Players take damage when stepping on tiles colored by the opponent, making aggressive movement risky and strategic positioning crucial.

Each shot consumes a portion of the player’s power bar. When the bar reaches zero, the player can no longer shoot, requiring careful management of offensive resources and timing.

Players can parry enemy beams. When a beam reaches a player, they can intercept it, reversing the beam back toward the opponent. Reversed beams can be parried again, creating fast-paced chains of beam counters. Each time a beam is parried, it increases in speed, adding intensity to repeated exchanges. The beam always obeys the same single-tile resolution rules when painting tiles.

If a beam reaches an opponent, it deals much more damage than simply stepping on an opponent-colored tile. The struck player is “carried along” the beam’s path until the beam stops according to the beam rules, creating high-risk moments and sudden swings in board control.

The board evolves dynamically as players race to claim territory, control key lanes, and outmaneuver their opponent. Victory comes from dominating the board, forcing the enemy into dangerous tiles or lethal beams, and skillfully balancing aggression, defense, and resource management.

## Installation

Download the latest release from the [releases page](../../releases/latest).

**Linux**
```
tar -xzf tyled-linux-x86_64.tar.gz
cd tyled-linux-x86_64
./tyled
```

**macOS**
```
tar -xzf tyled-macos-aarch64.tar.gz
cd tyled-macos-aarch64
./tyled
```

**Windows**

Extract `tyled-windows-x86_64.zip` and run `tyled.exe` from inside the extracted folder.

> The `assets/` folder must remain next to the binary.

**Build from source**

Requires Rust nightly.
```
cargo run --features dev
```

## Controls

Tyled is a 2-player local game. Both players share the same keyboard.

| Action | Player 1 | Player 2 |
|---|---|---|
| Move | WASD | Arrow keys |
| Lock direction | Q | Right Shift |
| Shoot beam | Tab | / |

Holding **Lock direction** freezes your aim while letting you move freely — useful for strafing or repositioning without changing where your next beam will go.
