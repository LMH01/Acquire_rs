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
use clap::{App, Arg};
use demo::test_things;
use game::{print_info_card, GameManager};
use network::{start_client, start_server};

//TODO Add flag with which the help card can be enabled. This will cause to print a copy of the
//info card from the real game to the console
//TODO Change $ to €

fn main() -> miette::Result<()> {
    let matches = App::new("Acquire_rs")
        .version("0.1.0")
        .author("LMH01")
        .about("The board game Acquire fia command line in Rust")
        .arg(Arg::new("players")
            .short('p')
            .long("players")
            .help("The number of players")
            .value_name("NUMBER")
            .possible_values(["2", "3", "4", "5", "6"])
            .required_unless_present_any(&["lan_client", "demo", "demo_type", "info_card"])
            .default_value_if("demo", None, Some("2")))
        .arg(Arg::new("hide_extra_info")
            .short('h')
            .long("hide-extra-info")
            .help("Use to hide additional information to the player")
            .long_help("Use to hide additional information to the player. This will hide information that the player would normally have to give the game more variation. The following is hidden:\n - Who the largest or second largest shareholders are\nWhen this flag is not set a little star next to your bought stocks displayes if you are the largest (golden star) or second largest shareholder for that chain (silver star)."))
        .arg(Arg::new("lan_client")
            .long("lan-client")
            .help("Use to play the game on multiplayer per lan and join a server")
            .conflicts_with_all(&["hide_extra_info", "players", "skip_dialogues", "lan_server"]))
        .arg(Arg::new("lan_server")
            .long("lan-server")
            .help("Start the game as server")
            .conflicts_with_all(&["lan_client"]))
        .arg(Arg::new("name")
            .short('n')
            .long("name")
            .help("The name of the player")
            .long_help("The name of the player. This can also be used to set the player name of the player that hosts the game.")
            .takes_value(true)
            .requires("lan_server"))
        .arg(Arg::new("ip")
            .long("ip")
            .help("The ip and port to which to connect")
            .long_help("The ip and port to wich to connect. Example: 192.168.178.10:11511")
            .requires("lan_client")
            .takes_value(true)
            .value_name("IP")
            .conflicts_with("lan_server"))
        .arg(Arg::new("port")
            .long("port")
            .help("Overwrite the port at wich the game should be hosted")
            .long_help("Overwrite the port at wich the game should be hosted\nDefault is 11511")
            .default_value_if("lan_server", None, Some("11511"))
            .requires("lan_server")
            )
        .arg(Arg::new("info_card")
            .long("info-card")
            .help("Print the stock info card")
            .long_help("Print the stocks info card. This card displayes information on how much a stock is worth depending on the length of the hotel chain")
            .exclusive(true))
        .arg(Arg::new("small_board")
            .short('s')
            .long("small-board")
            .help("Use to make the board smaller"))
        .arg(Arg::new("skip_dialogues")
            .long("skip-dialogues")
            .help("Use to always skip some dialogues")
            .long_help("Use to always skip some dialogues. Dialogues that are skipped include: The confirmation what card the player drew."))
        .arg(Arg::new("demo")
            .long("demo")
            .help("Use to run some demo on how the game looks like instead of the main game")
            .conflicts_with_all(&["lan_client", "lan_server"]))
        .arg(Arg::new("demo_type")
            .long("demo-type")
            .help("Set what demo type to run")
            .default_value_if("demo", None, Some("0"))
            .requires("demo"))
        .get_matches();
    set_terminal_output();
    print_welcome();
    let settings = Settings::new(
        matches.is_present("small_board"),
        matches.is_present("hide_extra_info"),
        matches.is_present("skip_dialogues"),
    );
    if matches.is_present("demo") {
        test_things(&matches, settings)?;
    } else if matches.is_present("lan_server") {
        start_server(&matches, settings)?;
    } else if matches.is_present("lan_client") {
        start_client(&matches)?;
    } else if matches.is_present("info_card") {
        print_info_card();
    } else {
        let mut game_manager = GameManager::new(
            matches.value_of("players").unwrap().parse().unwrap(),
            settings,
        )?;
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
