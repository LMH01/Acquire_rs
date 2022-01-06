/// Contains all functionalities that are required to play the game.
pub mod game {
    use std::collections::HashMap;

    use miette::{miette, IntoDiagnostic, Result};
    use owo_colors::{AnsiColors, OwoColorize};
    use rand::Rng;

    use crate::{
        base_game::{
            bank::Bank,
            board::{Board, Letter, Piece, Position},
            hotel::Hotel,
            player::Player,
            ui,
        },
        data_stream::{self, read_enter},
        game::game::round::start_round,
    };

    use self::{hotel_manager::HotelManager, round::Round};

    //TODO Check what field are required to be public, make private if not used
    /// Contains all variables required to play a game.\
    /// This is the main interface to access game functions. Everything that happens in the game
    /// will run through this object.\
    /// A new game can be started this way:
    /// ```
    /// use game::game::GameManager;
    /// fn main() {
    ///     let number_of_players = 3;
    ///     let large_board = false;
    ///     let game_manager = GameManager::new(number_of_players, large_board);
    ///     game_manager.start_game();
    /// }
    /// ```
    pub struct GameManager {
        /// The board that belongs to this game
        pub board: Board,
        /// The bank that manages the stocks and the money
        pub bank: Bank,
        /// The hotel manager for this game
        pub hotel_manager: HotelManager,
        /// The positions that can be drawn
        pub position_cards: Vec<Position>,
        //TODO Add player_manager that stores the players, the number of players and information if
        //a player is currently the largest or second largest share holder
        /// A vector that contains all players that participate in the game
        pub players: Vec<Player>,
        /// The number of the currently running round
        pub round_number: u32,
        pub round: Option<Round>,
        number_of_players: u32,
        game_started: bool,
    }

    impl GameManager {
        /// Initializes a new game
        pub fn new(number_of_players: u32, large_board: bool) -> Result<Self> {
            // verify that the amout of players entered is between 2 and 6
            if number_of_players < 2 || number_of_players > 6 {
                return Err(miette!("Unable to create new game: The amount of players is invalid. Valid: 2-6, entered: {}", number_of_players));
            }

            let mut position_cards = GameManager::init_position_cards();
            let players = GameManager::init_players(number_of_players, &mut position_cards)?;
            Ok(Self {
                board: Board::new(large_board),
                position_cards,
                bank: Bank::new(),
                hotel_manager: HotelManager::new(),
                players,
                round_number: 0,
                round: None,
                number_of_players,
                game_started: false,
            })
        }

        /// Starts the game that has been created previously.
        /// Returns an Error when the game has already been started.
        pub fn start_game(&mut self) -> Result<()> {
            println!("Starting game!");
            if self.game_started {
                return Err(miette!(
                    "Unable to start game: Game has already been started!"
                ));
            } else {
                self.game_started = true;
            }
            println!("Each player draws a card now, the player with the lowest card starts.");
            let mut cards: Vec<Position> = Vec::new();
            for i in 1..=self.players.len() {
                let card = self.draw_card().unwrap();
                println!("Press enter to draw the card.");
                read_enter()?;
                println!("Drew card {}", &card.color(AnsiColors::Green));
                println!();
                cards.push(card);
            }
            println!("Press enter to place these hotels and start the first round!");
            read_enter()?;
            for card in cards {
                self.board.place_hotel(&card)?;
            }
            self.start_rounds()?;
            Ok(())
        }

        /// Starts game rounds.
        /// If one round returns true no new round is started.
        fn start_rounds(&mut self) -> Result<()> {
            let mut game_running = true;
            while game_running {
                let round = Round::new();
                self.round = Some(round);
                let round_status = start_round(self)?;
                if round_status {
                    game_running = false;
                }
            }
            //TODO Add final account (=Endabrechnung)
            Ok(())
        }

        /// Initializes all position cards and puts them in the vector
        fn init_position_cards() -> Vec<Position> {
            let mut cards: Vec<Position> = Vec::new();
            for c in Letter::iterator() {
                for i in 1..=12 {
                    cards.push(Position::new(*c, i));
                }
            }
            cards
        }

        /// Initializes all players and puts them in the vector
        fn init_players(
            number_of_players: u32,
            position_cards: &mut Vec<Position>,
        ) -> Result<Vec<Player>> {
            let mut players: Vec<Player> = Vec::new();
            // Contains the position cards for each player.
            let mut player_cards: Vec<Vec<Position>> = Vec::new();
            // Put an empty vector in player_cards for each player
            for _i in 1..=number_of_players {
                player_cards.push(Vec::new());
            }
            // Get the starting cards for the player
            for _i in 1..=6 {
                for player in 0..=number_of_players - 1 {
                    let random_number = rand::thread_rng().gen_range(0..=position_cards.len() - 1);
                    if let Some(position) = position_cards.get(random_number) {
                        player_cards
                            .get_mut(usize::try_from(player).unwrap())
                            .unwrap()
                            .push(*position);
                        position_cards.remove(usize::try_from(random_number).unwrap());
                    } else {
                        println!("position_cards length: {}", position_cards.len());
                        return Err(miette!("Unable to add position to list. The index {} does not exist in the position_cards vector!", random_number));
                    }
                }
            }
            // Initialize new players and put them in the list
            while !player_cards.is_empty() {
                players.push(Player::new(player_cards.pop().unwrap()))
            }
            Ok(players)
        }

        pub fn print_player_cards(&self, player: usize) {
            self.players.get(player).unwrap().print_cards();
        }

        /// Draw a card from the ['game::Game#position_cards']position_cards deck
        fn draw_card(&mut self) -> Result<Position> {
            let random_number = rand::thread_rng().gen_range(0..=self.position_cards.len() - 1);
            if let None = self.position_cards.get(random_number) {
                println!("position_cards length: {}", self.position_cards.len());
                return Err(miette!("Unable to add position to list. The index {} does not exist in the position_cards vector!", random_number));
            }
            let position = self.position_cards.get(random_number).unwrap().clone();
            self.position_cards.remove(random_number);
            Ok(position)
        }
    }

    /// Manages the currently active hotel chains
    pub mod hotel_manager {
        use std::collections::HashMap;

        use miette::{miette, Result};

        use crate::base_game::hotel::Hotel;

        /// Store the currently active hotel chains
        pub struct HotelManager {
            /// Stores if the hotel is currently active on the board
            hotel_status: HashMap<Hotel, bool>,
            /// Stores the number of currently build hotels that belong to the chain
            hotel_buildings: HashMap<Hotel, u32>,
        }

        impl HotelManager {
            /// Create a new hotel manager that is used to manage the currently active hotel chains
            pub fn new() -> Self {
                let mut hotel_active: HashMap<Hotel, bool> = HashMap::new();
                let mut hotel_buildings: HashMap<Hotel, u32> = HashMap::new();
                // Fill the hash maps
                for hotel in Hotel::iterator() {
                    hotel_active.insert(*hotel, false);
                    hotel_buildings.insert(*hotel, 0);
                }
                Self {
                    hotel_status: hotel_active,
                    hotel_buildings,
                }
            }

            /// Returns the number of hotels currently built for the specified chain
            pub fn number_of_hotels(&self, hotel: &Hotel) -> u32 {
                *self.hotel_buildings.get(hotel).unwrap()
            }

            /// Returns true if the hotel is currently active
            pub fn hotel_status(&self, hotel: &Hotel) -> bool {
                *self.hotel_status.get(hotel).unwrap()
            }

            /// Returns the range in which the current number of hotels is
            pub fn hotel_range(&self, hotel: &Hotel) -> String {
                let hotels = self.hotel_buildings.get(hotel).unwrap();
                let range = match hotels {
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
                let string = format!("{}", range);
                string
            }

            //TODO Decide if i want to increase the amount of hotel buildings to 2 when this
            //function is called. Also decide if i would like to reset the hotel buildings to 0
            //when hotel is disabled
            /// Changes that status of the hotel.
            /// This decides if stocks can be bought for this hotel.
            /// # Arguments
            /// * `active` - If true the hotel is set active, if false the hotel is set inactive
            pub fn set_hotel_status(&mut self, hotel: &Hotel, active: bool) {
                *self.hotel_status.get_mut(&hotel).unwrap() = active;
            }

            /// Adds the specified amount of hotel buildings to the chain.
            /// # Returns
            /// * `Ok` when value was changed
            /// * `Err` when hotel is disabled
            pub fn add_hotel_buildings(&mut self, hotel: &Hotel, amount: u32) -> Result<()> {
                if *self.hotel_status.get(hotel).unwrap() {
                    *self.hotel_buildings.get_mut(&hotel).unwrap() += amount;
                    Ok(())
                } else {
                    Err(miette!(
                        "Unable to add hotel buildings: The hotel is not active"
                    ))
                }
            }
        }
    }

    /// Manages a single round. A round consists of each player doing a move.
    mod round {
        use std::slice::SliceIndex;

        use miette::{miette, Result};

        use crate::{base_game::{board::Board, player::Player, ui}, game::game::logic::place_hotel};

        use super::GameManager;

        pub struct Round {
            /// The index of the current player
            pub current_player_index: usize,
            pub started: bool,
        }

        impl Round {
            /// Creates a new round
            pub fn new() -> Self {
                Self {
                    current_player_index: 0,
                    started: false,
                }
            }

            /// Returns the current player
            pub fn current_player<'a>(&self, game_manager: &'a GameManager) -> &'a Player {
                game_manager.players.get(self.current_player_index).unwrap()
            }
        }

        /// Starts a new round consisting of each player doing a single turn.
        /// Does not automatically start a new round when the game is not over yet.
        /// When the game finishes in this round `true` is returned.
        /// The final account is not calculated in this function.
        pub fn start_round(game_manager: &mut GameManager) -> Result<bool> {
            if game_manager.round.as_ref().unwrap().started {
                return Err(miette!("Round was already started!"));
            }
            game_manager.round.as_mut().unwrap().started = true;
            game_manager.round_number += 1;
            // Make a surn for each player
            for i in 0..=game_manager.players.len()-1 {
                game_manager.round.as_mut().unwrap().current_player_index = i; 
                let status = player_turn(game_manager)?;
                if status {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        /// Plays a single player turn
        /// When this player finishes the game this round `true` is returned
        fn player_turn(game_manager: &mut GameManager) -> Result<bool> {
            ui::print_main_ui(game_manager);
            place_hotel(game_manager);
            //TODO Implemnt function
            //1. Place piece
            //2. Check if win condition is meet
            //      If yes ask give user the option to end the game here
            //3. Buy stocks
            //4. Draw new card
            //todo!("Implement function");
            Ok(false)
        }
    }

    /// Contains the most part of the game logic.
    /// Does not contain the logic of the different managers. Their logic is implemented in thair
    /// main impl block.
    /// This is also used to implement the required functions on existing structs.
    mod logic {
        use std::{slice::Iter, io::{Read, self}};

        use miette::{miette, Result};
        use read_input::{prelude::input, InputBuild};

        use crate::base_game::{bank::Bank, hotel::Hotel, player::Player, board::Position};

        use super::{hotel_manager::HotelManager, GameManager};

        /// The different ways the game can end.
        enum EndCondition {
            /// The game can be finished when all chains on the board have at least 10 hotels and
            /// when there is no space to found a new chain
            AllChainsMoreThan10HotelsAndNoSpaceForNewChain,
            /// The game can be finished when at least one chain has more than 41 hotels
            OneChainMoreThan41Hotels,
        }

        impl EndCondition {
            fn is_condition_meet(&self, game_manager: &GameManager) -> bool {
                match self {
                    Self::AllChainsMoreThan10HotelsAndNoSpaceForNewChain => {
                        todo!();
                    }
                    Self::OneChainMoreThan41Hotels => {
                        todo!();
                    }
                }
            }

            fn iterator() -> Iter<'static, EndCondition> {
                const END_CONDITION: [EndCondition; 2] = [
                    EndCondition::AllChainsMoreThan10HotelsAndNoSpaceForNewChain,
                    EndCondition::OneChainMoreThan41Hotels,
                ];
                END_CONDITION.iter()
            }
        }

        /// Checks if the game state meets at least one condition because of which the game can be
        /// finished.
        /// # Returns
        /// * `true` - When the game meets at leaste one end condition
        pub fn check_end_condition(game_manager: &GameManager) -> bool {
            for end_condition in EndCondition::iterator() {
                if end_condition.is_condition_meet(game_manager) {
                    return true;
                }
            }
            false
        }
        
        /// Place a hotel on the board.
        /// This function will abide by the game rules.
        /// The player is asked what card to play.
        pub fn place_hotel(game_manager: &mut GameManager) -> Result<()> {
            println!("Please choose what hotel card you would like to play.");
            //TODO Add function that checkes what cards can be played
            let player = game_manager.round.as_ref().unwrap().current_player(&game_manager);
            let card = read_card(player);
            game_manager.board.place_hotel(&card)?;
            //TODO Add logic for the following cases:
            //1. The board piece founds a new hotel chain
            //2. The board piece extends a existing chain
            //  2.1 The board piece extends a existing chain by more than 1 piece
            //3. The board piece creates a fusion between chains
            //  3.1 Add Logic that can handle fusions between two chains
            //  3.2 Add Logic that can handle fusions between two ore more chains
            Ok(())
        }

        /// Prompts the user to select a card
        fn read_card(player: &Player) -> Position {
            print!("Enter a number 1-6:");
            let number = input::<usize>().inside(1..=6).get();
            let position = player.sorted_cards().get(number-1).unwrap().clone();
            println!("Entered {}, card {}", &number, player.sorted_cards().get(number-1).unwrap());
            position
        }

        /// Implements all logic
        impl Bank {
            /// Buy a single stock from the bank.
            /// # Arguments
            /// * `hotel_manager` - The hotel manager for the current match
            /// * `hotel` - The hotel for which the player buys a stock
            /// * `player` - The player that buys the stock
            /// # Returns
            /// * `Ok(())` - When stock was successfully bought
            /// * `Err` - When something went wrong while buying the stock
            pub fn buy_stock(
                &mut self,
                hotel_manager: &HotelManager,
                hotel: &Hotel,
                player: &mut Player,
            ) -> Result<()> {
                // The currently available stocks for the given hotel that can still be bought
                let stock_available = self.hotel_stocks_available(hotel);
                // Check if the stock can be bought (= Is the hotel chain active)
                if !hotel_manager.hotel_status(hotel) {
                    return Err(miette!(
                        "Unable to buy stock from hotel {}: Hotel is not active.",
                        &hotel
                    ));
                }
                // Check if the desired stock can still be bought
                if *stock_available == 0 {
                    return Err(miette!(
                        "Unable to buy stock from hotel {}: No stocks left.",
                        &hotel
                    ));
                }
                let stock_price = Bank::stock_price(&hotel_manager, &hotel);
                // Check if player has enough money to buy the stock
                if player.money <= stock_price {
                    return Err(miette!(
                        "Unable to buy stock from hotel {}: Not enough money.",
                        &hotel
                    ));
                }
                // Finally buy the stock
                self.stocks_for_sale.decrease_stocks(hotel, 1);
                player.add_stocks(hotel, 1);
                player.remove_money(stock_price);
                Ok(())
            }
        }

        #[cfg(test)]
        mod tests {
            mod bank {

                use miette::Result;

                use crate::{base_game::hotel::Hotel, game::game::GameManager};

                #[test]
                fn test_buy_stock() {
                    let mut game = GameManager::new(2, false).unwrap();
                    // Test if Hotel is not active error works
                    let mut input = game.bank.buy_stock(
                        &game.hotel_manager,
                        &Hotel::Airport,
                        game.players.get_mut(0).unwrap(),
                    );
                    assert!(is_error(input));
                    // Test if no stocks left error works
                    game.bank.stocks_for_sale.set_stocks(&Hotel::Airport, 0);
                    game.hotel_manager.set_hotel_status(&Hotel::Airport, true);
                    input = game.bank.buy_stock(
                        &game.hotel_manager,
                        &Hotel::Airport,
                        game.players.get_mut(0).unwrap(),
                    );
                    assert!(is_error(input));
                    // Test if not enough money error works
                    game.bank.stocks_for_sale.set_stocks(&Hotel::Airport, 5);
                    game.players.get_mut(0).unwrap().money = 0;
                    input = game.bank.buy_stock(
                        &game.hotel_manager,
                        &Hotel::Airport,
                        game.players.get_mut(0).unwrap(),
                    );
                    assert!(is_error(input));
                }

                fn is_error(input: Result<()>) -> bool {
                    return match input {
                        Err(_) => true,
                        Ok(_) => false,
                    };
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::game::game::GameManager;

        #[test]
        fn test_draw_card() {
            let mut game = GameManager::new(2, false).unwrap();
            game.draw_card().unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 94);
            game = GameManager::new(6, false).unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 71);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::game::GameManager;

    #[test]
    fn test_position_card_amount() {
        let mut index = 0;
        while index <= 1000 {
            let game = GameManager::new(2, false).unwrap();
            assert_eq!(game.position_cards.len(), 96);
            index += 1;
        }
    }
}
