//! Pure placement rules: analyzing a position, legality, and chain length.
//!
//! The functions that used to prompt the user (choose card, choose chain,
//! resolve fusion order, ask about stocks) have been moved out of here and
//! into the [`super::game::Game`] state machine, which emits
//! [`super::game::InputRequest`]s instead of blocking on input.

use super::board::{Board, Position};
use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;

/// The different cases that can happen when a hotel is placed.
#[derive(PartialEq, Debug, Eq, Clone)]
pub enum PlaceHotelCase {
    /// The hotel is placed with nothing special happening.
    SingleHotel,
    /// The hotel starts a new chain.
    /// * The vector contains the positions that belong to the new chain.
    NewChain(Vec<Position>),
    /// The hotel extends an already existing chain.
    /// * The first parameter is the chain that is being extended.
    /// * The vector contains the pieces that extend the chain.
    ExtendsChain(HotelChain, Vec<Position>),
    /// The hotel fuses two or more chains.
    /// * The vector contains the hotel chains that are being fused.
    /// * The second parameter contains the position that causes the fusion.
    Fusion(Vec<HotelChain>, Position),
    /// The hotel can not be placed.
    /// * The parameter describes the reason why this hotel can not be placed.
    Illegal(IllegalPlacement),
}

/// The different ways a hotel placement can be illegal.
#[derive(PartialEq, Debug, Eq, Clone)]
pub enum IllegalPlacement {
    /// Signals that no more chains can be started.
    ChainStartIllegal,
    /// Signals that a fusion is illegal because it would fuse chains that can no
    /// longer be fused.
    FusionIllegal,
}

impl IllegalPlacement {
    /// Returns a string that contains the brief reason why this hotel can not be placed.
    pub fn reason(&self) -> String {
        match self {
            IllegalPlacement::FusionIllegal => String::from("Fusion illegal"),
            IllegalPlacement::ChainStartIllegal => String::from("Chain start illegal"),
        }
    }

    /// Returns a string that contains the detailed reason why this hotel can not be placed.
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

/// Analyzes the position of the card and returns the case to which the position belongs.
pub fn analyze_position(
    origin: &Position,
    board: &Board,
    hotel_chain_manager: &HotelChainManager,
) -> PlaceHotelCase {
    let surrounding_positions: Vec<Position> = surrounding_positions(origin);
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
        if hotel_chain_manager.is_chain_safe(chain) {
            cant_fuse += 1;
        }
    }
    // If more than two hotels are safe from being fused the placement of the hotel is illegal.
    if cant_fuse >= 2 {
        return PlaceHotelCase::Illegal(IllegalPlacement::FusionIllegal);
    }
    PlaceHotelCase::Fusion(surrounding_chains, *origin)
}

/// Analyzes the surrounding positions of the piece and returns them.
pub fn surrounding_positions(origin: &Position) -> Vec<Position> {
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

/// Determines what the longest chain is.
/// # Returns
/// * `Some(chain)` - The chain that is the longest
/// * `None` - No single chain is the longest
#[allow(clippy::unnecessary_unwrap)]
pub fn longest_chain<'a>(
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
            return Some(chain1);
        }
        if chain2_length > chain1_length
            && chain2_length > chain3_length
            && chain2_length > chain4_length
        {
            return Some(chain2);
        }
        if chain3_length > chain1_length
            && chain3_length > chain2_length
            && chain3_length > chain4_length
        {
            return Some(chain3.unwrap());
        }
        if chain4_length > chain1_length
            && chain4_length > chain2_length
            && chain4_length > chain3_length
        {
            return Some(chain4.unwrap());
        }
        return None;
    }

    if chain3.is_some() && chain4.is_none() {
        let chain3_length = hotel_chain_manager.chain_length(chain3.unwrap());
        // Determine what chain is the longest out of 3
        if chain1_length > chain2_length && chain1_length > chain3_length {
            return Some(chain1);
        }
        if chain2_length > chain1_length && chain2_length > chain3_length {
            return Some(chain2);
        }
        if chain3_length > chain1_length && chain3_length > chain2_length {
            return Some(chain3.unwrap());
        }
        return None;
    }
    // Determine what chain is the longest out of 2
    if hotel_chain_manager.chain_length(chain1) > hotel_chain_manager.chain_length(chain2) {
        Some(chain1)
    } else {
        Some(chain2)
    }
}

/// The hotel that is placed by the player extends a chain.
/// # Arguments
/// * `chain` - The chain that is extended
/// * `positions` - The positions that should extend the chain
pub fn extend_chain(
    chain: HotelChain,
    positions: Vec<Position>,
    hotel_chain_manager: &mut HotelChainManager,
    board: &mut Board,
) -> miette::Result<()> {
    for position in positions {
        hotel_chain_manager.add_hotel_to_chain(&chain, position, board)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use miette::Result;

    use super::{
        analyze_position, longest_chain, surrounding_positions, IllegalPlacement, PlaceHotelCase,
    };
    use crate::core::bank::Bank;
    use crate::core::board::{Board, Position};
    use crate::core::chains::HotelChain;
    use crate::core::chains_mgr::HotelChainManager;
    use crate::core::players::Player;

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
        let mut players = vec![
            Player::new(vec![], 0, String::from("Player 1")),
            Player::new(vec![], 1, String::from("Player 2")),
        ];
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
            &mut Player::new(vec![], 0, String::from("Player 1")),
            &mut bank,
        )?;
        hotel_chain_manager.start_chain(
            HotelChain::Continental,
            chain2,
            &mut board,
            &mut Player::new(vec![], 0, String::from("Player 2")),
            &mut bank,
        )?;
        // Case 1: Isolated hotel
        assert_eq!(
            type_name(&analyze_position(
                &Position::new('F', 2),
                &board,
                &hotel_chain_manager
            )),
            "SingleHotel"
        );
        // Case 2: Start new chain
        assert_eq!(
            type_name(&analyze_position(
                &Position::new('C', 2),
                &board,
                &hotel_chain_manager
            )),
            "NewChain"
        );
        // Case 3: Extend chain
        assert_eq!(
            type_name(&analyze_position(
                &Position::new('I', 4),
                &board,
                &hotel_chain_manager
            )),
            "ExtendsChain"
        );
        // Case 4: Fusion
        assert_eq!(
            type_name(&analyze_position(
                &Position::new('H', 5),
                &board,
                &hotel_chain_manager
            )),
            "Fusion"
        );
        Ok(())
    }

    fn type_name(place_hotel_case: &PlaceHotelCase) -> String {
        match place_hotel_case {
            PlaceHotelCase::SingleHotel => String::from("SingleHotel"),
            PlaceHotelCase::NewChain(_chain) => String::from("NewChain"),
            PlaceHotelCase::ExtendsChain(_chain, _pos) => String::from("ExtendsChain"),
            PlaceHotelCase::Fusion(_chains, _origin) => String::from("Fusion"),
            PlaceHotelCase::Illegal(_reason) => String::from("Illegal"),
        }
    }

    #[test]
    fn analyze_illegal_positions() -> Result<()> {
        let mut board = Board::new();
        let mut bank = Bank::new();
        let mut hotel_chain_manager = HotelChainManager::new();
        let mut player = Player::new(
            vec![Position::new('B', 3), Position::new('E', 6)],
            0,
            String::from("Player 1"),
        );
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
        player.analyze_cards(&board, &hotel_chain_manager);
        assert!(player.only_illegal_cards());
        assert_eq!(
            analyze_position(&Position::new('E', 6), &board, &hotel_chain_manager),
            PlaceHotelCase::Illegal(IllegalPlacement::ChainStartIllegal)
        );
        Ok(())
    }
}
