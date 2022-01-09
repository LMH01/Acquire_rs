use std::{cmp::Ordering, collections::HashMap, iter::Map, slice::Iter};

use miette::{miette, Result};
use owo_colors::{AnsiColors, OwoColorize};
use read_input::{prelude::input, InputBuild};

use crate::{
    base_game::{
        bank::{Bank, LargestShareholders},
        board::{AnalyzedPosition, Position},
        hotel_chains::HotelChain,
        player::Player,
    },
    data_stream::read_enter,
    game::game::{hotel_chain_manager::HotelChainManager, GameManager},
};

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

/// Promts the user to press enter to draw a new card. The card is removed from the
/// remaining cards and placed in the players inventory
pub fn draw_card(game_manager: &mut GameManager) {
    if !game_manager.settings.skip_dialogues {
        print!("Press enter to draw a new card");
        read_enter();
    }
    let card = game_manager.draw_card().unwrap();
    if !game_manager.settings.skip_dialogues {
        println!("Card drawn: {}", &card.to_string().color(AnsiColors::Green));
    }
    let player = game_manager
        .round
        .as_mut()
        .unwrap()
        .current_player_mut(&mut game_manager.players);
    player.add_card(
        &card,
        &game_manager.board,
        &game_manager.hotel_chain_manager,
    );
    if !game_manager.settings.skip_dialogues {
        print!("Press enter to finish your turn");
        read_enter();
    }
}

/// Prompts the user to select a card.
/// This card is then removed from the players inventory and returned.
fn read_card(player: &mut Player) -> Result<AnalyzedPosition> {
    loop {
        print!("Enter a number 1-6: ");
        let card_index = input::<usize>().inside(1..=6).get() - 1;
        player.sort_cards();
        let analyzed_position = *player.analyzed_cards.get(card_index).as_ref().unwrap();
        // Check if hotel placement is allowed
        if analyzed_position.is_illegal() {
            continue;
        }
        let position = analyzed_position.position.clone();
        //Remove the played card from the players hand cards
        return Ok(player.remove_card(&position)?);
    }
}

/// All functions related to placing a hotel
pub mod place_hotel {
    use std::collections::HashMap;

    use miette::Result;

    use crate::{
        base_game::{
            board::{AnalyzedPosition, Board, Position},
            hotel_chains::HotelChain,
            ui,
        },
        game::game::{
            hotel_chain_manager::{self, HotelChainManager},
            GameManager,
        },
    };

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
        //let played_card = read_card(player, analyzed_positions);
        //game_manager.board.place_hotel(&card)?;
        ui::print_main_ui(&game_manager);
        //TODO Add logic for the following cases:
        //1. The board piece founds a new hotel chain
        //2. The board piece extends a existing chain
        //  2.1 The board piece extends a existing chain by more than 1 piece
        //3. The board piece creates a fusion between chains
        //  3.1 Add Logic that can handle fusions between two chains
        //  3.2 Add Logic that can handle fusions between two ore more chains
        Ok(())
    }
    /// The different cases that can hapen when a hotel is placed
    #[derive(PartialEq, Debug, Eq)]
    pub enum PlaceHotelCase {
        SingleHotel,
        NewChain(Vec<Position>),
        ExtendsChain(HotelChain, Vec<Position>),
        Fusion(Vec<HotelChain>),
        Illegal(IllegalPlacement),
    }

    impl PlaceHotelCase {
        /// Returns the type name for the case
        pub fn type_name(&self) -> String {
            match self {
                Self::SingleHotel => String::from("SingleHotel"),
                Self::NewChain(_vec) => String::from("NewChain"),
                Self::ExtendsChain(_chain, _vec) => String::from("ExtendsChain"),
                Self::Fusion(_vec) => String::from("Fusion"),
                Self::Illegal(_illegal) => String::from("Illegal"),
            }
        }
    }

    /// The different ways a hotel placement can be illegal
    #[derive(PartialEq, Debug, Eq)]
    pub enum IllegalPlacement {
        /// Signals that no more chains can be started
        ChainStartIllegal,
        /// Signals that a fusion is illegal because it would fuse chains that can no
        /// longer be fused
        FusionIllegal,
    }

    impl IllegalPlacement {
        /// Returns a string that contains the brief reson why this hotel can not be placed
        pub fn reason_short(&self) -> String {
            match self {
                IllegalPlacement::FusionIllegal => String::from("Fusion illegal"),
                IllegalPlacement::ChainStartIllegal => String::from("Chain start illegal"),
            }
        }

        /// Returns a string that contains the detailed reson why this hotel can not be placed
        pub fn reason_long(&self) -> String {
            match self {
                IllegalPlacement::FusionIllegal => String::from("Fusion illegal: The piece would start a fusion between chains that can no longer be fused."),
                IllegalPlacement::ChainStartIllegal => String::from("Chain start illegal: All 7 chains are already active."),
            }
        }
    }

    /// Analyzes the players hand cards and returns a map of analyzed positons. The value is the case
    /// that will happen when the card is played. Illegal positions are inlcuded in the map
    fn analyze_cards(
        player_cards: &Vec<Position>,
        board: &Board,
        hotel_chain_manager: &HotelChainManager,
    ) -> Vec<AnalyzedPosition> {
        let mut analyzed_cards = Vec::new();
        for card in player_cards {
            analyzed_cards.push(AnalyzedPosition::new(*card, board, hotel_chain_manager));
        }
        analyzed_cards
    }

    /// Analyzes the position of the card.
    /// Returns the case to which the position belongs
    pub fn analyze_position(
        origin: &Position,
        board: &Board,
        hotel_chain_manager: &HotelChainManager,
    ) -> PlaceHotelCase {
        let surrounding_positions: Vec<Position> = surrounding_positions(&origin);
        // Stores the surrounding hotels that do belong to a chain
        let mut surrounding_chains: Vec<HotelChain> = Vec::new();
        // Stores the surrounding hotels that do not belong to any chain
        let mut surrounding_hotels: Vec<Position> = Vec::new();
        for position in surrounding_positions {
            if let Some(value) = board.is_hotel_placed(&position) {
                match value {
                    None => surrounding_hotels.push(position),
                    Some(chain) => surrounding_chains.push(chain),
                }
            }
        }
        // Case 1: No hotel is nearby
        if surrounding_chains.is_empty() && surrounding_hotels.is_empty() {
            return PlaceHotelCase::SingleHotel;
        }
        // Case 2: New chain
        if surrounding_chains.is_empty() {
            if hotel_chain_manager.available_chains().is_none() {
                return PlaceHotelCase::Illegal(IllegalPlacement::ChainStartIllegal);
            }
            let mut founding_members: Vec<Position> = Vec::new();
            for hotel in surrounding_hotels {
                founding_members.push(hotel);
            }
            return PlaceHotelCase::NewChain(founding_members);
        }
        // Case 3: Extends chain
        if surrounding_chains.len() == 1 {
            let mut new_members: Vec<Position> = Vec::new();
            for hotel in surrounding_hotels {
                new_members.push(hotel);
            }
            return PlaceHotelCase::ExtendsChain(*surrounding_chains.get(0).unwrap(), new_members);
        }
        // Case 4: Fusion
        let mut cant_fuse = 0;
        for chain in &surrounding_chains {
            if hotel_chain_manager.is_chain_safe(&chain) {
                cant_fuse += 1;
            }
        }
        // If more than two hotels are safe from being fused the placement of the hotel is
        // illegal.
        if cant_fuse >= 2 {
            return PlaceHotelCase::Illegal(IllegalPlacement::FusionIllegal);
        }
        return PlaceHotelCase::Fusion(surrounding_chains);
    }

    /// Analyzes the surrounding positions of the piece and returns them
    fn surrounding_positions(origin: &Position) -> Vec<Position> {
        let mut neighbours: Vec<Position> = Vec::new();
        if let Some(position) = origin.up() {
            neighbours.push(position);
        }
        if let Some(position) = origin.down() {
            neighbours.push(position);
        }
        if let Some(position) = origin.next() {
            neighbours.push(position);
        }
        if let Some(position) = origin.prev() {
            neighbours.push(position);
        }
        neighbours
    }

    //TODO When fusion has been completed check if surrounding pieces (of piece that caused
    //fusion) are correctly set to the hote chain

    #[cfg(test)]
    mod tests {
        use miette::Result;

        use crate::{
            base_game::{
                bank::Bank,
                board::{Board, Position},
                hotel_chains::HotelChain,
                player::Player,
            },
            game::game::hotel_chain_manager::HotelChainManager,
            logic::place_hotel::{analyze_position, IllegalPlacement, PlaceHotelCase},
        };

        use super::surrounding_positions;

        #[test]
        fn surrounding_positions_correct() -> Result<()> {
            let origin = Position::new('B', 2);
            let surrounding_positions = surrounding_positions(&origin);
            let should = vec![
                Position::new('B', 1),
                Position::new('B', 3),
                Position::new('A', 2),
                Position::new('C', 2),
            ];
            for position in should {
                assert!(surrounding_positions.contains(&position));
            }
            Ok(())
        }

        #[test]
        fn analyze_allowed_positions() -> Result<()> {
            let mut board = Board::new();
            let mut bank = Bank::new();
            let mut hotel_chain_manager = HotelChainManager::new();
            // Place some test hotels
            board.place_hotel(&Position::new('B', 2))?;
            let chain1 = vec![Position::new('H', 3), Position::new('H', 4)];
            let chain2 = vec![Position::new('G', 6), Position::new('H', 6)];
            for chain in &chain1 {
                board.place_hotel(&chain)?;
            }
            for chain in &chain2 {
                board.place_hotel(&chain)?;
            }
            hotel_chain_manager.start_chain(
                HotelChain::Airport,
                chain1,
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Continental,
                chain2,
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            board.print(false);
            println!(
                "Start new chain name: {}",
                PlaceHotelCase::NewChain(vec![]).type_name()
            );
            // Case 1: Isolated hotel
            assert_eq!(
                analyze_position(&Position::new('F', 2), &board, &hotel_chain_manager),
                PlaceHotelCase::SingleHotel
            );
            // Case 2: Start new chain
            assert_eq!(
                analyze_position(&Position::new('C', 2), &board, &hotel_chain_manager).type_name(),
                "NewChain"
            );
            // Case 3: Extend chain
            assert_eq!(
                analyze_position(&Position::new('I', 4), &board, &hotel_chain_manager).type_name(),
                "ExtendsChain"
            );
            // Case 4: Fusion
            assert_eq!(
                analyze_position(&Position::new('H', 5), &board, &hotel_chain_manager).type_name(),
                "Fusion"
            );
            Ok(())
        }

        #[test]
        fn analyze_illegal_positions() -> Result<()> {
            let mut board = Board::new();
            let mut bank = Bank::new();
            let mut hotel_chain_manager = HotelChainManager::new();
            // Place some test hotels
            let mut positions1 = Vec::new();
            let mut positions2 = Vec::new();
            for i in 1..=12 {
                let position1 = Position::new('A', i);
                let position2 = Position::new('C', i);
                board.place_hotel(&position1)?;
                board.place_hotel(&position2)?;
                positions1.push(position1);
                positions2.push(position2);
            }
            // Test fusion illegal
            hotel_chain_manager.start_chain(
                HotelChain::Airport,
                positions1,
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Continental,
                positions2,
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            assert_eq!(
                analyze_position(&Position::new('B', 3), &board, &hotel_chain_manager),
                PlaceHotelCase::Illegal(IllegalPlacement::FusionIllegal)
            );
            // Test start new chain illegal
            hotel_chain_manager.start_chain(
                HotelChain::Festival,
                vec![Position::new('E', 1), Position::new('E', 2)],
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Imperial,
                vec![Position::new('G', 1), Position::new('G', 2)],
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Luxor,
                vec![Position::new('I', 1), Position::new('I', 2)],
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Oriental,
                vec![Position::new('G', 11), Position::new('G', 12)],
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Prestige,
                vec![Position::new('E', 11), Position::new('E', 12)],
                &mut board,
                &mut Player::new(vec![], 0),
                &mut bank,
            )?;
            board.place_hotel(&Position::new('E', 5))?;
            println!(
                "Available chains: {:?}",
                hotel_chain_manager.available_chains()
            );
            assert_eq!(
                analyze_position(&Position::new('E', 6), &board, &hotel_chain_manager),
                PlaceHotelCase::Illegal(IllegalPlacement::ChainStartIllegal)
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    mod bank {

        use miette::Result;

        use crate::{
            base_game::{hotel_chains::HotelChain, settings::Settings},
            game::game::GameManager,
        };

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
                game_manager.players.get(0).unwrap(),
                &HotelChain::Airport
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Airport
            ));
            // Test case 2: multiple largest shareholerds (Airport)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap(),
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(0).unwrap(),
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(2).unwrap(),
                &HotelChain::Continental
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(2).unwrap(),
                &HotelChain::Continental
            ));
            // Test case 3: one largest and multiple second largest shareholders (Prestige)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap(),
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(2).unwrap(),
                &HotelChain::Festival
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(3).unwrap(),
                &HotelChain::Festival
            ));
            // Test case 4: one player is largest and second largest shareholder (Luxor)
            assert!(game_manager.bank.is_largest_shareholder(
                game_manager.players.get(0).unwrap(),
                &HotelChain::Imperial
            ));
            assert!(game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(0).unwrap(),
                &HotelChain::Imperial
            ));
            assert!(!game_manager.bank.is_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Imperial
            ));
            assert!(!game_manager.bank.is_second_largest_shareholder(
                game_manager.players.get(1).unwrap(),
                &HotelChain::Imperial
            ));
        }

        fn is_error(input: Result<()>) -> bool {
            return match input {
                Err(_) => true,
                Ok(_) => false,
            };
        }
    }
}
