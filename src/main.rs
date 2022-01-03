/// Contains all base functionalities that the game needs to work.
mod base_game;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, management of players and more.
mod game;
/// Contains functions that help to read and parse the user input
mod data_stream;

use std::slice::SliceIndex;

use base_game::{Board, Hotel, Letter, Position};
use game::game::Game;
use miette::Result;

fn main() -> miette::Result<()> {
//    let mut board = Board::new();
//    board.print();
//    board.place_hotel(Position::new(Letter::E, 6))?;
//    board.print();
//    board.place_hotel_debug(Position::new(Letter::D, 2), Hotel::Airport)?;
//    place_test_hotels(&mut board)?;
//    board.print();
    print_welcome();
    println!("Please enter the number of players:");
    let number_of_players = data_stream::read_number()?;
    let mut game = Game::new(number_of_players)?;
    game.start_game()?;
    game.print_player_cards(1);
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

