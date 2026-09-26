# Acquire_rs TUI Rework — Implementation Plan

This document is the execution plan for reworking `acquire_rs` from a blocking
command-line game into a ratatui-based TUI game with single-player mode against
bots. It is written to be followed phase by phase by an implementing agent.

---

## 1. Goals & Requirements

| # | Requirement |
|---|-------------|
| R1 | The game is played in a TUI built with **ratatui** (crossterm backend). |
| R2 | **Single-player vs bot must work properly** (2–6 total players: 1 human + N−1 bots). |
| R3 | Multiplayer (LAN) does **not** need to work, but clean placeholders must exist so it can be re-implemented easily: keep the CLI flags, keep the wire-protocol documentation, and design the core so a network backend can be slotted in without touching game logic. |
| R4 | **Keep all existing CLI options** that are still applicable: `-p/--players`, `--lan-client`, `--lan-server`, `-n/--name`, `--ip`, `--port`, `--info-card`, `--skip-dialogues`, `--demo`, `--demo-type`, `-h/--hide-extra-info`. |
| R5 | The old CLI playing mode (stdin/stdout prompt loop) is **entirely replaced** by the TUI. |
| R6 | **Do not change the game rules.** The existing rule implementation in `logic.rs` / `base_game.rs` is the source of truth. Only I/O and flow orchestration change. |
| R7 | All existing unit tests must keep passing (port them if they touch I/O). |

Environment notes (verified):

- Rust 1.98.1, edition 2021, `cargo test` currently passes (18 tests).
- `ratatui 0.26.3` and `crossterm 0.27.0` are present in the local cargo cache —
  use these versions (offline-safe). If a different ratatui version is used, pin the
  matching crossterm version.
- Existing deps to keep: `clap 3.x`, `rand 0.8`, `miette 3.x`, `local-ip-address 0.4`
  (for the LAN placeholder). Deps to **remove** once the refactor is done:
  `read_input`, `owo-colors` (colors move to ratatui; the only remaining non-TUI
  output is `--info-card`, which also becomes a TUI screen).

---

## 2. Current-State Analysis (what we are refactoring)

| File | Lines | Role | Verdict |
|------|-------|------|---------|
| `src/main.rs` | 135 | clap arg parsing, dispatch (game / demo / lan / info-card) | Rework: dispatch into TUI app |
| `src/base_game.rs` | 2529 | board, settings, chains, stock, bank, **player (mixed I/O+state)**, ui (text rendering) | Split: pure core + TUI rendering |
| `src/game.rs` | 1077 | `GameManager`, `Round`, `HotelChainManager`, `final_account`, `print_info_card` | Replaced by `Game` state machine + core managers |
| `src/logic.rs` | 1258 | `EndCondition`, `place_hotel` (analyze/chain/fusion rules, **mixed with prompts**) | Split: pure rules + decision points |
| `src/network.rs` | 307 | LAN client/server, broadcast, `send_string` | **Stub/placeholder** (R3) |
| `src/demo.rs` | 280 | demo board setup (random / clever), mostly pure except prints | Keep setup logic, drop prints, render via TUI |
| `src/data_stream.rs` | 6 | `read_enter()` helper | **Delete** |
| `src/utils.rs` | 53 | `generate_number_vector`, `remove_content_from_vec`, `chains_to_print` | Keep (used by core/bot) |

### The core problem

Game logic and I/O are interleaved:

- `Player` carries `tcp_stream: Option<TcpStream>`, `small_board`, and I/O methods
  (`read_input`, `get_enter`, `print_text_ln`) used throughout the game loop.
- `GameManager::start_game`, `Round::player_turn`, `place_hotel::*`, `Player::buy_stocks`,
  `Player::handle_fusion_stocks` all prompt the user inline and block on stdin/TCP.
- `network::broadcast` / `broadcast_others` are called from the middle of game logic.
- `ui::main_ui` builds a `Vec<String>` text dump of the whole state.

This blocking, I/O-in-the-middle design cannot be rendered incrementally in a TUI and
cannot be driven by a bot. **The fix: separate "what the game needs a decision about"
from "how the decision is obtained".**

---

## 3. Target Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                        main.rs (clap)                              │
│   parses ALL existing flags → builds Settings + player roster      │
└──────────────┬─────────────────────────────────────────────────────┘
               │
   ┌───────────┴───────────┐        ┌────────────────────────┐
   │  TUI app (tui/)       │        │  network/ (STUB)       │
   │  ratatui + crossterm  │        │  placeholder backend    │
   │  implements `Decider` │        │  implements `Decider`  │
   │  for the human        │        │  (not implemented yet) │
   └───────────┬───────────┘        └────────────┬───────────┘
               │            ┌──────────┐          │
               └───────────▶│  core/   │◀─────────┘
                            │ (pure)   │   bot/ implements `Decider`
                            │ no io,   │
                            │ no print │
                            └──────────┘
```

### 3.1 Core (`core/`) — pure game engine, zero I/O

New module layout (content migrated from `base_game.rs` / `logic.rs` / `game.rs`,
with all `print!`, `stdin`, `TcpStream`, `owo-colors` and `read_input` usage removed):

```
src/core/
  mod.rs        – re-exports
  board.rs      – Board, Piece, Position, AnalyzedPosition, letter (from base_game::board)
  chains.rs     – HotelChain, PriceLevel (from base_game::hotel_chains)
  stock.rs      – Stocks, STOCK_BASE_PRICE, stock_price (from base_game::stock)
  bank.rs       – Bank, LargestShareholders (from base_game::bank)
  players.rs    – Player STATE ONLY: id, name, money, owned_stocks,
                  analyzed_cards, decider (see 3.2). No I/O methods.
  rules.rs      – analyze_position, PlaceHotelCase, IllegalPlacement,
                  surrounding_positions, longest_chain (from logic.rs)
  endcond.rs    – EndCondition, check_end_condition (from logic.rs)
  fusion.rs     – fusion-order resolution logic (resolve_fusion_order*, fuse steps)
                  WITHOUT prompts — returns what decisions are needed
  chains_mgr.rs – HotelChainManager (from game.rs, unchanged logic)
  demo.rs       – set_hotel_chains_clever/random (from demo.rs, prints removed)
  game.rs       – Game state machine (replaces GameManager + Round)  ★ the key piece
  log.rs        – LogEntry / audience tagging (replaces broadcast/broadcast_others)
```

Rules the core must keep **byte-for-byte in behavior** (these tests guard them):
stock prices, chain start/extend/fuse bookkeeping, largest-shareholder computation,
majority bonuses (incl. rounding-up-to-100 split rules), exchange (2:1) rules,
end conditions, illegal-placement detection.

### 3.2 The `Game` state machine (replaces `GameManager` + `Round`)

The engine never asks a human anything. It advances until it needs a decision,
yields the decision request, and resumes when a decision is applied.

```rust
// core/game.rs (sketch — implement as such)

pub struct Game {
    pub board: Board,
    pub bank: Bank,
    pub chains: HotelChainManager,
    pub deck: Vec<Position>,
    pub players: Vec<Player>,
    pub settings: Settings,
    pub round_number: u32,
    pub current_player: usize,
    pub log: Vec<LogEntry>,
    phase: Phase,
    // fusion bookkeeping when phase == Fusion:
    fusion: Option<FusionState>,
}

pub enum Phase {
    Setup,            // determine first player (each draws 1 card, placed on board,
                      // lowest card goes first) — mirrors current start_game()
    PlaceCard,        // current player picks one legal card (or pass + optional redraw)
    ChooseChain,      // placement starts a chain → pick which chain
    Fusion,           // sub-stages: order → per-pair: bonuses → holders' stocks → fuse
    EndCondition,     // condition met → ask whether to end the game
    BuyStocks,        // buy up to 3 stocks total
    Confirm,          // confirm a pending transaction (respects skip_dialogues)
    DrawCard,         // draw 1 card (automatic, logged)
    GameOver,
}

/// One unit of progress the engine reports to the host (TUI / test harness).
pub enum Step {
    /// Something happened; render/log it and keep stepping.
    Event(LogEntry),
    /// Player `player` must answer `request`. Host gets the decision and calls
    /// `Game::apply_decision` with the matching Decision.
    Input(usize, InputRequest),
    /// Game finished; final ranking ready.
    Finished(FinalResult),
}

impl Game {
    pub fn new(settings: Settings, roster: Vec<(String, DeciderKind)>) -> Result<Self>;
    /// Advance the engine until it needs input or finishes. Never blocks.
    pub fn step(&mut self) -> Result<Step>;
    /// Apply a decision for the pending Input. Invalid decisions return Err
    /// (the host shows the error and re-prompts).
    pub fn apply_decision(&mut self, player: usize, decision: Decision) -> Result<()>;
    pub fn pending(&self) -> Option<(usize, &InputRequest)>;
    pub fn is_over(&self) -> bool;
}
```

Decision / request types (sketch):

```rust
pub enum InputRequest {
    ChooseCard { options: Vec<CardOption> },            // CardOption = analyzed card + legality
    Pass { can_redraw: bool },                           // only-illegal-cards situation
    ChooseChain { available: Vec<HotelChain> },
    FusionOrder { chains: Vec<HotelChain> },             // pick survivor (3–4 chain case)
    FusionPairOrder { chains: Vec<HotelChain> },         // equal-length 2-chain case
    FusionStocks { holder: u32, dead: HotelChain, alive: HotelChain,
                   max_exchange: u32, max_sell: u32 },
    EndGame { condition: EndCondition, description: String },
    BuyStocks { table: Vec<BuyRow>, remaining_slots: u32 },  // BuyRow: chain, price,
                                                             // available, max_affordable
    Confirm { summary: String },
}

pub enum Decision {
    Card(Position),
    Pass { redraw: bool },
    Chain(HotelChain),
    FusionSurvivor(HotelChain),
    FusionOrder(Vec<HotelChain>),                        // ordered, last = survivor
    FusionStocks { exchange: u32, sell: u32 },
    EndGame(bool),
    Buy(HashMap<HotelChain, u32>),
    Confirm(bool),
}
```

Flow mapping (current code → new phases), keep semantics identical:

| Current (blocking) | New phase sequence |
|---|---|
| `GameManager::start_game` (draw, place, order) | `Setup` (events only) |
| `Round::player_turn` → `place_hotel::place_hotel` | `PlaceCard` → (`ChooseChain` \| `Fusion`) |
| `fuse_chains` / `fuse_two_chains` / `handle_fusion_stocks` | `Fusion` sub-stages |
| `check_end_condition` + Y/n prompt | `EndCondition` |
| `Player::buy_stocks` + `get_correct` | `BuyStocks` → `Confirm` |
| `Player::draw_card` ("press enter" dialogs) | `DrawCard` (events; skipped prompts per `skip_dialogues`) |
| `final_account` | `Finished(FinalResult)` — ranking, bonuses, stock liquidation |

Log/audience (replaces `network::broadcast` / `broadcast_others`):

```rust
pub struct LogEntry { pub round: u32, pub audience: Audience, pub text: String }
pub enum Audience { Everyone, Not(u32) }   // Not(player_id) == broadcast_others
```

In single-player the TUI shows all entries (optionally dimming `Not(human)` —
"other players" chatter — behind a toggle; default: show everything). In future
multiplayer the audience field is exactly what routing needs.

### 3.3 Deciders (the multiplayer placeholder, R3)

```rust
// core/players.rs
pub enum DeciderKind { Human, Bot { seed: u64 }, Network /* placeholder */ }

// core/decider.rs
/// Anything that can answer the engine's InputRequests.
pub trait Decider: Send {
    fn decide(&self, request: &InputRequest, game: &Game) -> Decision;
    fn name(&self) -> &str;
}

pub struct Bot { /* seed, rng */ }                 // bot/bot.rs
impl Decider for Bot { /* heuristics, section 4 */ }

// network/mod.rs  (PLACEHOLDER)
/// Future implementation: a Decider that serializes the request over the
/// existing wire protocol ($Input/$Print/... — protocol spec kept below),
/// shows it to the remote user, and returns their answer.
pub struct NetworkDecider { /* ... */ }
```

- `Player` stores `decider: DeciderKind` (plus name/id/money/stocks/cards).
- `Game` resolves `DeciderKind` → `Box<dyn Decider>` when it needs to ask; the
  TUI hosts the human decider, `bot::Bot` the bots, `network::NetworkDecider`
  the (future) remote players. **Game logic never knows which one it is.**
- `network/mod.rs` keeps: `start_server` / `start_client` entry points that return
  a friendly "multiplayer is not implemented in this build" result, the
  `ClientPlayer` type, and a `///`-documented spec of the old wire protocol
  (`$Init`, `$Name`, `$Print`, `$Println`, `$Input`, `$Ping`, `$TERMINATE`,
  `$GameEnded`) so a future re-implementation has the contract. The old
  broadcast helpers move to `core/log.rs` as pure log tagging.

### 3.4 Bot (single-player mode, R2)

`bot/bot.rs` — deterministic given a seed (use `rand::StdRng::seed_from_u64`).
Heuristics (simple, explainable, good enough to be beatable by a human):

| Request | Strategy |
|---|---|
| `ChooseCard` | Score each **legal** card: `ExtendsChain` = `100·stock_price(chain, new_len) + 10·new_len` (+200 bonus if bot is L1/S2 of that chain); `NewChain` = `50 + 10·founding_size` (+50 if a High/Medium price-level chain is still available); `Fusion` = `20·stock_price(survivor, total_len)` (deliberately low: avoid fusions unless huge); `SingleHotel` = `5`. Pick max, ties broken randomly. |
| `Pass` | `redraw = true`. |
| `ChooseChain` | Among available chains, pick the one with the **highest price level** (L<M<H); tie → random. |
| `FusionOrder`/`FusionSurvivor` | Survivor = longest chain; tie → higher price level. Order = ascending length (rules already force this when lengths differ). |
| `FusionStocks` | Exchange if `price(alive) ≥ 2·price(dead)`, else sell all. (Keep = 0 value.) |
| `EndGame` | `true` when offered. |
| `BuyStocks` | Up to 3 total, affordable only: (a) 1 stock in each chain where bot is largest shareholder and the chain has < 11 hotels (protect bonus); (b) else, up to 2 stocks in a chain the bot can extend (it holds a hand card adjacent to it); (c) else pass. |
| `Confirm` | `true`. |

### 3.5 TUI design (ratatui)

`src/tui/`:

```
src/tui/
  mod.rs       – App struct, terminal init/restore (raw mode, alt screen,
                 restore on drop — a TerminalGuard struct), main run loop
  input.rs     – key mapping per pending InputRequest
  render.rs    – layout composition (ratatui Layout)
  widgets/
    board.rs   – 9×12 grid (rows A–I, cols 1–12)
    panel.rs   – player panel: money, hand, owned stocks
    table.rs   – chain table: name, price level [L/M/H], length, price range,
                 bank stocks, own stocks, stock price, 10× / 5× bonuses,
                 ★/☆ (largest / 2nd largest — hidden if hide_extra_info)
    log.rs     – scrollable event log (last N entries)
    dialog.rs  – generic centered dialog: prompt, options, error line
  screens/
    setup.rs   – players 2–6, human name, bot names, settings summary
    play.rs    – main board + panel + table + log + pending dialog
    gameover.rs– final ranking (1st gold, 2nd silver, 3rd bronze — port
                 the final_account colors), per-player wealth breakdown
    infocard.rs– the stock-value table (port of print_info_card)
    help.rs    – key bindings + brief rules reminder
    demo.rs    – demo board view (--demo), Esc/q quits
    lan.rs     – friendly "multiplayer placeholder" screen for --lan-* flags
```

**Layout (min. 110×32, else "terminal too small" overlay):**

```
┌──────────────────────────────────────────────────────────────────────┐
│ Acquire          Round 3 · Your turn: You                            │
├──────────────────────────────────────┬───────────────────────────────┤
│  1  2  3  4  5  6  7  8  9 10 11 12  │  YOU  (id 0)                  │
│ A ·  ·  F  F  ·  ·  ·  ·  ·  ·  ·  · │  Money: 3 400 €               │
│ B ·  A  A  ·  L  L  ·  ·  ·  ·  ·  · │  Hand:                        │
│ C ·  ·  ·  L  L  ·  ·  C  C  C  ·  · │   1) B3  [Extend Luxor +2]    │
│ D ·  ·  ·  ·  ·  ·  P  P  ·  ·  ·  · │   2) E7  [Start chain]        │
│ E F  F  F  ·  ·  ·  P  ·  ·  ·  ·  · │   3) H2  [Illegal: fusion]    │
│ F ·  ·  ·  ·  O  O  O  ·  ·  ·  ·  · │  Stocks: Luxor 2 · Airport 1  │
│ G ·  ·  ·  O  O  ·  ·  ·  ·  ·  ·  · │ ┌──────────────────────────┐  │
│ H ·  ·  ·  ·  ·  ·  ·  C  C  ·  ·  · │ │ Chains (7)               │  │
│ I ·  ·  ·  ·  ·  ·  ·  ·  ·  ·  P  P │ │ Luxor   M  4  [6-10] ... │  │
│ 1  2  3  4  5  6  7  8  9 10 11 12   │ │ Airport L  3  [3]    ... │  │
├──────────────────────────────────────┴─┴─────────────────────────────┤
│ Log: Bot 2 extended Airport by 2 · Bot 2 bought 1× Airport (400 €)   │
└──────────────────────────────────────────────────────────────────────┘
```

- Chain colors: map `HotelChain::color()` (`Rgb`) → `ratatui::style::Color::Rgb`
  (exact same 7 colors as today). Inactive chains dimmed. Unplaced cells dim;
  placed-unaffiliated hotels rendered as `X` in white; chain cells show the
  chain identifier (`A C F I L O P`) in the chain color.
- Highlighting: when a hand card is selected, highlight its board position
  (reversed) plus the positions it would found/extend (secondary style);
  illegal cards greyed out (as today, via `AnalyzedPosition::is_illegal`).
- `small_board` setting → compact variant: cell width 2 instead of 3, hide the
  log panel (accessible with `L`).

**Event loop** (`tui/mod.rs`):

```
init terminal (raw + alt screen, restore guard)
loop {
    crossterm::event::poll(Duration::from_millis(10))?
    if let Event::Key(k) = ev {
        if pending input → map key to Decision (input.rs) → game.apply_decision(...)
        else → global keys (i info, h help, L log, q quit-confirm, resize)
    }
    // drive the engine until it wants input or ends
    loop {
        match game.step()? {
            Step::Event(e)  => push to log buffer,
            Step::Input(p,r)=> { pending = Some((p,r)); break }
            Step::Finished(x) => { screen = GameOver(x); break }
        }
    }
    draw(frame)   // render.rs, always full redraw each loop
}
```

Bot turns therefore flow automatically: the engine yields `Event`s (bot
decisions are applied by the engine itself via the player's `Decider`), and the
UI just watches the log/board update. Optional `--slow-bots` later; default:
instant but visible via log entries.

**Key map** (per context; shown in `help.rs`):

| Context | Keys |
|---|---|
| Global | `i` info card · `h` help · `q` quit (confirm) · `L` toggle log · `Esc` cancel dialog |
| ChooseCard | `1`–`9` pick card · `p` pass · `Esc` back |
| ChooseChain | chain identifier keys `a c f i l o p` (case-insensitive) |
| FusionOrder | `1`–`4` pick survivor/next in order · `Esc` back |
| FusionStocks | `<` `>` or arrows adjust exchange count, `s` toggle sell count, `Enter` confirm |
| BuyStocks | `1`–`7` select chain row, `<` `>` adjust 0..max, `Enter` finish buying |
| EndGame / Confirm | `y` / `n` |
| GameOver | `q` quit |

### 3.6 `main.rs` & CLI (R4)

Keep clap 3.x and **all existing flags** with the same names/shorts. Behavior in the
new build:

| Flag | New behavior |
|---|---|
| `-p/--players` (2–6) | Total players: **1 human + (N−1) bots**. Becomes optional with **default 4**; the setup screen also allows changing it (2–6). |
| `-n/--name` | Human player name (no longer requires `--lan-server`; harmless if given with `--lan-*` too). |
| `-h/--hide-extra-info` | Hides ★/☆ largest-shareholder markers in the chain table. |
| `--skip-dialogues` | Engine auto-answers `Confirm` requests with `true` and omits `DrawCard` pause events. |
| `--info-card` | Opens the TUI directly on the info-card screen. |
| `--demo` / `--demo-type` (0/1) | Opens the TUI on the demo board (clever / random setup from `demo.rs`). |
| `--lan-server`, `--lan-client`, `--ip`, `--port` | Accepted (conflicts preserved), then show the **LAN placeholder screen**: "Multiplayer is not implemented in this build. The protocol spec lives in `src/network/mod.rs`." + `q` to quit. No crash. |

Dispatch order (as today): `--demo` → demo screen; `--info-card` → info screen;
`--lan-server/--lan-client` → placeholder screen; else → setup screen + game.

---

## 4. Bot Details (reference implementation)

Full behavior spec is in 3.4. Additional requirements:

- `Bot::decide` must be a **pure function of (request, game, rng)** — no printing,
  no sleeping, no panics on any engine state.
- Bot seeds: default `seed = 0x5EED + player_index`; a hidden `--seed N` flag may be
  added for reproducible sessions (used by the headless test).
- The bot must never select an illegal card, an unavailable chain, or exceed
  money/bank limits — the engine also rejects invalid decisions, but the bot
  should aim to not need retries.

## 5. Invariants (must hold in every state)

Add these as assertions in the headless test (and `debug_assert!` in the core):

1. Per chain: `bank.stocks_for_sale(chain) + Σ player.owned_stocks(chain) == 25`.
2. Every placed board cell belongs to ≤ 1 chain; chain membership in
   `HotelChainManager` matches board cells exactly.
3. `player.money` never underflows (u32 — engine must check before charging).
4. Deck never exceeds 108 − placed cards; total placed cards == `deck` removals.
5. Turn order cycles through all players exactly once per round until game over.

## 6. Phased Implementation Plan

Each phase ends in a green `cargo build && cargo test`. Work in this order; do not
start phase N+1 until phase N's acceptance criteria are met.

### Phase 0 — Dependencies & skeleton (small)
1. `Cargo.toml`: add `ratatui = { version = "0.26", features = ["crossterm"] }` and
   `crossterm = "0.27"`. Keep everything else for now.
2. Create empty module tree `src/core/mod.rs`, `src/bot/mod.rs`, `src/tui/mod.rs`,
   `src/network.rs` (stub) so later phases have homes.
- **Accept:** `cargo build` green; `cargo test` still 18/18.

### Phase 1 — Pure core (large, the heart of the rework)
1. Migrate `base_game.rs` → `core/{board,chains,stock,bank,players}.rs`:
   - Remove `owo-colors` from these types (keep `color() -> Rgb` data; drop
     `.color(...)` styling calls — rendering is the TUI's job).
   - `Player`: remove `tcp_stream`, `read_input`, `get_enter`, `print_text_ln`,
     `read_card`, `buy_stocks`, `handle_fusion_stocks`, `player_ui`, `draw_card`
     (I/O parts), `small_board`. Keep state + `only_illegal_cards`, `sort_cards`,
     `remove_card`, `add_card`, `analyze_cards`.
   - `Bank`: remove `print_largest_shareholders`; `give_majority_shareholder_bonuses`
     takes a `&mut Vec<LogEntry>` instead of printing/broadcasting.
2. Migrate `logic.rs` → `core/{rules,endcond,fusion}.rs`: split every function at
   its prompt boundaries into *pure compute* + *decision request*.
   - `analyze_position`, `PlaceHotelCase`, `IllegalPlacement`, `longest_chain`,
     `surrounding_positions`, `extend_chain` stay pure.
   - `place_hotel`, `start_chain`, `fuse_chains`, `fuse_two_chains`,
     `resolve_fusion_order*` become phase logic inside `Game` that emits
     `InputRequest`s instead of calling `player.read_input`.
3. Replace `GameManager` + `Round` with the `Game` state machine (3.2), including
   `Setup` (first-player determination exactly as in `start_game` today, incl. the
   placed-but-unaffiliated order cards) and `final_account` → `Finished(FinalResult)`.
4. Replace `network::broadcast*` with `core/log.rs` entries; keep `utils.rs`.
5. Port all 18 existing tests to the new layout (drop their `ui::print_main_ui_console`
   calls). Remove `data_stream.rs`, `demo.rs` prints (move setup into `core/demo.rs`),
   delete `src/base_game.rs`, `src/game.rs`, `src/logic.rs`.
6. `network/mod.rs`: placeholder per 3.3 (protocol doc + friendly error entry points).
- **Accept:**
  - `cargo build` green, `cargo test` green (all ported tests).
  - `grep -rE "print!|println!|eprint|stdin|TcpStream|read_input|owo_colors" src/core/`
    returns nothing.
  - A test `game_can_be_constructed_and_stepped` constructs a 2-player all-bot
    `Game` and steps `Setup` through `Finished` by answering every `Input` with a
    trivial valid `Decision` (a minimal "ScriptedDecider" — also the seed for the
    real bot).

### Phase 2 — Bot + headless full game
1. Implement `bot/bot.rs` per section 3.4/4 (`Bot` + `Decider` impl, seeded).
2. Add `core/game.rs::run_until_finished(&mut self)` helper (step loop applying
   each player's decider) — this is the headless driver.
3. Tests:
   - `full_game_2p_4p_6p_completes` — all-bot games with fixed seeds finish with
     exactly one ranking; invariants from section 5 asserted.
   - `bot_never_illegal` — during the game, assert every bot decision was accepted
     on first try (log rejected decisions; allow a small retry budget but expect 0).
   - `fusion_exercises` — at least one test script forces a 2-chain fusion and a
     3-chain equal-length fusion (re-use the setups from `logic.rs` tests) and
     checks bonuses/exchange/sell bookkeeping.
   - `buy_stock_limits` — bot can never exceed 3 stocks/turn or its money.
4. Remove `read_input` and `owo-colors` from `Cargo.toml` (now unused).
- **Accept:** `cargo test` green including 3+ full bot-vs-bot games; no I/O deps left
  outside `tui/`, `network/` (stub), `main.rs`.

### Phase 3 — TUI (large)
1. `tui/mod.rs`: terminal guard, event loop (3.5), screen state machine
   (`Setup → Play → GameOver`, plus `InfoCard`/`Help` overlays).
2. `widgets/board.rs`: render `Board` (9×12), chain colors, highlights for the
   selected card's effect (founding/extend positions), dim inactive.
3. `widgets/{panel,table,log}.rs`: player panel, chain table (with ★/☆ per
   `hide_extra_info`), scrolling log fed by `Step::Event`.
4. `screens/*`: setup (names/players), play (layout from 3.5), gameover
   (port `final_account` ranking colors), infocard (port `print_info_card`),
   help, demo, lan placeholder.
5. `input.rs`: key mapping per 3.5 table; invalid keys → inline error line
   (same text the engine would produce), no state corruption.
6. Human decider: pending `InputRequest` rendered as `dialog.rs`; keys produce the
   `Decision`; `apply_decision` errors re-render with the error.
7. Handle window resize (re-layout next frame; min-size overlay), and clean
   terminal restore on quit/panic (guard).
- **Accept (manual):** `cargo run -- -p 4` gives a playable game vs 3 bots covering:
  card pick (legal+illegal), chain start, 2-chain fusion with stock exchange/sell,
  stock buying (affordable + not), end-condition prompt, final ranking; `i`, `h`,
  `q`, resize all behave; terminal restored after quit.

### Phase 4 — CLI wiring & modes (small)
1. `main.rs`: build `Settings` + roster from args per 3.6; dispatch to TUI app
   (setup / demo / infocard / lan placeholder).
2. `--skip-dialogues` → engine auto-confirms; `--small-board` → compact layout;
   `--hide-extra-info` → no stars; `--demo-type` 0/1 → clever/random.
3. `--lan-server/--lan-client` (with `--ip/--port/--name`) → placeholder screen;
   keep clap conflicts exactly as today.
4. Update `README.md`: new usage, feature list, multiplayer status ("placeholder;
   protocol spec in `src/network/mod.rs`"), screenshot/ASCII of the TUI.
- **Accept:** `cargo run -- --help` lists all original flags; every flag in the 3.6
  table behaves as specified; `cargo test` still green.

### Phase 5 — Polish & hardening (small–medium)
1. `cargo clippy --all-targets` and `cargo fmt` clean (no new warnings).
2. 8-color terminal sanity (chain identifiers still distinguishable by letter),
   very small terminals (overlay instead of panic), `q` mid-dialog.
3. Edge cases from the rules tests: only-illegal-cards → pass/redraw path; 4-chain
   fusion; end-condition after last card drawn; deck exhaustion (no card drawn).
4. Re-run the full manual checklist (section 7) once more; fix anything found.
- **Accept:** all of the above pass; `cargo build --release` green.

## 7. Manual Verification Checklist (final gate)

- [ ] `cargo run` (defaults) → setup screen → playable game vs 3 bots, winnable, the TUI is fully working
- [ ] `-p 2` and `-p 6` work (1 + 1 / 1 + 5 bots).
- [ ] Chain start: correct chain chosen, founder bonus stock, table updates.
- [ ] 2-chain fusion: order by length, equal-length prompt, bonuses paid (10×/5×),
      exchange (2:1) / sell / keep all reachable, board recolored.
- [ ] Stock buying: max 3 total, affordability enforced, prices update with length.
- [ ] `--hide-extra-info` removes ★/☆; without it they appear when applicable.
- [ ] `--skip-dialogues` removes confirm pauses; without them they appear.
- [ ] `--small-board` compact layout; normal layout otherwise.
- [ ] `--info-card` shows the stock table; `i` in-game shows the same.
- [ ] `--demo` (and `--demo-type 1`) render the demo board; `q` exits cleanly.
- [ ] `--lan-server -p 3 --name X` and `--lan-client --ip 127.0.0.1:11511`
      show the friendly placeholder screen and exit cleanly (no crash, terminal
      restored).
- [ ] End condition (41+ chain or all-safe) prompts Y/n; ending runs final
      account, ranking with gold/silver/bronze, correct winner.
- [ ] Terminal size: 110×32 min enforced with overlay; resize mid-game OK;
      terminal always restored after quit (also on `q` from any dialog).

## 8. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Phase 1 refactor subtly changes rules | Existing 18 tests ported first; section-5 invariants in the headless test; bot-vs-bot games as integration test; diff behavior only at I/O boundaries. |
| Borrow-checker pain in the state machine (players + bank + chains all `&mut`) | `Game` owns everything; phases use explicit index-based access (as today's code does with `player_index`), no cross-borrows across the `step` boundary; fusion bookkeeping in a separate `FusionState` struct. |
| ratatui version mismatch with offline cache | Pin `ratatui 0.26` + `crossterm 0.27` (both cached); verify with `cargo build --offline` once. |
| Blocking prompts inside `apply_decision` re-introduce I/O in core | Lint rule: `src/core` may not `use std::io` (enforced by the Phase 1 grep acceptance check + clippy). |
| Bot loops (e.g. endless re-prompts) | Engine counts rejected decisions per request; >5 rejections for the same request → force a valid fallback decision (first legal option) and log it. |
| Scope creep into real multiplayer | R3 is placeholders only: no sockets, no threads, no async. `network/` must compile but contain no working network code. |

## 9. Definition of Done

- All phases' acceptance criteria met; `cargo build`, `cargo test`,
  `cargo clippy`, `cargo fmt --check` green.
- Checklist in section 7 fully ticked by a human.
- `README.md` reflects the new TUI usage and the multiplayer placeholder status.
- No `read_input` / `owo-colors` dependencies remain; `src/core` is I/O-free.
