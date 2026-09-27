//! The ratatui-based TUI for Acquire.
//!
//! This module is the *host* for the pure [`crate::core::game::Game`] engine. It
//!
//! * initialises/restores the terminal (raw mode + alternate screen) with a guard that
//!   cleans up even on panic,
//! * drives the engine, answering every request for a bot seat with a
//!   [`crate::bot::Bot`] and pausing for a human seat,
//! * renders the board, player panel, chain table and log, and
//! * maps key presses to [`crate::core::game::Decision`]s for the pending request.
//!
//! The engine itself never touches I/O; all of that lives here.

pub mod render;

use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, Result};
use ratatui::{backend::Backend, Terminal};

use crate::bot::Bot;
use crate::core::bank::Bank;
use crate::core::board::Board;
use crate::core::chains::HotelChain;
use crate::core::chains_mgr::HotelChainManager;
use crate::core::decider::{Decider, DeciderKind, ScriptedDecider};
use crate::core::demo;
use crate::core::game::{Decision, FinalResult, Game, InputRequest, Step};
use crate::core::players::Player;
use crate::core::settings::Settings;

/// Restores the terminal on drop. Constructed after the terminal is put into raw mode and the
/// alternate screen is entered, so that quitting *or* panicking always leaves a sane terminal.
pub struct TerminalGuard;

impl TerminalGuard {
    /// Puts the terminal into raw mode and the alternate screen.
    pub fn new() -> Result<Self> {
        enable_raw_mode().map_err(|e| miette!(format!("failed to enable raw mode: {e}")))?;
        execute!(io::stdout(), EnterAlternateScreen)
            .map_err(|e| miette!(format!("failed to enter alternate screen: {e}")))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// The top-level screen the app is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Choose the number of players and confirm the settings, then start.
    Setup,
    /// The main game: board + panel + table + log + pending dialog.
    Play,
    /// Final ranking.
    GameOver,
    /// A static, pre-built board (`--demo`).
    Demo,
    /// Friendly "multiplayer is not implemented" screen (`--lan-*`).
    Lan,
}

/// An overlay shown on top of the current screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    /// The stock-value table (`--info-card` or `i`).
    InfoCard,
    /// Key bindings + rules reminder (`h`).
    Help,
}

/// The result of a single engine step, decoupled from the borrow of `self.game`.
#[derive(Debug)]
pub enum StepAction {
    /// An event occurred (log text, possibly empty).
    Event(String),
    /// The human must decide.
    HumanInput(usize, InputRequest),
    /// A bot decision was rejected and the fallback also failed.
    BotError(String),
    /// The game finished with a final result.
    Finished(FinalResult),
}

/// A fully built demo board (no game in progress).
#[derive(Clone, Debug)]
pub struct DemoState {
    pub board: Board,
    pub chains: HotelChainManager,
    pub bank: Bank,
    #[allow(dead_code)]
    pub player: Player,
}

/// The whole TUI application state.
pub struct App {
    pub settings: Settings,
    pub screen: Screen,
    pub overlay: Option<Overlay>,
    /// The in-progress game (only on the `Play`/`GameOver` screens).
    pub game: Option<Game>,
    /// One decider per seat; `deciders[i]` acts on `game.players[i]`. The human's slot is never
    /// used (the human provides the decision interactively).
    pub deciders: Vec<Box<dyn Decider>>,
    /// The human's player name (used to tell their seat apart from the bots').
    pub human_name: String,
    /// Number of players (2-6) chosen on the setup screen.
    pub player_count: u8,
    /// The pending input request the human must answer, if any.
    pub pending: Option<(usize, InputRequest)>,
    /// The last error the engine reported for the human's decision (shown in the dialog).
    pub error: Option<String>,
    /// The human-readable game log (fed by `Step::Event` and the engine's `Game::log`).
    pub log_lines: Vec<String>,
    /// How many log lines to scroll back (0 = newest at the bottom).
    pub log_scroll: usize,
    /// Whether the log panel is visible (toggled with `l`).
    pub show_log: bool,
    /// The final ranking (set when the game finishes).
    pub final_result: Option<FinalResult>,
    // -- dialog state (per pending request) -------------------------------
    /// Index of the highlighted hand card (for `ChooseCard`).
    pub selected_card: usize,
    /// Index of the highlighted chain (for `ChooseChain`).
    pub selected_chain: usize,
    /// Chains selected so far for a `FusionOrder` request, in the order pressed.
    pub fusion_order_sel: Vec<HotelChain>,
    /// Current exchange/sell counts for a `FusionStocks` request.
    pub fusion_exchange: u32,
    pub fusion_sell: u32,
    /// Per-row buy counts for a `BuyStocks` request (parallel to the request's `table`).
    pub buy_counts: Vec<u32>,
    /// The currently selected row for a `BuyStocks` request.
    pub buy_row: usize,
    /// The demo board (only on the `Demo` screen).
    pub demo: Option<DemoState>,
    /// Which setup field is currently being edited.
    pub setup_field: SetupField,
}

/// Which field on the setup screen is currently focused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupField {
    /// Number of players (2-6).
    Players,
    /// The human's name.
    Name,
}

impl App {
    /// Creates a new app for a fresh game.
    pub fn new(settings: Settings, human_name: String, player_count: u8) -> Result<Self> {
        let count = player_count.clamp(2, 6);
        Ok(Self {
            settings,
            screen: Screen::Setup,
            overlay: None,
            game: None,
            deciders: Vec::new(),
            human_name,
            player_count: count,
            pending: None,
            error: None,
            log_lines: Vec::new(),
            log_scroll: 0,
            show_log: true,
            final_result: None,
            selected_card: 0,
            selected_chain: 0,
            fusion_order_sel: Vec::new(),
            fusion_exchange: 0,
            fusion_sell: 0,
            buy_counts: Vec::new(),
            buy_row: 0,
            demo: None,
            setup_field: SetupField::Players,
        })
    }

    /// Creates an app that opens directly on the demo board.
    pub fn new_demo(settings: Settings, human_name: String, demo_type: u8) -> Result<Self> {
        let mut app = Self::new(settings, human_name, 2)?;
        app.screen = Screen::Demo;
        app.demo = Some(Self::build_demo(demo_type)?);
        Ok(app)
    }

    /// Creates an app that opens directly on the LAN placeholder.
    pub fn new_lan(settings: Settings, human_name: String) -> Result<Self> {
        let mut app = Self::new(settings, human_name, 2)?;
        app.screen = Screen::Lan;
        Ok(app)
    }

    /// Builds a demo board using the pure setup helpers in `core::demo`.
    pub fn build_demo(demo_type: u8) -> Result<DemoState> {
        let mut board = Board::new();
        let mut chains = HotelChainManager::new();
        let mut bank = Bank::new();
        let mut player = Player::new(Vec::new(), 0, String::from("You"));
        let mut deck: Vec<crate::core::board::Position> = Vec::new();
        for c in crate::core::board::letter::LETTERS {
            for i in 1..=12 {
                deck.push(crate::core::board::Position::new(c, i));
            }
        }
        let mut active: Vec<HotelChain> = Vec::new();
        if demo_type == 1 {
            demo::set_hotel_chains_random(
                &mut active,
                &mut player,
                &mut deck,
                &mut board,
                &mut chains,
                &mut bank,
            )?;
        } else {
            demo::set_hotel_chains_clever(
                &mut active,
                &mut player,
                &mut deck,
                &mut board,
                &mut chains,
                &mut bank,
            )?;
        }
        bank.update_largest_shareholders(&[player.clone()]);
        Ok(DemoState {
            board,
            chains,
            bank,
            player,
        })
    }

    /// Starts a new game with the current player count, creating one bot per non-human seat.
    pub fn start_game(&mut self) -> Result<()> {
        let n = self.player_count as usize;
        let mut roster: Vec<(String, DeciderKind)> = Vec::with_capacity(n);
        roster.push((self.human_name.clone(), DeciderKind::HUMAN));
        for i in 1..n {
            roster.push((format!("Bot {i}"), DeciderKind::bot(Bot::default_seed(i))));
        }
        let mut game = Game::new(self.settings.clone(), roster)?;
        // One bot decider per seat, tied to the seat index. The human's slot is never called.
        let deciders: Vec<Box<dyn Decider>> = (0..n)
            .map(|i| Box::new(Bot::new(Bot::default_seed(i), i)) as Box<dyn Decider>)
            .collect();
        // Warm the engine: run the whole `Setup` phase (it is events-only) so the first
        // decision the host sees is a real turn.
        while let Step::Event(text) = game.step()? {
            self.push_log(&text);
        }
        self.game = Some(game);
        self.deciders = deciders;
        self.screen = Screen::Play;
        self.pending = None;
        Ok(())
    }

    /// Advances the engine until it needs the human's input, the game finishes, or it errors.
    ///
    /// Bot seats are answered immediately with their [`crate::bot::Bot`]; the loop pauses the
    /// moment a human seat is asked to decide.
    pub fn drive_engine(&mut self) -> Result<()> {
        let mut budget = 200_000u32;
        loop {
            budget = budget
                .checked_sub(1)
                .ok_or_else(|| miette!("engine did not reach a decision within the step budget"))?;
            let action = self.step_once()?;
            match action {
                StepAction::Event(text) => {
                    self.push_log(&text);
                }
                StepAction::HumanInput(player, request) => {
                    self.pending = Some((player, request.clone()));
                    self.init_dialog_state(&request);
                    break;
                }
                StepAction::BotError(msg) => {
                    self.push_log(&msg);
                    // The engine re-prompts on the next step; keep going.
                }
                StepAction::Finished(result) => {
                    self.final_result = Some(result);
                    self.screen = Screen::GameOver;
                    self.pending = None;
                    break;
                }
            }
        }
        Ok(())
    }

    /// The result of a single engine step, decoupled from the borrow of `self.game`.
    fn step_once(&mut self) -> Result<StepAction> {
        let game = self.game.as_mut().ok_or_else(|| {
            miette!("there is no game to drive (expected only on the Play screen)")
        })?;
        match game.step()? {
            Step::Event(text) => Ok(StepAction::Event(text)),
            Step::Input(player, request) => {
                let is_human = game.players[player].name == self.human_name;
                if is_human {
                    Ok(StepAction::HumanInput(player, request))
                } else {
                    let decision = self.deciders[player].decide(&request, game);
                    match game.apply_decision(player, decision) {
                        Ok(()) => Ok(StepAction::Event(String::new())),
                        Err(e) => {
                            // Fall back to a guaranteed-valid answer so the game cannot stall.
                            let fallback = ScriptedDecider.decide(&request, game);
                            match game.apply_decision(player, fallback) {
                                Ok(()) => Ok(StepAction::Event(String::new())),
                                Err(e2) => Ok(StepAction::BotError(format!(
                                    "bot decision rejected ({e}); fallback also rejected ({e2})"
                                ))),
                            }
                        }
                    }
                }
            }
            Step::Finished(result) => Ok(StepAction::Finished(result)),
        }
    }

    /// Pushes a line to the log and keeps the scroll offset in range.
    pub fn push_log(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        self.log_lines.push(line.to_string());
        // Keep the log from growing without bound.
        if self.log_lines.len() > 500 {
            let excess = self.log_lines.len() - 500;
            self.log_lines.drain(..excess);
            self.log_scroll = self.log_scroll.saturating_sub(excess);
        }
    }

    /// Initializes the dialog state for a freshly-pending request.
    fn init_dialog_state(&mut self, request: &InputRequest) {
        self.selected_card = 0;
        self.selected_chain = 0;
        self.fusion_order_sel.clear();
        self.fusion_exchange = 0;
        self.fusion_sell = 0;
        self.buy_row = 0;
        match request {
            InputRequest::BuyStocks { table, .. } => {
                self.buy_counts = vec![0; table.len()];
            }
            _ => self.buy_counts.clear(),
        }
    }

    /// Applies the human's decision. On error the request is re-prompted with the error shown.
    fn apply_human(&mut self, player: usize, request: &InputRequest, decision: Decision) {
        match self
            .game
            .as_mut()
            .expect("a pending request implies a running game")
            .apply_decision(player, decision)
        {
            Ok(()) => {
                self.pending = None;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.pending = Some((player, request.clone()));
            }
        }
    }

    /// Handles a key press. Returns `true` when the app should quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // `q` always quits, no matter what is on screen (overlay, pending dialog, setup, play).
        // It must be checked first so it is never swallowed by a context-specific handler.
        if key.code == KeyCode::Char('q') {
            return Ok(true);
        }

        // Overlay: Esc closes it, everything else is ignored while the overlay is up.
        if self.overlay.is_some() {
            return match key.code {
                KeyCode::Esc => {
                    self.overlay = None;
                    Ok(false)
                }
                _ => Ok(false),
            };
        }

        // A pending request is answered by the context-specific key map.
        if let Some((player, request)) = self.pending.clone() {
            return self.handle_pending_key(player, &request, key);
        }

        // Setup screen: edit fields and start the game.
        if self.screen == Screen::Setup {
            return self.handle_setup_key(key);
        }

        // Global keys.
        Ok(match key.code {
            KeyCode::Char('q') => true,
            KeyCode::Esc => {
                if self.screen == Screen::Play {
                    self.overlay = Some(Overlay::Help);
                }
                false
            }
            KeyCode::Char('i') => {
                self.overlay = Some(Overlay::InfoCard);
                false
            }
            KeyCode::Char('h') => {
                self.overlay = Some(Overlay::Help);
                false
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.show_log = !self.show_log;
                false
            }
            KeyCode::Up => {
                self.log_scroll = self.log_scroll.saturating_add(3);
                false
            }
            KeyCode::Down => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
                false
            }
            _ => false,
        })
    }

    /// Handles key presses on the setup screen. Returns `true` when the app should quit.
    fn handle_setup_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                self.setup_field = match self.setup_field {
                    SetupField::Players => SetupField::Name,
                    SetupField::Name => SetupField::Players,
                };
                Ok(false)
            }
            KeyCode::Enter => {
                if self.human_name.trim().is_empty() {
                    self.human_name = String::from("You");
                }
                self.start_game()?;
                Ok(false)
            }
            // Player count: digits 2-6.
            KeyCode::Char(c) if c.is_ascii_digit() && self.setup_field == SetupField::Players => {
                let n = c.to_digit(10).unwrap() as u8;
                if (2..=6).contains(&n) {
                    self.player_count = n;
                }
                Ok(false)
            }
            // Name field: printable characters.
            KeyCode::Char(c) if self.setup_field == SetupField::Name => {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    self.human_name.push(c);
                }
                Ok(false)
            }
            KeyCode::Backspace if self.setup_field == SetupField::Name => {
                self.human_name.pop();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Maps a key to a [`Decision`] for the pending request (see `render::dialog` for the map).
    fn handle_pending_key(
        &mut self,
        player: usize,
        request: &InputRequest,
        key: KeyEvent,
    ) -> Result<bool> {
        match request {
            InputRequest::ChooseCard { legal } => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let idx = c.to_digit(10).unwrap() as usize - 1;
                    if idx < legal.len() {
                        self.apply_human(player, request, Decision::Card(legal[idx]));
                    }
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected_card = self.selected_card.saturating_sub(1).max(0);
                    if self.selected_card >= legal.len() {
                        self.selected_card = legal.len() - 1;
                    }
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected_card = (self.selected_card + 1) % legal.len().max(1);
                    Ok(false)
                }
                KeyCode::Enter => {
                    if self.selected_card < legal.len() {
                        self.apply_human(
                            player,
                            request,
                            Decision::Card(legal[self.selected_card]),
                        );
                    }
                    Ok(false)
                }
                _ => Ok(false),
            },
            InputRequest::ChooseChain { available } => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected_chain = self.selected_chain.saturating_sub(1);
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected_chain = (self.selected_chain + 1) % available.len().max(1);
                    Ok(false)
                }
                KeyCode::Enter => {
                    if self.selected_chain < available.len() {
                        self.apply_human(
                            player,
                            request,
                            Decision::Chain(available[self.selected_chain]),
                        );
                    }
                    Ok(false)
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let idx = c.to_digit(10).unwrap() as usize - 1;
                    if idx < available.len() {
                        self.apply_human(player, request, Decision::Chain(available[idx]));
                    }
                    Ok(false)
                }
                _ => Ok(false),
            },
            InputRequest::FusionOrder { chains, .. } => match key.code {
                KeyCode::Char(c) => {
                    if let Some(chain) = HotelChain::from_identifier(c.to_ascii_lowercase()) {
                        if chains.contains(&chain) && !self.fusion_order_sel.contains(&chain) {
                            self.fusion_order_sel.push(chain);
                            if self.fusion_order_sel.len() == chains.len() {
                                let ordered = self.fusion_order_sel.clone();
                                self.pending = None;
                                self.error = None;
                                self.apply_human(player, request, Decision::FusionOrder(ordered));
                            }
                        }
                    }
                    Ok(false)
                }
                KeyCode::Backspace => {
                    self.fusion_order_sel.pop();
                    Ok(false)
                }
                _ => Ok(false),
            },
            InputRequest::FusionStocks {
                max_exchange,
                max_sell,
                ..
            } => {
                let max_exchange = *max_exchange;
                let max_sell = *max_sell;
                Ok(match key.code {
                    KeyCode::Left | KeyCode::Char('<') => {
                        self.fusion_exchange = self.fusion_exchange.saturating_sub(2);
                        false
                    }
                    KeyCode::Right | KeyCode::Char('>') => {
                        if self.fusion_exchange + 2 <= max_exchange {
                            self.fusion_exchange += 2;
                        }
                        false
                    }
                    KeyCode::Up | KeyCode::Char('+') => {
                        if self.fusion_sell < max_sell - self.fusion_exchange {
                            self.fusion_sell += 1;
                        }
                        false
                    }
                    KeyCode::Down | KeyCode::Char('-') => {
                        self.fusion_sell = self.fusion_sell.saturating_sub(1);
                        false
                    }
                    KeyCode::Enter => {
                        self.apply_human(
                            player,
                            request,
                            Decision::FusionStocks {
                                exchange: self.fusion_exchange,
                                sell: self.fusion_sell,
                            },
                        );
                        false
                    }
                    _ => false,
                })
            }
            InputRequest::EndGame { .. } => Ok(match key.code {
                KeyCode::Char('y') => {
                    self.apply_human(player, request, Decision::EndGame(true));
                    false
                }
                KeyCode::Char('n') => {
                    self.apply_human(player, request, Decision::EndGame(false));
                    false
                }
                _ => false,
            }),
            InputRequest::BuyStocks {
                table,
                remaining_slots,
            } => {
                let remaining = *remaining_slots;
                Ok(match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let idx = c.to_digit(10).unwrap() as usize - 1;
                        if idx < table.len() {
                            self.buy_row = idx;
                        }
                        false
                    }
                    KeyCode::Up => {
                        if self.buy_row > 0 {
                            self.buy_row -= 1;
                        }
                        false
                    }
                    KeyCode::Down => {
                        if self.buy_row + 1 < table.len() {
                            self.buy_row += 1;
                        }
                        false
                    }
                    KeyCode::Left | KeyCode::Char('<') => {
                        if self.buy_counts[self.buy_row] > 0 {
                            self.buy_counts[self.buy_row] -= 1;
                        }
                        false
                    }
                    KeyCode::Right | KeyCode::Char('>') => {
                        let total: u32 = self.buy_counts.iter().sum();
                        let cap = table[self.buy_row].max_affordable;
                        if self.buy_counts[self.buy_row] < cap && total < remaining {
                            self.buy_counts[self.buy_row] += 1;
                        }
                        false
                    }
                    KeyCode::Enter => {
                        let mut map: HashMap<HotelChain, u32> = HashMap::new();
                        for (row, count) in table.iter().zip(self.buy_counts.iter()) {
                            if *count > 0 {
                                map.insert(row.chain, *count);
                            }
                        }
                        self.apply_human(player, request, Decision::Buy(map));
                        false
                    }
                    _ => false,
                })
            }
            InputRequest::Pass { can_redraw } => Ok(match key.code {
                KeyCode::Char('y') | KeyCode::Char('r') => {
                    self.apply_human(
                        player,
                        request,
                        Decision::Pass {
                            redraw: *can_redraw,
                        },
                    );
                    false
                }
                KeyCode::Char('n') => {
                    self.apply_human(player, request, Decision::Pass { redraw: false });
                    false
                }
                _ => false,
            }),
        }
    }

    /// Runs the application until it is quit. Restores the terminal on the way out.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let guard = TerminalGuard::new()?;
        loop {
            // Drive the engine while we are playing and not waiting on the human.
            if self.screen == Screen::Play
                && self.pending.is_none()
                && self.overlay.is_none()
                && self.game.is_some()
            {
                if let Err(e) = self.drive_engine() {
                    self.error = Some(e.to_string());
                    break;
                }
            }
            terminal
                .draw(|f| render::render(f, self))
                .map_err(|e| miette!(format!("terminal draw failed: {e}")))?;
            // Sleep briefly so a full game does not burn 100% CPU while bots play.
            let has_event = event::poll(Duration::from_millis(16))
                .map_err(|e| miette!(format!("event poll failed: {e}")))?;
            if !has_event {
                continue;
            }
            let event = event::read().map_err(|e| miette!(format!("event read failed: {e}")))?;
            match event {
                Event::Key(key) => {
                    if self.handle_key(key)? {
                        break;
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        drop(guard);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives a 2-player (1 human + 1 bot) game to completion by answering every human request
    /// with the first valid option, mirroring what a human pressing the first key would do.
    #[test]
    fn tui_drives_a_game_to_finish() {
        let mut app = App::new(Settings::default(), String::from("You"), 2).unwrap();
        app.start_game().unwrap();
        let mut budget = 500_000;
        loop {
            budget -= 1;
            if budget == 0 {
                panic!("game did not finish within the budget");
            }
            if app.screen == Screen::GameOver {
                break;
            }
            if app.pending.is_none() {
                app.drive_engine().unwrap();
                continue;
            }
            let (player, request) = app.pending.clone().unwrap();
            let decision = ScriptedDecider.decide(&request, &app.game.as_ref().unwrap());
            app.apply_human(player, &request, decision);
        }
        assert!(app.final_result.is_some());
        assert!(!app.final_result.as_ref().unwrap().ranking.is_empty());
    }
}
