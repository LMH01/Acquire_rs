/// Contains all functionalities related to the game board. Like the pieces that are placed on the
/// board and the board itself.
pub mod board {
    use self::Letter::*;
    use miette::{miette, Result};
    use owo_colors::OwoColorize;
    use std::{
        fmt::{self, Display, Formatter},
        slice::Iter,
    };

    use super::hotel_chains::HotelChain;

    /// The board object that contains all information about the current state of the board.
    pub struct Board {
        pub pieces: Vec<Vec<Piece>>,
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

        /// Places a hotel at the designated coordinates. Does not check if this placement is valid
        /// acording to the game rules.
        /// # Return
        /// Ok when the hotel was placed correctly
        /// Error when the hotel was already placed
        pub fn place_hotel(&mut self, position: &Position) -> Result<()> {
            for x in self.pieces.iter_mut() {
                for y in x.iter_mut() {
                    if y.position.number.eq(&position.number)
                        && y.position.letter == position.letter
                    {
                        if y.piece_set {
                            //TODO Decide if i should remove this error from here or change that
                            //error handling in place hotel function does not ? this result instead
                            //analyzes it
                            return Err(miette!("Unable to set hotel at [{}{:2}] active: The hotel has already been placed!", position.letter, position.number));
                        } else {
                            y.piece_set = true;
                        }
                    }
                }
            }
            Ok(())
        }

        /// Place a hotel on the board without abiding by the game rules
        pub fn place_hotel_debug(&mut self, position: Position, chain: HotelChain) -> Result<()> {
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

        /// Updates the hotel of the piece at the specified position.
        /// Will overwrite any chain that stands there.
        /// # Arguments
        /// * `hotel_chain` - The hotel chain to which the piece should be updated
        /// * `position` - The position of the piece
        ///
        /// # Returns
        /// * `Ok(())` - When the piece as updated successfully
        /// * `Err(Error)` - When the piece is not placed on the board
        pub fn update_hotel(&mut self, hotel_chain: HotelChain, position: &Position) -> Result<()> {
            for line in self.pieces.iter_mut() {
                for piece in line {
                    if piece.position.eq(&position) {
                        piece.chain = Some(hotel_chain);
                        return Ok(());
                    }
                }
            }
            Err(miette!(
                "Unable to update hotel at position {} to chain {}: Hotel has not been placed yet",
                position,
                hotel_chain
            ))
        }
    }
    /// Symbolizes a position on the board
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
    pub struct Position {
        pub letter: Letter,
        pub number: u32,
    }

    impl Position {
        //TODO Change return type to Result<Self> and return error when number > 12 has been
        //entered.
        /// Creates a new position
        pub fn new(letter: Letter, number: u32) -> Self {
            Self { letter, number }
        }
    }

    impl Display for Position {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            write!(f, "{:?}{:?}", self.letter, self.number)
        }
    }

    /// This enum contains all letters of the board
    #[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd, Ord)]
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
        pub chain: Option<HotelChain>,
        /// Stores the position on the board of this piece
        pub position: Position,
        /// Stores if the piece has been set yet
        pub piece_set: bool,
    }

    impl Piece {
        //TODO Write new() method
        fn print_text(&self, compact: bool) -> String {
            if self.piece_set {
                if self.chain.is_some() {
                    if compact {
                        self.chain
                            .as_ref()
                            .unwrap()
                            .identifier()
                            .color(HotelChain::color(&self.chain.as_ref().unwrap()))
                            .to_string()
                    } else {
                        format!(" {} ", self.chain.as_ref().unwrap().identifier())
                            .color(HotelChain::color(&self.chain.as_ref().unwrap()))
                            .to_string()
                    }
                } else {
                    if compact {
                        "X".bright_white().to_string()
                    } else {
                        "XXX".bright_white().to_string()
                    }
                }
            } else {
                if compact {
                    ' '.white().to_string()
                } else {
                    format!("{}{:2}", self.position.letter, self.position.number)
                        .white()
                        .to_string()
                }
            }
        }
    }
}

/// Contains all functionalities related to the hotel chains. Like name, information about stock
/// values and more.
pub mod hotel_chains {
    use std::{
        fmt::{self, Display, Formatter},
        slice::Iter,
    };

    use owo_colors::Rgb;

    use super::stock;

    //TODO See if it would be a better idea to remove the clone, copy
    /// All different hotel types that exist in the game
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
        /// Returns the identifier for the hotel chain
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

        /// Returns the specific color for the hotel
        pub fn color(&self) -> Rgb {
            match *self {
                HotelChain::Airport => Rgb(107, 141, 165),
                HotelChain::Continental => Rgb(32, 64, 136),
                HotelChain::Festival => Rgb(12, 106, 88),
                HotelChain::Imperial => Rgb(198, 83, 80),
                HotelChain::Luxor => Rgb(231, 219, 0),
                HotelChain::Oriental => Rgb(184, 96, 20),
                HotelChain::Prestige => Rgb(99, 47, 107),
            }
        }

        pub fn iterator() -> Iter<'static, HotelChain> {
            const HOTELS: [HotelChain; 7] = [
                HotelChain::Airport,
                HotelChain::Continental,
                HotelChain::Festival,
                HotelChain::Imperial,
                HotelChain::Luxor,
                HotelChain::Oriental,
                HotelChain::Prestige,
            ];
            HOTELS.iter()
        }

        /// Returns the value of a single stock for the hotel chain.
        /// Value is returned with help of [`super::stock::stock_price`].
        /// # Arguments
        /// * 'number_of_hotels' - The number of hotels that currently belong to the hotel chain
        pub fn stock_value(&self, number_of_hotels: u32) -> u32 {
            stock::stock_price(self.price_level(), number_of_hotels)
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

        /// Returns the name of the hotel
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

    impl Display for HotelChain {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::base_game::hotel_chains::HotelChain;

        #[test]
        fn name() {
            assert_eq!("Airport", HotelChain::Airport.to_string());
            assert_eq!("Continental", HotelChain::Continental.to_string());
            assert_eq!("Festival", HotelChain::Festival.to_string());
            assert_eq!("Imperial", HotelChain::Imperial.to_string());
            assert_eq!("Luxor", HotelChain::Luxor.to_string());
            assert_eq!("Oriental", HotelChain::Oriental.to_string());
            assert_eq!("Prestige", HotelChain::Prestige.to_string());
        }
    }
}

/// Contains all functions for the stocks.
pub mod stock {
    use std::collections::HashMap;

    use super::hotel_chains::{HotelChain, PriceLevel};

    /// Used to symbolize how many stocks a player has/the bank has left for a specific hotel
    pub struct Stocks {
        // Contains the stocks.
        pub stocks: HashMap<HotelChain, u32>,
    }

    impl Stocks {
        /// Initializes a new stock struct. Member variables are set to 0
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

        /// Returns the amout of stocks available for the hotel
        pub fn stocks_for_hotel(&self, chain: &HotelChain) -> &u32 {
            self.stocks.get(chain).unwrap()
        }

        /// Set the stocks of the hotel to the amount.
        /// # Arguments
        /// * `hotel` - The hotel for which the stock value should be changed
        /// * `value` - The value to which the stock amount should be set
        pub fn set_stocks(&mut self, chain: &HotelChain, value: u32) {
            *self.stocks.get_mut(chain).unwrap() = value;
        }

        /// Increases stocks for the `hotel` by `value`
        pub fn increase_stocks(&mut self, chain: &HotelChain, value: u32) {
            *self.stocks.get_mut(chain).unwrap() += value;
        }

        /// Decreases stocks for the `hotel` by `value`
        pub fn decrease_stocks(&mut self, chain: &HotelChain, value: u32) {
            *self.stocks.get_mut(chain).unwrap() -= value;
        }
    }

    /// The base prices for a single stock
    const STOCK_BASE_PRICE: [u32; 11] = [200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200];

    /// Calculates the current stock price for the hotel.
    /// # Arguments
    /// * `price_level` - Of what price level the stock is
    /// * `number_of_hotels` - The number of hotels that belong to the chain
    pub fn stock_price(price_level: PriceLevel, number_of_hotels: u32) -> u32 {
        // Check if hotel has at least 2 buildings, otherwise the stock is not worth anything
        if number_of_hotels < 2 {
            return 0;
        }
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
        use crate::base_game::{hotel_chains::PriceLevel, stock::stock_price};

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
    use std::{slice::SliceIndex, collections::HashMap};

    use crate::{base_game::stock::Stocks, game::game::hotel_chain_manager::HotelChainManager};

    use super::{hotel_chains::HotelChain, player::Player};

    pub struct Bank<'a> {
        pub stocks_for_sale: Stocks,
        /// Stores the currently largest and second largest shareholders
        pub largest_shareholders: LargestShareholders<'a>,
    }

    impl Bank<'_> {
        /// Creates a new bank
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

        /// Returns the current price for a stock of the given chain
        pub fn stock_price(hotel_chain_manager: &HotelChainManager, chain: &HotelChain) -> u32 {
            chain.stock_value(hotel_chain_manager.chain_length(&chain))
        }

        /// Set the stocks of the given chain that the bank has left to sell
        pub fn set_stocks(&mut self, chain: &HotelChain, value: u32) {
            *self.stocks_for_sale.stocks.get_mut(chain).unwrap() = value;
        }
    }
    
    /// Used to store if the player is a largest or second largest shareholder
    pub struct LargestShareholders<'a> {
        /// Contains who the largest shareholder is for the specified hotel
        largest_shareholder: HashMap<HotelChain, Option<&'a Player>>,
        /// Chains where the player is the second largest shareholder
        second_largest_shareholder: HashMap<HotelChain, Option<&'a Player>>,
    }

    impl LargestShareholders<'_> {
        pub fn new() -> Self {
            let mut largest_shareholder: HashMap<HotelChain, Option<& Player>> = HashMap::new();
            let mut second_largest_shareholder: HashMap<HotelChain, Option<& Player>> = HashMap::new();
            for chain in HotelChain::iterator() {
                largest_shareholder.insert(*chain, None);
                second_largest_shareholder.insert(*chain, None);
            }
            Self {
                largest_shareholder,
                second_largest_shareholder,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use miette::Result;

        use crate::{
            base_game::{
                bank::Bank,
                board::{Letter, Position},
                hotel_chains::HotelChain,
                player::Player,
            },
            game::game::GameManager,
        };

        #[test]
        fn test_stock_price() -> Result<()> {
            let mut game_manager = GameManager::new(2, false).unwrap();
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Airport,
                vec![Position::new(Letter::A, 1), Position::new(Letter::A, 2)],
                &mut game_manager.board,
                &mut Player::new(Vec::new()),
                &mut game_manager.bank,
            )?;
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Imperial,
                vec![
                    Position::new(Letter::B, 3),
                    Position::new(Letter::C, 3),
                    Position::new(Letter::C, 4),
                ],
                &mut game_manager.board,
                &mut Player::new(Vec::new()),
                &mut game_manager.bank,
            )?;
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Continental,
                vec![
                    Position::new(Letter::H, 1),
                    Position::new(Letter::H, 2),
                    Position::new(Letter::H, 3),
                    Position::new(Letter::H, 4),
                ],
                &mut game_manager.board,
                &mut Player::new(Vec::new()),
                &mut game_manager.bank,
            )?;
            println!(
                "Number of hotels: {}",
                game_manager
                    .hotel_chain_manager
                    .chain_length(&HotelChain::Airport)
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_chain_manager, &HotelChain::Airport),
                200
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_chain_manager, &HotelChain::Imperial),
                400
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_chain_manager, &HotelChain::Continental),
                600
            );
            Ok(())
        }
    }
}

/// Player management
pub mod player {
    use crate::{
        base_game::board::Position,
        base_game::{hotel_chains::HotelChain, stock::Stocks},
        base_game::bank::Bank,
    };
    use owo_colors::OwoColorize;

    /// Stores all variables that belong to the player
    pub struct Player {
        /// The money the player currently has
        pub money: u32,
        /// The stocks that the player currently owns
        pub owned_stocks: Stocks,
        /// Contains the cards that the player currently has on his hand and that could be played
        pub cards: Vec<Position>,
    }

    impl Player {
        pub fn new(mut start_cards: Vec<Position>) -> Self {
            Self {
                money: 6000,
                owned_stocks: Stocks::new(),
                cards: start_cards,
            }
        }

        /// Add money to the player
        pub fn add_money(&mut self, money: u32) {
            self.money += money;
        }

        /// Remove money to the player
        pub fn remove_money(&mut self, money: u32) {
            self.money -= money;
        }

        /// Add stocks that the player owns
        pub fn add_stocks(&mut self, chain: &HotelChain, amount: u32) {
            self.owned_stocks.increase_stocks(chain, amount);
        }

        /// Remove stocks that the player owns
        pub fn remove_stocks(&mut self, chain: &HotelChain, amount: u32) {
            self.owned_stocks.decrease_stocks(chain, amount);
        }

        /// Print the cards the player currently has
        pub fn print_cards(&self) {
            println!();
            print!("Your current cards: ");
            for position in &self.cards {
                print!("[{}{:2}]", position.letter, position.number);
            }
            println!();
        }

        /// Sorts the current hand cards and returns a copy
        pub fn sorted_cards(&self) -> Vec<Position> {
            let mut cards = self.cards.clone();
            cards.sort();
            cards
        }

        /// Removes a card from the players inventory.
        /// Returns `true` when the card was removed successfully
        pub fn remove_card(&mut self, position: &Position) -> bool {
            for (index, card) in self.cards.iter().enumerate() {
                if card.letter.eq(&position.letter) {
                    if card.number.eq(&position.number) {
                        self.cards.remove(index);
                        return true;
                    }
                }
            }
            false
        }

        /// Checks if this player is the largest shareholder for the chain
        pub fn is_largest_shareholder(&self, bank: &Bank) -> bool {
            //self.bonuses.largest_shareholder.contains(&chain)
            todo!();
        }

        /// Checks if this player is the second largest shareholder for the chain
        pub fn is_second_largest_shareholder(&self, bank: &Bank) -> bool {
            //self.bonuses.second_largest_shareholder.contains(&chain)
            todo!();
        }

        /// Prints the current player to the console
        pub fn print_player_ui(&self) {
            // Print money
            println!("{} {}$", String::from("Money:").bright_green(), self.money);
            // Print cards
            print!("{}", String::from("Cards: ").bright_green());
            let mut first_card = true;
            for card in &self.sorted_cards() {
                if first_card {
                    first_card = false;
                } else {
                    print!(", ");
                }
                print!("{}", card);
            }
            println!();
            // Print stocks
            //TODO Add functionality that a * in gold is printed when the player is the largest
            //shareholder or a silver * when the player is the second largest shareholder.
            //The star is positioned here: Airport*:
            print!("{}", String::from("Stocks: ").bright_green());
            first_card = true;
            for chain in HotelChain::iterator() {
                if first_card {
                    first_card = false;
                } else {
                    print!(", ")
                }
                print!(
                    "{}: {}",
                    chain.name().color(chain.color()),
                    self.owned_stocks.stocks_for_hotel(chain)
                );
            }
            println!();
            //TODO Maybe add fields:
            //- "Current estimated wealth". That displayes the amount of
            //  money the player would have now if all shares where sold and the rewards for the
            //  largest shareholders where given now. (But is only enabled if special info flag is
            //  given)
            //- "Current stock value" - Value of alls stocks if sold now
            //- "Total stocks" - Amount of all stocks the player has
            //- Net profit: Stores all expenses the player made and calculate the net profit the
            // would make if all stocks where sold now
        }
    }

}

/// User interface drawing
pub mod ui {
    use crate::{
        base_game::{bank::Bank, hotel_chains::HotelChain},
        game::game::GameManager,
    };
    use owo_colors::{AnsiColors, DynColors, OwoColorize, Rgb};

    use super::player::Player;

    /// Prints the main user interface.
    pub fn print_main_ui(game_manager: &GameManager) {
        game_manager.board.print();
        println!();
        println!("Round {}", &game_manager.round_number);
        println!(
            "Player {}:",
            game_manager.round.as_ref().unwrap().current_player_index + 1
        );
        game_manager
            .round
            .as_ref()
            .unwrap()
            .current_player(&game_manager.players)
            .print_player_ui();
        //TODO Implement largest shareholder display
        //Maybe add commandline flag with which it can be enabled to show who is the largest
        //stareholder currently
        println!();
        println!("{}", String::from("Game stats:").bright_green());
        println!("{:15}||      Hotels       ||        Stocks          ||      Bonuses for the majority shareholders", "");
        println!("{:15}|| Number ||  Range  || Bank || Owned || Value || Largest shareholder || Second largest shareholder", "");
        println!("==================================================================================================================");
        for chain in HotelChain::iterator() {
            // Set the color of the values
            let enable_color = game_manager.hotel_chain_manager.chain_status(chain);
            let color = match enable_color {
                true => DynColors::Ansi(AnsiColors::White),
                false => DynColors::Rgb(105, 105, 105),
            };
            let chain_color = match enable_color {
                true => chain.color(),
                false => Rgb(105, 105, 105),
            };
            let formatted_string = format!(
                "||   {:2}   || {:7} ||  {:2}  ||   {:2}{} || {:4}$ ||    TO IMPLEMENT     || TO IMPLEMENT",
                game_manager.hotel_chain_manager.chain_length(chain),
                game_manager.hotel_chain_manager.price_range(chain),
                game_manager.bank.stocks_available(&chain, &game_manager.hotel_chain_manager),
                game_manager
                    .round
                    .as_ref()
                    .unwrap()
                    .current_player(&game_manager.players)
                    .owned_stocks
                    .stocks_for_hotel(chain),
                " ",
                Bank::stock_price(&game_manager.hotel_chain_manager, &chain),
                );
            println!(
                "{:15}{}",
                chain.name().color(chain_color),
                formatted_string.color(color),
            );
        }
    }

    /// Used to display a little star that indicates if the player is largest or second largest
    /// shareholder
    fn stock_status_symbol(player: &Player) -> String {
        // The star should probably be only displayed when a special terminal flag is set (mayber
        // --info or something like that)
        todo!()
    }
}
