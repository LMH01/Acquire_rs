/// Contains all base functionalities that the game needs to work.
/// This includes all basic data types and the playfield.
/// This module does not contain any logic related to the gameplay.
mod base_game;
/// Contains functions that help to read and parse the user input
mod data_stream;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, game logic, management of players and more.
mod game;

use std::slice::SliceIndex;

use base_game::board::{Board, Letter, Position};
use base_game::hotel::Hotel;
use base_game::{stock, ui};
use clap::Parser;
use game::game::round::Round;
use game::game::GameManager;
use miette::Result;
use rand::{random, Rng};

use crate::base_game::board::Piece;
use crate::base_game::player::Player;
use crate::base_game::ui::print_main_ui;
use crate::data_stream::read_enter;
//TODO Review struct fields in base_game.rs and decide if it would be a better idea to
//make them public. Also remove the getters/setters
//TODO Start with gameplay
//While doing that add functionality that stores the currently largest and second largest shareholder
//TODO See if i can remove the clone, copy trait from the hotel enum
//TODO Add flag with which the help card can be enabled. This will cause to print a copy of the
//info card from the real game to the console

#[derive(Parser)]
#[clap(about = "The board game Acquire fia command line in Rust")]
struct Opts {
    #[clap(short, long, help = "The number of players", possible_values = ["2", "3", "4", "5", "6"], required = true)]
    players: u32,
    #[clap(
        short,
        long,
        help = "Use to make the board larger and to write the coordinates into the field"
    )]
    large_board: bool,
    #[clap(long, help = "Use to run the test function instead of the main game")]
    test: bool,
}

fn main() -> miette::Result<()> {
    let opts = Opts::parse();
    //        let mut board = Board::new();
    //        board.print();
    //        board.place_hotel(Position::new(Letter::E, 6))?;
    //        board.print();
    //        board.place_hotel_debug(Position::new(Letter::D, 2), Hotel::Airport)?;
    //        place_test_hotels(&mut board)?;
    //        board.print();
    print_welcome();
    let mut game_manager = GameManager::new(opts.players, opts.large_board)?;
    if opts.test {
        test_things(game_manager)?;
    } else {
        game_manager.start_game()?;
    }
    Ok(())
}

fn print_welcome() {
    println!("Welcome to the Game Acquire!");
}

fn place_test_hotels(board: &mut Board) -> Result<()> {
    for (index, h) in Hotel::iterator().enumerate() {
        board.place_hotel_debug(Position::new(Letter::A, index.try_into().unwrap()), *h)?;
    }
    Ok(())
}

fn test_things(mut game_manager: GameManager) -> Result<()> {
    game_manager.round = Some(Round::new());
    let mut active_chains: Vec<Hotel> = Vec::new();
    for hotel_chain in Hotel::iterator() {
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
        game_manager.hotel_manager.start_chain(
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
        active_chains.push(*hotel_chain);
    }
    ui::print_main_ui(&game_manager);
    if active_chains.len() >= 2 {
        let rand1 = rand::thread_rng().gen_range(0..=active_chains.len()-1);
        let mut rand2 = rand::thread_rng().gen_range(0..=active_chains.len()-1);
        while rand1 == rand2 {
            rand2 = rand::thread_rng().gen_range(0..=active_chains.len()-1);
        }
        let chain1 = active_chains.get(rand1).unwrap();
        let chain2 = active_chains.get(rand2).unwrap();
        println!("Press enter to fuse {} into {}", chain2, chain1);
        read_enter();
        game_manager.hotel_manager.fuse_chains(chain1, chain2, &mut game_manager.board)?;
        ui::print_main_ui(&game_manager);
    }
    Ok(())
}
