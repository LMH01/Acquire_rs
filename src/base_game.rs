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

    use super::hotel::Hotel;

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
        pub chain: Option<Hotel>,
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
                            .color(Hotel::color(&self.chain.as_ref().unwrap()))
                            .to_string()
                    } else {
                        format!(" {} ", self.chain.as_ref().unwrap().identifier())
                            .color(Hotel::color(&self.chain.as_ref().unwrap()))
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

/// Contains all functionalities related to the hotel buildings. Like name, information about stock
/// values and more.
pub mod hotel {
    use std::{
        fmt::{self, Display, Formatter},
        slice::Iter,
    };

    use owo_colors::Rgb;

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
        pub fn color(&self) -> Rgb {
            match *self {
                Hotel::Airport => Rgb(107, 141, 165),
                Hotel::Continental => Rgb(32, 64, 136),
                Hotel::Festival => Rgb(12, 106, 88),
                Hotel::Imperial => Rgb(198, 83, 80),
                Hotel::Luxor => Rgb(231, 219, 0),
                Hotel::Oriental => Rgb(184, 96, 20),
                Hotel::Prestige => Rgb(99, 47, 107),
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
        pub fn stock_value(&self, number_of_hotels: u32) -> u32 {
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

        /// Returns the name of the hotel
        pub fn name(&self) -> &str {
            match *self {
                Hotel::Airport => "Airport",
                Hotel::Continental => "Continental",
                Hotel::Festival => "Festival",
                Hotel::Imperial => "Imperial",
                Hotel::Luxor => "Luxor",
                Hotel::Oriental => "Oriental",
                Hotel::Prestige => "Prestige",
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
        pub stocks: HashMap<Hotel, u32>,
    }

    impl Stocks {
        /// Initializes a new stock struct. Member variables are set to 0
        pub fn new() -> Self {
            let mut stocks: HashMap<Hotel, u32> = HashMap::new();
            for hotel in Hotel::iterator() {
                stocks.insert(*hotel, 0);
            }
            Self { stocks }
        }

        /// Initializes a new stock struct. Member variables are set to 25. This is used so that
        /// the bank works get all available stocks at the start.
        pub fn new_bank() -> Self {
            let mut stocks: HashMap<Hotel, u32> = HashMap::new();
            for hotel in Hotel::iterator() {
                stocks.insert(*hotel, 25);
            }
            Self { stocks }
        }

        /// Returns the amout of stocks available for the hotel
        pub fn stocks_for_hotel(&self, hotel: &Hotel) -> &u32 {
            self.stocks.get(hotel).unwrap()
        }

        /// Set the stocks of the hotel to the amount.
        /// # Arguments
        /// * `hotel` - The hotel for which the stock value should be changed
        /// * `value` - The value to which the stock amount should be set
        pub fn set_stocks(&mut self, hotel: &Hotel, value: u32) {
            *self.stocks.get_mut(hotel).unwrap() = value;
        }

        /// Increases stocks for the `hotel` by `value`
        pub fn increase_stocks(&mut self, hotel: &Hotel, value: u32) {
            *self.stocks.get_mut(hotel).unwrap() += value;
        }

        /// Decreases stocks for the `hotel` by `value`
        pub fn decrease_stocks(&mut self, hotel: &Hotel, value: u32) {
            *self.stocks.get_mut(hotel).unwrap() -= value;
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
    use crate::{base_game::stock::Stocks, game::game::hotel_manager::HotelManager};

    use super::hotel::Hotel;

    pub struct Bank {
        pub stocks_for_sale: Stocks,
    }

    impl Bank {
        /// Creates a new bank
        pub fn new() -> Self {
            Self {
                stocks_for_sale: Stocks::new_bank(),
            }
        }

        /// Returns how many stocks of the given hotel are still available to be bought
        pub fn hotel_stocks_available(&self, hotel: &Hotel) -> &u32 {
            self.stocks_for_sale.stocks_for_hotel(hotel)
        }

        /// Returns the current price for a stock of the given hotel
        pub fn stock_price(hotel_manager: &HotelManager, hotel: &Hotel) -> u32 {
            hotel.stock_value(hotel_manager.number_of_hotels(&hotel))
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            base_game::{bank::Bank, hotel::Hotel},
            game::game::GameManager,
        };

        #[test]
        fn test_stock_price() {
            let mut game_manager = GameManager::new(2, false).unwrap();
            game_manager
                .hotel_manager
                .set_hotel_status(&Hotel::Airport, true);
            game_manager
                .hotel_manager
                .set_hotel_status(&Hotel::Imperial, true);
            game_manager
                .hotel_manager
                .set_hotel_status(&Hotel::Continental, true);
            game_manager
                .hotel_manager
                .add_hotel_buildings(&Hotel::Airport, 20)
                .unwrap();
            game_manager
                .hotel_manager
                .add_hotel_buildings(&Hotel::Imperial, 15)
                .unwrap();
            game_manager
                .hotel_manager
                .add_hotel_buildings(&Hotel::Continental, 41)
                .unwrap();
            println!(
                "Number of hotels: {}",
                game_manager.hotel_manager.number_of_hotels(&Hotel::Airport)
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_manager, &Hotel::Airport),
                700
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_manager, &Hotel::Imperial),
                800
            );
            assert_eq!(
                Bank::stock_price(&game_manager.hotel_manager, &Hotel::Continental),
                1200
            );
        }
    }
}

/// Player management
pub mod player {
    use crate::{
        base_game::board::Position,
        base_game::{hotel::Hotel, stock::Stocks},
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
        pub bonuses: Bonuses,
    }

    impl Player {
        pub fn new(start_cards: Vec<Position>) -> Self {
            Self {
                money: 6000,
                owned_stocks: Stocks::new(),
                cards: start_cards,
                bonuses: Bonuses::new(),
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
        pub fn add_stocks(&mut self, hotel: &Hotel, amount: u32) {
            self.owned_stocks.increase_stocks(hotel, amount);
        }

        /// Remove stocks that the player owns
        pub fn remove_stocks(&mut self, hotel: &Hotel, amount: u32) {
            self.owned_stocks.decrease_stocks(hotel, amount);
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
            for hotel in Hotel::iterator() {
                if first_card {
                    first_card = false;
                } else {
                    print!(", ")
                }
                print!(
                    "{}: {}",
                    hotel.name().color(hotel.color()),
                    self.owned_stocks.stocks_for_hotel(hotel)
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

    /// Used to store if the player is a largest or second largest shareholder
    pub struct Bonuses {
        /// Hotels where the player is the larges shareholder
        largest_shareholder: Vec<Hotel>,
        /// Hotels where the player is the second largest shareholder
        second_largest_shareholder: Vec<Hotel>,
    }

    impl Bonuses {
        pub fn new() -> Self {
            Self {
                largest_shareholder: Vec::new(),
                second_largest_shareholder: Vec::new(),
            }
        }
    }
}

/// User interface drawing
pub mod ui {
    use crate::{
        base_game::{bank::Bank, hotel::Hotel},
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
            .current_player(&game_manager)
            .print_player_ui();
        //TODO Implement largest shareholder display
        //Maybe add commandline flag with which it can be enabled to show who is the largest
        //stareholder currently
        println!();
        println!("{}", String::from("Game stats:").bright_green());
        println!("{:15}||      Hotels       ||        Stocks          ||      Bonuses for the majority shareholders", "");
        println!("{:15}|| Number ||  Range  || Bank || Owned || Value || Largest shareholder || Second largest shareholder", "");
        println!("==================================================================================================================");
        for hotel in Hotel::iterator() {
            // Set the color of the values
            let enable_color = game_manager.hotel_manager.hotel_status(hotel);
            let color = match enable_color {
                true => DynColors::Ansi(AnsiColors::White),
                false => DynColors::Rgb(105, 105, 105),
            };
            let hotel_color = match enable_color {
                true => hotel.color(),
                false => Rgb(105, 105, 105),
            };
            let formatted_string = format!(
                "||   {:2}   || {:7} ||  {:2}  ||   {:2}{} || {:4}$ ||    TO IMPLEMENT     || TO IMPLEMENT",
                game_manager.hotel_manager.number_of_hotels(hotel),
                game_manager.hotel_manager.hotel_range(hotel),
                game_manager.bank.hotel_stocks_available(&hotel),
                game_manager
                    .round
                    .as_ref()
                    .unwrap()
                    .current_player(&game_manager)
                    .owned_stocks
                    .stocks_for_hotel(hotel),
                " ",
                Bank::stock_price(&game_manager.hotel_manager, &hotel),
                );
            println!(
                "{:15}{}",
                hotel.name().color(hotel_color),
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
