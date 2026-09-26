//! The `Game` state machine.
//!
//! This replaces the old `GameManager` + `Round` and the `place_hotel::*` functions in
//! `logic.rs`. The engine never blocks on input: `step()` advances the game until it needs
//! a decision, yields an [`InputRequest`], and resumes when the host feeds back a
//! [`Decision`] via [`Game::apply_decision`].
//!
//! All the prompting that used to live inline (`read_card`, `read_input`, `get_enter`,
//! `get_correct`, `broadcast`) has been split out into discrete phases.

use std::collections::HashMap;

use miette::{miette, Result};
use rand::Rng;

use super::bank::Bank;
use super::board::{Board, Position};
use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::decider::{Decider, DeciderKind};
use super::endcond::{check_end_condition, EndCondition};
use super::fusion::{order_to_result, resolve_order, OrderResolution};
use super::log::LogEntry;
use super::players::Player;
use super::rules::{analyze_position, IllegalPlacement, PlaceHotelCase};
use super::settings::Settings;

/// The board letters, re-exported for convenience.
use super::board::letter::LETTERS;

/// Draws a random card from the deck, removing it. Returns `None` when the deck is empty.
pub fn draw_card(position_cards: &mut Vec<Position>) -> Result<Option<Position>> {
    if position_cards.is_empty() {
        return Ok(None);
    }
    let random_number = rand::thread_rng().gen_range(0..position_cards.len());
    let position = position_cards.remove(random_number);
    Ok(Some(position))
}

/// One unit of progress the engine reports to the host (TUI / test harness).
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    /// Something happened; the summary is recorded and the detailed line is in `Game::log`.
    Event(String),
    /// Player `player` must answer `request`. The host obtains a [`Decision`] and calls
    /// [`Game::apply_decision`] with the matching decision.
    Input(usize, InputRequest),
    /// The game finished; the final ranking is ready.
    Finished(FinalResult),
}

/// A request for a decision from the current player.
#[derive(Clone, Debug, PartialEq)]
pub enum InputRequest {
    /// Pick one of the legal cards in `legal`.
    ChooseCard { legal: Vec<Position> },
    /// The player has no legal card; `can_redraw` offers to redraw the hand.
    Pass { can_redraw: bool },
    /// A new chain is being started; pick which of the `available` chains it is.
    ChooseChain { available: Vec<HotelChain> },
    /// A fusion's order must be chosen: order `chains` (the last element survives unless a
    /// fixed survivor already exists).
    FusionOrder { chains: Vec<HotelChain> },
    /// A player decides what to do with their stocks of a dying chain.
    FusionStocks {
        holder: u32,
        dead: HotelChain,
        alive: HotelChain,
        max_exchange: u32,
        max_sell: u32,
    },
    /// An end condition is met; ask whether to end the game now.
    EndGame {
        condition: EndCondition,
        description: String,
    },
    /// Buy up to `remaining_slots` stocks from the listed chains.
    BuyStocks { table: Vec<BuyRow>, remaining_slots: u32 },
}

/// A row describing what can be bought for a chain.
#[derive(Clone, Debug, PartialEq)]
pub struct BuyRow {
    pub chain: HotelChain,
    pub price: u32,
    pub available: u32,
    pub max_affordable: u32,
}

/// The answer to an [`InputRequest`].
#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    /// Play this card.
    Card(Position),
    /// Pass (optionally redraw the hand).
    Pass { redraw: bool },
    /// Start this chain.
    Chain(HotelChain),
    /// The ordered fusion list; the last element is the survivor (unless fixed).
    FusionOrder(Vec<HotelChain>),
    /// Exchange `exchange` stocks and sell `sell` stocks of the dying chain.
    FusionStocks { exchange: u32, sell: u32 },
    /// Whether to end the game.
    EndGame(bool),
    /// The map of chain -> number of stocks to buy.
    Buy(HashMap<HotelChain, u32>),
}

/// The final ranking of the game.
#[derive(Clone, Debug, PartialEq)]
pub struct FinalResult {
    /// Players in descending order of money: `(name, money)`.
    pub ranking: Vec<(String, u32)>,
    /// The winner's name.
    pub winner: String,
}

/// The internal phase of the engine.
#[derive(Debug)]
enum Phase {
    Setup,
    PlaceCard,
    ChooseChain(Vec<Position>),
    Fusion,
    EndCondition(EndCondition),
    BuyStocks,
    TurnEnd,
    GameOver,
}

/// A single bookkeeping step of an in-progress fusion.
#[derive(Clone, Debug)]
enum FusionItem {
    PayBonuses { dead: HotelChain, alive: HotelChain },
    HandleStocks { player_index: usize, dead: HotelChain, alive: HotelChain },
    FuseOnBoard { dead: HotelChain, alive: HotelChain },
    AddOrigin,
}

/// Bookkeeping for an in-progress fusion.
#[derive(Debug)]
struct FusionState {
    player_index: usize,
    origin: Position,
    chains: Vec<HotelChain>,
    order_determined: bool,
    pending_survivor: Option<HotelChain>,
    items: Vec<FusionItem>,
    item_index: usize,
}

/// The pure game engine.
pub struct Game {
    /// The board.
    pub board: Board,
    /// The bank.
    pub bank: Bank,
    /// The hotel chain manager.
    pub chains: HotelChainManager,
    /// The draw deck.
    pub deck: Vec<Position>,
    /// The players (index order == turn order after setup).
    pub players: Vec<Player>,
    /// The settings.
    pub settings: Settings,
    /// The current round number.
    pub round_number: u32,
    /// The index of the player whose turn it is.
    pub current_player: usize,
    /// The game log (replaces broadcasts).
    pub log: Vec<LogEntry>,
    /// The kind of each seat (for display/logging).
    pub roster: Vec<DeciderKind>,
    phase: Phase,
    game_ending: bool,
    placed_this_turn: bool,
    final_done: bool,
    fusion: Option<FusionState>,
}

impl Game {
    /// Creates a new game with the given settings and roster (one entry per seat).
    ///
    /// The first seat is seat 0; the roster order is the *initial* order, which is
    /// re-sorted by the `Setup` phase (lowest drawn card plays first).
    pub fn new(settings: Settings, roster: Vec<(String, DeciderKind)>) -> Result<Self> {
        let n: u32 = roster.len() as u32;
        if !(2u32..=6u32).contains(&n) {
            return Err(miette!(
                "Unable to create new game: The amount of players is invalid. Valid: 2-6, entered: {}",
                roster.len()
            ));
        }

        let mut deck = Self::init_deck();
        let number_of_players = roster.len() as u32;
        let player_cards = Self::init_player_cards(number_of_players, &mut deck);

        let mut players = Vec::new();
        for (index, (name, _kind)) in roster.iter().enumerate() {
            players.push(Player::new(
                player_cards.get(index).cloned().unwrap_or_default(),
                index as u32,
                name.clone(),
            ));
        }

        let kinds = roster.iter().map(|(_, k)| *k).collect();

        Ok(Self {
            board: Board::new(),
            bank: Bank::new(),
            chains: HotelChainManager::new(),
            deck,
            players,
            settings,
            round_number: 1,
            current_player: 0,
            log: Vec::new(),
            roster: kinds,
            phase: Phase::Setup,
            game_ending: false,
            placed_this_turn: false,
            final_done: false,
            fusion: None,
        })
    }

    /// Creates a deck with every position on the board.
    fn init_deck() -> Vec<Position> {
        let mut deck = Vec::new();
        for c in LETTERS {
            for i in 1..=12 {
                deck.push(Position::new(c, i));
            }
        }
        deck
    }

    /// Deals 6 starting cards to each player, removing them from the deck.
    fn init_player_cards(
        number_of_players: u32,
        position_cards: &mut Vec<Position>,
    ) -> Vec<Vec<Position>> {
        let mut player_cards: Vec<Vec<Position>> = Vec::new();
        for _i in 1..=number_of_players {
            player_cards.push(Vec::new());
        }
        for _i in 1..=6 {
            for player in 0..=number_of_players - 1 {
                if position_cards.is_empty() {
                    break;
                }
                let random_number = rand::thread_rng().gen_range(0..position_cards.len());
                let position = position_cards.remove(random_number);
                player_cards
                    .get_mut(usize::try_from(player).unwrap())
                    .unwrap()
                    .push(position);
            }
        }
        player_cards
    }

    /// Advances the engine until it needs input or finishes. Never blocks.
    pub fn step(&mut self) -> Result<Step> {
        match &self.phase {
            Phase::Setup => self.step_setup(),
            Phase::PlaceCard => self.step_place_card(),
            Phase::ChooseChain(positions) => self.step_choose_chain(positions.clone()),
            Phase::Fusion => self.step_fusion(),
            Phase::EndCondition(condition) => Ok(Step::Input(
                self.current_player,
                InputRequest::EndGame {
                    condition: *condition,
                    description: condition.description(),
                },
            )),
            Phase::BuyStocks => self.step_buy_stocks(),
            Phase::TurnEnd => self.step_turn_end(),
            Phase::GameOver => self.step_game_over(),
        }
    }

    fn step_setup(&mut self) -> Result<Step> {
        self.log
            .push(LogEntry::everyone(self.round_number, "Starting game!"));
        self.log.push(LogEntry::everyone(
            self.round_number,
            "Each player draws a card now, the player with the lowest card starts.",
        ));
        let mut cards_with_players = HashMap::new();
        let mut cards = Vec::new();
        for (index, player) in self.players.iter().enumerate() {
            let card = match draw_card(&mut self.deck)? {
                Some(card) => card,
                None => {
                    return Err(miette!(
                        "Unable to start the game: no cards left to draw a starting card."
                    ))
                }
            };
            self.log.push(LogEntry::everyone(
                self.round_number,
                format!("{} drew card {}", player.name, card),
            ));
            self.board.place_hotel(&card)?;
            cards_with_players.insert(card, index);
            cards.push(card);
        }
        cards.sort();
        for (index, card) in cards.iter().enumerate() {
            let player_index = cards_with_players.get(card).unwrap();
            let player_name = self
                .players
                .get(*player_index)
                .unwrap()
                .name
                .clone();
            self.players.get_mut(*player_index).unwrap().id = index as u32;
            self.log.push(LogEntry::everyone(
                self.round_number,
                format!("{} is the {}. player", player_name, index + 1),
            ));
        }
        self.players.sort();
        for player in &mut self.players {
            player.analyze_cards(&self.board, &self.chains);
        }
        self.current_player = 0;
        self.round_number = 1;
        self.phase = Phase::PlaceCard;
        self.log.push(LogEntry::everyone(
            self.round_number,
            "Game started.",
        ));
        Ok(Step::Event(String::from("Game started")))
    }

    fn step_place_card(&mut self) -> Result<Step> {
        let idx = self.current_player;
        self.players[idx].analyze_cards(&self.board, &self.chains);
        self.players[idx].sort_cards();
        if self.players[idx].only_illegal_cards() {
            self.log.push(LogEntry::others(
                self.round_number,
                self.players[idx].id,
                format!("{} has no card that could be played.", self.players[idx].name),
            ));
            self.after_placement();
            return Ok(Step::Event(String::from("passed (no legal card)")));
        }
        let legal: Vec<Position> = self.players[idx]
            .analyzed_cards
            .iter()
            .filter(|c| !c.is_illegal())
            .map(|c| c.position)
            .collect();
        Ok(Step::Input(
            idx,
            InputRequest::ChooseCard { legal },
        ))
    }

    fn step_choose_chain(&mut self, _positions: Vec<Position>) -> Result<Step> {
        let available = self.chains.available_chains().ok_or_else(|| {
            miette!("Unable to start a chain: no chains are left to be founded.")
        })?;
        Ok(Step::Input(
            self.current_player,
            InputRequest::ChooseChain { available },
        ))
    }

    fn step_fusion(&mut self) -> Result<Step> {
        let f = self.fusion.as_mut().unwrap();
        if !f.order_determined {
            let resolution = resolve_order(&f.chains, &self.chains);
            match resolution {
                OrderResolution::Determined { survivor, order } => {
                    f.order_determined = true;
                    f.pending_survivor = None;
                    f.items = Self::build_fusion_items(&order, survivor, &self.players);
                    f.item_index = 0;
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[f.player_index].id,
                        format!(
                            "Fusion between {} chains at {} (order determined).",
                            f.chains.len(),
                            f.origin
                        ),
                    ));
                    return Ok(Step::Event(String::from("fusion order determined")));
                }
                OrderResolution::NeedsOrder {
                    survivor,
                    order_chains,
                } => {
                    f.pending_survivor = survivor;
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[f.player_index].id,
                        format!(
                            "{} is deciding the fusion order.",
                            self.players[f.player_index].name
                        ),
                    ));
                    return Ok(Step::Input(
                        f.player_index,
                        InputRequest::FusionOrder {
                            chains: order_chains,
                        },
                    ));
                }
            }
        }

        if f.item_index >= f.items.len() {
            // Fusion fully processed; the AddOrigin item is the last one, so this
            // normally is not reached. Transition to the end-condition check.
            self.fusion = None;
            self.placed_this_turn = true;
            self.after_placement();
            return Ok(Step::Event(String::from("fusion complete")));
        }

        let item = f.items[f.item_index].clone();
        match item {
            FusionItem::PayBonuses { dead, alive: _ } => {
                self.bank.update_largest_shareholders(&self.players);
                self.bank
                    .give_majority_shareholder_bonuses(
                        &mut self.players,
                        &dead,
                        &self.chains,
                        true,
                        self.round_number,
                        &mut self.log,
                    )?;
                f.item_index += 1;
                Ok(Step::Event(format!(
                    "paid majority bonuses for {}",
                    dead
                )))
            }
            FusionItem::HandleStocks {
                player_index,
                dead,
                alive,
            } => {
                let player_id = self.players[player_index].id;
                let holdings = *self.players[player_index]
                    .owned_stocks
                    .stocks_for_hotel(&dead);
                Ok(Step::Input(
                    player_index,
                    InputRequest::FusionStocks {
                        holder: player_id,
                        dead,
                        alive,
                        max_exchange: holdings,
                        max_sell: holdings,
                    },
                ))
            }
            FusionItem::FuseOnBoard { dead, alive } => {
                self.log.push(LogEntry::others(
                    self.round_number,
                    self.players[f.player_index].id,
                    format!("Chain {} is being fused into {}", dead, alive),
                ));
                self.chains.fuse_chains(&alive, &dead, &mut self.board)?;
                f.item_index += 1;
                Ok(Step::Event(format!("fused {} into {}", dead, alive)))
            }
            FusionItem::AddOrigin => {
                let case = analyze_position(&f.origin, &self.board, &self.chains);
                if let PlaceHotelCase::ExtendsChain(chain, positions) = case {
                    for pos in positions {
                        self.chains
                            .add_hotel_to_chain(&chain, pos, &mut self.board)?;
                    }
                }
                f.item_index += 1;
                self.fusion = None;
                self.placed_this_turn = true;
                self.after_placement();
                Ok(Step::Event(String::from("fusion complete")))
            }
        }
    }

    /// Builds the ordered list of fusion bookkeeping items for a determined order.
    fn build_fusion_items(
        order: &[HotelChain],
        survivor: HotelChain,
        players: &[Player],
    ) -> Vec<FusionItem> {
        let mut items = Vec::new();
        for dead in order {
            items.push(FusionItem::PayBonuses {
                dead: *dead,
                alive: survivor,
            });
            for (player_index, player) in players.iter().enumerate() {
                if *player.owned_stocks.stocks_for_hotel(dead) > 0 {
                    items.push(FusionItem::HandleStocks {
                        player_index,
                        dead: *dead,
                        alive: survivor,
                    });
                }
            }
            items.push(FusionItem::FuseOnBoard {
                dead: *dead,
                alive: survivor,
            });
        }
        items.push(FusionItem::AddOrigin);
        items
    }

    fn step_buy_stocks(&mut self) -> Result<Step> {
        if self.chains.active_chains().is_empty() {
            self.after_buy();
            return Ok(Step::Event(String::from("no stocks to buy")));
        }
        let active = self.chains.active_chains();
        let money = self.players[self.current_player].money;
        let mut table = Vec::new();
        for chain in active {
            let price = Bank::stock_price(&self.chains, &chain);
            let available = *self.bank.stocks_available(&chain, &self.chains);
            let max_affordable = if price == 0 {
                0
            } else {
                std::cmp::min(3, std::cmp::min(available, money / price))
            };
            table.push(BuyRow {
                chain,
                price,
                available,
                max_affordable,
            });
        }
        Ok(Step::Input(
            self.current_player,
            InputRequest::BuyStocks {
                table,
                remaining_slots: 3,
            },
        ))
    }

    fn step_turn_end(&mut self) -> Result<Step> {
        let idx = self.current_player;
        if self.placed_this_turn {
            match draw_card(&mut self.deck)? {
                Some(card) => {
                    self.players[idx].add_card(&card, &self.board, &self.chains);
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[idx].id,
                        format!("{} drew card {}", self.players[idx].name, card),
                    ));
                }
                None => {
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[idx].id,
                        format!(
                            "{}: no card can be drawn because no cards are left.",
                            self.players[idx].name
                        ),
                    ));
                }
            }
            self.advance_player();
            return Ok(Step::Event(String::from("turn end: drew card")));
        }
        if only_illegal_fusion(&self.players[idx]) {
            return Ok(Step::Input(
                idx,
                InputRequest::Pass { can_redraw: true },
            ));
        }
        self.advance_player();
        Ok(Step::Event(String::from("turn end: passed")))
    }

    fn step_game_over(&mut self) -> Result<Step> {
        if !self.final_done {
            let active = self.chains.active_chains();
            for chain in active {
                self.bank.update_largest_shareholders(&self.players);
                self.bank
                    .give_majority_shareholder_bonuses(
                        &mut self.players,
                        &chain,
                        &self.chains,
                        false,
                        self.round_number,
                        &mut self.log,
                    )?;
                for player in &mut self.players {
                    let held = *player.owned_stocks.stocks_for_hotel(&chain);
                    if held > 0 {
                        self.bank.sell_stock(player, held, &chain, &self.chains)?;
                    }
                }
            }
            self.final_done = true;
        }
        let mut ranking: Vec<(String, u32)> = self
            .players
            .iter()
            .map(|p| (p.name.clone(), p.money))
            .collect();
        ranking.sort_by(|a, b| b.1.cmp(&a.1));
        let winner = ranking
            .first()
            .map(|(n, _)| n.clone())
            .unwrap_or_default();
        self.log.push(LogEntry::everyone(
            self.round_number,
            format!("Final ranking: winner is {}", winner),
        ));
        Ok(Step::Finished(FinalResult {
            ranking,
            winner,
        }))
    }

    /// Applies a decision for the pending [`InputRequest`] and advances the engine.
    ///
    /// Invalid decisions return `Err` (the host shows the error and re-prompts).
    pub fn apply_decision(&mut self, player: usize, decision: Decision) -> Result<()> {
        match decision {
            Decision::Card(position) => self.apply_card(player, position),
            Decision::Pass { redraw } => self.apply_pass(player, redraw),
            Decision::Chain(chain) => self.apply_chain(player, chain),
            Decision::FusionOrder(ordered) => self.apply_fusion_order(player, ordered),
            Decision::FusionStocks { exchange, sell } => {
                self.apply_fusion_stocks(player, exchange, sell)
            }
            Decision::EndGame(end) => {
                self.game_ending = end;
                self.phase = Phase::BuyStocks;
                Ok(())
            }
            Decision::Buy(map) => self.apply_buy(player, map),
        }
    }

    fn apply_card(&mut self, player: usize, position: Position) -> Result<()> {
        let card = self.players[player]
            .analyzed_cards
            .iter()
            .find(|c| c.position == position)
            .ok_or_else(|| miette!("the chosen card is not in the player's hand"))?
            .clone();
        if card.is_illegal() {
            return Err(miette!("the chosen card is illegal: {}",
                match &card.place_hotel_case {
                    PlaceHotelCase::Illegal(reason) => reason.description(),
                    _ => String::new(),
                }));
        }
        let card_index = self
            .players[player]
            .analyzed_cards
            .iter()
            .position(|c| c.position == position)
            .unwrap();
        let card = self.players[player].analyzed_cards.remove(card_index);
        self.board.place_hotel(&position)?;

        match card.place_hotel_case {
            PlaceHotelCase::SingleHotel => {
                self.log.push(LogEntry::others(
                    self.round_number,
                    self.players[player].id,
                    format!("{} has placed a hotel on {}", self.players[player].name, position),
                ));
                self.placed_this_turn = true;
                self.after_placement();
            }
            PlaceHotelCase::NewChain(positions) => {
                self.phase = Phase::ChooseChain(positions);
            }
            PlaceHotelCase::ExtendsChain(chain, positions) => {
                let count = positions.len();
                for pos in positions {
                    self.chains
                        .add_hotel_to_chain(&chain, pos, &mut self.board)?;
                }
                self.log.push(LogEntry::everyone(
                    self.round_number,
                    format!(
                        "{} has extended the chain {} by {} hotel(s)",
                        self.players[player].name,
                        chain,
                        count
                    ),
                ));
                self.placed_this_turn = true;
                self.after_placement();
            }
            PlaceHotelCase::Fusion(chains, origin) => {
                self.log.push(LogEntry::everyone(
                    self.round_number,
                    format!("Fusion between {} chains at {}!", chains.len(), origin),
                ));
                self.fusion = Some(FusionState {
                    player_index: player,
                    origin,
                    chains,
                    order_determined: false,
                    pending_survivor: None,
                    items: Vec::new(),
                    item_index: 0,
                });
                self.phase = Phase::Fusion;
            }
            PlaceHotelCase::Illegal(reason) => {
                return Err(miette!("the chosen card is illegal: {}", reason.description()));
            }
        }
        Ok(())
    }

    fn apply_chain(&mut self, player: usize, chain: HotelChain) -> Result<()> {
        let positions = match &self.phase {
            Phase::ChooseChain(positions) => positions.clone(),
            _ => {
                return Err(miette!(
                    "a chain can only be chosen while starting a new chain"
                ))
            }
        };
        let available = self
            .chains
            .available_chains()
            .ok_or_else(|| miette!("no chains are available to start"))?;
        if !available.contains(&chain) {
            return Err(miette!("the chosen chain is not available to start"));
        }
        self.chains
            .start_chain(chain, positions, &mut self.board, &mut self.players[player], &mut self.bank)?;
        self.bank.update_largest_shareholders(&self.players);
        self.log.push(LogEntry::everyone(
            self.round_number,
            format!("{} has started the new chain {}", self.players[player].name, chain),
        ));
        self.placed_this_turn = true;
        self.after_placement();
        Ok(())
    }

    fn apply_fusion_order(&mut self, player: usize, ordered: Vec<HotelChain>) -> Result<()> {
        let f = self.fusion.as_ref().unwrap();
        if f.order_determined {
            return Err(miette!("the fusion order has already been determined"));
        }
        // Validate that `ordered` is a permutation of the requested chains.
        let expected = match &self.phase {
            Phase::Fusion => f.chains.clone(),
            _ => return Err(miette!("a fusion order can only be given during a fusion")),
        };
        if !same_multiset(&ordered, &expected) {
            return Err(miette!(
                "the fusion order must contain exactly the chains being fused"
            ));
        }
        let (survivor, order) = order_to_result(f.pending_survivor, ordered);
        if let Some(f) = self.fusion.as_mut() {
            f.order_determined = true;
            f.items = Self::build_fusion_items(&order, survivor, &self.players);
            f.item_index = 0;
        }
        self.log.push(LogEntry::others(
            self.round_number,
            self.players[player].id,
            format!("The fusion will have {} survive.", survivor),
        ));
        Ok(())
    }

    fn apply_fusion_stocks(
        &mut self,
        player: usize,
        exchange: u32,
        sell: u32,
    ) -> Result<()> {
        let f = self.fusion.as_ref().unwrap();
        if f.item_index >= f.items.len() {
            return Err(miette!("there are no more fusion stock decisions to make"));
        }
        let (dead, alive) = match &f.items[f.item_index] {
            FusionItem::HandleStocks { dead, alive, .. } => (*dead, *alive),
            _ => return Err(miette!("the current fusion step is not a stock decision")),
        };
        let holdings = *self.players[player].owned_stocks.stocks_for_hotel(&dead);
        if exchange % 2 != 0 {
            return Err(miette!("the number of stocks to exchange must be even"));
        }
        let available = *self.bank.stocks_available(&alive, &self.chains);
        if exchange / 2 > available {
            return Err(miette!("not enough stocks of the surviving chain to exchange"));
        }
        if exchange + sell > holdings {
            return Err(miette!("cannot exchange or sell more stocks than are owned"));
        }
        if exchange > 0 {
            self.bank
                .exchange_stock(&mut self.players[player], exchange, &dead, &alive)?;
        }
        if sell > 0 {
            self.bank.sell_stock(
                &mut self.players[player],
                sell,
                &dead,
                &self.chains,
            )?;
        }
        self.log.push(LogEntry::others(
            self.round_number,
            self.players[player].id,
            format!(
                "{} did the following with their {} stocks: Exchanged: {}, Sold: {}, Kept: {}",
                self.players[player].name,
                dead,
                exchange,
                sell,
                holdings - exchange - sell
            ),
        ));
        if let Some(f) = self.fusion.as_mut() {
            f.item_index += 1;
        }
        Ok(())
    }

    fn apply_buy(&mut self, player: usize, map: HashMap<HotelChain, u32>) -> Result<()> {
        let total: u32 = map.values().sum();
        if total > 3 {
            return Err(miette!("cannot buy more than 3 stocks in a turn"));
        }
        let mut cost = 0u32;
        for (chain, amount) in &map {
            if *amount == 0 {
                continue;
            }
            let available = *self.bank.stocks_available(chain, &self.chains);
            if *amount > available {
                return Err(miette!(
                    "not enough stocks of {} available to buy",
                    chain
                ));
            }
            let price = Bank::stock_price(&self.chains, chain);
            cost = cost.saturating_add(price.saturating_mul(*amount));
        }
        if cost > self.players[player].money {
            return Err(miette!("not enough money to buy the selected stocks"));
        }
        for (chain, amount) in map {
            if amount == 0 {
                continue;
            }
            for _ in 0..amount {
                self.bank
                    .buy_stock(&self.chains, &chain, &mut self.players[player])?;
            }
        }
        self.bank.update_largest_shareholders(&self.players);
        if total == 0 {
            self.log.push(LogEntry::others(
                self.round_number,
                self.players[player].id,
                format!("{} bought no stocks.", self.players[player].name),
            ));
        } else {
            let mut out = format!("{} bought the following stocks:", self.players[player].name);
            for (chain, amount) in self.players[player].owned_stocks.stocks.iter() {
                if *amount > 0 {
                    out.push_str(&format!("\n{}: {}", chain, amount));
                }
            }
            self.log.push(LogEntry::others(
                self.round_number,
                self.players[player].id,
                out,
            ));
        }
        self.after_buy();
        Ok(())
    }

    fn apply_pass(&mut self, player: usize, redraw: bool) -> Result<()> {
        if redraw {
            self.players[player].analyzed_cards.clear();
            match draw_card(&mut self.deck)? {
                Some(card) => {
                    self.players[player].add_card(&card, &self.board, &self.chains);
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[player].id,
                        format!("{} redrew their hand: new card {}", self.players[player].name, card),
                    ));
                }
                None => {
                    self.log.push(LogEntry::others(
                        self.round_number,
                        self.players[player].id,
                        format!(
                            "{}: no card can be drawn because no cards are left.",
                            self.players[player].name
                        ),
                    ));
                }
            }
        }
        self.advance_player();
        Ok(())
    }

    /// Returns the pending input request, if the engine is waiting for a decision.
    pub fn pending(&self) -> Option<(usize, InputRequest)> {
        match &self.phase {
            Phase::PlaceCard => {
                if self.players[self.current_player].only_illegal_cards() {
                    None
                } else {
                    let legal: Vec<Position> = self.players[self.current_player]
                        .analyzed_cards
                        .iter()
                        .filter(|c| !c.is_illegal())
                        .map(|c| c.position)
                        .collect();
                    Some((
                        self.current_player,
                        InputRequest::ChooseCard { legal },
                    ))
                }
            }
            _ => None,
        }
    }

    /// Returns `true` once the game has finished.
    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::GameOver)
    }

    /// Drives the game to completion using `decider` to answer every request.
    ///
    /// This is the headless driver used by tests and (in the future) by the TUI to run
    /// bot turns automatically.
    pub fn run_until_finished<D: Decider>(&mut self, decider: &D) -> Result<FinalResult> {
        loop {
            match self.step()? {
                Step::Event(_) => {}
                Step::Input(player, request) => {
                    let decision = decider.decide(&request, self);
                    self.apply_decision(player, decision)?;
                }
                Step::Finished(result) => return Ok(result),
            }
        }
    }

    fn after_placement(&mut self) {
        if let Some(condition) = check_end_condition(&self.board, &self.chains) {
            self.phase = Phase::EndCondition(condition);
        } else {
            self.phase = Phase::BuyStocks;
        }
    }

    fn after_buy(&mut self) {
        if self.game_ending {
            self.phase = Phase::GameOver;
        } else {
            self.phase = Phase::TurnEnd;
        }
    }

    fn advance_player(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
        if self.current_player == 0 {
            self.round_number += 1;
        }
        self.placed_this_turn = false;
        self.phase = Phase::PlaceCard;
    }
}

/// Returns `true` when the two slices contain the same elements with the same
/// multiplicities (i.e. one is a permutation of the other).
fn same_multiset(a: &[HotelChain], b: &[HotelChain]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut freq: HashMap<HotelChain, u32> = HashMap::new();
    for chain in a {
        *freq.entry(*chain).or_insert(0) += 1;
    }
    for chain in b {
        let count = freq.get_mut(chain).unwrap();
        if *count == 0 {
            return false;
        }
        *count -= 1;
    }
    true
}

/// Returns `true` when every illegal card of the player is a *fusion* illegal card
/// (i.e. none of them is a chain-start illegal card).
fn only_illegal_fusion(player: &Player) -> bool {
    let mut only_illegal_fusion = true;
    for position in &player.analyzed_cards {
        if let PlaceHotelCase::Illegal(reason) = &position.place_hotel_case {
            if reason == &IllegalPlacement::ChainStartIllegal {
                only_illegal_fusion = false;
            }
        }
    }
    only_illegal_fusion
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::decider::ScriptedDecider;

    #[test]
    fn game_can_be_constructed_and_stepped() -> Result<()> {
        let roster: Vec<(String, DeciderKind)> = vec![
            (String::from("Bot 1"), DeciderKind::bot(1)),
            (String::from("Bot 2"), DeciderKind::bot(2)),
        ];
        let mut game = Game::new(Settings::default(), roster)?;
        let decider = ScriptedDecider;
        let mut steps = 0u32;
        loop {
            let step = game.step()?;
            steps += 1;
            if steps > 50_000 {
                return Err(miette!(
                    "game did not finish within the step budget (possible infinite loop)"
                ));
            }
            match step {
                Step::Event(_) => {}
                Step::Input(player, request) => {
                    let decision = decider.decide(&request, &game);
                    game.apply_decision(player, decision)?;
                }
                Step::Finished(res) => {
                    assert!(!res.ranking.is_empty());
                    assert!(res.ranking.len() == 2);
                    return Ok(());
                }
            }
        }
    }

    #[test]
    fn game_rejects_too_few_players() {
        let roster: Vec<(String, DeciderKind)> = vec![(String::from("Bot 1"), DeciderKind::bot(1))];
        assert!(Game::new(Settings::default(), roster).is_err());
    }
}
