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
            hotel_chains::HotelChain,
            player::Player,
            ui, settings::Settings,
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
                if self.active_chains.contains_key(&hotel_chain) {
                    return Err(miette!("Unable to start new chain of hotel {}: The chain has already been founded!", &hotel_chain));
                }
                self.active_chains.insert(hotel_chain, positions.clone());
                // Update hotels on board
                for position in positions {
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
        }

        #[cfg(test)]
        mod tests {
            use miette::Result;

            use crate::{
                base_game::{board::Position, hotel_chains::HotelChain, ui},
                game::game::{round::Round, GameManager},
            };

            #[test]
            fn test_chain_status_and_length() -> Result<()> {
                let mut game_manager = GameManager::new(2, false).unwrap();
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
            fn test_fusion() -> Result<()> {
                let mut game_manager = GameManager::new(2, false).unwrap();
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

        use crate::{
            base_game::{board::Board, player::Player, ui},
            game::game::logic::place_hotel,
        };

        use super::{logic::draw_card, GameManager};

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
            //todo!("Implement function");
            draw_card(game_manager);
            Ok(false)
        }
    }

    /// Contains the most part of the game logic.
    /// Does not contain the logic of the different managers. Their logic is implemented in thair
    /// main impl block.
    /// This is also used to implement the required functions on existing structs.
    mod logic {
        use std::{cmp::Ordering, collections::HashMap, iter::Map, slice::Iter};

        use miette::{miette, Result};
        use owo_colors::{AnsiColors, OwoColorize};
        use read_input::{prelude::input, InputBuild};

        use crate::{
            base_game::{bank::{Bank, LargestShareholders}, board::Position, hotel_chains::HotelChain, player::Player},
            data_stream::read_enter,
        };

        use super::{hotel_chain_manager::HotelChainManager, GameManager};

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
            let player = game_manager
                .round
                .as_mut()
                .unwrap()
                .current_player_mut(&mut game_manager.players);
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

        /// Promts the user to press enter to draw a new card. The card is removed from the
        /// remaining cards and placed in the players inventory
        pub fn draw_card(game_manager: &mut GameManager) {
            print!("Press enter to draw a new card");
            read_enter();
            let card = game_manager.draw_card().unwrap();
            println!("Card drawn: {}", &card.to_string().color(AnsiColors::Green));
            let player = game_manager
                .round
                .as_mut()
                .unwrap()
                .current_player_mut(&mut game_manager.players);
            player.cards.push(card);
            print!("Press enter to finish your turn");
            read_enter();
        }

        /// Prompts the user to select a card.
        /// This card is then removed from the players inventory and returned
        fn read_card(player: &mut Player) -> Position {
            print!("Enter a number 1-6: ");
            let card_index = input::<usize>().inside(1..=6).get() - 1;
            let position = player.sorted_cards().get(card_index).unwrap().clone();
            //Remove the played card from the players hand cards
            player.remove_card(&position);
            position
        }

        /// Implements all logic
        impl <'b>Bank<'b> {
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
                if player.money <= stock_price {
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

            /// Gives one stock of the hotel chain to the player for free
            /// # Arguments
            /// * `players` - The list of players playing the game. Used to update largest
            /// shareholders.
            pub fn give_bonus_stock(
                &mut self,
                chain: &HotelChain,
                player: &mut Player,
            ) -> Result<()> {
                // Check if stocks are left
                if *self.stocks_for_sale.stocks.get(&chain).unwrap() == 0 {
                    return Err(miette!(
                        "Free stock could not be given to the player: No stocks available!"
                    ));
                }
                *self.stocks_for_sale.stocks.get_mut(&chain).unwrap() -= 1;
                // Give stock to player
                *player.owned_stocks.stocks.get_mut(&chain).unwrap() += 1;
                Ok(())
            }

            /// Updates who the largest and second largest shareholders are.
            /// For that the stocks of earch player are compared to one another.
            pub fn update_largest_shareholders(&mut self, players: &'b Vec<Player>) {
                // Clear currently largest shareholder vectors and initialize new
                self.largest_shareholders = LargestShareholders::new();
                for chain in HotelChain::iterator() {
                    let mut largest_shareholder: Vec<&'b Player> = Vec::new();
                    let mut second_largest_shareholder: Vec<&'b Player> = Vec::new();
                    for player in players {
                        // Check if player owns stocks for that chain
                        if *player.owned_stocks.stocks.get(&chain).unwrap() == 0 {
                            continue;
                        }
                        // Set first player to largest and second largest shareholder
                        if largest_shareholder.is_empty() && second_largest_shareholder.is_empty() {
                            largest_shareholder.push(player);
                            second_largest_shareholder.push(player);
                            continue;
                        }
                        // Determine currently largest shareholders
                        // On cases where continue has been called the second largest shareholder
                        // has been set already.
                        let largest_shareholder_stocks = *largest_shareholder
                            .get(0)
                            .unwrap()
                            .owned_stocks
                            .stocks
                            .get(&chain)
                            .unwrap();
                        let second_largest_shareholder_stocks = *second_largest_shareholder
                            .get(0)
                            .unwrap()
                            .owned_stocks
                            .stocks
                            .get(&chain)
                            .unwrap();
                        let current_player_stocks =
                            *player.owned_stocks.stocks.get(&chain).unwrap();
                        if largest_shareholder.len() == 1 {
                            // Currently only one player is largest shareholder
                            match current_player_stocks.cmp(&largest_shareholder_stocks) {
                                // The player will be set second largest shareholder if the
                                // currently second largest shareholder has less stocks then them
                                Ordering::Less => {
                                    println!("Hotel {}, Player {}: Less stocks than largest shareholder", chain.name(), player.id);
                                },
                                // Player has equal stocks => booth players will be set to largest
                                // and second largest shareholder
                                Ordering::Equal => {
                                    largest_shareholder.push(player);
                                    second_largest_shareholder.push(player);
                                    continue;
                                }
                                // Player has more stocks => The player will be set largest
                                // shareholder and the previously largest shareholder will become
                                // second largest shareholder
                                Ordering::Greater => {
                                    largest_shareholder.remove(0);
                                    largest_shareholder.push(player);
                                }
                            }
                        } else if largest_shareholder.len() > 1 {
                            // Currently more than one players are largest shareholders
                            match current_player_stocks.cmp(&second_largest_shareholder_stocks) {
                                Ordering::Less => (),
                                // Player has equal stocks => current player will be added to the largest
                                // and second largest shareholder vector
                                Ordering::Equal => {
                                    largest_shareholder.push(player);
                                    second_largest_shareholder.push(player);
                                    continue;
                                }
                                // The players that where stored as largest and second largest
                                // shareholder will now only be second largest shareholder. The
                                // current player will be set largest shareholder
                                Ordering::Greater => {
                                    largest_shareholder.clear();
                                    largest_shareholder.push(player);
                                    continue;
                                }
                            }
                        }
                        // Determine the currently second largest shareholder
                        match second_largest_shareholder_stocks.cmp(&current_player_stocks) {
                            Ordering::Less => (),
                            // Player has equal stocks => current player will be added to the second largest shareholder vector
                            Ordering::Equal => {
                                second_largest_shareholder.push(player);
                                continue;
                            }
                            // Player has more stocks => all currently second largest
                            // shareholders will be removed and replaced by the current player
                            Ordering::Greater => {
                                second_largest_shareholder.clear();
                                second_largest_shareholder.push(player);
                                continue;
                            }
                        }
                    }
                    // Insert largest shareholders for chain
                    self.largest_shareholders.largest_shareholder.insert(*chain, largest_shareholder);
                    self.largest_shareholders.second_largest_shareholder.insert(*chain, second_largest_shareholder);
                }
            }

            /// Checks if the player is one of the largest shareholders for the chain.
            pub fn is_largest_shareholder(&self, player: &Player, chain: &HotelChain) -> bool{
                self.largest_shareholders.largest_shareholder.get(chain).unwrap().contains(&player)
            }

            /// Checks if the player is one of the second largest shareholders for the chain.
            pub fn is_second_largest_shareholder(&self, player: &Player, chain: &HotelChain) -> bool {
                self.largest_shareholders.second_largest_shareholder.get(chain).unwrap().contains(&player)
            }
        }

        #[cfg(test)]
        mod tests {
            mod bank {

                use miette::Result;

                use crate::{base_game::hotel_chains::HotelChain, game::game::GameManager};

                #[test]
                fn test_buy_stock() {
                    let mut game = GameManager::new(2, false).unwrap();
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
                fn test_largest_shareholders() {
                    let mut game_manager = GameManager::new(4, false).unwrap();
                   
                    let mut index = 0;
                    while index < 4 {
                        let mut player = game_manager.players.get_mut(index).unwrap();
                        match index {
                            0 => {
                                player.owned_stocks.set_stocks(&HotelChain::Airport, 7);
                                player.owned_stocks.set_stocks(&HotelChain::Continental, 10);
                                player.owned_stocks.set_stocks(&HotelChain::Festival, 5);
                                player.owned_stocks.set_stocks(&HotelChain::Imperial, 7);
                            },
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
                    game_manager.bank.update_largest_shareholders(&game_manager.players);
                    game_manager.bank.print_largest_shareholders();
                    // Test case 1: one largest and one second largest shareholder (Continental)
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Airport));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Airport));
                    // Test case 2: multiple largest shareholerds (Airport)
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Continental));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Continental));
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Continental));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Continental));
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(2).unwrap(), &HotelChain::Continental));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(2).unwrap(), &HotelChain::Continental));
                    // Test case 3: one largest and multiple second largest shareholders (Prestige)
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Festival));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Festival));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(2).unwrap(), &HotelChain::Festival));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(3).unwrap(), &HotelChain::Festival));
                    // Test case 4: one player is largest and second largest shareholder (Luxor)
                    assert!(game_manager.bank.is_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Imperial));
                    assert!(game_manager.bank.is_second_largest_shareholder(game_manager.players.get(0).unwrap(), &HotelChain::Imperial));
                    assert!(!game_manager.bank.is_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Imperial));
                    assert!(!game_manager.bank.is_second_largest_shareholder(game_manager.players.get(1).unwrap(), &HotelChain::Imperial));
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
