# Acquire_rs — Implementation Status

Cross-check of the current codebase against `PLAN.md`. Generated 2026-09-27.

## Summary

| Phase | Status | Notes |
|-------|--------|-------|
| 0 — Dependencies & skeleton | **Done** | ratatui 0.26 + crossterm 0.27 added; module tree exists |
| 1 — Pure core | **Done** | `core/` is I/O-free; all rules ported; `ScriptedDecider` present |
| 2 — Bot + headless game | **Done** | `bot/bot.rs` implements all strategies; 19 tests pass |
| 3 — TUI | **Done (functional)** | Playable single-player game; all screens render; key handling works |
| 4 — CLI wiring & modes | **Done (partial)** | All flags present; dispatch works; README not updated |
| 5 — Polish & hardening | **Partial** | Build clean, 0 warnings; some edge cases not verified |

**Build:** `cargo build` — 0 errors, 0 warnings.
**Tests:** `cargo test` — 19/19 passing.
**Clippy:** No errors; some style suggestions in core/bot (non-blocking).

---

## Phase-by-Phase Detail

### Phase 0 — Dependencies & skeleton ✅

- [x] `ratatui 0.26` + `crossterm 0.27` in `Cargo.toml`
- [x] Module tree: `src/core/`, `src/bot/`, `src/tui/` exist and are wired in `main.rs`
- [x] `cargo build` green

**Remaining:**
- [ ] `read_input` and `owo-colors` still in `Cargo.toml` (no longer used by active code — should be removed)
- [ ] `local-ip-address` still in `Cargo.toml` (unused by active code; only the dead `network.rs` references it — remove or keep for future LAN work)
- [ ] `colored` (windows-only target) still present — unused by active code

### Phase 1 — Pure core ✅

- [x] `core/board.rs` — `Board`, `Piece`, `Position`, `AnalyzedPosition`, `letter`
- [x] `core/chains.rs` — `HotelChain`, `PriceLevel`
- [x] `core/stock.rs` — `Stocks`, `STOCK_BASE_PRICE`, `stock_price`
- [x] `core/bank.rs` — `Bank`, `LargestShareholders`
- [x] `core/players.rs` — `Player` state only (no I/O)
- [x] `core/rules.rs` — `analyze_position`, `PlaceHotelCase`, `IllegalPlacement`, `surrounding_positions`, `longest_chain`
- [x] `core/endcond.rs` — `EndCondition`, `check_end_condition`
- [x] `core/fusion.rs` — fusion-order resolution
- [x] `core/chains_mgr.rs` — `HotelChainManager`
- [x] `core/demo.rs` — `set_hotel_chains_clever/random`
- [x] `core/game.rs` — `Game` state machine with `Step`/`InputRequest`/`Decision`
- [x] `core/log.rs` — `LogEntry` / `Audience`
- [x] `core/decider.rs` — `Decider` trait, `DeciderKind`, `ScriptedDecider`
- [x] `core/settings.rs` — `Settings` struct
- [x] `grep -rE "print!|println!|eprint|stdin|TcpStream|read_input|owo_colors" src/core/` → only in doc comments
- [x] `game_can_be_constructed_and_stepped` test exists and passes

**Deviations from plan (acceptable):**
- No `Confirm` phase — the engine auto-confirms transactions inline (simpler, no user-visible pause). The plan's `Confirm` phase is effectively absorbed.
- No `DrawCard` phase — card drawing is logged as an `Event`, not a distinct phase.
- `InputRequest::ChooseChain` uses `available: Vec<HotelChain>` (not `Vec<CardOption>` from the sketch) — functionally equivalent.
- `Decision` has no `Confirm(bool)` variant — not needed without a `Confirm` phase.

### Phase 2 — Bot + headless game ✅

- [x] `bot/bot.rs` — seeded, pure `Decider` implementation
- [x] All strategies from §3.4 implemented:
  - [x] `ChooseCard` — scored (extend/new/fusion/single)
  - [x] `Pass` — `redraw = true`
  - [x] `ChooseChain` — highest price level, tie → random
  - [x] `FusionOrder` — survivor = longest, order ascending
  - [x] `FusionStocks` — exchange if `price(alive) ≥ 2·price(dead)`, else sell all
  - [x] `EndGame` — `true`
  - [x] `BuyStocks` — 3-tier strategy (protect bonus / extend / pass)
- [x] `Game::run_until_finished` headless driver
- [x] Tests:
  - [x] `full_game_2p_4p_6p` — all-bot games complete
  - [x] `bot_never_illegal` — all decisions accepted
  - [x] `fusion_exercises` — 2-chain and 3-chain fusion
  - [x] `buy_stock_limits` — max 3/turn, money enforced
- [ ] `read_input` and `owo-colors` still in `Cargo.toml` (see Phase 0)

### Phase 3 — TUI ✅ (functional)

**Architecture:** Monolithic — `tui/mod.rs` (App state + event loop + key handling) + `tui/render.rs` (all rendering). The plan's `widgets/` and `screens/` subdirectories exist but are empty.

- [x] `tui/mod.rs` — `App` struct, `TerminalGuard`, event loop, screen state machine
- [x] `tui/render.rs` — all screens rendered (setup, play, gameover, demo, lan, infocard, help)
- [x] Board rendering (9×12 grid, chain colors, card highlight, small_board mode)
- [x] Player panel (money, hand with descriptions, owned stocks)
- [x] Chain table (name, level, length, range, bank, own, price, 10×, 5×, ★/☆)
- [x] Scrolling event log
- [x] Dialog for all pending `InputRequest` types
- [x] Setup screen (player count, name entry, Tab to switch fields, Enter to start)
- [x] Game over screen (ranking with gold/silver/bronze colors)
- [x] Info card (stock value table)
- [x] Help screen (key bindings + rules)
- [x] Demo board view
- [x] LAN placeholder screen
- [x] Minimum size check (100×24) with "too small" overlay
- [x] Terminal restore on quit/panic (`TerminalGuard`)
- [x] Resize handled (re-layout next frame)

**Key map (implemented):**

| Context | Plan | Actual |
|---------|------|--------|
| Global | `i`, `h`, `q`, `L`, `Esc` | ✅ All present |
| Setup | digits, Enter, Tab | ✅ (added: Tab to switch field, name typing) |
| ChooseCard | `1`–`9`, `p`, `Esc` | ✅ `1`–`9` + `↑/↓` + `Enter` (no `p` pass key — pass is auto-offered by engine) |
| ChooseChain | chain letters | ✅ `↑/↓` + `Enter` + digits (user requested arrow-key selection) |
| FusionOrder | `1`–`4` | ✅ chain letters in order + Backspace |
| FusionStocks | `<` `>`, `↑↓`, `Enter` | ✅ `<` `>` + `↑↓` + `Enter` |
| BuyStocks | `1`–`7`, `<` `>`, `Enter` | ✅ `1`–`7` + `<` `>` + `Enter` |
| EndGame | `y` / `n` | ✅ |
| GameOver | `q` | ✅ |

**Deviations from plan:**
- `widgets/` and `screens/` are empty — all code is in `mod.rs` + `render.rs`. Functionally complete; restructuring would be refactoring, not a gap.
- No dedicated `input.rs` — key handling is inline in `mod.rs::handle_pending_key`.
- `ChooseChain` uses arrow keys + Enter (user preference) instead of the plan's chain-letter keys.

### Phase 4 — CLI wiring & modes ⚠️ (partial)

- [x] All flags present and functional: `-p`, `-n`, `-h`, `--lan-client`, `--lan-server`, `--ip`, `--port`, `--info-card`, `--skip-dialogues`, `--demo`, `--demo-type`, `-s`
- [x] Dispatch: demo → lan → info-card → setup + game
- [x] `--small-board` → compact layout
- [x] `--hide-extra-info` → hides ★/☆
- [x] `--info-card` → opens on info card screen
- [x] `--demo` / `--demo-type` → demo board
- [x] `--lan-server` / `--lan-client` → placeholder screen
- [x] Clap conflicts preserved
- [ ] **`--skip-dialogues` is stored in `Settings` but has no behavioral effect** — the engine has no `Confirm` phase or `DrawCard` pause to skip. In practice this flag is a no-op. This is acceptable since the TUI flow has no explicit confirm pauses, but should be documented.
- [ ] **README.md is stale** — still describes the old CLI game with zip downloads and LAN multiplayer as a feature

### Phase 5 — Polish & hardening ⚠️ (partial)

- [x] `cargo build` — 0 errors, 0 warnings
- [x] `cargo test` — 19/19 passing
- [x] `cargo fmt` clean
- [x] Clippy: no errors (some style suggestions in core/bot: manual `div_ceil`, `&mut Vec`, non-canonical `partial_cmp` — non-blocking)
- [ ] 8-color terminal sanity check (manual)
- [ ] Very small terminal behavior verified (min-size overlay works, but not stress-tested)
- [ ] `q` mid-dialog from any screen (implemented, but not manually verified for all screens)
- [ ] Only-illegal-cards → pass/redraw path (tested in bot, not manually verified in TUI)
- [ ] 4-chain fusion (tested in bot `fusion_exercises`, not manually verified in TUI)
- [ ] End-condition after last card drawn (edge case)
- [ ] Deck exhaustion (no card drawn) (edge case)
- [ ] `cargo build --release` green (not yet verified)
- [ ] Full manual checklist from §7 not yet executed by a human

---

## Cleanup Items

| Item | Action | Priority |
|------|--------|----------|
| `read_input` in Cargo.toml | Remove (unused by active code) | Low |
| `owo-colors` in Cargo.toml | Remove (unused by active code) | Low |
| `local-ip-address` in Cargo.toml | Remove or keep (only dead `network.rs` uses it) | Low |
| `colored` in Cargo.toml | Remove (windows-only, unused) | Low |
| 7 dead files in `src/` | `base_game.rs`, `game.rs`, `logic.rs`, `network.rs`, `demo.rs`, `data_stream.rs`, `utils.rs` — delete or move to `src/legacy/` | Low |
| Empty `tui/widgets/` and `tui/screens/` dirs | Remove or populate (cosmetic) | Low |
| README.md | Rewrite for TUI usage, feature list, multiplayer placeholder status | Medium |
| `--skip-dialogues` no-op | Add a note in help text or implement a confirm phase | Low |

---

## What Would Make This "Done" per PLAN.md §9

1. ✅ All phases' acceptance criteria met (Phases 0-3 fully; Phase 4 partial — README)
2. ⬜ Checklist in §7 fully ticked by a human
3. ⬜ README.md reflects new TUI usage and multiplayer placeholder status
4. ⬜ No `read_input` / `owo-colors` dependencies remain
5. ✅ `src/core` is I/O-free
6. ⬜ Old dead files deleted (or archived)
7. ⬜ `cargo build --release` verified

**Bottom line:** The game is playable end-to-end (setup → play vs bots → game over). The core engine, bot, and TUI are all functional. Remaining work is cleanup (dead files, unused deps, README) and manual verification of edge cases.
