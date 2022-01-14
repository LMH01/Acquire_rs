use std::{cmp::Ordering, collections::HashMap, iter::Map, slice::Iter};

use miette::{miette, Result};
use owo_colors::{AnsiColors, OwoColorize};
use read_input::{prelude::input, InputBuild};

use crate::{
    base_game::{
        bank::{Bank, LargestShareholders},
        board::{AnalyzedPosition, Board, Position},
        hotel_chains::HotelChain,
        player::Player,
    },
    data_stream::read_enter,
    game::game::{
        hotel_chain_manager::{self, HotelChainManager},
        GameManager,
    },
    logic::place_hotel::{analyze_position, PlaceHotelCase},
};

use self::place_hotel::IllegalPlacement;

/// The different ways the game can end.
#[derive(Clone, Copy)]
pub enum EndCondition {
    /// The game can be finished when all chains on the board have at least 10 hotels and
    /// when there is no space to found a new chain
    AllChainsMoreThan10HotelsAndNoSpaceForNewChain,
    /// The game can be finished when at least one chain has 41 or more hotels
    OneChain41OrMoreHotels,
}

impl EndCondition {
    fn is_condition_met(&self, board: &Board, hotel_chain_manager: &HotelChainManager) -> bool {
        match self {
            Self::AllChainsMoreThan10HotelsAndNoSpaceForNewChain => {
                let mut all_chains_safe = true;
                for chain in HotelChain::iterator() {
                    if hotel_chain_manager.chain_status(chain) {
                        if hotel_chain_manager.chain_length(chain) <= 10 {
                            all_chains_safe = false;
                        }
                    }
                }
                if !all_chains_safe {
                    return false;
                }
                for line in &board.pieces {
                    for piece in line {
                        match analyze_position(&piece.position, board, hotel_chain_manager) {
                            PlaceHotelCase::NewChain(_positions) => return false,
                            PlaceHotelCase::SingleHotel => {
                                let neighbours = piece.position.neighbours();
                                // Check if one of the neighbours is free for a single hotel.
                                // If yes two single hotels stand next to each other and could
                                // found a new chain.
                                for neighbour in neighbours {
                                    match analyze_position(&neighbour, board, hotel_chain_manager) {
                                        PlaceHotelCase::SingleHotel => return false,
                                        _ => continue,
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
                }
                return true;
            }
            Self::OneChain41OrMoreHotels => {
                for chain in HotelChain::iterator() {
                    if hotel_chain_manager.chain_length(chain) >= 41 {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Returns a description on the end condition
    pub fn description(&self) -> String {
        match self {
            Self::AllChainsMoreThan10HotelsAndNoSpaceForNewChain => {
                String::from("All chains have at least 10 hotels and no new chains can be founded")
            }
            Self::OneChain41OrMoreHotels => String::from("One chain has 41 or more hotels"),
        }
    }

    fn iterator() -> Iter<'static, EndCondition> {
        const END_CONDITION: [EndCondition; 2] = [
            EndCondition::AllChainsMoreThan10HotelsAndNoSpaceForNewChain,
            EndCondition::OneChain41OrMoreHotels,
        ];
        END_CONDITION.iter()
    }
}

/// Checks if the game state mets at least one condition because of which the game can be
/// finished.
/// # Returns
/// * `None` - No ending condition is met.
/// * `Some(condition)` - One condition is met.
/// * `true` - When the game mets at leaste one end condition
pub fn check_end_condition(
    board: &Board,
    hotel_chain_manager: &HotelChainManager,
) -> Option<EndCondition> {
    for end_condition in EndCondition::iterator() {
        if end_condition.is_condition_met(board, hotel_chain_manager) {
            return Some(*end_condition);
        }
    }
    None
}

/// Checks if there are still positions on the board left that could be played.
/// If there are none true is returned.
/// Used to determine if the game will finish definitly.
pub fn can_game_continue(board: &Board, hotel_chain_manager: &HotelChainManager) -> bool {
    for line in &board.pieces {
        for piece in line {
            if piece.chain.is_none() {
                match analyze_position(&piece.position, board, hotel_chain_manager) {
                    PlaceHotelCase::Illegal(_reason) => continue,
                    _ => {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// All functions related to placing a hotel
pub mod place_hotel {
    use std::{cmp::Ordering, collections::HashMap, slice::SliceIndex};

    use miette::{miette, Result};
    use owo_colors::{AnsiColors, OwoColorize};
    use rand::seq::index;

    use crate::{
        base_game::{
            bank::{self, Bank},
            board::{AnalyzedPosition, Board, Piece, Position},
            hotel_chains::{self, HotelChain},
            player::Player,
            settings::Settings,
            ui,
        },
        game::game::{
            hotel_chain_manager::{self, HotelChainManager},
            round::Round,
            GameManager,
        },
        utils::remove_content_from_vec,
    };

    /// Place a hotel on the board.
    /// This function will abide by the game rules.
    /// The player is asked what card to play.
    /// # Returns
    /// * `Ok(true)` - A hotel has been placed
    /// * `Ok(false)` - No hotel has been placed
    pub fn place_hotel(
        player_index: usize,
        players: &mut Vec<Player>,
        board: &mut Board,
        settings: &Settings,
        round: &Round,
        bank: &mut Bank,
        hotel_chain_manager: &mut HotelChainManager,
    ) -> Result<bool> {
        let mut player = players.get_mut(player_index).unwrap();
        player.print_text_ln("Please choose what hotel card you would like to play.");
        //TODO Add function that checkes what cards can be played
        // Check if player has at least one card that can be played
        if player.only_illegal_cards() {
            player.print_text_ln("You have no card that could be played.");
            return Ok(false);
        }
        let played_position = player.read_card()?;
        // Place hotel
        board.place_hotel(&played_position.position)?;
        ui::print_main_ui(
            Some(player),
            board,
            settings,
            Some(round),
            bank,
            hotel_chain_manager,
        );
        match played_position.place_hotel_case {
            PlaceHotelCase::SingleHotel => (),
            PlaceHotelCase::NewChain(positions) => start_chain(
                positions,
                player_index,
                players,
                hotel_chain_manager,
                board,
                bank,
            )?,
            PlaceHotelCase::ExtendsChain(chain, positions) => {
                extend_chain(chain, positions, hotel_chain_manager, board)?
            }
            PlaceHotelCase::Fusion(chains, origin) => fuse_chains(
                chains,
                origin,
                player_index,
                players,
                board,
                bank,
                hotel_chain_manager,
                round,
                settings,
            )?,
            _ => (),
        }
        // Handle cases
        //TODO Add logic for the following cases:
        //1. The board piece founds a new hotel chain
        //2. The board piece extends a existing chain - Done
        //  2.1 The board piece extends a existing chain by more than 1 piece - Done
        //3. The board piece creates a fusion between chains
        //  3.1 Add Logic that can handle fusions between two chains
        //  3.2 Add Logic that can handle fusions between two ore more chains
        Ok(true)
    }

    /// The player will start a new chain.
    /// # Arguments
    /// * `positions` - The positions that will belong to the new chain
    /// * `player` - The player that founds the new chain
    pub fn start_chain(
        positions: Vec<Position>,
        player_index: usize,
        players: &mut Vec<Player>,
        hotel_chain_manager: &mut HotelChainManager,
        board: &mut Board,
        bank: &mut Bank,
    ) -> Result<()> {
        let player = players.get_mut(player_index).unwrap();
        //TODO Add logic that makes the player select a new chain
        let mut available_chains = HashMap::new();
        let mut available_chains_identifier = Vec::new();
        for chain in HotelChain::iterator() {
            if hotel_chain_manager
                .available_chains()
                .unwrap()
                .contains(chain)
            {
                available_chains.insert(chain.identifier(), *chain);
                available_chains_identifier.push(chain.identifier());
            }
        }
        let mut available_chains_help = String::new();
        let mut first = true;
        for (k, v) in &available_chains {
            if first {
                first = false;
            } else {
                available_chains_help.push_str(", ");
            }
            available_chains_help.push_str(&format!("{}", k.color(v.color()).to_string()));
        }
        let input = player.read_input(
            format!(
                "What chain would you like to start? [{}]: ",
                available_chains_help
            ),
            available_chains_identifier,
        );

        let chain = available_chains.get(&input).unwrap();
        hotel_chain_manager.start_chain(*chain, positions, board, player, bank)?;
        bank.update_largest_shareholders(players);
        Ok(())
    }

    /// The hotel that is placed by the player extends a chain
    /// # Arguments
    /// * `chain` - The chain that is extended
    /// * `positions` - The positions that should extend the chain
    pub fn extend_chain(
        chain: HotelChain,
        positions: Vec<Position>,
        hotel_chain_manager: &mut HotelChainManager,
        board: &mut Board,
    ) -> Result<()> {
        for position in positions {
            hotel_chain_manager.add_hotel_to_chain(&chain, position, board)?;
        }
        Ok(())
    }

    /// Analyses the length of input chains. When some chains are equally long the player that
    /// started the fusion is asked which chain should survive.
    pub fn fuse_chains(
        chains: Vec<HotelChain>,
        origin: Position,
        player_index: usize,
        players: &mut Vec<Player>,
        board: &mut Board,
        bank: &mut Bank,
        hotel_chain_manager: &mut HotelChainManager,
        round: &Round,
        settings: &Settings,
    ) -> Result<()> {
        let player = players.get_mut(player_index).unwrap();
        // Contains the order in which the hotels are fused with the surviving chain.
        let mut fuse_order = Vec::new();
        let surviving_chain;
        //TODO This text should be broadcasted to each player
        player.print_text_ln(&format!(
            "Fusion between {} chains at {}!",
            chains.len(),
            &origin.color(AnsiColors::Green).to_string()
        ));
        // Determine the order in which the hotels are fused
        match chains.len() {
            2 => {
                let chain1 = chains.get(0).unwrap();
                let chain2 = chains.get(1).unwrap();
                let resolved_order =
                    resolve_fusion_order(player, chain1, chain2, hotel_chain_manager);
                fuse_order.push(*resolved_order.get(0).unwrap());
                surviving_chain = *resolved_order.get(1).unwrap();
            }
            3 => {
                let chain1 = chains.get(0).unwrap();
                let chain2 = chains.get(1).unwrap();
                let chain3 = chains.get(2).unwrap();
                match longest_chain(chain1, chain2, Some(chain3), None, hotel_chain_manager) {
                    Some(chain) => {
                        let mut resolved_order = Vec::new();
                        if chain == chain1 {
                            resolved_order =
                                resolve_fusion_order(player, chain2, chain3, hotel_chain_manager);
                        }
                        if chain == chain2 {
                            resolved_order =
                                resolve_fusion_order(player, chain1, chain3, hotel_chain_manager);
                        }
                        if chain == chain3 {
                            resolved_order =
                                resolve_fusion_order(player, chain1, chain2, hotel_chain_manager);
                        }
                        fuse_order.push(resolved_order.get(0).unwrap());
                        fuse_order.push(resolved_order.get(1).unwrap());
                        surviving_chain = chain;
                    }
                    None => {
                        // All three chains are equally long
                        player.print_text_ln("All three chains are equally long.");
                        player.print_text_ln("Note: The chain that you pic first will be fused into the second and the second will be fused into the third.");
                        let resolved_order =
                            resolve_fusion_order_three_and_four_chains(player, &chains)?;
                        fuse_order.push(resolved_order.get(0).unwrap());
                        fuse_order.push(resolved_order.get(1).unwrap());
                        surviving_chain = resolved_order.get(2).unwrap();
                    }
                }
            }
            4 => {
                //TODO this should be broadcasted to every player
                player.print_text_ln("Concratulations, you are fusing 4 chains into one.");
                player.print_text_ln("Because this scenario is so unlikely i did not code a way to automatically detect the fusion order.");
                player
                    .print_text_ln("You will have to do that manually and acording to the rules:");
                player
                    .print_text_ln("1. The chain with the most hotels absorbs all smaller chains");
                player
                    .print_text_ln("2. The order in which the smaller chains are fused is determined by thair size.\n   The smallest chain fuses into the second smallest chain and so on.");
                player.print_text_ln("3. The player that stared the fusion can decide the fusion order, if all chains are the same size");
                player.print_text_ln("Note: The chain that you pic first will be fused into the second, second will be fused into the third and the third will be fused into the fourth.");
                let resolved_order = resolve_fusion_order_three_and_four_chains(player, &chains)?;
                fuse_order.push(resolved_order.get(0).unwrap());
                fuse_order.push(resolved_order.get(1).unwrap());
                fuse_order.push(resolved_order.get(2).unwrap());
                surviving_chain = resolved_order.get(3).unwrap();
            }
            _ => return Err(miette!("Unable to fuse chains: The amount of input chains is invalid. Should be 1-4, was {}", chains.len())),
        };
        // Fuse oder has been determined
        let chain1 = *fuse_order.get(0).unwrap();
        fuse_two_chains(
            surviving_chain,
            chain1,
            player_index,
            players,
            board,
            hotel_chain_manager,
            bank,
        )?;
        if fuse_order.len() > 1 {
            let player = players.get_mut(player_index).unwrap();
            ui::print_main_ui(
                Some(player),
                board,
                settings,
                Some(round),
                bank,
                hotel_chain_manager,
            );
            let chain2 = *fuse_order.get(1).unwrap();
            fuse_two_chains(
                surviving_chain,
                chain2,
                player_index,
                players,
                board,
                hotel_chain_manager,
                bank,
            )?;
            if fuse_order.len() > 2 {
                let player = players.get_mut(player_index).unwrap();
                ui::print_main_ui(
                    Some(player),
                    board,
                    settings,
                    Some(round),
                    bank,
                    hotel_chain_manager,
                );
                let chain3 = *fuse_order.get(2).unwrap();
                fuse_two_chains(
                    surviving_chain,
                    chain3,
                    player_index,
                    players,
                    board,
                    hotel_chain_manager,
                    bank,
                )?;
            }
        }
        // Add the hotel that caused the fusion the the chain that survived
        hotel_chain_manager.add_hotel_to_chain(surviving_chain, origin, board)?;
        Ok(())
    }

    /// Determines which chain will surivive the fusion.
    /// If the two chains are equal in size the player that started the fusion is asked which chain
    /// should survive.
    /// # Returns
    /// A vector: The first element will be fused into the second element
    fn resolve_fusion_order<'a>(
        player: &Player,
        chain1: &'a HotelChain,
        chain2: &'a HotelChain,
        hotel_chain_manager: &HotelChainManager,
    ) -> Vec<&'a HotelChain> {
        let mut fuse_order = Vec::new();
        match hotel_chain_manager
            .chain_length(chain1)
            .cmp(&hotel_chain_manager.chain_length(chain2))
        {
            Ordering::Greater => {
                fuse_order.push(chain2);
                fuse_order.push(chain1);
            }
            Ordering::Less => {
                fuse_order.push(chain1);
                fuse_order.push(chain2);
            }
            Ordering::Equal => {
                // Player decides which chain should fuse into which
                let fusion_case = player.read_input(
                    format!(
                        "[1] = Fuse {} in {}\n[2] = Fuse {} in {}\nChoose a case: ",
                        chain1.name().color(chain1.color()),
                        chain2.name().color(chain2.color()),
                        chain2.name().color(chain2.color()),
                        chain1.name().color(chain1.color())
                    ),
                    vec![1, 2],
                );
                match fusion_case {
                    1 => {
                        fuse_order.push(chain1);
                        fuse_order.push(chain2);
                    }
                    2 => {
                        fuse_order.push(chain2);
                        fuse_order.push(chain1);
                    }
                    _ => (),
                }
            }
        }
        fuse_order
    }

    /// Asks the player the order in which order the three or four chains should be fused.
    fn resolve_fusion_order_three_and_four_chains<'a>(
        player: &Player,
        chains: &'a Vec<HotelChain>,
    ) -> Result<Vec<&'a HotelChain>> {
        if chains.len() <= 2 || chains.len() > 4 {
            return Err(miette!(
                "Unable to resolve fusion order: Not enough/too many chains where provided!"
            ));
        }
        let mut fuse_order = Vec::new();
        loop {
            // Setup variables for user input
            let mut available_chains_identifier = Vec::new();
            let mut available_chains = HashMap::new();
            for chain in chains {
                available_chains_identifier.push(chain.identifier());
                available_chains.insert(chain.identifier(), *chain);
            }
            // Setup pretty print for user
            let mut available_chains_help = String::new();
            let mut first = true;
            for (k, v) in &available_chains {
                if first {
                    first = false;
                } else {
                    available_chains_help.push_str(", ");
                }
                available_chains_help.push_str(&format!("{}", k.color(v.color()).to_string()));
            }
            let surviving_chain = player.read_input(
                format!(
                    "Which chain should surivive the fusion? [{}]: ",
                    available_chains_help
                ),
                available_chains_identifier,
            );
            // Contains the chain that the player decided should survive.
            let surviving_chain_temp = *available_chains.get(&surviving_chain).unwrap();
            player.print_text_ln(&format!(
                "Please choose the order in which the hotels should be fused into {}:",
                surviving_chain_temp
                    .name()
                    .color(surviving_chain_temp.color())
            ));
            let mut available_positions: Vec<u32>;
            if chains.len() == 3 {
                available_positions = vec![1, 2];
            } else {
                available_positions = vec![1, 2, 3];
            }
            let mut determined_positions = HashMap::new();
            let mut surviving_chain = None;
            for chain in chains {
                if *chain == surviving_chain_temp {
                    // Surviving chain is now a reference that is not owned by the function
                    surviving_chain = Some(chain);
                    continue;
                }
                let mut allowed_values_string = String::new();
                let mut first_position = true;
                for p in &available_positions {
                    if !first_position {
                        allowed_values_string.push_str(", ");
                    } else {
                        first_position = false;
                    }
                    allowed_values_string.push_str(&p.to_string());
                }
                let pos = player.read_input(
                    format!(
                        "At which position should {} be? [{}]: ",
                        chain.name().color(chain.color()),
                        allowed_values_string,
                    ),
                    available_positions.clone(),
                );
                determined_positions.insert(pos, chain);
                remove_content_from_vec(pos, &mut available_positions)?;
            }
            let surviving_chain = surviving_chain.unwrap();
            // Show summary
            player.print_text_ln("The fusion will take place as followed:");
            let chain1 = *determined_positions.get(&1).unwrap();
            let chain2 = *determined_positions.get(&2).unwrap();
            if chains.len() == 3 {
                player.print_text_ln(&format!(
                    "1. {} -> {}\n2. {} -> {}",
                    chain1.name().color(chain1.color()),
                    surviving_chain.name().color(surviving_chain.color()),
                    chain2.name().color(chain2.color()),
                    surviving_chain.name().color(surviving_chain.color())
                ));
            } else {
                let chain3 = *determined_positions.get(&3).unwrap();
                player.print_text_ln(&format!(
                    "1. {} -> {}\n2. {} -> {}\n3. {} -> {}",
                    chain1.name().color(chain1.color()),
                    surviving_chain.name().color(surviving_chain.color()),
                    chain2.name().color(chain2.color()),
                    surviving_chain.name().color(surviving_chain.color()),
                    chain3.name().color(chain3.color()),
                    surviving_chain.name().color(surviving_chain.color()),
                ));
            }
            match player.get_correct() {
                false => continue,
                true => {
                    fuse_order.push(*determined_positions.get(&1).unwrap());
                    fuse_order.push(*determined_positions.get(&2).unwrap());
                    fuse_order.push(*determined_positions.get(&3).unwrap());
                    if chains.len() == 4 {
                        fuse_order.push(surviving_chain);
                    }
                    break;
                }
            }
        }
        Ok(fuse_order)
    }

    /// Determines what the longest chain is.
    /// # Returns
    /// * 'Some(chain)' - The chain that is the longest
    /// * `None` - No chain is the longest
    fn longest_chain<'a>(
        chain1: &'a HotelChain,
        chain2: &'a HotelChain,
        chain3: Option<&'a HotelChain>,
        chain4: Option<&'a HotelChain>,
        hotel_chain_manager: &HotelChainManager,
    ) -> Option<&'a HotelChain> {
        let chain1_length = hotel_chain_manager.chain_length(chain1);
        let chain2_length = hotel_chain_manager.chain_length(chain2);
        if chain3.is_some() && chain4.is_some() {
            let chain3_length = hotel_chain_manager.chain_length(chain3.unwrap());
            let chain4_length = hotel_chain_manager.chain_length(chain4.unwrap());
            // Determine what chain is the longest out of 4
            if chain1_length > chain2_length
                && chain1_length > chain3_length
                && chain1_length > chain4_length
            {
                // Chain 1 is the longest
                return Some(chain1);
            }
            if chain2_length > chain1_length
                && chain2_length > chain3_length
                && chain2_length > chain4_length
            {
                // Chain 2 is the longest
                return Some(chain2);
            }
            if chain3_length > chain1_length
                && chain3_length > chain2_length
                && chain3_length > chain4_length
            {
                // Chain 3 is the longest
                return Some(chain3.unwrap());
            }
            if chain4_length > chain1_length
                && chain4_length > chain2_length
                && chain4_length > chain3_length
            {
                // Chain 4 is the longest
                return Some(chain4.unwrap());
            }
            // No chain is the longest
            return None;
        }

        if chain3.is_some() && chain4.is_none() {
            let chain3_length = hotel_chain_manager.chain_length(chain3.unwrap());
            // Determine what chain is the longest out of 3
            if chain1_length > chain2_length && chain1_length > chain3_length {
                // Chain 1 is the longest
                return Some(chain1);
            }
            if chain2_length > chain1_length && chain2_length > chain3_length {
                // Chain 2 is the longest
                return Some(chain2);
            }
            if chain3_length > chain1_length && chain3_length > chain2_length {
                // Chain 3 is the longest
                return Some(chain3.unwrap());
            }
            // No chain is the longest
            return None;
        }
        // Determine what chain is the longest out of 2
        if hotel_chain_manager.chain_length(chain1) > hotel_chain_manager.chain_length(chain2) {
            return Some(chain1);
        } else {
            return Some(chain2);
        }
    }

    /// The stocks will be sold or exchanged and the chains will be fused.
    /// This function uses [`crate::game::game::hotel_chain_manager::HotelChainManager::fuse_chains`] to update
    /// the active chains and the board.
    /// The currently playing player is asked to press enter do start the fusion.
    fn fuse_two_chains(
        alive: &HotelChain,
        dead: &HotelChain,
        player_index: usize,
        players: &mut Vec<Player>,
        board: &mut Board,
        hotel_chain_manager: &mut HotelChainManager,
        bank: &mut Bank,
    ) -> Result<()> {
        let player = players.get_mut(player_index).unwrap();
        player.get_enter(&format!(
            "Press enter to fuse {} into {} ",
            dead.name().color(dead.color()).to_string(),
            alive.name().color(alive.color()).to_string()
        ));
        // 1. Payout the majority shareholder bonuses
        bank.update_largest_shareholders(players);
        bank.give_majority_shareholder_bonuses(players, dead, hotel_chain_manager, true)?;
        // 2. Trade stocks
        for i in 0..=players.len() {
            let mut index = player_index + i;
            if index > players.len() - 1 {
                index = 0;
            }
            let player = players.get_mut(index).unwrap();
            // check if player has stocks if yes let them handle the fusion stocks
            if *player.owned_stocks.stocks_for_hotel(dead) > 0 {
                player.handle_fusion_stocks(dead, alive, bank, hotel_chain_manager)?;
            }
        }
        // 3. Fuse chains on board
        hotel_chain_manager.fuse_chains(alive, dead, board)?;
        Ok(())
    }

    /// The different cases that can hapen when a hotel is placed
    #[derive(PartialEq, Debug, Eq)]
    pub enum PlaceHotelCase {
        //TODO Add rustdoc for the enum variants that describes the arguments (link to functions)
        SingleHotel,
        NewChain(Vec<Position>),
        ExtendsChain(HotelChain, Vec<Position>),
        Fusion(Vec<HotelChain>, Position),
        Illegal(IllegalPlacement),
    }

    impl PlaceHotelCase {
        /// Returns the type name for the case
        pub fn type_name(&self) -> String {
            match self {
                Self::SingleHotel => String::from("SingleHotel"),
                Self::NewChain(_vec) => String::from("NewChain"),
                Self::ExtendsChain(_chain, _vec) => String::from("ExtendsChain"),
                Self::Fusion(_vec, _origin) => String::from("Fusion"),
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
        pub fn reason(&self) -> String {
            match self {
                IllegalPlacement::FusionIllegal => String::from("Fusion illegal"),
                IllegalPlacement::ChainStartIllegal => String::from("Chain start illegal"),
            }
        }

        /// Returns a string that contains the detailed reson why this hotel can not be placed
        pub fn description(&self) -> String {
            match self {
                IllegalPlacement::FusionIllegal => String::from(
                    "The piece would start a fusion between chains that can no longer be fused.",
                ),
                IllegalPlacement::ChainStartIllegal => String::from(
                    "The piece would start a new chain but all 7 chains are already active.",
                ),
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
        // Stores the surrounding chains
        let mut surrounding_chains: Vec<HotelChain> = Vec::new();
        // Stores the surrounding hotels that do not belong to any chain
        let mut surrounding_hotels: Vec<Position> = Vec::new();
        for position in surrounding_positions {
            if let Some(value) = board.is_hotel_placed(&position) {
                match value {
                    None => surrounding_hotels.push(position),
                    Some(chain) => {
                        // Add each chain only once
                        if !surrounding_chains.contains(&chain) {
                            surrounding_chains.push(chain);
                        }
                    }
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
            founding_members.push(*origin);
            return PlaceHotelCase::NewChain(founding_members);
        }
        // Case 3: Extends chain
        if surrounding_chains.len() == 1 {
            let mut new_members: Vec<Position> = Vec::new();
            for hotel in surrounding_hotels {
                new_members.push(hotel);
            }
            new_members.push(*origin);
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
        return PlaceHotelCase::Fusion(surrounding_chains, *origin);
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
        use std::slice::SliceIndex;

        use miette::Result;

        use crate::{
            base_game::{
                bank::Bank,
                board::{Board, Position},
                hotel_chains::HotelChain,
                player::Player,
                settings::Settings,
                ui,
            },
            game::game::hotel_chain_manager::{self, HotelChainManager},
            logic::{
                check_end_condition,
                place_hotel::{analyze_position, fuse_chains, IllegalPlacement, PlaceHotelCase},
            },
        };

        use super::{longest_chain, surrounding_positions};
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
        fn longest_chain_works() -> Result<()> {
            let mut board = Board::new();
            let mut bank = Bank::new();
            let mut hotel_chain_manager = HotelChainManager::new();
            let mut players = vec![Player::new(vec![], 0), Player::new(vec![], 1)];
            let chain1 = &HotelChain::Luxor;
            let chain2 = &HotelChain::Festival;
            let chain3 = &HotelChain::Imperial;
            let chain4 = &HotelChain::Continental;
            hotel_chain_manager.start_chain(
                *chain1,
                vec![Position::new('E', 3), Position::new('E', 4)],
                &mut board,
                players.get_mut(0).unwrap(),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                *chain2,
                vec![Position::new('C', 5), Position::new('D', 5)],
                &mut board,
                players.get_mut(0).unwrap(),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                *chain3,
                vec![
                    Position::new('F', 5),
                    Position::new('G', 5),
                    Position::new('H', 5),
                ],
                &mut board,
                players.get_mut(0).unwrap(),
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                *chain4,
                vec![Position::new('E', 6), Position::new('E', 7)],
                &mut board,
                players.get_mut(0).unwrap(),
                &mut bank,
            )?;
            board.print(false);
            assert_eq!(
                longest_chain(chain1, chain3, None, None, &hotel_chain_manager).unwrap(),
                chain3
            );
            assert_eq!(
                longest_chain(chain1, chain2, Some(chain3), None, &hotel_chain_manager).unwrap(),
                chain3
            );
            assert_eq!(
                longest_chain(
                    chain1,
                    chain2,
                    Some(chain3),
                    Some(chain4),
                    &hotel_chain_manager
                )
                .unwrap(),
                chain3
            );
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
            let mut player = Player::new(vec![Position::new('B', 3), Position::new('E', 6)], 0);
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
                &mut player,
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Continental,
                positions2,
                &mut board,
                &mut player,
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
                &mut player,
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Imperial,
                vec![Position::new('G', 1), Position::new('G', 2)],
                &mut board,
                &mut player,
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Luxor,
                vec![Position::new('I', 1), Position::new('I', 2)],
                &mut board,
                &mut player,
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Oriental,
                vec![Position::new('G', 11), Position::new('G', 12)],
                &mut board,
                &mut player,
                &mut bank,
            )?;
            hotel_chain_manager.start_chain(
                HotelChain::Prestige,
                vec![Position::new('E', 11), Position::new('E', 12)],
                &mut board,
                &mut player,
                &mut bank,
            )?;
            board.place_hotel(&Position::new('E', 5))?;
            println!(
                "Available chains: {:?}",
                hotel_chain_manager.available_chains()
            );
            player.analyze_cards(&board, &hotel_chain_manager);
            player.print_cards();
            assert!(player.only_illegal_cards());
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
    use miette::Result;

    use crate::{
        base_game::{
            bank::Bank,
            board::{Board, Position},
            hotel_chains::HotelChain,
            player::Player,
            settings::Settings,
            ui,
        },
        game::game::hotel_chain_manager::HotelChainManager,
        logic::{can_game_continue, check_end_condition},
    };

    #[test]
    fn is_end_game_condition_met_working() -> Result<()> {
        let mut board = Board::new();
        let mut hotel_chain_manager = HotelChainManager::new();
        let mut bank = Bank::new();
        let mut player = Player::new(vec![], 0);
        let mut positions = Vec::new();
        // Check no end condition is met
        assert!(check_end_condition(&board, &hotel_chain_manager).is_none());
        for c in vec!['A', 'B', 'C', 'D'] {
            for i in 1..=12 {
                positions.push(Position::new(c, i));
            }
        }
        hotel_chain_manager.start_chain(
            HotelChain::Luxor,
            positions,
            &mut board,
            &mut player,
            &mut bank,
        )?;
        // Check end condition is met when one hotel has 41 or more hotels
        assert!(check_end_condition(&board, &hotel_chain_manager).is_some());
        let mut board = Board::new();
        let mut hotel_chain_manager = HotelChainManager::new();
        for c in vec!['A', 'C', 'E', 'G', 'I'] {
            let mut positions = Vec::new();
            for i in 1..=12 {
                positions.push(Position::new(c, i));
            }
            let chain = match c {
                'A' => HotelChain::Airport,
                'C' => HotelChain::Continental,
                'E' => HotelChain::Luxor,
                'G' => HotelChain::Oriental,
                'I' => HotelChain::Prestige,
                _ => HotelChain::Imperial,
            };
            hotel_chain_manager.start_chain(
                chain,
                positions,
                &mut board,
                &mut player,
                &mut bank,
            )?;
        }
        ui::print_main_ui(
            Some(&player),
            &board,
            &Settings::new(false, false, false),
            None,
            &bank,
            &hotel_chain_manager,
        );
        // Check all hotels 10 or more and no place to found new
        assert!(check_end_condition(&board, &hotel_chain_manager).is_some());
        assert!(!can_game_continue(&board, &hotel_chain_manager));
        Ok(())
    }
}
