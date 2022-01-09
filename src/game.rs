/// Contains all functionalities that are required to play the game.
pub mod game {
    use std::collections::HashMap;

    use miette::{miette, IntoDiagnostic, Result};
    use owo_colors::{AnsiColors, OwoColorize};
    use rand::Rng;

    use crate::{
        base_game::{
            bank::Bank,
            board::{letter::LETTERS, Board, Piece, Position},
            hotel_chains::HotelChain,
            player::Player,
            settings::Settings,
            ui,
        },
        data_stream::{self, read_enter},
        game::game::round::start_round,
    };

    use self::{hotel_chain_manager::HotelChainManager, round::Round};

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
    pub struct GameManager<'a> {
        /// The board that belongs to this game
        pub board: Board,
        /// The bank that manages the stocks and the money
        pub bank: Bank<'a>,
        /// The hotel manager for this game
        pub hotel_chain_manager: HotelChainManager,
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
        /// Stores the settings
        pub settings: Settings,
    }

    impl GameManager<'_> {
        /// Initializes a new game
        pub fn new(number_of_players: u32, settings: Settings) -> Result<Self> {
            // verify that the amout of players entered is between 2 and 6
            if number_of_players < 2 || number_of_players > 6 {
                return Err(miette!("Unable to create new game: The amount of players is invalid. Valid: 2-6, entered: {}", number_of_players));
            }

            let mut position_cards = GameManager::init_position_cards();
            let players = GameManager::init_players(number_of_players, &mut position_cards)?;
            Ok(Self {
                board: Board::new(),
                position_cards,
                bank: Bank::new(),
                hotel_chain_manager: HotelChainManager::new(),
                players,
                round_number: 0,
                round: None,
                number_of_players,
                game_started: false,
                settings,
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
                read_enter();
                println!("Drew card {}", &card.color(AnsiColors::Green));
                println!();
                cards.push(card);
            }
            println!("Press enter to place these hotels and start the first round!");
            read_enter();
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
            for c in LETTERS {
                for i in 1..=12 {
                    cards.push(Position::new(c, i));
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
            let mut player_id = 0;
            while !player_cards.is_empty() {
                players.push(Player::new(player_cards.pop().unwrap(), player_id));
                player_id += 1;
            }
            Ok(players)
        }

        pub fn print_player_cards(&self, player: usize) {
            self.players.get(player).unwrap().print_cards();
        }

        /// Draw a card from the ['game::Game#position_cards']position_cards deck
        pub fn draw_card(&mut self) -> Result<Position> {
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
    pub mod hotel_chain_manager {
        use std::collections::HashMap;

        use miette::{miette, Result};

        use crate::base_game::{
            bank::Bank,
            board::{Board, Position},
            hotel_chains::HotelChain,
            player::Player,
        };

        /// Store the currently active hotel chains
        pub struct HotelChainManager {
            /// Stores the active hotel chains and the buildings that belong to the chain
            active_chains: HashMap<HotelChain, Vec<Position>>,
        }

        impl HotelChainManager {
            /// Create a new hotel manager that is used to manage the currently active hotel chains
            pub fn new() -> Self {
                Self {
                    active_chains: HashMap::new(),
                }
            }

            /// Returns the number of hotels currently built for the specified chain.
            /// If the chain is not active 0 is returned
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

            /// Returns true if the chain is currently active
            pub fn chain_status(&self, hotel: &HotelChain) -> bool {
                self.active_chains.contains_key(hotel)
            }

            /// Returns the range in which the current price level of the chain is
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
                let string = format!("{}", range);
                string
            }

            /// Start a new chain.
            /// The hotels on the board are updated to show the chain.
            /// The player will be given one stock as start-up bonus.
            /// When the hotel pieces have not been set on the board they will be placed but a
            /// warning will be shown.
            /// # Arguments
            /// * `hotel` - The hotel type that is founded
            /// * `positions` - The initial positions of the hotels that belong to this chain
            /// * `board` - The board on which the hotels should be updated
            /// * `player` - The player that is the founder of the new chain
            /// * `bank` - The bank that manages the available stocks
            ///
            /// # Returns
            /// A result containing 'Ok()' when the chain has been founded successfully
            pub fn start_chain(
                &mut self,
                hotel_chain: HotelChain,
                positions: Vec<Position>,
                board: &mut Board,
                player: &mut Player,
                bank: &mut Bank,
            ) -> Result<()> {
                //TODO Add check if each position is next to each other. If not return error.
                //      Add check if no other chains are next to this chain.
                //      Maybe these checks are overkill and not worth to implement as these
                //      conditions are normally checked before this function is called.
                if self.active_chains.contains_key(&hotel_chain) {
                    return Err(miette!("Unable to start new chain of hotel {}: The chain has already been founded!", &hotel_chain));
                }
                self.active_chains.insert(hotel_chain, positions.clone());
                // Update hotels on board
                for position in positions {
                    if board.is_hotel_placed(&position).is_none() {
                        board.place_hotel(&position)?;
                        eprintln!("Warning: Hotel at {} was not placed but has been placed to start the chain {}. Please place the hotel before the chain is stared!", &position, &hotel_chain);
                    }
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
            /// * `Ok()` - When the hotel was successfully added
            /// * `Err(Error)` - Whe the hotel chain does not exist
            pub fn add_hotel_to_chain(
                &mut self,
                hotel_chain: &HotelChain,
                position: Position,
                mut board: Board,
            ) -> Result<()> {
                if !self.active_chains.contains_key(&hotel_chain) {
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
            /// # Arguments
            /// * `alive` - The hotel chain that survives the fusion
            /// * `dead` - The hotel chain that dies
            /// * `board` - The board where the pieces should be updated
            ///
            /// # Returns
            /// * `Ok()` - When the hotels where merged successfully
            /// * `Err(Error)` - Whe the merge was not successfull
            pub fn fuse_chains(
                &mut self,
                alive: &HotelChain,
                dead: &HotelChain,
                board: &mut Board,
            ) -> Result<()> {
                // Check if the two chains exist
                if !(self.active_chains.contains_key(&alive)
                    && self.active_chains.contains_key(&dead))
                {
                    return Err(miette!("Unable to fuse chain {} into {}: At least one of the two chains does not exist!", &dead, &alive));
                }
                // Transfer positions and uptate board
                for position in self.active_chains.get(&dead).unwrap().clone() {
                    self.active_chains.get_mut(&alive).unwrap().push(position);
                    board.update_hotel(*alive, &position)?;
                }
                // Remove old chain
                self.active_chains.remove(&dead);
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
                self.chain_length(&chain) >= 11
            }
        }

        #[cfg(test)]
        mod tests {
            use miette::Result;

            use crate::{
                base_game::{board::Position, hotel_chains::HotelChain, settings::Settings, ui},
                game::game::{round::Round, GameManager},
            };

            #[test]
            fn chain_status_and_length_correct() -> Result<()> {
                let mut game_manager = GameManager::new(2, Settings::new_default()).unwrap();
                game_manager.round = Some(Round::new());
                for hotel_chain in HotelChain::iterator() {
                    setup_hotel(&mut game_manager, hotel_chain)?;
                    assert_eq!(
                        game_manager.hotel_chain_manager.chain_status(hotel_chain),
                        true
                    );
                    assert_eq!(
                        game_manager.hotel_chain_manager.chain_length(hotel_chain),
                        13
                    );
                }
                Ok(())
            }

            #[test]
            fn fusion_correct() -> Result<()> {
                let mut game_manager = GameManager::new(2, Settings::new_default()).unwrap();
                game_manager.round = Some(Round::new());
                let hotel_chain_1 = &HotelChain::Airport;
                let hotel_chain_2 = &HotelChain::Continental;
                setup_hotel(&mut game_manager, hotel_chain_1)?;
                setup_hotel(&mut game_manager, hotel_chain_2)?;
                ui::print_main_ui(&game_manager);
                game_manager.hotel_chain_manager.fuse_chains(
                    &HotelChain::Continental,
                    &HotelChain::Airport,
                    &mut game_manager.board,
                )?;
                assert_eq!(
                    game_manager.hotel_chain_manager.chain_status(hotel_chain_1),
                    false
                );
                assert_eq!(
                    game_manager.hotel_chain_manager.chain_status(hotel_chain_2),
                    true
                );
                assert_eq!(
                    game_manager.hotel_chain_manager.chain_length(hotel_chain_1),
                    0
                );
                assert_eq!(
                    game_manager.hotel_chain_manager.chain_length(hotel_chain_2),
                    26
                );
                Ok(())
            }

            fn setup_hotel(game_manager: &mut GameManager, hotel_chain: &HotelChain) -> Result<()> {
                let mut cards: Vec<Position> = Vec::new();
                for _i in 1..=13 {
                    cards.push(game_manager.draw_card().unwrap());
                }
                for card in &cards {
                    game_manager.board.place_hotel(&card)?;
                }
                game_manager.hotel_chain_manager.start_chain(
                    *hotel_chain,
                    cards,
                    &mut game_manager.board,
                    game_manager
                        .round
                        .as_ref()
                        .unwrap()
                        .current_player_mut(&mut game_manager.players),
                    &mut game_manager.bank,
                )?;
                Ok(())
            }
        }
    }

    /// Manages a single round. A round consists of each player doing a move.
    pub mod round {
        use std::slice::SliceIndex;

        use miette::{miette, Result};

        use crate::base_game::{board::Board, player::Player, logic::{self, place_hotel::place_hotel, draw_card}, ui};

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
            pub fn current_player<'a>(&self, players: &'a Vec<Player>) -> &'a Player {
                players.get(self.current_player_index).unwrap()
            }

            /// Returns the current player as mutuable
            pub fn current_player_mut<'a>(&self, players: &'a mut Vec<Player>) -> &'a mut Player {
                players.get_mut(self.current_player_index).unwrap()
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
            for i in 0..=game_manager.players.len() - 1 {
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
            place_hotel(game_manager)?;
            //TODO board should be updated when the hotel has been placed
            //TODO Implemnt function
            //1. Place piece
            //2. Check if win condition is meet
            //      If yes ask give user the option to end the game here
            //3. Buy stocks
            //4. Draw new card
            draw_card(game_manager);
            Ok(false)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{base_game::settings::Settings, game::game::GameManager};

        #[test]
        fn draw_card_works() {
            let mut game = GameManager::new(2, Settings::new(false, false, false)).unwrap();
            game.draw_card().unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 94);
            game = GameManager::new(6, Settings::new(false, false, false)).unwrap();
            game.draw_card().unwrap();
            assert_eq!(game.position_cards.len(), 71);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{base_game::settings::Settings, game::game::GameManager};

    #[test]
    fn position_card_amount_correctly_initialized() {
        let mut index = 0;
        while index <= 1000 {
            let game = GameManager::new(2, Settings::new_default()).unwrap();
            assert_eq!(game.position_cards.len(), 96);
            index += 1;
        }
    }
}
