use std::{
    io::{self, stdout, BufRead, BufReader, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    process::exit,
    str, thread, time,
};

use local_ip_address::local_ip;
use miette::{miette, IntoDiagnostic, Result};

use crate::{
    base_game::{player::Player, settings::Settings},
    data_stream::read_enter,
    game::game::GameManager,
    Opts,
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
pub fn start_client() -> Result<()> {
    let mut port = String::new();
    let stdin = io::stdin();

    //    print!("Enter port: ");
    //    stdout().flush().into_diagnostic()?;
    //    stdin.read_line(&mut port).into_diagnostic()?;
    //    let mut ip = String::new();
    //    ip.push_str("192.168.188.5:");
    //    ip.push_str(&port.trim());
    let ip = String::from("192.168.188.5:11511");

    //TODO Add these lines back when i am no longer testing
    //    print!("Enter ip: ");
    //    stdout().flush().into_diagnostic()?;
    //    stdin.read_line(&mut port).into_diagnostic()?;
    //    let mut ip = String::new();
    //    ip.push_str(&port.trim());
    match TcpStream::connect(ip) {
        Ok(mut tcp_stream) => {
            println!("Connection established!");
            //TODO Player name should be transmitted fia cli argument
            print!("Enter name: ");
            stdout().flush().into_diagnostic()?;
            let mut buffer = String::new();
            stdin.read_line(&mut buffer).into_diagnostic()?;
            buffer = buffer.trim().to_string();
            tcp_stream
                .write_all(format!("$PlayerName{}\n", buffer).as_bytes())
                .into_diagnostic()?;
            println!("Waiting for the game to start...");

            let mut br = BufReader::new(tcp_stream.try_clone().into_diagnostic()?);
            // Player recieving loop
            loop {
                let mut input_buffer = String::new();
                br.read_line(&mut input_buffer).into_diagnostic()?;
                if input_buffer.starts_with("$Println") {
                    let mut to_print = String::from(input_buffer.replacen("$Println", "", 1));
                    to_print.pop();
                    println!("{}", to_print);
                } else if input_buffer.starts_with("$Print") {
                    let mut to_print = String::from(input_buffer.replacen("$Print", "", 1));
                    to_print.pop();
                    print!("{}", to_print);
                } else if input_buffer.starts_with("$Input") {
                    let mut to_print = String::from(input_buffer.replacen("$Input", "", 1));
                    to_print.pop();
                    print!("{}", to_print);
                    stdout().flush().into_diagnostic()?;
                    let mut output_buffer = String::new();
                    stdin.read_line(&mut output_buffer).into_diagnostic()?;
                    let output = output_buffer;
                    tcp_stream.write_all(output.as_bytes()).into_diagnostic()?;
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
}

impl ClientPlayer {
    fn new(name: String, tcp_stream: TcpStream) -> Self {
        Self { name, tcp_stream }
    }
}

/// Starts the server to play the game on multiplayer per lan.
pub fn start_server(opts: &Opts, settings: Settings) -> Result<()> {
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
    let socket = SocketAddrV4::new(local_ip, 11511);
    let listener = TcpListener::bind(socket).into_diagnostic()?;
    let port = listener.local_addr().into_diagnostic()?;
    println!("Game has been hosted on {}", port);
    println!(
        "The game can be stared when {} player(s) have connected.",
        opts.players - 1
    );
    let mut client_players = Vec::new();
    // Number of players determines how many clients can connect to the game.
    // When the last client has been connected the host player can start the game.
    for i in 1..=opts.players - 1 {
        let (tcp_stream, addr) = listener.accept().into_diagnostic()?;
        let mut br = BufReader::new(tcp_stream.try_clone().into_diagnostic()?);
        let mut input_buffer = String::new();
        br.read_line(&mut input_buffer).into_diagnostic()?;
        if input_buffer.starts_with("$PlayerName") {
            let name = String::from(input_buffer.replacen("$PlayerName", "", 1).trim());
            println!("{} joined from {}!", name, addr);
            client_players.push(ClientPlayer::new(name, tcp_stream));
        }
        println!(
            "The game can be stared when {} more player(s) have connected.",
            opts.players - 1 - i
        );
    }
    // All players have connected to the game, game will start
    println!("Setting up game...");
    let mut game_manager = GameManager::new_server(client_players, settings)?;
    println!("Game has been setup.");
    println!("Press enter to start the game!");
    read_enter();
    game_manager.start_game()?;
    // game is over, stream will be closed
    for player in game_manager.players {
        if player.tcp_stream.is_some() {
            player.tcp_stream.unwrap().shutdown(std::net::Shutdown::Both).into_diagnostic()?;
        }
    }
    Ok(())
}

/// Send a message to every player (including the local player).
/// If the game is only played local the message is only written once to the console.
pub fn broadcast(message: &str, players: &Vec<Player>) {
    let mut written_to_console = false;
    for player in players {
        if player.tcp_stream.is_none() {
            if !written_to_console {
                player.print_text_ln(message);
                written_to_console = true;
            }
        } else {
            player.print_text_ln(message);
        }
    }
}

/// Send a message to every player except for the player that currently has thair turn.
/// If the game is only played local the message is only written once to the console.
pub fn broadcast_others(message: &str, current_player_name: &String, players: &Vec<Player>) {
    for player in players {
        if player.name != *current_player_name {
            player.print_text_ln(message);
        }
    }
}

/// Sends a string to the client.
/// The text is split at `\n`. These slices are send individually.
pub fn send_string(player: &Player, text: &str, command: &str) {
    let mut stream = player.tcp_stream.as_ref().unwrap();
    let text = String::from(text);
    let text = text.split("\n");
    for split in text {
        let mut out = String::new();
        out.push_str(command);
        out.push_str(split);
        out.push_str("\n");
        if let Err(err) = stream.write_all(out.as_bytes()) {
            println!("Unable to send data to player {}: {}", player.name, err);
        }
    }
}
