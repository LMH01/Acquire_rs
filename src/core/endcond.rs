//! Game end conditions.

use std::slice::Iter;

use super::board::Board;
use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::rules::{analyze_position, PlaceHotelCase};

/// The different ways the game can end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndCondition {
    /// The game can be finished when all chains on the board have at least 10 hotels and
    /// when there is no space to found a new chain.
    AllChainsMoreThan10HotelsAndNoSpaceForNewChain,
    /// The game can be finished when at least one chain has 41 or more hotels.
    OneChain41OrMoreHotels,
}

impl EndCondition {
    fn is_condition_met(&self, board: &Board, hotel_chain_manager: &HotelChainManager) -> bool {
        match self {
            Self::AllChainsMoreThan10HotelsAndNoSpaceForNewChain => {
                let mut all_chains_safe = true;
                for chain in HotelChain::iterator() {
                    if hotel_chain_manager.chain_status(chain)
                        && hotel_chain_manager.chain_length(chain) <= 10
                    {
                        all_chains_safe = false;
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
                                    match analyze_position(&neighbour, board, hotel_chain_manager)
                                    {
                                        PlaceHotelCase::SingleHotel => return false,
                                        _ => continue,
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
                }
                true
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

    /// Returns a description of the end condition.
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

/// Checks if the game state meets at least one condition because of which the game can be
/// finished.
/// # Returns
/// * `None` - No ending condition is met.
/// * `Some(condition)` - One condition is met.
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

#[cfg(test)]
mod tests {
    use miette::Result;

    use super::check_end_condition;
    use crate::core::bank::Bank;
    use crate::core::board::{Board, Position};
    use crate::core::chains::HotelChain;
    use crate::core::chains_mgr::HotelChainManager;
    use crate::core::players::Player;

    #[test]
    fn is_end_game_condition_met_working() -> Result<()> {
        let mut board = Board::new();
        let mut hotel_chain_manager = HotelChainManager::new();
        let mut bank = Bank::new();
        let mut player = Player::new(vec![], 0, String::from("Player 1"));
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
        // Check all hotels 10 or more and no place to found new
        assert!(check_end_condition(&board, &hotel_chain_manager).is_some());
        Ok(())
    }
}
