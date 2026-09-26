//! Pure player state.
//!
//! The original `Player` carried a `TcpStream` and a `small_board` display flag and a
//! number of I/O methods (`draw_card`, `read_card`, `get_enter`, ...). Those belong to the
//! I/O layer, so the core `Player` only keeps the game-relevant data and the pure
//! bookkeeping methods.

use std::cmp::Ordering;

use miette::{miette, Result};

use super::board::{AnalyzedPosition, Board, Position};
use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::stock::Stocks;

/// Stores all game-relevant variables that belong to the player.
#[derive(Clone, Debug)]
pub struct Player {
    /// The money the player currently has.
    pub money: u32,
    /// The stocks that the player currently owns.
    pub owned_stocks: Stocks,
    /// Contains the cards that the player currently has on his hand.
    pub analyzed_cards: Vec<AnalyzedPosition>,
    /// The id of the player (This should be the index at which this player is stored in the
    /// players vector in the game manager).
    pub id: u32,
    /// The name of the player.
    pub name: String,
}

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        self.id == other.id && self.name == other.name && self.money == other.money
    }
}

/// Players will be sorted by id, if id is same than by name.
impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.id.cmp(&other.id) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => match self.name.cmp(&other.name) {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => Some(Ordering::Equal),
            },
        }
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.id.cmp(&other.id) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match self.name.cmp(&other.name) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => Ordering::Equal,
            },
        }
    }
}

impl Eq for Player {}

impl Player {
    /// Creates a new player with a custom name.
    pub fn new(start_cards: Vec<Position>, id: u32, name: String) -> Self {
        let mut cards = Vec::new();
        for position in start_cards {
            cards.push(AnalyzedPosition::new_unchecked(position));
        }
        Self {
            money: 6000,
            owned_stocks: Stocks::new(),
            analyzed_cards: cards,
            id,
            name,
        }
    }

    /// Add money to the player.
    pub fn add_money(&mut self, money: u32) {
        self.money += money;
    }

    /// Remove money from the player.
    pub fn remove_money(&mut self, money: u32) {
        self.money -= money;
    }

    /// Add stocks that the player owns.
    pub fn add_stocks(&mut self, chain: &HotelChain, amount: u32) {
        self.owned_stocks.increase_stocks(chain, amount);
    }

    /// Remove stocks that the player owns.
    pub fn remove_stocks(&mut self, chain: &HotelChain, amount: u32) {
        self.owned_stocks.decrease_stocks(chain, amount);
    }

    /// Returns true if the player has no cards that can be played.
    pub fn only_illegal_cards(&self) -> bool {
        for card in &self.analyzed_cards {
            if !card.is_illegal() {
                return false;
            }
        }
        true
    }

    /// Sorts the player's current hand cards.
    pub fn sort_cards(&mut self) {
        self.analyzed_cards.sort()
    }

    /// Removes a card from the player's inventory.
    /// Returns the removed card when the card has been removed successfully.
    /// Otherwise `None` is returned.
    #[allow(dead_code)]
    pub fn remove_card(&mut self, position: &Position) -> Result<AnalyzedPosition> {
        self.sort_cards();
        for (index, analyzed_card) in self.analyzed_cards.iter().enumerate() {
            if analyzed_card.position.letter.eq(&position.letter)
                && analyzed_card.position.number.eq(&position.number)
            {
                return Ok(self.analyzed_cards.remove(index));
            }
        }
        Err(miette!(
            "Unable to remove card from player, the requested card {} could not be found.",
            position
        ))
    }

    /// Adds a card to the player's inventory.
    /// Analyzes the position.
    pub fn add_card(
        &mut self,
        position: &Position,
        board: &Board,
        hotel_chain_manager: &HotelChainManager,
    ) {
        self.analyzed_cards
            .push(AnalyzedPosition::new(*position, board, hotel_chain_manager));
        self.sort_cards();
    }

    /// Analyzes the player's hand cards and updates the place hotel case value.
    pub fn analyze_cards(&mut self, board: &Board, hotel_chain_manager: &HotelChainManager) {
        for card in &mut self.analyzed_cards {
            card.check(board, hotel_chain_manager);
        }
    }
}

/// Returns a reference to the player with the entered id.
pub fn player_by_id(id: u32, players: &[Player]) -> Option<&Player> {
    for player in players {
        if player.id == id {
            return Some(player);
        }
    }
    None
}
