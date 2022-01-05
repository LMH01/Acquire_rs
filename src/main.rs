/// Contains all base functionalities that the game needs to work.
mod base_game;
/// Contains functions that help to read and parse the user input
mod data_stream;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, game logic, management of players and more.
mod game;

use base_game::board::{Board, Letter, Position};
use base_game::hotel::Hotel;
use base_game::stock;
use clap::Parser;
use game::game::Game;
use miette::Result;

//TODO Make all fields that i made public in the struct private and provide getters and, where
//needed setters
//TODO See if i can remove the clone, copy trait from the hotel enum

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
    let mut game = Game::new(opts.players, opts.large_board)?;
    //game.start_game()?;
    test_things(game)?;
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

fn test_things(mut game: Game) -> Result<()> {
    //    game.board.place_hotel_debug(Position::new(Letter::A, 2), Hotel::Luxor);
    //    game.board.print();
    //        Board::print(&game.board);
    //        game.board.print();
    //    for position in &game.position_cards {
    //        game.board.place_hotel(*position)?;
    //    }
    //    place_test_hotels(&mut game.board)?;
    //    place_test_hotels(&mut game.board)?;
    game.board
        .place_hotel_debug(Position::new(Letter::B, 10), Hotel::Oriental)?;
    game.board
        .place_hotel_debug(Position::new(Letter::E, 9), Hotel::Continental)?;
    game.board.print();
    println!(
        "High Price Hotel with 40 Hotels: {}",
        stock::stock_price(base_game::hotel::PriceLevel::High, 40)
    );
    println!(
        "High Price Hotel with 41 Hotels: {}",
        stock::stock_price(base_game::hotel::PriceLevel::High, 41)
    );
    game.bank.buy_stock(&game.hotel_manager, &Hotel::Airport, game.players.get_mut(0).unwrap())?;
    Ok(())
}
