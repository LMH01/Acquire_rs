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
use game::game::GameManager;
use miette::Result;
use rand::{random, Rng};

use crate::base_game::board::Piece;
use crate::base_game::ui::print_main_ui;
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
        test_things(game_manager);
    } else {
        game_manager.start_game();
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
    //    game.board.place_hotel_debug(Position::new(Letter::A, 2), Hotel::Luxor);
    //    game.board.print();
    //        Board::print(&game.board);
    //        game.board.print();
    //    for position in &game.position_cards {
    //        game.board.place_hotel(*position)?;
    //    }
    //    place_test_hotels(&mut game.board)?;
    //    place_test_hotels(&mut game.board)?;
    game_manager
        .board
        .place_hotel_debug(Position::new(Letter::B, 10), Hotel::Oriental)?;
    game_manager
        .board
        .place_hotel_debug(Position::new(Letter::E, 9), Hotel::Continental)?;
    game_manager.board.print();
    println!(
        "High Price Hotel with 40 Hotels: {}",
        stock::stock_price(base_game::hotel::PriceLevel::High, 40)
    );
    println!(
        "High Price Hotel with 41 Hotels: {}",
        stock::stock_price(base_game::hotel::PriceLevel::High, 41)
    );
    //    game_manager.bank.buy_stock(
    //        &game_manager.hotel_manager,
    //        &Hotel::Airport,
    //        game_manager.players.get_mut(0).unwrap(),
    //    )?;
    game_manager.start_game()?;
    for hotel in Hotel::iterator() {
        if rand::thread_rng().gen_bool(0.4) {
            continue;
        }
        game_manager.hotel_manager.set_hotel_status(&hotel, true);
        let random_number = rand::thread_rng().gen_range(2..=41);
        game_manager
            .hotel_manager
            .add_hotel_buildings(hotel, random_number)
            .unwrap();
        for player in &mut game_manager.players {
            for i in 1..=rand::thread_rng().gen_range(1..=5) {
                if game_manager
                    .bank
                    .buy_stock(&game_manager.hotel_manager, hotel, player)
                    .is_err()
                {
                    //Stock could not be bought
                };
            }
        }
    }
    print_main_ui(&game_manager);
    Ok(())
}
