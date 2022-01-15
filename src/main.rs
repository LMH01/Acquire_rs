/// Contains all base functionalities that the game needs to work.
/// This includes all basic data types and the playfield, some game logic and more.
mod base_game;
/// Contains functions that help to read and parse the user input
mod data_stream;
/// Contains some code to print the board without that the game has to be started
mod demo;
/// Contains all functionalities that are required to play the game. This includes the setting up
/// of new games, round, turn and player managemnt and more.
mod game;
/// Contains the most part of the game logic.
/// Does not contain the logic of the different managers. Their logic is implemented in their main impl block.
mod logic;
/// Contains all functionalities required to play the game fia lan.
mod network;
/// Contains some functions that dont fit in another module.
mod utils;

use base_game::settings::Settings;
use clap::Parser;
use demo::test_things;
use game::game::GameManager;
use network::{start_client, start_server};

//TODO Add flag with which the help card can be enabled. This will cause to print a copy of the
//info card from the real game to the console
//TODO Change $ to €
//TODO Add: Player can rerol their hand cards if all cards are unplayable because of illegal fusion
//TODO Investigate leaderboard print wrong. (But the message to the players on who won is correct)

#[derive(Parser)]
#[clap(about = "The board game Acquire fia command line in Rust")]
pub struct Opts {
    #[clap(short, long, help = "The number of players", possible_values = ["2", "3", "4", "5", "6"], value_name = "NUMBER")]
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
    #[clap(
        long,
        help = "Use to run some demo on how the game looks like instead of the main game"
    )]
    demo: bool,
    #[clap(long, help = "Set what demo type to run", default_value = "0")]
    demo_type: u32,
    #[clap(
        short,
        long,
        help = "Use to always skip some dialogues",
        long_help = "Use to always skip some dialogues. Dialogues that are skipped include: The confirmation what card the player drew."
    )]
    skip_dialogues: bool,
    //    #[clap(long, help = "Use to play the game on multiplayer per lan and join a server", value_name = "IP", required = false, conflicts_with_all = &["demo", "demo-type", "extra-info", "lan-server", "large-board", "skip-dialogues"])]
    //    lan_client: String,
    //    #[clap(long, help = "Use to play the game on multiplayer per lan and host the server", conflicts_with_all = &["demo", "demo-type", "lan-client", "skip-dialogues"])]
    //    lan_server: bool,
    //    #[clap(subcommand)]
    //    lan: Lan,
    #[clap(long)]
    lan_client: bool,
    #[clap(long)]
    lan_server: bool,
}

//#[derive(Parser)]
//enum Lan {
//    Client,
//    Server,
//}

fn main() -> miette::Result<()> {
    let opts = Opts::parse();
    set_terminal_output();
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
    } else if opts.lan_server {
        start_server(&opts, settings)?;
    } else if opts.lan_client {
        start_client();
    } else {
        let mut game_manager = GameManager::new(opts.players, settings)?;
        game_manager.start_game()?;
    }
    Ok(())
}

fn print_welcome() {
    println!("Welcome to the Game Acquire!");
}

// If the os is windows the virtual terminal will be set to true

#[cfg(windows)]
fn set_terminal_output() {
    colored::control::set_virtual_terminal(true).unwrap();
}

#[cfg(unix)]
fn set_terminal_output() {
    // Nothing additional has to be setup
}
