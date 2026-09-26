//! Demo board setup.
//!
//! Ported from the old `demo.rs` with the prints / `read_enter` / `ui` calls removed.
//! These functions build a partially-filled board used by the `--demo` screen.

use std::collections::HashMap;

use rand::Rng;

use super::bank::Bank;
use super::board::{letter::LETTERS, Board, Position};
use super::chains::HotelChain;
use super::chains_mgr::HotelChainManager;
use super::players::Player;

/// Draws a random position from the given deck and removes it.
fn draw_card(allowed_cards: &mut Vec<Position>) -> Position {
    let random_number = rand::thread_rng().gen_range(0..=allowed_cards.len() - 1);
    let position = *allowed_cards.get(random_number).unwrap();
    allowed_cards.remove(random_number);
    position
}

/// Builds a random set of chains on the board.
pub fn set_hotel_chains_random(
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
    position_cards: &mut Vec<Position>,
    board: &mut Board,
    hotel_chain_manager: &mut HotelChainManager,
    bank: &mut Bank,
) -> miette::Result<()> {
    for hotel_chain in HotelChain::iterator() {
        if rand::thread_rng().gen_bool(0.4) {
            continue;
        }
        let mut cards: Vec<Position> = Vec::new();
        for _i in 1..=20 {
            if rand::thread_rng().gen_bool(0.1) {
                break;
            }
            if position_cards.is_empty() {
                break;
            }
            cards.push(draw_card(position_cards));
        }
        for card in &cards {
            board.place_hotel(card)?;
        }
        if cards.len() < 2 {
            break;
        }
        hotel_chain_manager.start_chain(*hotel_chain, cards, board, player, bank)?;
        active_chains.push(*hotel_chain);
    }
    Ok(())
}

/// Builds a "clever" set of chains on the board (concatenated, non-overlapping chains).
pub fn set_hotel_chains_clever(
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
    _position_cards: &mut Vec<Position>,
    board: &mut Board,
    hotel_chain_manager: &mut HotelChainManager,
    bank: &mut Bank,
) -> miette::Result<()> {
    let mut allowed_positions: Vec<Position> = Vec::new();
    let mut placed_hotels: HashMap<Position, HotelChain> = HashMap::new();
    // initialize pieces
    for c in LETTERS {
        for i in 1..=12 {
            allowed_positions.push(Position::new(c, i));
        }
    }
    for hotel_chain in HotelChain::iterator() {
        if rand::thread_rng().gen_bool(0.4) {
            continue;
        }
        let mut origin;
        loop {
            origin = draw_card(&mut allowed_positions);
            if is_neighbour_free(hotel_chain, origin, &mut placed_hotels) {
                break;
            }
        }
        let positions = random_concatenated_positions(
            hotel_chain,
            origin,
            &mut allowed_positions,
            &mut placed_hotels,
        );
        if positions.len() < 2 {
            continue;
        }
        update_placed_hotels(hotel_chain, &positions, &mut placed_hotels);
        hotel_chain_manager.start_chain(*hotel_chain, positions, board, player, bank)?;
        active_chains.push(*hotel_chain);
    }
    Ok(())
}

fn update_placed_hotels(
    chain: &HotelChain,
    new_hotels: &[Position],
    placed_hotels: &mut HashMap<Position, HotelChain>,
) {
    for hotel in new_hotels {
        placed_hotels.insert(*hotel, *chain);
    }
}

fn random_concatenated_positions(
    chain: &HotelChain,
    origin: Position,
    allowed_positions: &mut Vec<Position>,
    placed_hotels: &mut HashMap<Position, HotelChain>,
) -> Vec<Position> {
    let size = rand::thread_rng().gen_range(1..=41);
    let mut positions = vec![origin];
    for i in 0..=size - 1 {
        match random_neighbour(
            chain,
            *positions.get(i).unwrap(),
            allowed_positions,
            placed_hotels,
        ) {
            Some(value) => positions.push(value),
            None => break,
        }
    }
    positions
}

fn random_neighbour(
    chain: &HotelChain,
    origin: Position,
    allowed_positions: &mut Vec<Position>,
    placed_hotels: &mut HashMap<Position, HotelChain>,
) -> Option<Position> {
    for _i in 1..=2 {
        let direction = rand::thread_rng().gen_range(0..=3);
        let position = match direction {
            0 => origin.next(),
            1 => origin.down(),
            2 => origin.prev(),
            3 => origin.up(),
            _ => continue,
        };
        if position.is_none() {
            continue;
        }
        if !allowed_positions.contains(&position.unwrap()) {
            continue;
        }
        for (index, allowed_position) in allowed_positions.iter().enumerate() {
            if allowed_position.letter.eq(&position.unwrap().letter)
                && allowed_position.number == position.unwrap().number
            {
                if !is_neighbour_free(chain, position.unwrap(), placed_hotels) {
                    continue;
                }
                allowed_positions.remove(index);
                return position;
            }
        }
    }
    None
}

fn is_neighbour_free(
    chain: &HotelChain,
    origin: Position,
    placed_hotels: &mut HashMap<Position, HotelChain>,
) -> bool {
    for i in 0..=3 {
        let position = match i {
            0 => origin.next(),
            1 => origin.down(),
            2 => origin.prev(),
            3 => origin.up(),
            _ => continue,
        };
        if position.is_none() {
            continue;
        }
        if placed_hotels.contains_key(&position.unwrap())
            && placed_hotels.get(&position.unwrap()).unwrap() != chain
        {
            return false;
        }
    }
    true
}
