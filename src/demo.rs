use std::collections::HashMap;

use clap::{App, ArgMatches};
use miette::Result;
use owo_colors::{AnsiColors, OwoColorize};
use rand::Rng;

use crate::{
    base_game::{
        bank::{self, Bank},
        board::{letter::LETTERS, Board, Position},
        hotel_chains::HotelChain,
        player::Player,
        settings::Settings,
        ui,
    },
    data_stream::read_enter,
    game::game::{
        self, final_account,
        hotel_chain_manager::{self, HotelChainManager},
        round::Round,
        GameManager,
    },
    logic::place_hotel::fuse_chains,
};

fn place_test_hotels(board: &mut Board) -> Result<()> {
    for (index, h) in HotelChain::iterator().enumerate() {
        board.place_hotel_debug(Position::new('A', index.try_into().unwrap()), *h)?;
    }
    Ok(())
}

pub fn test_things(matches: &ArgMatches, settings: Settings) -> Result<()> {
    let mut game_manager = GameManager::new(
        matches.value_of("players").unwrap().parse().unwrap(),
        settings,
    )?;
    let mut active_chains: Vec<HotelChain> = Vec::new();
    let round = Round::new(1);
    let mut player_cards = Vec::new();
    for _i in 1..=6 {
        player_cards.push(draw_card(&mut game_manager.position_cards));
    }
    let player = game_manager.players.get_mut(0).unwrap();
    if matches.value_of("demo_type").unwrap() == "0" {
        set_hotel_chains_clever(
            &mut active_chains,
            player,
            &mut game_manager.position_cards,
            &mut game_manager.board,
            &mut game_manager.hotel_chain_manager,
            &mut game_manager.bank,
        )?;
    } else {
        set_hotel_chains_random(
            &mut active_chains,
            player,
            &mut game_manager.position_cards,
            &mut game_manager.board,
            &mut game_manager.hotel_chain_manager,
            &mut game_manager.bank,
        )?;
    }
    game_manager
        .bank
        .update_largest_shareholders(&game_manager.players);
    game_manager.bank.print_largest_shareholders();
    let player = game_manager.players.get_mut(0).unwrap();
    player.analyze_cards(&game_manager.board, &game_manager.hotel_chain_manager);
    ui::print_main_ui_console(
        Some(&player),
        Some(&player.name),
        &game_manager.board,
        &game_manager.settings,
        Some(&round),
        &game_manager.bank,
        &game_manager.hotel_chain_manager,
    );
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
        ui::print_main_ui_console(
            Some(&player),
            Some(&player.name),
            &game_manager.board,
            &game_manager.settings,
            Some(&round),
            &game_manager.bank,
            &game_manager.hotel_chain_manager,
        );
    }
    Ok(())
}

pub fn set_hotel_chains_random(
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
    position_cards: &mut Vec<Position>,
    board: &mut Board,
    hotel_chain_manager: &mut HotelChainManager,
    bank: &mut Bank,
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
            cards.push(game::draw_card(position_cards)?.unwrap());
        }
        for card in &cards {
            board.place_hotel(&card)?;
        }
        if cards.len() < 2 {
            break;
        }
        hotel_chain_manager.start_chain(*hotel_chain, cards, board, player, bank)?;
        active_chains.push(*hotel_chain);
    }
    Ok(())
}

pub fn set_hotel_chains_clever(
    active_chains: &mut Vec<HotelChain>,
    player: &mut Player,
    position_cards: &mut Vec<Position>,
    board: &mut Board,
    hotel_chain_manager: &mut HotelChainManager,
    bank: &mut Bank,
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
        hotel_chain_manager.start_chain(*hotel_chain, positions, board, player, bank)?;
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
fn fuse_chains_works_with_three() -> Result<()> {
    let mut board = Board::new();
    let mut bank = Bank::new();
    let mut hotel_chain_manager = HotelChainManager::new();
    let mut players = vec![Player::new(vec![], 0, false), Player::new(vec![], 1, false)];
    let round = Round::new(1);
    let settings = Settings::new(false, false, false);
    hotel_chain_manager.start_chain(
        HotelChain::Imperial,
        vec![Position::new('E', 3), Position::new('E', 4)],
        &mut board,
        players.get_mut(0).unwrap(),
        &mut bank,
    )?;
    hotel_chain_manager.start_chain(
        HotelChain::Oriental,
        vec![Position::new('C', 5), Position::new('D', 5)],
        &mut board,
        players.get_mut(0).unwrap(),
        &mut bank,
    )?;
    hotel_chain_manager.start_chain(
        HotelChain::Airport,
        vec![Position::new('F', 5), Position::new('G', 5)],
        &mut board,
        players.get_mut(0).unwrap(),
        &mut bank,
    )?;
    hotel_chain_manager.start_chain(
        HotelChain::Prestige,
        vec![Position::new('E', 6), Position::new('E', 7)],
        &mut board,
        players.get_mut(0).unwrap(),
        &mut bank,
    )?;
    board.print(false);
    let chains = vec![
        HotelChain::Imperial,
        HotelChain::Oriental,
        HotelChain::Airport,
        HotelChain::Prestige,
    ];
    let origin = Position::new('E', 5);
    board.place_hotel(&origin)?;
    fuse_chains(
        chains,
        origin,
        0,
        &mut players,
        &mut board,
        &mut bank,
        &mut hotel_chain_manager,
        &round,
        &settings,
    )?;
    board.print(false);
    final_account(&mut players, &mut bank, &hotel_chain_manager)?;
    Ok(())
}
