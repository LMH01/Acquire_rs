## Acquire_rs

The board game **Acquire**, rebuilt as a terminal UI (TUI) in Rust. Play single-player
against seeded bot opponents; the game rules, board, chains, stocks and bonuses follow
the original game.

### Getting started

Build from source (no zip distribution anymore — this is a TUI, so it runs in any
terminal):

```sh
cargo run                 # start a single-player game vs 3 bots (default 4 players)
cargo run -- -p 2         # 1 human + 1 bot
cargo run -- -p 6 --name Alice   # 1 human + 5 bots, custom name
```

Release build:

```sh
cargo build --release
./target/release/acquire_rs --help
```

> Requires a terminal of at least ~100×24 characters; smaller windows show a
> "terminal too small" notice.

### How a turn works

Each turn you **place one card** (found/extend a chain, or trigger a fusion), then
**buy up to 3 stocks**. Majority shareholders collect 10×/5× bonuses. The player with
the most money at the end wins.

### Keys

| Context | Keys |
|---|---|
| Global | `i` info card · `h` help · `l` toggle log · `q` quit |
| Setup | `Tab`/`↑`/`↓` switch field · `2`–`6` player count · type your name · `Enter` start |
| Choose card | `1`–`9` play · `↑`/`↓` select · `Enter` confirm |
| Choose chain | `↑`/`↓` select · `Enter` confirm |
| Fusion order | press chain letters in fusion order (last = survivor), `Backspace` undo |
| Fusion stocks | `<`/`>` exchange · `↑`/`↓` sell · `Enter` confirm |
| Buy stocks | `1`–`7` select row · `<`/`>` adjust · `Enter` confirm |
| End game | `y` / `n` |
| Game over | `q` |

### Command line options

Run `acquire_rs --help` for the full list. The most useful:

| Flag | Meaning |
|---|---|
| `-p/--players` (2–6, default 4) | Total players: 1 human + (N−1) bots |
| `-n/--name` | Your player name |
| `-h/--hide-extra-info` | Hide the ★/☆ largest-shareholder markers |
| `-s/--small-board` | Compact board layout |
| `--info-card` | Open on the stock-value reference card |
| `--demo` / `--demo-type 0\|1` | Open a demo board (0 = clever, 1 = random) |
| `--skip-dialogues` | Accepted for compatibility; currently a no-op |
| `--lan-server` / `--lan-client` / `--ip` / `--port` | Multiplayer placeholder (see below) |

### Features

- Single-player vs bots (2–6 players) — the bots use a fixed, reproducible seed.
- Full rules: chain founding, extension, fusion (incl. 3/4-chain and stock
  exchange/sell), majority bonuses (10×/5×), stock market, end conditions.
- Colored board and chain table; selected-card highlighting; scrolling event log.
- Setup screen, stock info card, help screen, demo boards, and a game-over ranking
  (gold/silver/bronze).

### Multiplayer (LAN)

Multiplayer is **not implemented in this build**. The `--lan-*` flags are kept and
show a friendly placeholder screen so they can be re-implemented later without touching
the game logic. The engine is I/O-free and decider-based (`core/`), so a network backend
can be slotted in behind the existing `Decider` abstraction.

### Project layout

```
src/
  main.rs    CLI parsing + dispatch (clap)
  core/      Pure game engine — no I/O (board, chains, bank, players, rules, game)
  bot/       Seeded bot that implements the `Decider` trait
  tui/       ratatui TUI — app state, event loop, key handling, rendering
```
