//! Hotel chains: their identity, price level and stock values.
//!
//! The color is stored as plain RGB data (no styling library). Rendering is
//! the host's (TUI's) job.

use std::fmt::{self, Display, Formatter};

/// A plain RGB color, independent of any styling library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChainColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ChainColor {
    /// Creates a new color from its RGB components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// All different hotel types that exist in the game.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HotelChain {
    Airport,
    Continental,
    Festival,
    Imperial,
    Luxor,
    Oriental,
    Prestige,
}

impl HotelChain {
    /// Returns the identifier for the hotel chain.
    pub fn identifier(&self) -> char {
        match *self {
            HotelChain::Airport => 'A',
            HotelChain::Continental => 'C',
            HotelChain::Festival => 'F',
            HotelChain::Imperial => 'I',
            HotelChain::Luxor => 'L',
            HotelChain::Oriental => 'O',
            HotelChain::Prestige => 'P',
        }
    }

    /// Returns the specific color for the hotel.
    pub fn color(&self) -> ChainColor {
        match *self {
            HotelChain::Airport => ChainColor::new(107, 141, 165),
            HotelChain::Continental => ChainColor::new(32, 64, 136),
            HotelChain::Festival => ChainColor::new(12, 106, 88),
            HotelChain::Imperial => ChainColor::new(198, 83, 80),
            HotelChain::Luxor => ChainColor::new(231, 219, 0),
            HotelChain::Oriental => ChainColor::new(184, 96, 20),
            HotelChain::Prestige => ChainColor::new(99, 47, 107),
        }
    }

    /// Returns an iterator over all hotel chains.
    pub fn iterator() -> impl Iterator<Item = &'static HotelChain> {
        const HOTELS: [HotelChain; 7] = [
            HotelChain::Airport,
            HotelChain::Festival,
            HotelChain::Imperial,
            HotelChain::Luxor,
            HotelChain::Oriental,
            HotelChain::Prestige,
            HotelChain::Continental,
        ];
        HOTELS.iter()
    }

    /// Returns the value of a single stock for the hotel chain.
    ///
    /// # Arguments
    /// * `number_of_hotels` - The number of hotels that currently belong to the hotel chain
    pub fn stock_value(&self, number_of_hotels: u32) -> u32 {
        super::stock::stock_price(self.price_level(), number_of_hotels)
    }

    /// Returns the price level of the hotel. This has an influence on the stock value.
    pub fn price_level(&self) -> PriceLevel {
        match *self {
            HotelChain::Airport => PriceLevel::Low,
            HotelChain::Continental => PriceLevel::High,
            HotelChain::Festival => PriceLevel::Low,
            HotelChain::Imperial => PriceLevel::Medium,
            HotelChain::Luxor => PriceLevel::Medium,
            HotelChain::Oriental => PriceLevel::Medium,
            HotelChain::Prestige => PriceLevel::High,
        }
    }

    /// Returns the name of the hotel.
    pub fn name(&self) -> &str {
        match *self {
            HotelChain::Airport => "Airport",
            HotelChain::Continental => "Continental",
            HotelChain::Festival => "Festival",
            HotelChain::Imperial => "Imperial",
            HotelChain::Luxor => "Luxor",
            HotelChain::Oriental => "Oriental",
            HotelChain::Prestige => "Prestige",
        }
    }

    /// Returns the hotel chain for its identifier, if it exists.
    pub fn from_identifier(identifier: char) -> Option<HotelChain> {
        for chain in HotelChain::iterator() {
            if chain.identifier() == identifier {
                return Some(*chain);
            }
        }
        None
    }
}

/// Used to set the price level for an hotel. This has an influence on the stock value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PriceLevel {
    Low,
    Medium,
    High,
}

impl Display for HotelChain {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::HotelChain;

    #[test]
    fn hotel_names_correct() {
        assert_eq!("Airport", HotelChain::Airport.to_string());
        assert_eq!("Continental", HotelChain::Continental.to_string());
        assert_eq!("Festival", HotelChain::Festival.to_string());
        assert_eq!("Imperial", HotelChain::Imperial.to_string());
        assert_eq!("Luxor", HotelChain::Luxor.to_string());
        assert_eq!("Oriental", HotelChain::Oriental.to_string());
        assert_eq!("Prestige", HotelChain::Prestige.to_string());
    }
}
