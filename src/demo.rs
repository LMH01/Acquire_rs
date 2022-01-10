use std::collections::HashMap;

use miette::Result;
use owo_colors::{AnsiColors, OwoColorize};
use rand::Rng;

use crate::{
    base_game::{
        board::{letter::LETTERS, Board, Position},
        hotel_chains::HotelChain,
        player::Player,
        settings::Settings,
        ui,
    },
    data_stream::read_enter,
    game::game::{player_by_id_mut, round::Round, GameManager},
    Opts,
};

fn place_test_hotels(board: &mut Board) -> Result<()> {
    for (index, h) in HotelChain::iterator().enumerate() {
        board.place_hotel_debug(Position::new('A', index.try_into().unwrap()), *h)?;
    }
    Ok(())
}

pub fn test_things(opts: &Opts, settings: Settings) -> Result<()> {
    let mut game_manager = GameManager::new(opts.players, settings)?;
    game_manager.round = Some(Round::new());
    let mut active_chains: Vec<HotelChain> = Vec::new();
    let mut player = Player::new(vec![Position::new('A', 3)], 0);
    if opts.demo_type == 0 {
        set_hotel_chains_clever(&mut game_manager, &mut active_chains, &mut player)?;
    } else {
        set_hotel_chains_random(&mut game_manager, &mut active_chains, &mut player)?;
    }
    game_manager
        .bank
        .update_largest_shareholders(&game_manager.players);
    player.analyze_cards(&game_manager.board, &game_manager.hotel_chain_manager);
    ui::print_main_ui(&game_manager);
    if active_chains.len() >= 2 {
        let rand1 = rand::thread_rng().gen_range(0..=active_chains.len() - 1);
        let mut rand2 = rand::thread_rng().gen_range(0..=active_chains.len() - 1);
        while rand1 == rand2 {
            rand2 = rand::thread_rng().gen_range(0..=active_chains.len() - 1);
        }
        let mut chain1 = active_chains.get(rand1).unwrap();
        let mut chain2 = active_chains.get(rand2).unwrap();
        if game_manager.hotel_chain_manager.chain_length(chain1)
            < game_manager.hotel_chain_manager.chain_length(chain2)
        {
            let buffer = chain1;
            chain1 = chain2;
            chain2 = buffer;
        }
        println!("Press enter to fuse {} into {}", chain2, chain1);
        read_enter();
        game_manager
            .hotel_chain_manager
            .fuse_chains(chain1, chain2, &mut game_manager.board)?;
        player.analyze_cards(&game_manager.board, &game_manager.hotel_chain_manager);
        ui::print_main_ui(&game_manager);
    }
    Ok(())
}

pub fn set_hotel_chains_random(
    game_manager: &mut GameManager,
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
) -> Result<()> {
    for hotel_chain in HotelChain::iterator() {
        if rand::thread_rng().gen_bool(0.4) {
            continue;
        }
        let mut cards: Vec<Position> = Vec::new();
        for _i in 1..=20 {
            if rand::thread_rng().gen_bool(0.1) {
                break;
            }
            cards.push(game_manager.draw_card().unwrap());
        }
        for card in &cards {
            game_manager.board.place_hotel(&card)?;
        }
        if cards.len() < 2 {
            break;
        }
        game_manager.hotel_chain_manager.start_chain(
            *hotel_chain,
            cards,
            &mut game_manager.board,
            player,
            &mut game_manager.bank,
        )?;
        active_chains.push(*hotel_chain);
    }
    Ok(())
}

pub fn set_hotel_chains_clever(
    game_manager: &mut GameManager,
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
) -> Result<()> {
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
        println!(
            "Origin of chain {} is at {}",
            hotel_chain.name().color(hotel_chain.color()),
            origin.color(AnsiColors::Green)
        );
        game_manager.hotel_chain_manager.start_chain(
            *hotel_chain,
            positions,
            &mut game_manager.board,
            player,
            &mut game_manager.bank,
        )?;
        active_chains.push(*hotel_chain);
    }
    Ok(())
}

fn update_placed_hotels(
    chain: &HotelChain,
    new_hotels: &Vec<Position>,
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
    let mut positions = Vec::new();
    positions.push(origin);
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
        for (index, allowed_position) in allowed_positions.iter_mut().enumerate() {
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
        if placed_hotels.contains_key(&position.unwrap()) {
            if placed_hotels.get(&position.unwrap()).unwrap() != chain {
                println!(
                    "Unable to add to hotel at {} to chain {}: Illegal position",
                    origin, chain
                );
                return false;
            }
        }
    }
    return true;
}

pub fn draw_card(allowed_cards: &mut Vec<Position>) -> Position {
    let random_number = rand::thread_rng().gen_range(0..=allowed_cards.len() - 1);
    let position = allowed_cards.get(random_number).unwrap().clone();
    allowed_cards.remove(random_number);
    position
}
