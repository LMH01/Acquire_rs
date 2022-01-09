/// Contains all base functionalities that the game needs to work.
/// This includes all basic data types and the playfield, some game logic and more.
mod base_game;
/// Contains functions that help to read and parse the user input
mod data_stream;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, round, turn and player managemnt and more.
mod game;
/// Contains the most part of the game logic.
/// Does not contain the logic of the different managers. Their logic is implemented in their main impl block.
mod logic;
/// Contains some code to print the board without that the game has to be started
mod demo;

use std::slice::SliceIndex;

use base_game::board::letter::LETTERS;
use base_game::board::{Board, Position};
use base_game::hotel_chains::HotelChain;
use base_game::settings::Settings;
use base_game::{stock, ui};
use clap::Parser;
use demo::test_things;
use game::game::round::Round;
use game::game::GameManager;
use miette::{miette, Result};
use rand::{random, Rng};

use crate::base_game::board::Piece;
use crate::base_game::player::Player;
use crate::base_game::ui::print_main_ui;
use crate::data_stream::read_enter;
use crate::game::game::hotel_chain_manager;
//TODO Review struct fields in base_game.rs and decide if it would be a better idea to
//make them public. Also remove the getters/setters
//TODO Start with gameplay
//While doing that add functionality that stores the currently largest and second largest shareholder
//  -> Continue working on that!
//  Maybe rename LargestShareholders to MajorityShareholders
//TODO See if i can remove the clone, copy trait from the hotel enum
//TODO Add flag with which the help card can be enabled. This will cause to print a copy of the
//info card from the real game to the console

#[derive(Parser)]
#[clap(about = "The board game Acquire fia command line in Rust")]
pub struct Opts {
    #[clap(short, long, help = "The number of players", possible_values = ["2", "3", "4", "5", "6"], required = true)]
    players: u32,
    #[clap(
        short,
        long,
        help = "Use to make the board larger and to write the coordinates into the field"
    )]
    large_board: bool,
    #[clap(
        short,
        long,
        help = "Use to show additional information to the player.",
        long_help = "Use to show additional information to the player. This will show information that the player would normally not have. The following is shown:\n - If the player is largest or second largest shareholder"
    )]
    extra_info: bool,
    #[clap(long, help = "Use to run some demo on how the game looks like instead of the main game")]
    demo: bool,
    #[clap(long, help = "Set what demo type to run", requires = "demo", default_value = "0")]
    demo_type: u32,
    #[clap(
        short,
        long,
        help = "Use to always skip some dialogues",
        long_help = "Use to always skip some dialogues. Dialogues that are skipped include: The confirmation what card the player drew."
    )]
    skip_dialogues: bool,
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
    let settings = Settings::new(opts.large_board, opts.extra_info, opts.skip_dialogues);
    if opts.demo {
        test_things(&opts, settings)?;
    } else {
        let mut game_manager = GameManager::new(opts.players, settings)?;
        game_manager.start_game()?;
    }
    Ok(())
}

fn print_welcome() {
    println!("Welcome to the Game Acquire!");
}
