use std::{
    io::{self, stdin, stdout, BufRead, BufReader, Write},
    net::{IpAddr, SocketAddrV4, TcpListener, TcpStream},
    str, thread, time,
};

use clap::ArgMatches;
use local_ip_address::local_ip;
use miette::{miette, IntoDiagnostic, Result};
use owo_colors::{OwoColorize, AnsiColors};

use crate::{
    base_game::{player::Player, settings::Settings},
    data_stream::read_enter,
    game::GameManager,
};

/// Starts a client of the game.
/// The client will act upon recieving command.
///
/// The commands that the client understands are the following:
/// * `$Print` - Prints the text
/// * `$Println` - Prints the text with new line
/// * `$Input` - Prompts the user to input something
///
/// Everything emidiadly after the command is printed to the player.
/// A message always ends with `\n`.
pub fn start_client(matches: &ArgMatches) -> Result<()> {
    let stdin = io::stdin();
    let ip;
    if matches.is_present("ip") {
        ip = String::from(matches.value_of("ip").unwrap());
    } else {
        // Ip was not privided fia command line
        let mut buffer = String::new();
        print!("Enter ip and port: ");
        stdout().flush().into_diagnostic()?;
        stdin.read_line(&mut buffer).into_diagnostic()?;
        ip = String::from(buffer.trim());
    }
    println!("Connecting to {}...", &ip);
    match TcpStream::connect(ip) {
        Ok(mut tcp_stream) => {
            println!("Connection established!");
            let name;
            if matches.is_present("name") {
                name = String::from(matches.value_of("name").unwrap().trim());
            } else {
                print!("Enter name: ");
                stdout().flush().into_diagnostic()?;
                let mut buffer = String::new();
                stdin.read_line(&mut buffer).into_diagnostic()?;
                name = buffer.trim().to_string();
            }
            tcp_stream
                .write_all(
                    format!("$Init{}$Name{}\n", matches.is_present("small_board"), name).as_bytes(),
                )
                .into_diagnostic()?;
            println!("Waiting for the game to start...");

            let mut br = BufReader::new(tcp_stream.try_clone().into_diagnostic()?);
            // Player recieving loop
            loop {
                let stdin = io::stdin();
                let mut input_buffer = String::new();
                br.read_line(&mut input_buffer).into_diagnostic()?;
                if input_buffer.starts_with("$Println") {
                    let mut to_print = input_buffer.replacen("$Println", "", 1);
                    to_print.pop();
                    println!("{}", to_print);
                } else if input_buffer.starts_with("$Print") {
                    let mut to_print = input_buffer.replacen("$Print", "", 1);
                    to_print.pop();
                    print!("{}", to_print);
                } else if input_buffer.starts_with("$Input") {
                    let mut to_print = input_buffer.replacen("$Input", "", 1);
                    to_print.pop();
                    print!("{}", to_print);
                    stdout().flush().into_diagnostic()?;
                    let mut output_buffer = String::new();
                    stdin.read_line(&mut output_buffer).into_diagnostic()?;
                    let output = output_buffer;
                    tcp_stream.write_all(output.as_bytes()).into_diagnostic()?;
                } else if input_buffer.starts_with("$Ping") {
                    let _buffer = input_buffer.replacen("$Ping", "", 0);
                    tcp_stream
                        .write_all("$Here\n".as_bytes())
                        .into_diagnostic()?;
                } else if input_buffer.starts_with("$TERMINATE") {
                    let reason = input_buffer.replacen("$TERMINATE", "", 1);
                    println!("{}", "Game has been canceled!".color(AnsiColors::Red));
                    println!("Reason: {}", reason);
                    break;
                } else if input_buffer.starts_with("$GameEnded") {
                    break;
                } else {
                    // This is for now a work around until i can figgure out, how i can make the
                    // process sleep until new date is comming in.
                    thread::sleep(time::Duration::from_millis(100));
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

/// Symbolizes a client player.
pub struct ClientPlayer {
    pub name: String,
    pub tcp_stream: TcpStream,
    pub small_board: bool,
}

impl ClientPlayer {
    fn new(name: String, tcp_stream: TcpStream, small_board: bool) -> Self {
        Self {
            name,
            tcp_stream,
            small_board,
        }
    }
}

/// Starts the server to play the game on multiplayer per lan.
pub fn start_server(matches: &ArgMatches, settings: Settings) -> Result<()> {
    let local_ip = local_ip().unwrap();
    let local_ip = match local_ip {
        IpAddr::V4(ip4) => ip4,
        IpAddr::V6(ip6) => {
            return Err(miette!(
                "Unable to resolve local ip address. Ip should be ipv4 but was ipv6: {}",
                ip6
            ))
        }
    };
    let port = matches.value_of("port").unwrap();
    let socket = SocketAddrV4::new(local_ip, port.parse().into_diagnostic()?);
    let listener = TcpListener::bind(socket).into_diagnostic()?;
    let port = listener.local_addr().into_diagnostic()?;
    println!("Game has been hosted on {}", port);
    println!(
        "The game can be stared when {} more player(s) have connected.",
        matches.value_of("players").unwrap().parse::<u32>().unwrap() - 1
    );
    let mut client_players = Vec::new();
    // Number of players determines how many clients can connect to the game.
    // When the last client has been connected the host player can start the game.
    for i in 1..=matches.value_of("players").unwrap().parse::<u32>().unwrap() - 1 {
        let (tcp_stream, addr) = listener.accept().into_diagnostic()?;
        let mut br = BufReader::new(tcp_stream.try_clone().into_diagnostic()?);
        let mut input_buffer = String::new();
        br.read_line(&mut input_buffer).into_diagnostic()?;
        if input_buffer.starts_with("$Init") {
            let input = input_buffer.replacen("$Init", "", 1);
            let mut splits = input.splitn(2, "$Name");
            let small_board = matches!(splits.next().unwrap(), "true");
            let name = splits.next().unwrap().trim();
            println!("{} joined from {}!", name, addr);
            client_players.push(ClientPlayer::new(
                String::from(name),
                tcp_stream,
                small_board,
            ));
        }
        let remaining_players =
            matches.value_of("players").unwrap().parse::<u32>().unwrap() - 1 - i;
        if remaining_players > 0 {
            println!(
                "The game can be stared when {} more player(s) have connected.",
                remaining_players
            );
        }
    }
    // All players have connected to the game, game will start
    println!("Setting up game...");
    let host_name;
    if matches.is_present("name") {
        host_name = String::from(matches.value_of("name").unwrap());
    } else {
        let mut buffer = String::new();
        print!("Please enter your name: ");
        stdout().flush().into_diagnostic()?;
        stdin().read_line(&mut buffer).into_diagnostic()?;
        host_name = String::from(buffer.trim());
    }
    let mut game_manager = GameManager::new_server(client_players, settings, host_name)?;
    println!("Game has been setup.");
    println!("Press enter to start the game!");
    read_enter();
    if let Err(err) = game_manager.start_game() {
        // Some error occured because of which the game is canceled
        println!("{}", "An unrecoverable error occured, the game is canceled!".color(AnsiColors::Red));
        abort_game(&game_manager.players, err.to_string());
        println!("Reason the game had to be canceled:");
        return Err(err);
    }
    // game is over, stream will be closed
    for player in game_manager.players {
        if player.tcp_stream.is_some() {
            player
                .tcp_stream
                .unwrap()
                .shutdown(std::net::Shutdown::Both)
                .into_diagnostic()?;
        }
    }
    Ok(())
}

/// Send a message to every player (including the local player).
/// If the game is only played local the message is only written once to the console.
/// # Returns
/// * `Ok(())` - When the message was send successfully
/// * `Err(err)` - When the mesage could not be sent to at least one player
pub fn broadcast(message: &str, players: &[Player]) -> Result<()> {
    let mut written_to_console = false;
    for player in players {
        if player.tcp_stream.is_none() {
            if !written_to_console {
                player.print_text_ln(message)?;
                written_to_console = true;
            }
        } else {
            player.print_text_ln(message)?;
        }
    }
    Ok(())
}

/// Send a message to every player except for the player that currently has their turn.
/// If the game is only played local the message is only written once to the console.
/// # Returns
/// * `Ok(())` - When the message was send successfully
/// * `Err(err)` - When the message was not sent to at least one player
pub fn broadcast_others(
    message: &str,
    current_player_name: &str,
    players: &[Player],
) -> Result<()> {
    for player in players {
        if player.name != *current_player_name {
            player.print_text_ln(message)?;
        }
    }
    Ok(())
}

/// Sends a string to the client.
/// The text is split at `\n`. These slices are send individually.
/// # Returns
/// * `Ok(())` - When the string was send successfully
/// * `Err(err)` - When the string could not be sent
pub fn send_string(player: &Player, text: &str, command: &str) -> Result<()> {
    let mut stream = player.tcp_stream.as_ref().unwrap();
    let text = String::from(text);
    let text = text.split('\n');
    for split in text {
        let mut out = String::new();
        out.push_str(command);
        out.push_str(split);
        out.push('\n');
        if let Err(err) = stream.write_all(out.as_bytes()) {
            return Err(miette!(
                "Unable to send data to player {}: {}",
                player.name,
                err
            ));
        }
    }
    Ok(())
}

/// Sends a message to each player that the game is canceled
pub fn abort_game(players: &[Player], reason: String) {
    // Message players and terminate game
    for player in players {
        if player.tcp_stream.is_some() {
            if let Ok(()) = send_string(
                player,
                &(&reason).color(AnsiColors::Red).to_string(),
                "$TERMINATE",
            ) {
                println!("Stop command has been sent to {}", &player.name);
            }
        }
    }
}
