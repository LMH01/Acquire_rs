//! Stock bookkeeping and stock price calculation.

use std::collections::HashMap;

use super::chains::{HotelChain, PriceLevel};

/// Used to symbolize how many stocks a player has / the bank has left for a specific hotel.
#[derive(Clone, Debug, PartialEq)]
pub struct Stocks {
    /// Contains the stocks.
    pub stocks: HashMap<HotelChain, u32>,
}

impl Stocks {
    /// Initializes a new stock struct. Member variables are set to 0.
    pub fn new() -> Self {
        let mut stocks: HashMap<HotelChain, u32> = HashMap::new();
        for hotel in HotelChain::iterator() {
            stocks.insert(*hotel, 0);
        }
        Self { stocks }
    }

    /// Initializes a new stock struct. Member variables are set to 25. This is used so that
    /// the bank gets all available stocks at the start.
    pub fn new_bank() -> Self {
        let mut stocks: HashMap<HotelChain, u32> = HashMap::new();
        for chain in HotelChain::iterator() {
            stocks.insert(*chain, 25);
        }
        Self { stocks }
    }

    /// Returns the amount of stocks available for the hotel.
    pub fn stocks_for_hotel(&self, chain: &HotelChain) -> &u32 {
        self.stocks.get(chain).unwrap()
    }

    /// Sets the stocks of the hotel to the amount.
    pub fn set_stocks(&mut self, chain: &HotelChain, value: u32) {
        *self.stocks.get_mut(chain).unwrap() = value;
    }

    /// Increases stocks for the `chain` by `value`.
    pub fn increase_stocks(&mut self, chain: &HotelChain, value: u32) {
        *self.stocks.get_mut(chain).unwrap() += value;
    }

    /// Decreases stocks for the `chain` by `value`.
    pub fn decrease_stocks(&mut self, chain: &HotelChain, value: u32) {
        *self.stocks.get_mut(chain).unwrap() -= value;
    }
}

/// The base prices for a single stock.
pub const STOCK_BASE_PRICE: [u32; 11] = [200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200];

/// Calculates the current stock price for the hotel.
/// # Arguments
/// * `price_level` - Of what price level the stock is
/// * `number_of_hotels` - The number of hotels that belong to the chain
pub fn stock_price(price_level: PriceLevel, number_of_hotels: u32) -> u32 {
    // Check if hotel has at least 2 buildings, otherwise the stock is not worth anything
    if number_of_hotels < 2 {
        return 0;
    }
    // Offset is added to increase the stock price for hotels that have higher prices
    let offset = match price_level {
        PriceLevel::Low => 0,
        PriceLevel::Medium => 1,
        PriceLevel::High => 2,
    };
    // The index that should be pulled from the array, determined by number of hotels
    let stock_price_level = match number_of_hotels {
        2 => 0,
        3 => 1,
        4 => 2,
        5 => 3,
        6..=10 => 4,
        11..=20 => 5,
        21..=30 => 6,
        31..=40 => 7,
        _ => 8,
    };
    *STOCK_BASE_PRICE.get(stock_price_level + offset).unwrap()
}

#[cfg(test)]
mod tests {
    use super::super::chains::PriceLevel;
    use super::stock_price;

    #[test]
    fn stock_price_correct() {
        assert_eq!(stock_price(PriceLevel::Low, 2), 200);
        assert_eq!(stock_price(PriceLevel::Low, 40), 900);
        assert_eq!(stock_price(PriceLevel::Medium, 4), 500);
        assert_eq!(stock_price(PriceLevel::Medium, 20), 800);
        assert_eq!(stock_price(PriceLevel::High, 4), 600);
        assert_eq!(stock_price(PriceLevel::High, 20), 900);
    }
}
