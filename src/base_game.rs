/// Contains all functionalities related to the game board. Like the pieces that are placed on the
/// board and the board itself.
pub mod board {
    use crate::{
        game::game::hotel_chain_manager::HotelChainManager,
        logic::place_hotel::{analyze_position, PlaceHotelCase},
    };

    use self::letter::{next_letter, prev_letter, LETTERS};
    use super::hotel_chains::HotelChain;
    use core::borrow;
    use miette::{miette, Result};
    use owo_colors::{colors, AnsiColors, OwoColorize, Rgb};
    use std::cmp::Ordering;
    use std::fmt::{self, Display, Formatter};

    /// The board object that contains all information about the current state of the board.
    pub struct Board {
        pub pieces: Vec<Vec<Piece>>,
    }

    impl Board {
        /// Creates a new board and initializes it
        pub fn new() -> Self {
            let mut pieces: Vec<Vec<Piece>> = Vec::new();
            // initialize pieces
            for c in LETTERS {
                let mut x_pieces: Vec<Piece> = Vec::new();
                for i in 1..=12 {
                    x_pieces.push(Piece {
                        chain: None,
                        position: Position::new(c, i),
                        piece_set: false,
                    })
                }
                pieces.push(x_pieces);
            }
            Self { pieces }
        }

        /// Prints the current stage of the board
        pub fn print(&self, small_board: bool) {
            println!();
            let mut letters = LETTERS.iter();
            let mut first_line = true;
            for x in &self.pieces {
                if !first_line {
                    if !small_board {
                        println!("--------------------------------------------------");
                    }
                } else {
                    first_line = false;
                }
                print!("{} ", letters.next().unwrap());
                for y in x {
                    if !small_board {
                        print!("| {} ", y.print_text(true));
                    } else {
                        print!("{}  ", y.print_text(true))
                    }
                }
                println!();
            }
            if !small_board {
                print!("   ");
                for x in 1..=12 {
                    print!("{:2}  ", &x);
                }
            } else {
                print!(" ");
                for x in 1..=12 {
                    print!("{:2} ", &x);
                }
            }
            println!("");
        }

        /// Returns a vector that contains strings that describe the current state of the board.
        pub fn get_board_state(&self, small_board: bool) -> Vec<String> {
            let mut board_state = Vec::new();
            let mut letters = LETTERS.iter();
            let mut first_line = true;
            for x in &self.pieces {
                if !first_line {
                    if !small_board {
                        board_state.push(String::from(
                            "--------------------------------------------------",
                        ));
                    }
                } else {
                    first_line = false;
                }
                let mut current_line = String::new();
                current_line.push_str(&format!("{} ", letters.next().unwrap()));
                for y in x {
                    if !small_board {
                        current_line.push_str(&format!("| {} ", y.print_text(true)));
                    } else {
                        current_line.push_str(&format!("{}  ", y.print_text(true)));
                    }
                }
                board_state.push(String::from(current_line));
            }
            let mut current_line = String::new();
            if !small_board {
                current_line.push_str("   ");
                for x in 1..=12 {
                    current_line.push_str(&format!("{:2}  ", &x));
                }
            } else {
                current_line.push_str(" ");
                for x in 1..=12 {
                    current_line.push_str(&format!("{:2} ", &x));
                }
            }
            board_state.push(String::from(current_line));
            board_state
        }

        /// Places a hotel at the designated coordinates. Does not check if this placement is valid acording to the game rules.
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
                    if piece.position.eq(&position) && piece.piece_set {
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

        /// Checks if a hotel has been placed at the position.
        /// # Returns
        /// * `None` - Hotel has not been placed
        /// * `Some(None)` - When the hotel has been placed but it does not belong to any chain
        /// * `Some(HotelChain)` - When the hotel has been placed and belongs to a chain
        pub fn is_hotel_placed(&self, position: &Position) -> Option<Option<HotelChain>> {
            for line in &self.pieces {
                for piece in line {
                    if piece.position.eq(&position) {
                        if !piece.piece_set {
                            return None;
                        }
                        return Some(piece.chain);
                    }
                }
            }
            None
        }
    }

    /// Functions related to the letter
    pub mod letter {
        pub const LETTERS: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'];

        /// Returns the index for this letter in the `LETTERS` array
        fn letter_to_index(letter: char) -> Option<usize> {
            for (index, l) in LETTERS.iter().enumerate() {
                if letter == *l {
                    return Some(index);
                }
            }
            return None;
        }

        /// Returns the next letter if there is one.
        /// B would return C
        pub fn next_letter(letter: char) -> Option<char> {
            let index = letter_to_index(letter);
            if index.is_none() {
                return None;
            }
            if index.unwrap() == 8 {
                return None;
            }
            Some(*LETTERS.get(index.unwrap() + 1).unwrap())
        }

        /// Returns the previous letter if there is one.
        /// B would return A
        pub fn prev_letter(letter: char) -> Option<char> {
            let index = letter_to_index(letter);
            if index.is_none() {
                return None;
            }
            if index.unwrap() == 0 {
                return None;
            }
            Some(*LETTERS.get(index.unwrap() - 1).unwrap())
        }
    }

    /// Symbolizes a position on the board
    #[derive(Clone, Copy, Debug, PartialEq, Ord, Eq, Hash)]
    pub struct Position {
        pub letter: char,
        pub number: u32,
    }

    impl PartialOrd for Position {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self.number.cmp(&other.number) {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => match self.letter.cmp(&other.letter) {
                    Ordering::Less => Some(Ordering::Less),
                    Ordering::Greater => Some(Ordering::Greater),
                    Ordering::Equal => Some(Ordering::Equal),
                },
            }
        }
    }

    impl Position {
        /// Creates a new position
        pub fn new(letter: char, number: u32) -> Self {
            Self { letter, number }
        }

        /// Returns the next position.
        /// Input B3 would return B4.
        pub fn next(&self) -> Option<Position> {
            // Check if position has a next
            if self.number > 12 {
                return None;
            }
            return Some(Position::new(self.letter, self.number + 1));
        }

        /// Returns the previous position.
        /// Input B3 would return B2.
        pub fn prev(&self) -> Option<Position> {
            // Check if position has a prev
            if self.number < 1 {
                return None;
            }
            return Some(Position::new(self.letter, self.number - 1));
        }

        /// Returns the position that is above this position.
        /// Input B3 would return A3.
        pub fn up(&self) -> Option<Position> {
            match prev_letter(self.letter) {
                Some(letter) => Some(Position::new(letter, self.number)),
                None => None,
            }
        }

        /// Returns the position that is below this position.
        /// Input B3 would return C3.
        pub fn down(&self) -> Option<Position> {
            match next_letter(self.letter) {
                Some(letter) => Some(Position::new(letter, self.number)),
                None => None,
            }
        }

        /// Returns the neighbouring positions
        pub fn neighbours(&self) -> Vec<Position> {
            let mut neighbours = Vec::new();
            if let Some(next) = self.next() {
                neighbours.push(next);
            }
            if let Some(down) = self.down() {
                neighbours.push(down);
            }
            if let Some(prev) = self.prev() {
                neighbours.push(prev);
            }
            if let Some(up) = self.up() {
                neighbours.push(up);
            }
            neighbours
        }
    }

    impl Display for Position {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            write!(f, "{}{:?}", self.letter, self.number)
        }
    }

    /// Symbolizes a position on the board that has been analyzed
    #[derive(PartialEq, Eq)]
    pub struct AnalyzedPosition {
        pub position: Position,
        pub place_hotel_case: PlaceHotelCase,
    }

    impl PartialOrd for AnalyzedPosition {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.position.cmp(&other.position))
        }
    }

    impl Ord for AnalyzedPosition {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.position.cmp(&other.position)
        }
    }

    impl Display for AnalyzedPosition {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            let action = match &self.place_hotel_case {
                PlaceHotelCase::NewChain(_positions) => String::from("Start chain")
                    .color(AnsiColors::Green)
                    .to_string(),
                PlaceHotelCase::ExtendsChain(chain, positions) => format!(
                    "Extend {} by {} hotel(s)",
                    chain.name().color(chain.color()),
                    positions.len(),
                ),
                PlaceHotelCase::Fusion(_chains, origin) => String::from("Fuse chains")
                    .color(AnsiColors::Green)
                    .to_string(),
                PlaceHotelCase::Illegal(reason) => format!("Illegal: {}", reason.reason()),
                PlaceHotelCase::SingleHotel => String::new(),
            };
            // No special formatting is needed
            if action.is_empty() {
                return write!(f, "{}", self.position);
            }
            if action.contains("Illegal") {
                let content = format!("{} [{}]", self.position, action)
                    .color(Rgb(105, 105, 105))
                    .to_string();
                return write!(f, "{}", content);
            }
            write!(f, "{} [{}]", self.position, action)
        }
    }

    impl AnalyzedPosition {
        /// Analyzes the position to return a new analyzed position
        pub fn new(
            position: Position,
            board: &Board,
            hotel_chain_manager: &HotelChainManager,
        ) -> Self {
            let place_hotel_case = analyze_position(&position, board, hotel_chain_manager);
            Self {
                position,
                place_hotel_case,
            }
        }

        /// Creates a new analyzed position without analyzing the position.
        /// Should only be used when the players cards are initialized for the first time.
        /// The place hotel case will be set to single hotel.
        pub fn new_unchecked(position: Position) -> Self {
            Self {
                position,
                place_hotel_case: PlaceHotelCase::SingleHotel,
            }
        }

        /// Analyzes the position again and updates the place hotel case value
        pub fn check(&mut self, board: &Board, hotel_chain_manager: &HotelChainManager) {
            self.place_hotel_case = analyze_position(&self.position, board, hotel_chain_manager);
        }

        /// Checks if this position is illegal
        pub fn is_illegal(&self) -> bool {
            match &self.place_hotel_case {
                PlaceHotelCase::Illegal(_reason) => true,
                _ => false,
            }
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

    #[cfg(test)]
    mod tests {
        use miette::Result;

        use crate::base_game::hotel_chains::HotelChain;

        use super::{Board, Position};

        #[test]
        fn surrounding_positions_correct() {
            let position = Position::new('B', 3);
            let position_prev = Position::new('B', 2);
            let position_next = Position::new('B', 4);
            let position_up = Position::new('A', 3);
            let position_down = Position::new('C', 3);
            assert_eq!(position_prev, position.prev().unwrap());
            assert_eq!(position_next, position.next().unwrap());
            assert_eq!(position_up, position.up().unwrap());
            assert_eq!(position_down, position.down().unwrap());
        }

        #[test]
        fn is_hotel_placed() -> Result<()> {
            let mut board = Board::new();
            let position = Position::new('H', 5);
            board.place_hotel(&position)?;
            assert!(board.is_hotel_placed(&position).is_some());
            let position2 = Position::new('G', 3);
            board.place_hotel_debug(position2, HotelChain::Luxor)?;
            assert!(board.is_hotel_placed(&position2).unwrap().is_some());
            assert!(board.is_hotel_placed(&Position::new('F', 4)).is_none());
            Ok(())
        }
    }
}

/// Stores and handels the settings that are provided fia the command line
pub mod settings {
    //TODO Maybe add settings with which the board dimensions can be changed
    /// Stores the settings
    pub struct Settings {
        pub small_board: bool,
        /// Stores if some extra information should be shown to the player.
        ///
        /// E.g. If the player is the largest shareholder
        pub hide_extra_info: bool,
        /// Stores if some dialogues should be skipped
        pub skip_dialogues: bool,
    }

    impl Settings {
        pub fn new(large_board: bool, hide_extra_info: bool, skip_dialogues: bool) -> Self {
            Self {
                small_board: large_board,
                hide_extra_info,
                skip_dialogues,
            }
        }

        /// Returns a new Settings object with default settings
        pub fn new_default() -> Self {
            Self {
                small_board: false,
                hide_extra_info: false,
                skip_dialogues: false,
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
}

/// Contains all functions for the stocks.
pub mod stock {
    use std::{collections::HashMap, hash::Hasher};

    use super::hotel_chains::{HotelChain, PriceLevel};

    /// Used to symbolize how many stocks a player has/the bank has left for a specific hotel
    #[derive(PartialEq)]
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
    pub const STOCK_BASE_PRICE: [u32; 11] =
        [200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200];

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
        fn stock_price_correct() {
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
    use std::{cmp::Ordering, collections::HashMap, slice::SliceIndex};

    use miette::{miette, Result, SpanContents};
    use owo_colors::OwoColorize;

    use crate::{
        base_game::stock::Stocks,
        game::game::{hotel_chain_manager::HotelChainManager, player_by_id},
        network::broadcast_others,
    };

    use super::{hotel_chains::HotelChain, player::Player};

    pub struct Bank {
        pub stocks_for_sale: Stocks,
        /// Stores the currently largest and second largest shareholders
        pub largest_shareholders: LargestShareholders,
    }

    impl Bank {
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

        /// Prints the current largest shareholders
        pub fn print_largest_shareholders(&self) {
            println!("Largest shareholders:");
            println!("      Chain     || Largest shareholder || Second largest shareholder");
            println!("====================================================================");
            for chain in HotelChain::iterator() {
                let mut ls = Vec::<String>::new();
                let mut sls = Vec::<String>::new();
                for player_id in self
                    .largest_shareholders
                    .largest_shareholder
                    .get(chain)
                    .unwrap()
                {
                    ls.push(format!("{}, ", player_id));
                }
                for player in self
                    .largest_shareholders
                    .second_largest_shareholder
                    .get(chain)
                    .unwrap()
                {
                    sls.push(format!("{}, ", player));
                }
                println!(
                    "{:15} || {:19} || {}",
                    chain.name().color(chain.color()),
                    ls.join(""),
                    sls.join("")
                );
            }
        }

        /// Buy a single stock from the bank.
        /// # Arguments
        /// * `hotel_chain_manager` - The hotel manager for the current match
        /// * `hotel` - The hotel for which the player buys a stock
        /// * `player` - The player that buys the stock
        /// # Returns
        /// * `Ok(())` - When stock was successfully bought
        /// * `Err` - When something went wrong while buying the stock
        pub fn buy_stock(
            &mut self,
            hotel_chain_manager: &HotelChainManager,
            hotel: &HotelChain,
            player: &mut Player,
        ) -> Result<()> {
            // The currently available stocks for the given hotel that can still be bought
            let stock_available = self.stocks_available(hotel, &hotel_chain_manager);
            // Check if the desired stock can still be bought
            if *stock_available == 0 {
                return Err(miette!(
                    "Unable to buy stock from chain {}: No stocks available.",
                    &hotel
                ));
            }
            let stock_price = Bank::stock_price(&hotel_chain_manager, &hotel);
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

        /// Sell a number of stocks back to the bank
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
            // Move stocks from players inventory to the bank
            player.owned_stocks.set_stocks(chain, 0);
            self.stocks_for_sale.increase_stocks(chain, player_stocks);
            // Give money to player
            player.add_money(stock_price * player_stocks);
            Ok(())
        }

        /// Exchanges the stocks of one chain into another
        /// # Arguments
        /// * `to_exchange` - The number of stocks that should be exchanged
        /// # Returns
        /// * `Err` - When `to_exchange` is odd, when no stocks are left for the hotel_chain into
        /// which the stocks should be exchanged
        pub fn exchange_stock(
            &mut self,
            player: &mut Player,
            to_exchange: u32,
            dead: &HotelChain,
            alive: &HotelChain,
        ) -> Result<()> {
            let available_to_exchange = self.stocks_for_sale.stocks_for_hotel(&alive);
            if to_exchange % 2 != 0 {
                return Err(miette!("Unable to echange stocks: {} is odd", to_exchange));
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

        /// Gives one stock of the hotel chain to the player for free
        /// # Arguments
        /// * `players` - The list of players playing the game. Used to update largest
        /// shareholders.
        pub fn give_bonus_stock(&mut self, chain: &HotelChain, player: &mut Player) {
            // Check if stocks are left
            if *self.stocks_for_sale.stocks.get(&chain).unwrap() == 0 {
                player
                    .print_text_ln("You did not recieve a bonus stock because no stocks are left!");
            }
            *self.stocks_for_sale.stocks.get_mut(&chain).unwrap() -= 1;
            // Give stock to player
            *player.owned_stocks.stocks.get_mut(&chain).unwrap() += 1;
        }

        /// Updates who the largest and second largest shareholders are.
        /// For that the stocks of earch player are compared to one another.
        pub fn update_largest_shareholders(&mut self, players: &Vec<Player>) {
            // Clear currently largest shareholder vectors and initialize new
            self.largest_shareholders = LargestShareholders::new();
            for chain in HotelChain::iterator() {
                let mut largest_shareholder: Vec<u32> = Vec::new();
                let mut second_largest_shareholder: Vec<u32> = Vec::new();
                for player in players {
                    // Check if player owns stocks for that chain
                    if *player.owned_stocks.stocks.get(&chain).unwrap() == 0 {
                        continue;
                    }
                    // Set first player to largest and second largest shareholder
                    if largest_shareholder.is_empty() && second_largest_shareholder.is_empty() {
                        largest_shareholder.push(player.id);
                        second_largest_shareholder.push(player.id);
                        continue;
                    }
                    // Determine currently largest shareholders
                    // On cases where continue has been called the second largest shareholder
                    // has been set already.
                    let largest_shareholder_id = *largest_shareholder.get(0).unwrap();
                    let largest_shareholder_stocks = player_by_id(largest_shareholder_id, players)
                        .unwrap()
                        .owned_stocks
                        .stocks
                        .get(&chain)
                        .unwrap();
                    let second_largest_shareholder_id = *second_largest_shareholder.get(0).unwrap();
                    let second_largest_shareholder_stocks =
                        player_by_id(second_largest_shareholder_id, players)
                            .unwrap()
                            .owned_stocks
                            .stocks
                            .get(&chain)
                            .unwrap();
                    let current_player_stocks = *player.owned_stocks.stocks.get(&chain).unwrap();
                    if largest_shareholder.len() == 1 {
                        // Currently only one player is largest shareholder
                        match current_player_stocks.cmp(&largest_shareholder_stocks) {
                            // The player will be set second largest shareholder if the
                            // currently second largest shareholder has less stocks then them
                            Ordering::Less => (),
                            // Player has equal stocks => booth players will be set to largest
                            // and second largest shareholder
                            Ordering::Equal => {
                                largest_shareholder.push(player.id);
                                second_largest_shareholder.push(player.id);
                                continue;
                            }
                            // Player has more stocks => The player will be set largest
                            // shareholder and the previously largest shareholder will become
                            // second largest shareholder
                            Ordering::Greater => {
                                largest_shareholder.remove(0);
                                largest_shareholder.push(player.id);
                            }
                        }
                    } else if largest_shareholder.len() > 1 {
                        // Currently more than one players are largest shareholders
                        match current_player_stocks.cmp(&second_largest_shareholder_stocks) {
                            Ordering::Less => (),
                            // Player has equal stocks => current player will be added to the largest
                            // and second largest shareholder vector
                            Ordering::Equal => {
                                largest_shareholder.push(player.id);
                                second_largest_shareholder.push(player.id);
                                continue;
                            }
                            // The players that where stored as largest and second largest
                            // shareholder will now only be second largest shareholder. The
                            // current player will be set largest shareholder
                            Ordering::Greater => {
                                largest_shareholder.clear();
                                largest_shareholder.push(player.id);
                                continue;
                            }
                        }
                    }
                    // Determine the currently second largest shareholder
                    match second_largest_shareholder_stocks.cmp(&current_player_stocks) {
                        Ordering::Less => (),
                        // Player has equal stocks => current player will be added to the second largest shareholder vector
                        Ordering::Equal => {
                            second_largest_shareholder.push(player.id);
                            continue;
                        }
                        // Player has more stocks => all currently second largest
                        // shareholders will be removed and replaced by the current player
                        Ordering::Greater => {
                            second_largest_shareholder.clear();
                            second_largest_shareholder.push(player.id);
                            continue;
                        }
                    }
                }
                // Insert largest shareholders for chain
                self.largest_shareholders
                    .largest_shareholder
                    .insert(*chain, largest_shareholder);
                self.largest_shareholders
                    .second_largest_shareholder
                    .insert(*chain, second_largest_shareholder);
            }
        }

        /// Gives the largest and second largest shareholders the bonus.
        /// A player that is given a bonus will recieve a message.
        /// # Arguments
        /// * `players` - The playrs that play the game
        /// * `chain` - The chain for which the bonuses should be payed
        /// * `inform_player` - If true the player will recieve a message that they got their
        /// shareholder bonus. This message has to be confirmed by enter.
        pub fn give_majority_shareholder_bonuses(
            &self,
            players: &mut Vec<Player>,
            chain: &HotelChain,
            hotel_chain_manager: &HotelChainManager,
            inform_player: bool,
        ) -> Result<()> {
            //TODO Add functionality that the summary is printed to each player
            let largest_shareholders = self
                .largest_shareholders
                .largest_shareholder
                .get(chain)
                .unwrap();
            let second_largest_shareholders = self
                .largest_shareholders
                .second_largest_shareholder
                .get(chain)
                .unwrap();
            // Check if largest shareholders are set.
            if largest_shareholders.len() == 0 && second_largest_shareholders.len() == 0 {
                return Err(miette!("Unable to give majority shareholder bonuses: The largest shareholders are not set for chain {}", chain));
            }
            let largest_shareholder_bonus = Bank::stock_price(hotel_chain_manager, chain) * 10;
            let second_largest_shareholder_bonus =
                Bank::stock_price(hotel_chain_manager, chain) * 5;
            match largest_shareholders.len() {
                1 => {
                    let largest_shareholder_name =
                        players[largest_shareholders[0] as usize].name.clone();
                    players[largest_shareholders[0] as usize].add_money(largest_shareholder_bonus);
                    if inform_player {
                        broadcast_others(
                            &format!(
                                "{}, recieved {}€ because they where the largest shareholder.",
                                largest_shareholder_name, largest_shareholder_bonus
                            ),
                            &largest_shareholder_name,
                            players,
                        );
                        players[largest_shareholders[0] as usize].get_enter(&format!(
                            "{}, you recieved {}€ because you where the largest shareholder. (press enter to continue)",
                            &largest_shareholder_name, largest_shareholder_bonus
                        ));
                    }
                    match second_largest_shareholders.len() {
                        1 => {
                            let second_largest_shareholder_name = players
                                [second_largest_shareholders[0] as usize]
                                .name
                                .clone();
                            players[second_largest_shareholders[0] as usize]
                                .add_money(second_largest_shareholder_bonus);
                            if inform_player {
                                broadcast_others(
                            &format!(
                                "{}, recieved {}€ because they where the second largest shareholder.",
                                second_largest_shareholder_name, second_largest_shareholder_bonus
                            ),
                            &second_largest_shareholder_name,
                            players,
                        );
                                players[second_largest_shareholders[0] as usize].get_enter(&format!(
                            "{}, you recieved {}€ because you where the seond largest shareholder. (press enter to continue)",
                            &second_largest_shareholder_name, second_largest_shareholder_bonus
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
                                let name = players[*i as usize].name.clone();
                                players[*i as usize].add_money(bonus);
                                if inform_player {
                                    broadcast_others(&format!("{}, recieved {}€ because they where one of the second largest shareholders.", &name, bonus), &name, players);
                                    players[*i as usize].get_enter(&format!("{}, you recieved {}€ because you where one of the second largest shareholders. (press enter to continue)", &name, bonus));
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
                        let player = players.get_mut(*i as usize).unwrap();
                        player.add_money(bonus);
                        if inform_player {
                            player.get_enter(&format!("{}, you recieved {}€ because you where one of the largest shareholders. (press enter to continue)", player.name, bonus));
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

    /// Used to store if the player is a largest or second largest shareholder
    pub struct LargestShareholders {
        /// Contains what the player ids of the largest shareholder for the specified hotel are
        pub largest_shareholder: HashMap<HotelChain, Vec<u32>>,
        /// Contains what the player ids of the second largest shareholder for the specified chain are
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

    #[cfg(test)]
    mod tests {
        use miette::Result;

        use crate::{
            base_game::{
                bank::Bank,
                board::{Board, Position},
                hotel_chains::HotelChain,
                player::Player,
                settings::Settings,
            },
            game::game::{
                hotel_chain_manager::{self, HotelChainManager},
                GameManager,
            },
        };

        #[test]
        fn stock_price_correct() -> Result<()> {
            let mut game_manager = GameManager::new(2, Settings::new_default()).unwrap();
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Airport,
                vec![Position::new('A', 1), Position::new('A', 2)],
                &mut game_manager.board,
                &mut Player::new(Vec::new(), 1, false),
                &mut game_manager.bank,
            )?;
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Imperial,
                vec![
                    Position::new('B', 3),
                    Position::new('C', 3),
                    Position::new('C', 4),
                ],
                &mut game_manager.board,
                &mut Player::new(Vec::new(), 1, false),
                &mut game_manager.bank,
            )?;
            game_manager.hotel_chain_manager.start_chain(
                HotelChain::Continental,
                vec![
                    Position::new('H', 1),
                    Position::new('H', 2),
                    Position::new('H', 3),
                    Position::new('H', 4),
                ],
                &mut game_manager.board,
                &mut Player::new(Vec::new(), 1, false),
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

        #[test]
        fn buy_stock_errors_work() {
            let mut game = GameManager::new(2, Settings::new_default()).unwrap();
            // Test if Hotel is not active error works
            let mut input = game.bank.buy_stock(
                &game.hotel_chain_manager,
                &HotelChain::Airport,
                game.players.get_mut(0).unwrap(),
            );
            assert!(is_error(input));
            // Test if no stocks left error works
            game.bank
                .stocks_for_sale
                .set_stocks(&HotelChain::Airport, 0);
            input = game.bank.buy_stock(
                &game.hotel_chain_manager,
                &HotelChain::Airport,
                game.players.get_mut(0).unwrap(),
            );
            assert!(is_error(input));
            // Test if not enough money error works
            game.bank
                .stocks_for_sale
                .set_stocks(&HotelChain::Airport, 5);
            game.players.get_mut(0).unwrap().money = 0;
            input = game.bank.buy_stock(
                &game.hotel_chain_manager,
                &HotelChain::Airport,
                game.players.get_mut(0).unwrap(),
            );
            assert!(is_error(input));
        }

        #[test]
        fn exchange_stocks_works() -> Result<()> {
            let mut bank = Bank::new();
            let mut player = Player::new(vec![], 0, false);
            let dead = HotelChain::Airport;
            let alive = HotelChain::Festival;
            player.owned_stocks.increase_stocks(&dead, 6);
            bank.exchange_stock(&mut player, 6, &dead, &HotelChain::Festival)?;
            assert_eq!(*player.owned_stocks.stocks_for_hotel(&alive), 3);
            Ok(())
        }

        #[test]
        fn largest_shareholders_correct() {
            let mut game_manager = GameManager::new(4, Settings::new_default()).unwrap();

            let mut index = 0;
            while index < 4 {
                let mut player = game_manager.players.get_mut(index).unwrap();
                match index {
                    0 => {
                        player.owned_stocks.set_stocks(&HotelChain::Airport, 7);
                        player.owned_stocks.set_stocks(&HotelChain::Continental, 10);
                        player.owned_stocks.set_stocks(&HotelChain::Festival, 5);
                        player.owned_stocks.set_stocks(&HotelChain::Imperial, 7);
                    }
                    1 => {
                        player.owned_stocks.set_stocks(&HotelChain::Airport, 2);
                        player.owned_stocks.set_stocks(&HotelChain::Continental, 10);
                        player.owned_stocks.set_stocks(&HotelChain::Festival, 3);
                    }
                    2 => {
                        player.owned_stocks.set_stocks(&HotelChain::Festival, 3);
                        player.owned_stocks.set_stocks(&HotelChain::Continental, 10);
                    }
                    3 => {
                        player.owned_stocks.set_stocks(&HotelChain::Festival, 3);
                    }
                    _ => (),
                }
                index += 1;
            }
            game_manager
                .bank
                .update_largest_shareholders(&game_manager.players);
            game_manager.bank.print_largest_shareholders();
            // Test case 1: one largest and one second largest shareholder (Continental)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Airport
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Airport
            ));
            // Test case 2: multiple largest shareholerds (Airport)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(2).unwrap().id,
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(2).unwrap().id,
                &HotelChain::Continental
            ));
            // Test case 3: one largest and multiple second largest shareholders (Prestige)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(2).unwrap().id,
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(3).unwrap().id,
                &HotelChain::Festival
            ));
            // Test case 4: one player is largest and second largest shareholder (Luxor)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Imperial
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(0).unwrap().id,
                &HotelChain::Imperial
            ));
            assert!(!game_manager.bank.is_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Imperial
            ));
            assert!(!game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap().id,
                &HotelChain::Imperial
            ));
        }

        #[test]
        fn sell_stock_works() -> Result<()> {
            let mut bank = Bank::new();
            let mut board = Board::new();
            let mut hotel_chain_manager = HotelChainManager::new();
            let player = Player::new(vec![], 0, false);
            let mut players = vec![player];
            let chain = HotelChain::Airport;
            // Test error
            assert!(bank
                .sell_stock(players.get_mut(0).unwrap(), 4, &chain, &hotel_chain_manager)
                .is_err());
            hotel_chain_manager.start_chain(
                chain,
                vec![Position::new('A', 1), Position::new('A', 2)],
                &mut board,
                &mut players.get_mut(0).unwrap(),
                &mut bank,
            )?;
            bank.update_largest_shareholders(&players);
            bank.give_majority_shareholder_bonuses(
                &mut players,
                &chain,
                &hotel_chain_manager,
                false,
            )?;
            assert_eq!(players.get(0).unwrap().money, 9000);
            Ok(())
        }

        #[test]
        fn give_majority_shareholder_bonuses_works() -> Result<()> {
            use crate::{
                base_game::board::Board,
                game::game::hotel_chain_manager::{self, HotelChainManager},
            };

            // Basic scenario setup
            let mut bank = Bank::new();
            let mut board = Board::new();
            let mut players = Vec::new();
            players.push(Player::new(vec![], 0, false));
            players.push(Player::new(vec![], 1, false));
            players.push(Player::new(vec![], 2, false));
            let mut hotel_chain_manager = HotelChainManager::new();
            let chain = HotelChain::Imperial;
            let player = players.get_mut(0).unwrap();
            hotel_chain_manager.start_chain(
                chain,
                vec![Position::new('A', 1), Position::new('A', 2)],
                &mut board,
                player,
                &mut bank,
            )?;
            bank.buy_stock(&hotel_chain_manager, &chain, player)?;
            player.money = 6000;
            // Test cases:
            // 1. 1 Player largest and second largest
            bank.update_largest_shareholders(&players);
            bank.print_largest_shareholders();
            bank.give_majority_shareholder_bonuses(
                &mut players,
                &chain,
                &hotel_chain_manager,
                false,
            )?;
            let player = players.get_mut(0).unwrap();
            assert_eq!(player.money, 10500);
            // 2. More than 1 player largest and second largest
            player.money = 6000;
            let player2 = players.get_mut(1).unwrap();
            bank.buy_stock(&hotel_chain_manager, &chain, player2)?;
            bank.buy_stock(&hotel_chain_manager, &chain, player2)?;
            player2.money = 6000;
            bank.update_largest_shareholders(&players);
            bank.print_largest_shareholders();
            bank.give_majority_shareholder_bonuses(
                &mut players,
                &chain,
                &hotel_chain_manager,
                false,
            )?;
            let player = players.get_mut(0).unwrap();
            assert_eq!(player.money, 8300);
            let player2 = players.get_mut(1).unwrap();
            assert_eq!(player2.money, 8300);
            // 3. 1 Player largest and more than 1 player second largest
            //  Player is largest shareholder and player 2 and 3 are second largest shareholder
            let player = players.get_mut(0).unwrap();
            bank.buy_stock(&hotel_chain_manager, &chain, player)?;
            player.money = 6000;
            let player2 = players.get_mut(1).unwrap();
            player2.money = 6000;
            let player3 = players.get_mut(2).unwrap();
            bank.buy_stock(&hotel_chain_manager, &chain, player3)?;
            bank.buy_stock(&hotel_chain_manager, &chain, player3)?;
            player3.money = 6000;
            bank.update_largest_shareholders(&players);
            bank.print_largest_shareholders();
            bank.give_majority_shareholder_bonuses(
                &mut players,
                &chain,
                &hotel_chain_manager,
                false,
            )?;
            let player = players.get_mut(0).unwrap();
            assert_eq!(player.money, 9000);
            let player2 = players.get_mut(1).unwrap();
            assert_eq!(player2.money, 6800);
            let player3 = players.get_mut(1).unwrap();
            assert_eq!(player3.money, 6800);
            Ok(())
        }

        fn is_error(input: Result<()>) -> bool {
            return match input {
                Err(_) => true,
                Ok(_) => false,
            };
        }
    }
}

/// Player management
pub mod player {
    use std::{
        cmp::min,
        cmp::Ordering,
        cmp::PartialEq,
        cmp::PartialOrd,
        collections::HashMap,
        fmt::Debug,
        io::{BufRead, BufReader, Write},
        net::TcpStream,
        ops::RangeInclusive,
        process::exit,
        slice::{IterMut, SliceIndex},
        str::FromStr,
    };

    use crate::{
        base_game::bank::Bank,
        base_game::board::Position,
        base_game::{hotel_chains::HotelChain, stock::Stocks},
        data_stream::read_enter,
        game::game::{
            hotel_chain_manager::{self, HotelChainManager},
            GameManager,
        },
        logic::place_hotel::{IllegalPlacement, PlaceHotelCase},
        network::{broadcast, send_string},
        utils::generate_number_vector,
    };
    use miette::{miette, Result};
    use owo_colors::{AnsiColors, OwoColorize, Rgb};
    use read_input::{prelude::input, InputBuild};

    use super::board::{self, AnalyzedPosition, Board};

    /// Stores all variables that belong to the player
    //#[derive(PartialEq)]
    pub struct Player {
        /// The money the player currently has
        pub money: u32,
        /// The stocks that the player currently owns
        pub owned_stocks: Stocks,
        /// Contains the cards that the player currently has on his hand
        pub analyzed_cards: Vec<AnalyzedPosition>,
        /// The id of the player (This should be the index at which this player is stored in the players vecor in the game manager).
        pub id: u32,
        /// The name of the player
        pub name: String,
        /// The tcp stream that belongs to this player. Is used to communicate with the players client.
        pub tcp_stream: Option<TcpStream>,
        /// If the board should be printed small
        /// Determines how the board should be printed.
        /// This behaviour can be set with the -s flag.
        /// If false or not set the (empty) board is printed this way:
        /// ```None
        /// A |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// B |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// C |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// D |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// E |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// F |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// G |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// H |   |   |   |   |   |   |   |   |   |   |   |
        /// --------------------------------------------------
        /// I |   |   |   |   |   |   |   |   |   |   |   |
        ///     1   2   3   4   5   6   7   8   9  10  11  12
        /// ```
        /// If true the (empty) board is printed this way:
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
        pub small_board: bool,
    }

    impl PartialEq for Player {
        fn eq(&self, other: &Player) -> bool {
            self.id == other.id && self.name == other.name && self.money == other.money
        }
    }

    /// Players will be sorted by id, if id is same than by name
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
        /// creates a new player
        pub fn new(start_cards: Vec<Position>, id: u32, small_board: bool) -> Self {
            Player::new_named(start_cards, id, small_board, format!("Player {}", id + 1))
        }

        /// Creates a new player with a custom name
        pub fn new_named(
            start_cards: Vec<Position>,
            id: u32,
            small_board: bool,
            name: String,
        ) -> Self {
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
                tcp_stream: None,
                small_board,
            }
        }

        /// Creates a new client player
        pub fn new_client(
            start_cards: Vec<Position>,
            id: u32,
            name: String,
            tcp_stream: TcpStream,
            small_board: bool,
        ) -> Self {
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
                tcp_stream: Some(tcp_stream),
                small_board,
            }
        }

        /// Add money to the player
        pub fn add_money(&mut self, money: u32) {
            self.money += money;
        }

        /// Remove money from the player
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
            for analyzed_card in &self.analyzed_cards {
                print!(
                    "[{}{:2}]",
                    analyzed_card.position.letter, analyzed_card.position.number
                );
            }
            println!();
        }

        /// Returns true if the player has no cards that can be played
        pub fn only_illegal_cards(&self) -> bool {
            for card in &self.analyzed_cards {
                if !card.is_illegal() {
                    return false;
                }
            }
            true
        }

        /// Sorts the players current hand cards
        pub fn sort_cards(&mut self) {
            self.analyzed_cards.sort()
        }

        /// Removes a card from the players inventory.
        /// Returns the removed card when the card has been removed successfully.
        /// Otherwise `None` is returned.
        pub fn remove_card(&mut self, position: &Position) -> Result<AnalyzedPosition> {
            self.sort_cards();
            for (index, analyzed_card) in self.analyzed_cards.iter().enumerate() {
                if analyzed_card.position.letter.eq(&position.letter) {
                    if analyzed_card.position.number.eq(&position.number) {
                        return Ok(self.analyzed_cards.remove(index));
                    }
                }
            }
            Err(miette!(
                "Unable to remove card from player, the requested card {} could not be found.",
                position
            ))
        }

        /// Adds a card to the players inventory.
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

        /// Analyzes the players hand cards and updates the place hotel case value
        pub fn analyze_cards(&mut self, board: &Board, hotel_chain_manager: &HotelChainManager) {
            for card in &mut self.analyzed_cards {
                card.check(board, hotel_chain_manager);
            }
        }

        /// Returns the current state of the player
        pub fn player_ui(&self) -> Vec<String> {
            let mut ui = Vec::new();
            // Print money
            ui.push(format!(
                "{} {}€",
                String::from("Money:").bright_green(),
                self.money
            ));
            // Print cards
            let mut cards = String::new();
            cards.push_str(&String::from("Cards: ").bright_green().to_string());
            let mut first_card = true;
            for (index, analyzed_card) in self.analyzed_cards.iter().enumerate() {
                if first_card {
                    first_card = false;
                } else {
                    cards.push_str(", ");
                }
                if analyzed_card.is_illegal() {
                    cards.push_str(&format!(
                        "{} {}",
                        format!("({})", index + 1).color(Rgb(105, 105, 105)),
                        analyzed_card
                    ));
                } else {
                    cards.push_str(&format!(
                        "({}) {}",
                        format!("{}", index + 1).color(AnsiColors::BrightBlue),
                        analyzed_card
                    ));
                }
            }
            ui.push(cards);
            // Print stocks
            //shareholder or a silver * when the player is the second largest shareholder.
            //The star is positioned here: Airport*:
            let mut stocks = String::new();
            stocks.push_str(&String::from("Stocks: ").bright_green().to_string());
            first_card = true;
            for chain in HotelChain::iterator() {
                if first_card {
                    first_card = false;
                } else {
                    stocks.push_str(", ");
                }
                stocks.push_str(&format!(
                    "{}: {}",
                    chain.name().color(chain.color()),
                    self.owned_stocks.stocks_for_hotel(chain)
                ));
            }
            ui.push(stocks);
            ui
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

        /// Prints the current player to the console
        pub fn print_player_ui(&self) {
            // Print money
            println!("{} {}€", String::from("Money:").bright_green(), self.money);
            // Print cards
            print!("{}", String::from("Cards: ").bright_green());
            let mut first_card = true;
            for analyzed_card in &self.analyzed_cards {
                if first_card {
                    first_card = false;
                } else {
                    print!(", ");
                }
                print!("{}", analyzed_card);
            }
            println!();
            // Print stocks
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

        /// Promts the user to press enter to draw a new card.
        /// The card is drawn beforehand.
        pub fn draw_card(
            &mut self,
            card: Position,
            skip_dialogues: bool,
            board: &Board,
            hotel_chain_manager: &HotelChainManager,
        ) {
            if !skip_dialogues {
                self.get_enter("Press enter to draw a new card");
            }
            if !skip_dialogues {
                self.print_text_ln(&format!(
                    "Card drawn: {}",
                    &card.to_string().color(AnsiColors::Green)
                ));
            }
            self.add_card(&card, &board, &hotel_chain_manager);
            if !skip_dialogues {
                self.get_enter("Press enter to finish your turn");
            }
        }

        /// Prompts the user to select a card.
        /// This card is then removed from the players inventory and returned.
        pub fn read_card(&mut self) -> Result<AnalyzedPosition> {
            loop {
                let card_index = self.read_input(
                    format!("Enter a number 1-{}: ", self.analyzed_cards.len()),
                    generate_number_vector(1, self.analyzed_cards.len() as u32),
                ) as usize
                    - 1;
                let analyzed_position = *self.analyzed_cards.get(card_index).as_ref().unwrap();
                // Check if hotel placement is allowed
                if analyzed_position.is_illegal() {
                    let reason =
                        match analyzed_position
                            .place_hotel_case
                            .eq(&PlaceHotelCase::Illegal(
                                IllegalPlacement::ChainStartIllegal,
                            )) {
                            true => IllegalPlacement::ChainStartIllegal.description(),
                            false => IllegalPlacement::FusionIllegal.description(),
                        };
                    self.print_text_ln(&format!(
                        "This position is illegal [{}]: {}",
                        analyzed_position
                            .position
                            .color(Rgb(105, 105, 105))
                            .to_string(),
                        reason.color(AnsiColors::Red).to_string()
                    ));
                    self.print_text_ln("Please select another card!");
                    continue;
                }
                let position = analyzed_position.position.clone();
                //Remove the played card from the players hand cards
                return Ok(self.remove_card(&position)?);
            }
        }

        /// The player is involved in a fusion.
        /// This function will ask the player what they would like to do with the stocks that they
        /// have of the chain that is being fused.
        /// # Returns
        /// *`Ok(u32, u32)` - Contains the amount of stocks the player traded, sold and keept.
        pub fn handle_fusion_stocks(
            &mut self,
            dead: &HotelChain,
            alive: &HotelChain,
            bank: &mut Bank,
            hotel_chain_manager: &HotelChainManager,
        ) -> Result<(u32, u32, u32)> {
            let number_of_stocks = *self.owned_stocks.stocks_for_hotel(dead);
            self.print_text_ln(&format!(
                "{}, it's your turn to decide what you would like to do with your {} stock(s):",
                self.name, number_of_stocks
            ));
            let mut stocks_unasigned;
            let mut stocks_to_exchange = 0;
            let mut stocks_to_sell;
            // loop that runs until the player has decided what they would like to do with the
            // stocks
            loop {
                //TODO Test if stocks can no longer be exchanged when the stock does not have any
                //more stocks (do this when buy stocks is implemented).
                // First ask how many stocks should be exchanged
                stocks_unasigned = number_of_stocks;
                let mut allowed_values = vec![];
                // fill allowed values
                let mut allowed_string = String::new();
                let mut new_alive_stocks_number = 0;
                // Stores how many stocks the bank has left of the chain that survives the fusion
                let stocks_left_to_exchange = bank.stocks_for_sale.stocks_for_hotel(alive);
                for i in 0..=stocks_unasigned {
                    if i % 2 == 0 && *stocks_left_to_exchange >= i / 2 {
                        // i/2 is calculated because two stocks will be traded into one
                        if i != 0 {
                            allowed_string.push_str(", ");
                        }
                        allowed_values.push(i);
                        allowed_string.push_str(&i.to_string());
                    }
                }
                if allowed_values.len() != 1 {
                    stocks_to_exchange = self.read_input(
                        format!(
                            "Please enter how many stocks you would like to exchange [{}]: ",
                            allowed_string
                        ),
                        allowed_values,
                    );
                    new_alive_stocks_number = stocks_to_exchange / 2;
                } else {
                    // No stocks available for trade
                    if *stocks_left_to_exchange == 0 {
                        self.print_text_ln(&format!(
                        "Please enter how many stocks you would like to exchange [{}]: 0 {}",
                        allowed_string,
                        "- the bank does not have any stocks left that could be exchanged to you".color(Rgb(105, 105, 105))
                    ));
                    } else {
                        self.print_text_ln(&format!(
                            "Please enter how many stocks you would like to exchange [{}]: 0 {}",
                            allowed_string,
                            "- you don't have enough stocks to exchange them"
                                .color(Rgb(105, 105, 105))
                        ));
                    }
                }
                stocks_unasigned -= stocks_to_exchange;
                // Check if stocks are left that could be sold
                stocks_to_sell = 0;
                if stocks_unasigned != 0 {
                    stocks_to_sell = self.read_input(
                        format!(
                            "Please enter how many stocks you would like to sell [0-{}]: ",
                            stocks_unasigned
                        ),
                        generate_number_vector(0, stocks_unasigned),
                    );
                    stocks_unasigned -= stocks_to_sell;
                } else {
                    // No stocks left to sell
                    self.print_text_ln(&format!(
                        "Please enter how many stocks you would like to sell [0-0]: 0 {}",
                        "- not stocks left to sell".color(Rgb(105, 105, 105))
                    ));
                }
                self.print_text_ln(&format!(
                    "The following will happen to your stocks:\nTotal {} stocks: {} - {} = {}\nTotal {} stocks: {} + {} = {}\nMoney: {}€ + {}€ = {}€",
                    dead.name().color(dead.color()), self.owned_stocks.stocks_for_hotel(dead), stocks_to_sell+&stocks_to_exchange, self.owned_stocks.stocks_for_hotel(dead)-(stocks_to_sell+stocks_to_exchange),
                    alive.name().color(alive.color()), self.owned_stocks.stocks_for_hotel(alive), new_alive_stocks_number, self.owned_stocks.stocks_for_hotel(alive)+new_alive_stocks_number,
                    self.money, Bank::stock_price(hotel_chain_manager, dead)*stocks_to_sell, self.money+Bank::stock_price(hotel_chain_manager, dead)*stocks_to_sell,
                ));
                match self.read_input(
                    String::from("Is this correct? [Y/n]: "),
                    vec!['Y', 'y', 'N', 'n'],
                ) {
                    'Y' => break,
                    'y' => break,
                    'N' => continue,
                    'n' => continue,
                    _ => (),
                }
            }
            // Exchange stocks
            if stocks_to_exchange > 0 {
                bank.exchange_stock(self, stocks_to_exchange, dead, alive)?;
            }
            // Sell stocks
            if stocks_to_sell > 0 {
                bank.sell_stock(self, stocks_to_sell, dead, hotel_chain_manager)?;
            }
            Ok((stocks_to_exchange, stocks_to_sell, stocks_unasigned))
        }

        /// If chains are active, the player is asked if they would like to buy a maximum of three
        /// stocks from available chains.
        /// # Returns
        /// * `None` - The player did not buy any stocks
        /// * `Some(HashMap(HotelChain, u32))` - The player bought stocks, what stocks and how many is stored in the hashmap
        pub fn buy_stocks(
            &mut self,
            bank: &mut Bank,
            hotel_chain_manager: &HotelChainManager,
        ) -> Option<HashMap<HotelChain, u32>> {
            // Check if stocks are available to be bought
            if hotel_chain_manager.active_chains().len() == 0 {
                return None;
            }
            // Check if player has enough money to buy the lowest costing stock
            // Stores the value of the stock that costs the least
            let mut min_stock_value = 2000;
            for chain in hotel_chain_manager.active_chains() {
                let value = Bank::stock_price(hotel_chain_manager, &chain);
                if value < min_stock_value {
                    min_stock_value = value;
                }
            }
            self.print_text_ln(&format!(
                "{}, you can buy a maximum of three stocks now:",
                self.name
            ));
            // Runs until the player confirms the stocks bought
            loop {
                // Stores how many stockes the player is allowed to buy
                let mut stocks_left = 3;
                let mut stocks_bought = HashMap::new();
                // Stores the money available for the current trade
                let mut money_available = self.money;
                for chain in hotel_chain_manager.active_chains() {
                    // Check conditions under which no stocks can be bought
                    let main_message = format!(
                        "How many stocks would you like to buy of {}?",
                        chain.name().color(chain.color())
                    );
                    if stocks_left == 0 {
                        // Player has already bought 3 stocks
                        self.print_text_ln(&format!(
                            "{} [0-0]: 0 {}",
                            main_message,
                            "- already bought 3 stocks".color(Rgb(105, 105, 105))
                        ));
                        continue;
                    }
                    if *bank.stocks_available(&chain, hotel_chain_manager) == 0 {
                        // No stocks left
                        self.print_text_ln(&format!(
                            "{} [0-0]: 0 {}",
                            main_message,
                            "- no stocks left".color(Rgb(105, 105, 105))
                        ));
                        continue;
                    }
                    let stock_price = Bank::stock_price(hotel_chain_manager, &chain);
                    if money_available < stock_price {
                        // Player does not have enough money
                        self.print_text_ln(&format!(
                            "{} [0-0]: 0 {}",
                            main_message,
                            "- not enough money".color(Rgb(105, 105, 105))
                        ));
                        continue;
                    }
                    // Check how many stocks the player could buy with their current money
                    let mut money_for_stocks = 0;
                    if money_available >= stock_price {
                        money_for_stocks = 1;
                    }
                    if money_available >= stock_price * 2 {
                        money_for_stocks = 2;
                    }
                    if money_available >= stock_price * 3 {
                        money_for_stocks = 3;
                    }
                    let mut stocks_can_be_bought = min(money_for_stocks, stocks_left);
                    // Check if the stocks available in the bank are less then the stocks that the
                    // player could buy
                    stocks_can_be_bought = min(
                        stocks_can_be_bought,
                        *bank.stocks_available(&chain, hotel_chain_manager),
                    );
                    let bought = self.read_input(
                        format!(
                            "How many stocks would you like to buy of {}? [0-{}]: ",
                            chain.name().color(chain.color()),
                            stocks_can_be_bought,
                        ),
                        generate_number_vector(0, stocks_can_be_bought),
                    );
                    if bought > 0 {
                        stocks_bought.insert(chain, bought);
                        stocks_left -= bought;
                        money_available -= bought * stock_price;
                    }
                }
                // Check if player bought any stocks
                if stocks_bought.is_empty() {
                    self.print_text_ln("You did not buy any stocks.");
                    if self.get_correct() {
                        return None;
                    }
                    continue;
                }
                self.print_text_ln("The following will happen to your stocks:");
                let mut expanses = 0;
                for (k, v) in &stocks_bought {
                    let current_stocks = self.owned_stocks.stocks_for_hotel(&k);
                    self.print_text_ln(&format!(
                        "Total {} stocks: {} + {} = {}",
                        k.name().color(k.color()),
                        current_stocks,
                        v,
                        current_stocks + v
                    ));
                    expanses += Bank::stock_price(hotel_chain_manager, &k) * v;
                }
                self.print_text_ln(&format!(
                    "Money: {}€ - {}€ = {}€",
                    self.money,
                    expanses,
                    self.money - expanses
                ));
                if !self.get_correct() {
                    continue;
                }
                // Player confirmed transaction
                for (k, v) in &stocks_bought {
                    for _i in 1..=*v {
                        bank.buy_stock(hotel_chain_manager, &k, self).unwrap();
                    }
                }
                return Some(stocks_bought);
            }
        }

        /// Promts the user to enter something.
        ///
        /// If the player is a client, only the text before the first `\n` is transmitted.
        /// # Arguments
        /// * `text` - The text that is displayed
        /// * `allowed_values` - The values that are allowed to be entered
        /// * `T` - The data type that should be read
        pub fn read_input<T: 'static + FromStr + PartialEq>(
            &self,
            text: String,
            allowed_values: Vec<T>,
        ) -> T {
            if self.tcp_stream.is_none() {
                // Player does not play fia lan
                print!("{}", text);
                input::<T>().inside(allowed_values).get()
            } else {
                // Player plays fia lan
                let message = text.split("\n").nth(0).unwrap();
                let mut br = BufReader::new(self.tcp_stream.as_ref().unwrap());
                loop {
                    send_string(self, message, "$Input");
                    let mut buffer = String::new();
                    if let Err(err) = br.read_line(&mut buffer) {
                        println!("Unable to send data to player {}: {}", self.name, err);
                        exit(1);
                    }
                    let input = buffer.trim();
                    match input.parse::<T>() {
                        Ok(ok) => {
                            if !allowed_values.contains(&ok) {
                                self.print_text_ln("That value did not pass, please try again!");
                                continue;
                            }
                            return ok;
                        }
                        Err(_err) => {
                            self.print_text_ln("That value did not pass, please try again!");
                        }
                    };
                }
            }
        }

        /// Prints a text to the player and waits until they pressed enter.
        ///
        /// If the player is a client, only the text before the first `\n` is transmitted.
        pub fn get_enter(&self, text: &str) {
            if self.tcp_stream.is_none() {
                // Player does not play fia lan
                print!("{}", &text);
                read_enter();
            } else {
                // Player plays fia lan
                let message = text.split("\n").nth(0).unwrap();
                send_string(self, message, "$Input");
                let mut br = BufReader::new(self.tcp_stream.as_ref().unwrap());
                let mut buffer = String::new();
                if let Err(err) = br.read_line(&mut buffer) {
                    println!("Unable to send data to player {}: {}", self.name, err);
                    exit(1);
                }
            }
        }

        /// Displayes the message `Is this correct? [Y/n]: ` to the player and returns if they
        /// pressed yes or no.
        pub fn get_correct(&self) -> bool {
            match self.read_input(
                String::from("Is this correct? [Y/n]: "),
                vec!['Y', 'y', 'N', 'n'],
            ) {
                'Y' => return true,
                'y' => return true,
                'N' => return false,
                'n' => return false,
                _ => false,
            }
        }

        /// Prints the text to the player.
        /// Linebreak is not written.
        pub fn print_text(&self, text: &str) {
            if self.tcp_stream.is_none() {
                // Player does not play fia lan
                print!("{}", &text);
            } else {
                // Player plays fia lan
                send_string(self, text, "$Print");
            }
        }

        /// Prints the text to the player.
        /// A linebreak is written.
        pub fn print_text_ln(&self, text: &str) {
            if self.tcp_stream.is_none() {
                // Player does not play fia lan
                println!("{}", &text);
            } else {
                // Player plays fia lan
                send_string(self, text, "$Println");
            }
        }
    }

    /// Returns the player with the name if they exist.
    pub fn player_by_name<'a>(name: &str, players: &'a Vec<Player>) -> Option<&'a Player> {
        for player in players {
            if player.name == name {
                return Some(&player);
            }
        }
        None
    }
}

/// User interface drawing
pub mod ui {
    use crate::{
        base_game::{bank::Bank, board::Board, hotel_chains::HotelChain, settings::Settings},
        game::game::{
            hotel_chain_manager::{self, HotelChainManager},
            round::Round,
        },
    };
    use owo_colors::{AnsiColors, DynColors, OwoColorize, Rgb};

    use super::player::{player_by_name, Player};

    /// Prints the main ui for every player.
    /// If all players are on the same machine the ui is only printed once.
    pub fn print_main_ui_players(
        current_player_name: String,
        players: &Vec<Player>,
        board: &Board,
        settings: &Settings,
        round: Option<&Round>,
        bank: &Bank,
        hotel_chain_manager: &HotelChainManager,
    ) {
        let mut written_to_console = false;
        for player in players {
            player.print_text_ln("");
            if all_players_local(players) {
                let current_player = player_by_name(&current_player_name, players).unwrap();
                print_main_ui_console(
                    Some(current_player),
                    Some(&current_player_name),
                    board,
                    settings,
                    round,
                    bank,
                    hotel_chain_manager,
                );
                written_to_console = true;
            }
            if player.tcp_stream.is_none() {
                if !written_to_console {
                    print_main_ui_console(
                        Some(player),
                        Some(&current_player_name),
                        board,
                        settings,
                        round,
                        bank,
                        hotel_chain_manager,
                    );
                    written_to_console = true;
                }
            } else {
                for line in main_ui(
                    Some(player),
                    Some(&current_player_name),
                    board,
                    settings,
                    round,
                    bank,
                    hotel_chain_manager,
                ) {
                    player.print_text_ln(&line);
                }
            }
        }
    }

    /// Checks if all playing players are playing on one pc
    fn all_players_local(players: &Vec<Player>) -> bool {
        for player in players {
            if player.tcp_stream.is_some() {
                return false;
            }
        }
        true
    }

    /// Prints the main ui to the console
    pub fn print_main_ui_console(
        player: Option<&Player>,
        current_player_name: Option<&String>,
        board: &Board,
        settings: &Settings,
        round: Option<&Round>,
        bank: &Bank,
        hotel_chain_manager: &HotelChainManager,
    ) {
        let main_ui = main_ui(
            player,
            current_player_name,
            board,
            settings,
            round,
            bank,
            hotel_chain_manager,
        );
        for line in main_ui {
            println!("{}", line);
        }
    }

    /// Returns the main user interface.
    /// # Arguments
    /// * `player` - The player for which the money, cards and stocks should be displayed
    /// * `current_player_name` - The name of the player whos turn it is
    /// * `board` - The current game board
    /// * `settings` - The games settings
    /// * `round` - The current game round
    /// * `bank` - The bank of the game
    /// * `hotel_chain_manager` - The hotel chain manager of the game
    /// # Returns
    /// * `Vec<String>` - This vector contains the contents of the main ui
    pub fn main_ui(
        player: Option<&Player>,
        current_player_name: Option<&String>,
        board: &Board,
        settings: &Settings,
        round: Option<&Round>,
        bank: &Bank,
        hotel_chain_manager: &HotelChainManager,
    ) -> Vec<String> {
        let mut main_ui = Vec::new();
        let small_board;
        if player.is_some() {
            small_board = player.unwrap().small_board;
        } else {
            small_board = settings.small_board;
        }
        for line in board.get_board_state(small_board) {
            main_ui.push(line);
        }
        main_ui.push(String::new());
        match round {
            None => main_ui.push(String::from("Round 0 - Game has not been started yet")),
            Some(round) => {
                main_ui.push(format!("Round {}", round.number));
                match current_player_name {
                    None => main_ui.push(String::from("Current player: None")),
                    Some(name) => main_ui.push(format!("Current player: {}", name)),
                }
                match player {
                    None => main_ui.push(String::from("Player unavailable")),
                    Some(player) => {
                        main_ui.push(format!("{}, your status:", player.name));
                        for line in player.player_ui() {
                            main_ui.push(line);
                        }
                    }
                };
            }
        };
        main_ui.push(String::new());
        main_ui.push(format!("{}", String::from("Game stats:").bright_green()));
        main_ui.push(format!("{:15}||      Hotels       ||        Stocks          ||      Bonuses for the majority shareholders", ""));
        main_ui.push(format!("{:15}|| Number ||  Range  || Bank || Owned || Value || Largest shareholder || Second largest shareholder", ""));
        main_ui.push(format!("=================================================================================================================="));
        for chain in HotelChain::iterator() {
            // Set the color of the values
            let enable_color = hotel_chain_manager.chain_status(chain);
            let color = match enable_color {
                true => DynColors::Ansi(AnsiColors::White),
                false => DynColors::Rgb(105, 105, 105),
            };
            let chain_color = match enable_color {
                true => chain.color(),
                false => Rgb(105, 105, 105),
            };
            let player_stocks = match player {
                None => 0 as u32,
                Some(player) => *player.owned_stocks.stocks_for_hotel(chain),
            };
            let formatted_string1 = format!(
                "||   {:2}   || {:7} ||  {:2}  ||   {:2}",
                hotel_chain_manager.chain_length(chain),
                hotel_chain_manager.price_range(chain),
                bank.stocks_available(&chain, hotel_chain_manager),
                player_stocks,
            );
            let formatted_string2 = format!(
                " || {:4}€ ||        {:5}€       ||        {:5}€",
                Bank::stock_price(&hotel_chain_manager, &chain),
                Bank::stock_price(&hotel_chain_manager, &chain) * 10,
                Bank::stock_price(&hotel_chain_manager, &chain) * 5,
            );
            let stock_status_symbol = match player {
                None => String::from(" "),
                Some(player) => stock_status_symbol(
                    &bank,
                    &hotel_chain_manager,
                    &chain,
                    player.id,
                    !settings.hide_extra_info,
                ),
            };
            let hotel_price_color;
            if !enable_color {
                hotel_price_color = color;
            } else {
                hotel_price_color = match chain.price_level() {
                    super::hotel_chains::PriceLevel::Low => DynColors::Ansi(AnsiColors::Red),
                    super::hotel_chains::PriceLevel::Medium => DynColors::Ansi(AnsiColors::Yellow),
                    super::hotel_chains::PriceLevel::High => DynColors::Ansi(AnsiColors::Green),
                }
            };
            let hotel_price = match chain.price_level() {
                super::hotel_chains::PriceLevel::Low => {
                    if enable_color {
                        format!("[{}]", "L".color(hotel_price_color))
                    } else {
                        format!("[{}]", "L").color(color).to_string()
                    }
                }
                super::hotel_chains::PriceLevel::Medium => {
                    if enable_color {
                        format!("[{}]", "M".color(hotel_price_color))
                    } else {
                        format!("[{}]", "M").color(color).to_string()
                    }
                }
                super::hotel_chains::PriceLevel::High => {
                    if enable_color {
                        format!("[{}]", "H".color(hotel_price_color))
                    } else {
                        format!("[{}]", "H").color(color).to_string()
                    }
                }
            };
            main_ui.push(format!(
                "{:12}{}{}{}{}",
                chain.name().color(chain_color),
                hotel_price,
                formatted_string1.color(color),
                stock_status_symbol,
                formatted_string2.color(color),
            ));
        }
        main_ui
    }

    /// Used to display a little star that indicates if the player is largest or second largest
    /// shareholder
    fn stock_status_symbol(
        bank: &Bank,
        hotel_manager: &HotelChainManager,
        chain: &HotelChain,
        player_id: u32,
        show_symbol: bool,
    ) -> String {
        if !hotel_manager.chain_status(&chain) || !show_symbol {
            return String::from(" ");
        }
        if bank.is_largest_shareholder(player_id, chain) {
            return "*".color(Rgb(225, 215, 0)).to_string();
        }
        if bank.is_second_largest_shareholder(player_id, chain) {
            return "*".color(Rgb(192, 192, 192)).to_string();
        }
        // The star should probably be only displayed when a special terminal flag is set (mayber
        // --info or something like that)
        String::from(" ")
    }
}
