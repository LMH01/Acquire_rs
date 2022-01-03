/// Contains all base functionalities that the game needs to work.
mod base_game;
/// Contains functions that help to read and parse the user input
mod data_stream;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, management of players and more.
mod game;

use base_game::{Board, Hotel, Letter, Position};
use game::game::Game;
use miette::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(about = "The board game Acquire fia command line in Rust")]
struct Opts {
    #[clap(short, long, help = "The number of players", possible_values = ["2", "3", "4", "5", "6"], required = true)]
    players: u8,
}


fn main() -> miette::Result<()> {
    let opts = Opts::parse();
    //    let mut board = Board::new();
    //    board.print();
    //    board.place_hotel(Position::new(Letter::E, 6))?;
    //    board.print();
    //    board.place_hotel_debug(Position::new(Letter::D, 2), Hotel::Airport)?;
    //    place_test_hotels(&mut board)?;
    //    board.print();
    print_welcome();
    let mut game = Game::new(opts.players)?;
    game.start_game()?;
    //Board::print(&game.board);
    //    game.board.print();
    //    for position in game.position_cards {
    //        game.board.place_hotel(position)?;
    //    }
    //    game.board.print();
    Ok(())
}

fn print_welcome() {
    println!("Welcome to the Game Acquire!");
}

fn place_test_hotels(board: &mut Board) -> Result<()> {
    for (index, h) in Hotel::iterator().enumerate() {
        board.place_hotel_debug(Position::new(Letter::A, u8::try_from(index).unwrap()), *h)?;
    }
    Ok(())
}
