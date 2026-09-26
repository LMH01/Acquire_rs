//! Manages the currently active hotel chains.

use std::collections::HashMap;

use miette::{miette, Result};

use super::bank::Bank;
use super::board::{Board, Position};
use super::chains::HotelChain;
use super::players::Player;
use super::rules::{analyze_position, PlaceHotelCase};

/// Stores the currently active hotel chains.
#[derive(Clone, Debug)]
pub struct HotelChainManager {
    /// Stores the active hotel chains and the buildings that belong to the chain.
    active_chains: HashMap<HotelChain, Vec<Position>>,
}

impl HotelChainManager {
    /// Creates a new hotel manager that is used to manage the currently active hotel chains.
    pub fn new() -> Self {
        Self {
            active_chains: HashMap::new(),
        }
    }

    /// Returns the number of hotels currently built for the specified chain.
    /// If the chain is not active 0 is returned.
    pub fn chain_length(&self, hotel: &HotelChain) -> u32 {
        if !self.active_chains.contains_key(hotel) {
            return 0;
        }
        self.active_chains
            .get(hotel)
            .unwrap()
            .len()
            .try_into()
            .unwrap()
    }

    /// Returns a vector of currently active chains.
    pub fn active_chains(&self) -> Vec<HotelChain> {
        let mut chains = Vec::new();
        for k in self.active_chains.keys() {
            chains.push(*k);
        }
        chains
    }

    /// Returns true if the chain is currently active.
    pub fn chain_status(&self, hotel: &HotelChain) -> bool {
        self.active_chains.contains_key(hotel)
    }

    /// Returns the range in which the current price level of the chain is.
    pub fn price_range(&self, hotel: &HotelChain) -> String {
        let chains = match self.active_chains.contains_key(hotel) {
            true => self
                .active_chains
                .get(hotel)
                .unwrap()
                .len()
                .try_into()
                .unwrap(),
            false => 0,
        };
        let range = match chains {
            0 => "",
            2 => "    [2]",
            3 => "    [3]",
            4 => "    [4]",
            5 => "    [5]",
            6..=10 => " [6-10]",
            11..=20 => "[11-20]",
            21..=30 => "[21-30]",
            31..=40 => "[31-40]",
            _ => " [41++]",
        };

        range.to_string()
    }

    /// Returns the positions that belong to the given chain.
    pub fn chain_positions(&self, hotel: &HotelChain) -> Vec<Position> {
        self.active_chains
            .get(hotel)
            .cloned()
            .unwrap_or_default()
    }

    /// Start a new chain.
    /// The hotels on the board are updated to show the chain.
    /// The player will be given one stock as start-up bonus.
    /// When the hotel pieces have not been set on the board they will be placed.
    /// # Arguments
    /// * `hotel` - The hotel type that is founded
    /// * `positions` - The initial positions of the hotels that belong to this chain
    /// * `board` - The board on which the hotels should be updated
    /// * `player` - The player that is the founder of the new chain
    /// * `bank` - The bank that manages the available stocks
    ///
    /// # Returns
    /// A result containing `Ok(())` when the chain has been founded successfully.
    pub fn start_chain(
        &mut self,
        hotel_chain: HotelChain,
        positions: Vec<Position>,
        board: &mut Board,
        player: &mut Player,
        bank: &mut Bank,
    ) -> Result<()> {
        if positions.len() < 2 {
            return Err(miette!(
                "Unable to start new chain of hotel {}: Not enough buildings!",
                &hotel_chain
            ));
        }

        if self.active_chains.contains_key(&hotel_chain) {
            return Err(miette!(
                "Unable to start new chain of hotel {}: The chain has already been started!",
                &hotel_chain
            ));
        }
        self.active_chains.insert(hotel_chain, positions.clone());
        // Update hotels on board
        for position in positions {
            if board.is_hotel_placed(&position).is_none() {
                board.place_hotel(&position)?;
            }
            // Update single hotels that surround the placed hotel
            let analyzed_position = analyze_position(&position, board, self);
            if let PlaceHotelCase::NewChain(positions_ext) = analyzed_position {
                for p in positions_ext {
                    board.update_hotel(hotel_chain, &p)?
                }
            };
            board.update_hotel(hotel_chain, &position)?;
        }
        // Update player stocks
        bank.give_bonus_stock(&hotel_chain, player)?;
        Ok(())
    }

    /// Adds a hotel to an existing chain.
    /// Also updates the entry in the board.
    /// # Arguments
    /// * `hotel_chain` - The hotel chain to which the hotel should be added
    /// * `position` - The position of the hotel
    /// * `board` - The board on which the hotels should be updated
    ///
    /// # Returns
    /// * `Ok(())` - When the hotel was successfully added
    /// * `Err(Error)` - When the hotel chain does not exist
    pub fn add_hotel_to_chain(
        &mut self,
        hotel_chain: &HotelChain,
        position: Position,
        board: &mut Board,
    ) -> Result<()> {
        if !self.active_chains.contains_key(hotel_chain) {
            return Err(miette!("Unable to add hotel at position {} to chain {}: The chain has not been founded yet!", &position, &hotel_chain));
        }
        self.active_chains
            .get_mut(hotel_chain)
            .unwrap()
            .push(position);
        // Update hotel on board
        board.update_hotel(*hotel_chain, &position)?;
        Ok(())
    }

    /// Fuses the two hotel chains into one.
    /// Will update the board and the active chains.
    /// Will not do anything with the shares.
    /// # Arguments
    /// * `alive` - The hotel chain that survives the fusion
    /// * `dead` - The hotel chain that dies
    /// * `board` - The board where the pieces should be updated
    ///
    /// # Returns
    /// * `Ok(())` - When the hotels were merged successfully
    /// * `Err(Error)` - When the merge was not successful
    pub fn fuse_chains(
        &mut self,
        alive: &HotelChain,
        dead: &HotelChain,
        board: &mut Board,
    ) -> Result<()> {
        // Check if the two chains exist
        if !(self.active_chains.contains_key(alive) && self.active_chains.contains_key(dead)) {
            return Err(miette!("Unable to fuse chain {} into {}: At least one of the two chains does not exist!", &dead, &alive));
        }
        // Transfer positions and update board
        for position in self.active_chains.get(dead).unwrap().clone() {
            self.active_chains.get_mut(alive).unwrap().push(position);
            board.update_hotel(*alive, &position)?;
        }
        // Remove old chain
        self.active_chains.remove(dead);
        Ok(())
    }

    /// Returns a vector of hotel chains that can still be started.
    /// If no hotel chains are left `None` is returned.
    pub fn available_chains(&self) -> Option<Vec<HotelChain>> {
        let mut available = Vec::new();
        for chain in HotelChain::iterator() {
            if !self.active_chains.contains_key(chain) {
                available.push(*chain);
            }
        }
        if available.is_empty() {
            return None;
        }
        Some(available)
    }

    /// Returns true if the chain is safe. This means that it can no longer be fused into another chain.
    pub fn is_chain_safe(&self, chain: &HotelChain) -> bool {
        self.chain_length(chain) >= 11
    }
}

impl Default for HotelChainManager {
    fn default() -> Self {
        Self::new()
    }
}
