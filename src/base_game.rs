/// Contains all functionalities related to the game board. Like the pieces that are placed on the
/// board and the board itself.
pub mod board {
    use self::Letter::*;
    use colored::{ColoredString, Colorize};
    use miette::{miette, Result};
    use std::{
        fmt::{self, Display},
        slice::Iter,
    };

    use super::hotel::Hotel;

    /// The board object that contains all information about the current state of the board.
    pub struct Board {
        pieces: Vec<Vec<Piece>>,
        /// Determines how the board should be printed.
        /// This behaviour can be set with the -l flag.
        /// If true the (empty) board is printed this way:
        /// ``` none
        /// A [A 1] [A 2] [A 3] [A 4] [A 5] [A 6] [A 7] [A 8] [A 9] [A10] [A11] [A12]
        /// B [B 1] [B 2] [B 3] [B 4] [B 5] [B 6] [B 7] [B 8] [B 9] [B10] [B11] [B12]
        /// C [C 1] [C 2] [C 3] [C 4] [C 5] [C 6] [C 7] [C 8] [C 9] [C10] [C11] [C12]
        /// D [D 1] [D 2] [D 3] [D 4] [D 5] [D 6] [D 7] [D 8] [D 9] [D10] [D11] [D12]
        /// E [E 1] [E 2] [E 3] [E 4] [E 5] [E 6] [E 7] [E 8] [E 9] [E10] [E11] [E12]
        /// F [F 1] [F 2] [F 3] [F 4] [F 5] [F 6] [F 7] [F 8] [F 9] [F10] [F11] [F12]
        /// G [G 1] [G 2] [G 3] [G 4] [G 5] [G 6] [G 7] [G 8] [G 9] [G10] [G11] [G12]
        /// H [H 1] [H 2] [H 3] [H 4] [H 5] [H 6] [H 7] [H 8] [H 9] [H10] [H11] [H12]
        /// I [I 1] [I 2] [I 3] [I 4] [I 5] [I 6] [I 7] [I 8] [I 9] [I10] [I11] [I12]
        ///      1     2     3     4     5     6     7     8     9    10    11    12
        /// ```
        /// If false the (empty) board is printed this way:
        /// ```none
        /// A
        /// B
        /// C
        /// D
        /// E
        /// F
        /// G
        /// H
        /// I
        ///   1  2  3  4  5  6  7  8  9 10 11 12
        /// ```
        large_board: bool,
    }

    impl Board {
        /// Creates a new board and initializes it
        pub fn new(large_board: bool) -> Self {
            let mut pieces: Vec<Vec<Piece>> = Vec::new();
            // initialize pieces
            for c in Letter::iterator() {
                let mut x_pieces: Vec<Piece> = Vec::new();
                for i in 1..=12 {
                    x_pieces.push(Piece {
                        chain: None,
                        position: Position::new(*c, i),
                        piece_set: false,
                    })
                }
                pieces.push(x_pieces);
            }
            Self {
                pieces,
                large_board,
            }
        }

        /// Prints the current stage of the board
        pub fn print(&self) {
            println!();
            let mut letters = Letter::iterator();
            for x in &self.pieces {
                print!("{} ", letters.next().unwrap());
                for y in x {
                    if self.large_board {
                        print!("[{}] ", y.print_text(false));
                    } else {
                        print!("{}  ", y.print_text(true))
                    }
                }
                println!()
            }
            if self.large_board {
                print!("    ");
                for x in 1..=12 {
                    print!("{:2}    ", &x);
                }
            } else {
                print!(" ");
                for x in 1..=12 {
                    print!("{:2} ", &x);
                }
            }
            println!("");
        }

        //TODO Add all functionalities required to place a hotel correctly and accoring to the game
        //rules. Decide if i would like that check to be performed here or reather in the game module.
        //If i decide to do it this way this function will not check if the placement of the hotel is
        //valid.
        /// Places a hotel at the designated coordinates. Does not check if this placement is valid
        /// acording to the game rules.
        /// # Return
        /// Ok when the hotel was placed correctly
        /// Error when the hotel was already placed
        pub fn place_hotel(&mut self, position: Position) -> Result<()> {
            for x in self.pieces.iter_mut() {
                for y in x.iter_mut() {
                    if y.position.number.eq(&position.number)
                        && y.position.letter == position.letter
                    {
                        if y.piece_set {
                            return Err(miette!("Unable to set hotel at [{}{:2}] active: The hotel has already been placed!", position.letter, position.number));
                        } else {
                            y.piece_set = true;
                        }
                    }
                }
            }
            Ok(())
        }

        pub fn place_hotel_debug(&mut self, position: Position, chain: Hotel) -> Result<()> {
            'outer: for x in self.pieces.iter_mut() {
                for y in x.iter_mut() {
                    if y.position.number.eq(&position.number)
                        && y.position.letter == position.letter
                    {
                        if y.piece_set {
                            return Err(miette!("Unable to set hotel at [{}{:2}] active: The hotel has already been placed!", position.letter, position.number));
                        } else {
                            y.piece_set = true;
                            y.chain = Some(chain);
                            break 'outer;
                        }
                    }
                }
            }
            Ok(())
        }
    }
    /// Symbolizes a position on the board
    #[derive(Clone, Copy)]
    pub struct Position {
        pub letter: Letter,
        pub number: u8,
    }

    impl Position {
        //TODO Change return type to Result<Self> and return error when number > 12 has been
        //entered.
        /// Creates a new position
        pub fn new(letter: Letter, number: u8) -> Self {
            Self { letter, number }
        }
    }
    /// This enum contains all letters of the board
    #[derive(Clone, Copy, PartialEq)]
    pub enum Letter {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
    }

    impl Letter {
        pub fn iterator() -> Iter<'static, Letter> {
            static LETTERS: [Letter; 9] = [A, B, C, D, E, F, G, H, I];
            LETTERS.iter()
        }

        pub fn letter(&self) -> char {
            match *self {
                Letter::A => 'A',
                Letter::B => 'B',
                Letter::C => 'C',
                Letter::D => 'D',
                Letter::E => 'E',
                Letter::F => 'F',
                Letter::G => 'G',
                Letter::H => 'H',
                Letter::I => 'I',
            }
        }
    }

    impl Display for Letter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.letter())
        }
    }
    /// Symbolizes a single piece that can be placed on the board
    pub struct Piece {
        /// Stores what hotel chain this piece belongs to
        chain: Option<Hotel>,
        /// Stores the position on the board of this piece
        position: Position,
        /// Stores if the piece has been set yet
        piece_set: bool,
    }

    impl Piece {
        fn print_text(&self, compact: bool) -> ColoredString {
            if self.piece_set {
                if self.chain.is_some() {
                    if compact {
                        String::from(self.chain.as_ref().unwrap().identifier())
                            .color(Hotel::color(&self.chain.as_ref().unwrap()))
                    } else {
                        let mut string = String::from(" ");
                        string.push(self.chain.as_ref().unwrap().identifier());
                        string.push(' ');
                        string.color(Hotel::color(&self.chain.as_ref().unwrap()))
                    }
                } else {
                    if compact {
                        String::from("X").bright_white()
                    } else {
                        String::from("XXX").bright_white()
                    }
                }
            } else {
                if compact {
                    String::from(' ').white()
                } else {
                    String::from(format!(
                        "{}{:2}",
                        self.position.letter, self.position.number
                    ))
                    .white()
                }
            }
        }
    }
}

/// Contains all functionalities related to the hotel buildings. Like name, information about stock
/// values and more.
pub mod hotel {
    use std::{
        fmt::{self, Display, Formatter},
        slice::Iter,
    };

    use colored::Color;

    use super::stock;

    //TODO See if it would be a better idea to remove the clone, copy
    /// All different hotel types that exist in the game
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub enum Hotel {
        Airport,
        Continental,
        Festival,
        Imperial,
        Luxor,
        Oriental,
        Prestige,
    }

    impl Hotel {
        /// Returns the identifier for the hotel
        pub fn identifier(&self) -> char {
            match *self {
                Hotel::Airport => 'A',
                Hotel::Continental => 'C',
                Hotel::Festival => 'F',
                Hotel::Imperial => 'I',
                Hotel::Luxor => 'L',
                Hotel::Oriental => 'O',
                Hotel::Prestige => 'P',
            }
        }

        /// Returns the specific color for the hotel
        pub fn color(&self) -> Color {
            match *self {
                Hotel::Airport => Color::TrueColor {
                    r: 107,
                    g: 141,
                    b: 165,
                },
                Hotel::Continental => Color::TrueColor {
                    r: 32,
                    g: 64,
                    b: 136,
                },
                Hotel::Festival => Color::TrueColor {
                    r: 12,
                    g: 106,
                    b: 88,
                },
                Hotel::Imperial => Color::TrueColor {
                    r: 198,
                    g: 83,
                    b: 80,
                },
                Hotel::Luxor => Color::TrueColor {
                    r: 231,
                    g: 219,
                    b: 0,
                },
                Hotel::Oriental => Color::TrueColor {
                    r: 184,
                    g: 96,
                    b: 20,
                },
                Hotel::Prestige => Color::TrueColor {
                    r: 99,
                    g: 47,
                    b: 107,
                },
            }
        }

        pub fn iterator() -> Iter<'static, Hotel> {
            const HOTELS: [Hotel; 7] = [
                Hotel::Airport,
                Hotel::Continental,
                Hotel::Festival,
                Hotel::Imperial,
                Hotel::Luxor,
                Hotel::Oriental,
                Hotel::Prestige,
            ];
            HOTELS.iter()
        }

        /// Returns the value of a single stock for the hotel chain.
        /// Value is returned with help of [`super::stock::stock_price`].
        /// # Arguments
        /// * 'number_of_hotels' - The number of hotels that currently belong to the hotel chain
        pub fn stock_value(&self, number_of_hotels: u8) -> i32 {
            stock::stock_price(self.price_level(), number_of_hotels)
        }

        /// Returns the price level of the hotel. This has an influence on the stock value.
        pub fn price_level(&self) -> PriceLevel {
            match *self {
                Hotel::Airport => PriceLevel::Low,
                Hotel::Continental => PriceLevel::High,
                Hotel::Festival => PriceLevel::Low,
                Hotel::Imperial => PriceLevel::Medium,
                Hotel::Luxor => PriceLevel::Medium,
                Hotel::Oriental => PriceLevel::Medium,
                Hotel::Prestige => PriceLevel::High,
            }
        }
    }

    /// Used to set the price level for an hotel. This has an influence on the stock value.
    pub enum PriceLevel {
        Low,
        Medium,
        High,
    }

    impl PriceLevel {
        pub fn iterator() -> Iter<'static, PriceLevel> {
            const PRICE_LEVELS: [PriceLevel; 3] =
                [PriceLevel::Low, PriceLevel::Medium, PriceLevel::High];
            PRICE_LEVELS.iter()
        }
    }

    impl Display for Hotel {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::base_game::hotel::Hotel;

        #[test]
        fn name() {
            assert_eq!("Airport", Hotel::Airport.to_string());
            assert_eq!("Continental", Hotel::Continental.to_string());
            assert_eq!("Festival", Hotel::Festival.to_string());
            assert_eq!("Imperial", Hotel::Imperial.to_string());
            assert_eq!("Luxor", Hotel::Luxor.to_string());
            assert_eq!("Oriental", Hotel::Oriental.to_string());
            assert_eq!("Prestige", Hotel::Prestige.to_string());
        }
    }
}

/// Contains all functions for the stocks.
pub mod stock {
    use std::collections::HashMap;

    use super::hotel::{Hotel, PriceLevel};

    /// Used to symbolize how many stocks a player has/the bank has left for a specific hotel
    pub struct Stocks {
        // Contains the stocks.
        stocks: HashMap<Hotel, u8>,
    }

    impl Stocks {
        /// Initializes a new stock struct. Member variables are set to 0
        pub fn new() -> Self {
            let mut stocks: HashMap<Hotel, u8> = HashMap::new();
            for hotel in Hotel::iterator() {
                stocks.insert(*hotel, 0);
            }
            Self { stocks }
        }

        /// Initializes a new stock struct. Member variables are set to 25. This is used so that
        /// the bank works get all available stocks at the start.
        pub fn new_bank() -> Self {
            let mut stocks: HashMap<Hotel, u8> = HashMap::new();
            for hotel in Hotel::iterator() {
                stocks.insert(*hotel, 25);
            }
            Self { stocks }
        }
    }

    /// The base prices for a single stock
    const STOCK_BASE_PRICE: [i32; 11] = [200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200];

    /// Calculates the current stock price for the hotel.
    /// # Arguments
    /// * `price_level` - Of what price level the stock is
    /// * `number_of_hotels` - The number of hotels that belong to the chain
    pub fn stock_price(price_level: PriceLevel, number_of_hotels: u8) -> i32 {
        // Offset ist added to increate the stock price for hotels that have higher prices
        let offset = match price_level {
            PriceLevel::Low => 0,
            PriceLevel::Medium => 1,
            PriceLevel::High => 2,
        };
        // The index that should be pulled from the vector, determined by number of hotels
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
        use crate::base_game::{hotel::PriceLevel, stock::stock_price};

        #[test]
        fn test_stock_price() {
            assert_eq!(stock_price(PriceLevel::Low, 2), 200);
            assert_eq!(stock_price(PriceLevel::Low, 40), 900);
            assert_eq!(stock_price(PriceLevel::Medium, 4), 500);
            assert_eq!(stock_price(PriceLevel::Medium, 20), 800);
            assert_eq!(stock_price(PriceLevel::High, 4), 600);
            assert_eq!(stock_price(PriceLevel::High, 20), 900);
        }
    }
}

/// Manages the currently available stocks and the money.
pub mod bank {
    use crate::base_game::stock::Stocks;

    pub struct Bank {
        available_stocks: Stocks,
    }

    impl Bank {
        /// Creates a new bank
        pub fn new() -> Self {
            Self {
                available_stocks: Stocks::new_bank(),
            }
        }
    }
}
