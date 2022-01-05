/// Contains all functionalities that are required to play the game.
pub mod game {
    use std::collections::HashMap;

    use miette::{miette, IntoDiagnostic, Result};
    use rand::Rng;

    use crate::base_game::{
        bank::Bank,
        board::{Board, Letter, Piece, Position},
        hotel::Hotel, player::Player,
    };

    use self::hotel_manager::HotelManager;

    //TODO Rename to GameManager
    //TODO Check what field are required to be public, make private if not used
    /// Contains all variables required to play a game
    pub struct Game {
        /// The board that belongs to this game
        pub board: Board,
        /// The bank that manages the stocks and the money
        pub bank: Bank,
        /// The hotel manager for this game
        pub hotel_manager: HotelManager,
        /// The positions that can be drawn
        pub position_cards: Vec<Position>,
        /// A vector that contains all players that participate in the game
        pub players: Vec<Player>,
        number_of_players: u32,
        game_started: bool,
    }

    impl Game {
        /// Initializes a new game
        pub fn new(number_of_players: u32, large_board: bool) -> Result<Self> {
            // verify that the amout of players entered is between 2 and 6
            if number_of_players < 2 || number_of_players > 6 {
                return Err(miette!("Unable to create new game: The amount of players is invalid. Valid: 2-6, entered: {}", number_of_players));
            }

            let mut position_cards = Game::init_position_cards();
            let players = Game::init_players(number_of_players, &mut position_cards)?;
            Ok(Self {
                board: Board::new(large_board),
                position_cards,
                bank: Bank::new(),
                hotel_manager: HotelManager::new(),
                players,
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
            //TODO Continue to work here
            // Write function that prints current game information to console:
            //  Current player: Money, stocks they have, cards they have
            //  Game stats: How many hotels a chain has, how many stocks are left, how much a stock
            //  is worth
            //
            //  Maybe print the same info card that exists in the real game where the current
            //  amount of hotels is highlighted
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
        }
    }

    /// Manages a single round. A round consists of each player doing a move.
    mod round {}

    #[cfg(test)]
    mod game_tests {
        use crate::game::game::Game;

        #[test]
        fn test_draw_card() {
            let mut game = Game::new(2, false).unwrap();
            game.draw_card().unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 94);
            game = Game::new(6, false).unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 71);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::game::Game;

    #[test]
    fn test_position_card_amount() {
        let mut index = 0;
        while index <= 1000 {
            let game = Game::new(2, false).unwrap();
            assert_eq!(game.position_cards.len(), 96);
            index += 1;
        }
    }
}
