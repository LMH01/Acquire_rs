//! The bank: available stocks and money handling.
//!
//! All the I/O that used to live here (printing largest shareholders,
//! broadcasting bonus messages, waiting for enter) is replaced by appending
//! [`LogEntry`]s. The money bookkeeping is byte-for-byte the same as before.

use std::{cmp::Ordering, collections::HashMap};

use miette::{miette, Result};

use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::log::LogEntry;
use super::players::{player_by_id, Player};
use super::stock::Stocks;

/// Manages the currently available stocks and the money.
#[derive(Clone, Debug)]
pub struct Bank {
    pub stocks_for_sale: Stocks,
    /// Stores the currently largest and second largest shareholders.
    pub largest_shareholders: LargestShareholders,
}

impl Bank {
    /// Creates a new bank.
    pub fn new() -> Self {
        Self {
            stocks_for_sale: Stocks::new_bank(),
            largest_shareholders: LargestShareholders::new(),
        }
    }

    /// Returns how many stocks of the given chain are available to be bought.
    /// If the chain does not exist 0 is returned.
    pub fn stocks_available(
        &self,
        chain: &HotelChain,
        hotel_chain_manager: &HotelChainManager,
    ) -> &u32 {
        if !hotel_chain_manager.chain_status(chain) {
            return &0;
        }
        self.stocks_for_sale.stocks_for_hotel(chain)
    }

    /// Returns the current price for a stock of the given chain.
    pub fn stock_price(hotel_chain_manager: &HotelChainManager, chain: &HotelChain) -> u32 {
        chain.stock_value(hotel_chain_manager.chain_length(chain))
    }

    /// Buy a single stock from the bank.
    pub fn buy_stock(
        &mut self,
        hotel_chain_manager: &HotelChainManager,
        hotel: &HotelChain,
        player: &mut Player,
    ) -> Result<()> {
        // The currently available stocks for the given hotel that can still be bought
        let stock_available = self.stocks_available(hotel, hotel_chain_manager);
        // Check if the desired stock can still be bought
        if *stock_available == 0 {
            return Err(miette!(
                "Unable to buy stock from chain {}: No stocks available.",
                &hotel
            ));
        }
        let stock_price = Bank::stock_price(hotel_chain_manager, hotel);
        // Check if player has enough money to buy the stock
        if player.money < stock_price {
            return Err(miette!(
                "Unable to buy stock from chain {}: Not enough money.",
                &hotel
            ));
        }
        // Finally buy the stock
        self.stocks_for_sale.decrease_stocks(hotel, 1);
        player.add_stocks(hotel, 1);
        player.remove_money(stock_price);
        Ok(())
    }

    /// Sell a number of stocks back to the bank.
    /// # Returns
    /// * `Err` - When the player tries to sell more stocks than they have
    pub fn sell_stock(
        &mut self,
        player: &mut Player,
        amount: u32,
        chain: &HotelChain,
        hotel_chain_manager: &HotelChainManager,
    ) -> Result<()> {
        let player_stocks = *player.owned_stocks.stocks.get(chain).unwrap();
        if player_stocks < amount {
            return Err(miette!(
                "Unable to sell stocks: The player tried to sell {} stocks but only has {}.",
                amount,
                player.owned_stocks.stocks.get(chain).unwrap()
            ));
        }
        let stock_price = Bank::stock_price(hotel_chain_manager, chain);
        // Move stocks from player's inventory to the bank
        player.owned_stocks.set_stocks(chain, 0);
        self.stocks_for_sale.increase_stocks(chain, player_stocks);
        // Give money to player
        player.add_money(stock_price * player_stocks);
        Ok(())
    }

    /// Exchanges the stocks of one chain into another.
    /// # Arguments
    /// * `to_exchange` - The number of stocks that should be exchanged
    /// # Returns
    /// * `Err` - When `to_exchange` is odd, when no stocks are left for the hotel_chain into
    ///   which the stocks should be exchanged
    pub fn exchange_stock(
        &mut self,
        player: &mut Player,
        to_exchange: u32,
        dead: &HotelChain,
        alive: &HotelChain,
    ) -> Result<()> {
        let available_to_exchange = self.stocks_for_sale.stocks_for_hotel(alive);
        if to_exchange % 2 != 0 {
            return Err(miette!("Unable to exchange stocks: {} is odd", to_exchange));
        }
        if available_to_exchange < &(to_exchange / 2) {
            // Not enough stocks available for exchange
            return Err(miette!(
                "Unable to exchange stocks: Not enough stocks left to exchange."
            ));
        }
        // Trade stocks
        player.remove_stocks(dead, to_exchange);
        self.stocks_for_sale.increase_stocks(dead, to_exchange);
        self.stocks_for_sale.decrease_stocks(alive, to_exchange / 2);
        player.add_stocks(alive, to_exchange / 2);
        Ok(())
    }

    /// Gives one stock of the hotel chain to the player for free.
    pub fn give_bonus_stock(&mut self, chain: &HotelChain, player: &mut Player) -> Result<()> {
        // Check if stocks are left
        if *self.stocks_for_sale.stocks.get(chain).unwrap() == 0 {
            // No stocks left; the founder simply does not receive a bonus stock.
        }
        *self.stocks_for_sale.stocks.get_mut(chain).unwrap() -= 1;
        // Give stock to player
        *player.owned_stocks.stocks.get_mut(chain).unwrap() += 1;
        Ok(())
    }

    /// Updates who the largest and second largest shareholders are.
    /// For that the stocks of each player are compared to one another.
    pub fn update_largest_shareholders(&mut self, players: &[Player]) {
        // Clear currently largest shareholder vectors and initialize new
        self.largest_shareholders = LargestShareholders::new();
        for chain in HotelChain::iterator() {
            let mut largest_shareholders: Vec<u32> = Vec::new();
            let mut second_largest_shareholders: Vec<u32> = Vec::new();
            for player in players {
                // Check if player owns stocks for that chain
                if *player.owned_stocks.stocks.get(chain).unwrap() == 0 {
                    continue;
                }
                // Set first player to largest and second largest shareholder
                if largest_shareholders.is_empty() && second_largest_shareholders.is_empty() {
                    largest_shareholders.push(player.id);
                    second_largest_shareholders.push(player.id);
                    continue;
                }
                // Determine currently largest shareholders
                let largest_shareholder_id = *largest_shareholders.get(0).unwrap();
                let largest_shareholder_stocks = player_by_id(largest_shareholder_id, players)
                    .unwrap()
                    .owned_stocks
                    .stocks
                    .get(chain)
                    .unwrap();
                let second_largest_shareholder_id = *second_largest_shareholders.get(0).unwrap();
                let second_largest_shareholder_stocks =
                    player_by_id(second_largest_shareholder_id, players)
                        .unwrap()
                        .owned_stocks
                        .stocks
                        .get(chain)
                        .unwrap();
                let current_player_stocks = *player.owned_stocks.stocks.get(chain).unwrap();
                match largest_shareholders.len().cmp(&1) {
                    Ordering::Equal => {
                        // Currently only one player is largest shareholder
                        match current_player_stocks.cmp(largest_shareholder_stocks) {
                            Ordering::Less => (),
                            // Player has equal stocks => both players will be set to largest
                            // and second largest shareholder
                            Ordering::Equal => {
                                largest_shareholders.push(player.id);
                                second_largest_shareholders.clear();
                                for ls in &largest_shareholders {
                                    second_largest_shareholders.push(*ls);
                                }
                                continue;
                            }
                            // Player has more stocks => The player will be set largest
                            // shareholder and the previously largest shareholder will become
                            // second largest shareholder
                            Ordering::Greater => {
                                match largest_shareholder_stocks
                                    .cmp(second_largest_shareholder_stocks)
                                {
                                    Ordering::Greater => {
                                        second_largest_shareholders.clear();
                                        for s in &largest_shareholders {
                                            second_largest_shareholders.push(*s);
                                        }
                                    }
                                    Ordering::Equal => {
                                        for s in &largest_shareholders {
                                            second_largest_shareholders.push(*s);
                                        }
                                    }
                                    _ => (),
                                }
                                largest_shareholders.clear();
                                largest_shareholders.push(player.id);
                                continue;
                            }
                        }
                    }
                    Ordering::Greater => {
                        // Currently more than one player is largest shareholder
                        match current_player_stocks.cmp(largest_shareholder_stocks) {
                            Ordering::Less => (),
                            // Player has equal stocks => current player will be added to the largest
                            // and second largest shareholder vector
                            Ordering::Equal => {
                                largest_shareholders.push(player.id);
                                second_largest_shareholders.push(player.id);
                                continue;
                            }
                            // The players that were stored as largest and second largest
                            // shareholder will now only be second largest shareholder. The
                            // current player will be set largest shareholder
                            Ordering::Greater => {
                                match largest_shareholder_stocks
                                    .cmp(second_largest_shareholder_stocks)
                                {
                                    Ordering::Greater => {
                                        second_largest_shareholders.clear();
                                        for s in &largest_shareholders {
                                            second_largest_shareholders.push(*s);
                                        }
                                    }
                                    Ordering::Equal => {
                                        for s in &largest_shareholders {
                                            second_largest_shareholders.push(*s);
                                        }
                                    }
                                    _ => (),
                                }
                                largest_shareholders.clear();
                                largest_shareholders.push(player.id);
                                continue;
                            }
                        }
                    }
                    _ => (),
                }
                // Determine the currently second largest shareholder
                match current_player_stocks.cmp(second_largest_shareholder_stocks) {
                    Ordering::Less => {
                        if current_player_stocks > 0 && second_largest_shareholders.len() == 1 {
                            // Check if currently largest shareholder is also second largest
                            // shareholder. If yes, largest shareholder is no longer also second
                            // largest shareholder
                            if second_largest_shareholders.contains(&largest_shareholders[0]) {
                                second_largest_shareholders.clear();
                                second_largest_shareholders.push(player.id);
                            }
                        }
                    }
                    // Player has equal stocks => current player will be added to the second largest shareholder vector
                    Ordering::Equal => {
                        second_largest_shareholders.push(player.id);
                        continue;
                    }
                    // Player has more stocks => all currently second largest
                    // shareholders will be removed and replaced by the current player
                    Ordering::Greater => {
                        second_largest_shareholders.clear();
                        second_largest_shareholders.push(player.id);
                        continue;
                    }
                }
            }
            // Sort and remove duplicate player ids
            largest_shareholders.sort_unstable();
            largest_shareholders.dedup();
            second_largest_shareholders.sort_unstable();
            second_largest_shareholders.dedup();
            // Insert largest shareholders for chain
            self.largest_shareholders
                .largest_shareholder
                .insert(*chain, largest_shareholders);
            self.largest_shareholders
                .second_largest_shareholder
                .insert(*chain, second_largest_shareholders);
        }
    }

    /// Gives the largest and second largest shareholders the bonus.
    ///
    /// When `inform_player` is `true` a [`LogEntry`] is appended for each bonus
    /// that is paid (this replaces the old broadcast / enter prompts).
    pub fn give_majority_shareholder_bonuses(
        &self,
        players: &mut Vec<Player>,
        chain: &HotelChain,
        hotel_chain_manager: &HotelChainManager,
        inform_player: bool,
        round: u32,
        log: &mut Vec<LogEntry>,
    ) -> Result<()> {
        let largest_shareholders = self
            .largest_shareholders
            .largest_shareholder
            .get(chain)
            .unwrap()
            .clone();
        let second_largest_shareholders = self
            .largest_shareholders
            .second_largest_shareholder
            .get(chain)
            .unwrap()
            .clone();
        // A chain that has no shareholders (founded but never bought) simply pays no
        // bonuses — this is a legitimate game state, not an error.
        if largest_shareholders.is_empty() && second_largest_shareholders.is_empty() {
            if inform_player {
                log.push(LogEntry::others(
                    round,
                    0,
                    format!("Chain {} has no shareholders, no bonuses are paid.", chain),
                ));
            }
            return Ok(());
        }
        let largest_shareholder_bonus = Bank::stock_price(hotel_chain_manager, chain) * 10;
        let second_largest_shareholder_bonus =
            Bank::stock_price(hotel_chain_manager, chain) * 5;
        match largest_shareholders.len() {
            1 => {
                let largest_shareholder_id = largest_shareholders[0];
                let largest_shareholder_name =
                    players[largest_shareholder_id as usize].name.clone();
                players[largest_shareholder_id as usize]
                    .add_money(largest_shareholder_bonus);
                if inform_player {
                    log.push(LogEntry::others(
                        round,
                        largest_shareholder_id,
                        format!(
                            "{}, received {}€ because they were the largest shareholder.",
                            largest_shareholder_name, largest_shareholder_bonus
                        ),
                    ));
                }
                match second_largest_shareholders.len() {
                    1 => {
                        let second_largest_shareholder_id = second_largest_shareholders[0];
                        let second_largest_shareholder_name =
                            players[second_largest_shareholder_id as usize].name.clone();
                        players[second_largest_shareholder_id as usize]
                            .add_money(second_largest_shareholder_bonus);
                        if inform_player {
                            log.push(LogEntry::others(
                                round,
                                second_largest_shareholder_id,
                                format!(
                                    "{}, received {}€ because they were the second largest shareholder.",
                                    second_largest_shareholder_name, second_largest_shareholder_bonus
                                ),
                            ));
                        }
                    }
                    _ => {
                        let number_of_second_largest_shareholders =
                            second_largest_shareholders.len();
                        let bonus = second_largest_shareholder_bonus
                            / number_of_second_largest_shareholders as u32;
                        // Round to next 100
                        let bonus = (bonus + 99) / 100 * 100;
                        for i in second_largest_shareholders {
                            let name = players[i as usize].name.clone();
                            players[i as usize].add_money(bonus);
                            if inform_player {
                                log.push(LogEntry::others(
                                    round,
                                    i,
                                    format!(
                                        "{}, received {}€ because they were one of the second largest shareholders.",
                                        name, bonus
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            _ => {
                let number_of_largest_shareholders = largest_shareholders.len();
                let bonus = (largest_shareholder_bonus + second_largest_shareholder_bonus)
                    / number_of_largest_shareholders as u32;
                // Round to next 100
                let bonus = (bonus + 99) / 100 * 100;
                for i in largest_shareholders {
                    let player = players.get_mut(i as usize).unwrap();
                    player.add_money(bonus);
                    if inform_player {
                        log.push(LogEntry::others(
                            round,
                            i,
                            format!(
                                "{}, received {}€ because they were one of the largest shareholders.",
                                player.name, bonus
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks if the player is one of the largest shareholders for the chain.
    pub fn is_largest_shareholder(&self, player_id: u32, chain: &HotelChain) -> bool {
        self.largest_shareholders
            .largest_shareholder
            .get(chain)
            .unwrap()
            .contains(&player_id)
    }

    /// Checks if the player is one of the second largest shareholders for the chain.
    pub fn is_second_largest_shareholder(&self, player_id: u32, chain: &HotelChain) -> bool {
        self.largest_shareholders
            .second_largest_shareholder
            .get(chain)
            .unwrap()
            .contains(&player_id)
    }
}

impl Default for Bank {
    fn default() -> Self {
        Self::new()
    }
}

/// Used to store if the player is a largest or second largest shareholder.
#[derive(Clone, Debug)]
pub struct LargestShareholders {
    /// Contains what the player ids of the largest shareholder for the specified hotel are.
    pub largest_shareholder: HashMap<HotelChain, Vec<u32>>,
    /// Contains what the player ids of the second largest shareholder for the specified chain are.
    pub second_largest_shareholder: HashMap<HotelChain, Vec<u32>>,
}

impl LargestShareholders {
    pub fn new() -> Self {
        let mut largest_shareholder = HashMap::new();
        let mut second_largest_shareholder = HashMap::new();
        for chain in HotelChain::iterator() {
            largest_shareholder.insert(*chain, Vec::new());
            second_largest_shareholder.insert(*chain, Vec::new());
        }
        Self {
            largest_shareholder,
            second_largest_shareholder,
        }
    }
}

impl Default for LargestShareholders {
    fn default() -> Self {
        Self::new()
    }
}
