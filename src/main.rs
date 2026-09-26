/// TUI-based Acquire.
///
/// The pure game engine lives in `core/`; this binary only parses the CLI flags, builds the
/// settings + roster, and hands control to the ratatui TUI.
mod bot;
/// Pure game engine with zero I/O
mod core;
/// TUI built with ratatui
mod tui;

use clap::{App as ClapApp, Arg};
use miette::Result;

use core::settings::Settings;
use tui::App;

fn main() -> Result<()> {
    let matches = ClapApp::new("Acquire_rs")
        .version("1.0.1")
        .author("LMH01")
        .about("The board game Acquire as a TUI in Rust (single-player vs bots)")
        .arg(
            Arg::new("players")
                .short('p')
                .long("players")
                .help("The total number of players (1 human + N-1 bots)")
                .value_name("NUMBER")
                .possible_values(&["2", "3", "4", "5", "6"])
                .default_value("4"),
        )
        .arg(
            Arg::new("hide_extra_info")
                .short('h')
                .long("hide-extra-info")
                .help("Hide the largest/second-largest shareholder markers")
                .long_help(
                    "Hide the ★/☆ markers that show whether you are the largest (★) or second \
                     largest (☆) shareholder of a chain.",
                ),
        )
        .arg(
            Arg::new("lan_client")
                .long("lan-client")
                .help("Join a LAN game (multiplayer placeholder)")
                .conflicts_with_all(&[
                    "hide_extra_info",
                    "players",
                    "skip_dialogues",
                    "lan_server",
                ]),
        )
        .arg(
            Arg::new("lan_server")
                .long("lan-server")
                .help("Host a LAN game (multiplayer placeholder)")
                .conflicts_with("lan_client"),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("The human player's name"),
        )
        .arg(
            Arg::new("ip")
                .long("ip")
                .help("The ip and port to connect to (e.g. 192.168.178.10:11511)")
                .requires("lan_client")
                .value_name("IP")
                .conflicts_with("lan_server"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .help("The port to host on (default 11511)")
                .requires("lan_server"),
        )
        .arg(
            Arg::new("info_card")
                .long("info-card")
                .help("Open the TUI directly on the stock info card"),
        )
        .arg(
            Arg::new("small_board")
                .short('s')
                .long("small-board")
                .help("Use a compact board layout"),
        )
        .arg(
            Arg::new("skip_dialogues")
                .long("skip-dialogues")
                .help("Auto-answer confirmation prompts"),
        )
        .arg(
            Arg::new("demo")
                .long("demo")
                .help("Open the TUI on a demo board")
                .conflicts_with_all(&["lan_client", "lan_server"]),
        )
        .arg(
            Arg::new("demo_type")
                .long("demo-type")
                .help("Demo type: 0 = clever, 1 = random")
                .default_value_if("demo", None, Some("0"))
                .requires("demo"),
        )
        .get_matches();

    let settings = Settings::new(
        matches.is_present("small_board"),
        matches.is_present("hide_extra_info"),
        matches.is_present("skip_dialogues"),
    );
    let human_name = matches
        .value_of("name")
        .map(String::from)
        .unwrap_or_else(|| String::from("You"));

    // Build the terminal once.
    let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)
        .map_err(|e| miette::miette!(format!("failed to initialize terminal: {e}")))?;

    // Dispatch: demo → lan → info-card → normal game.
    let mut app = if matches.is_present("demo") {
        let demo_type: u8 = matches
            .value_of("demo_type")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        App::new_demo(settings, human_name, demo_type)?
    } else if matches.is_present("lan_server") || matches.is_present("lan_client") {
        App::new_lan(settings, human_name)?
    } else if matches.is_present("info_card") {
        let mut app = App::new(settings.clone(), human_name, 4)?;
        app.overlay = Some(tui::Overlay::InfoCard);
        app
    } else {
        let players: u8 = matches
            .value_of("players")
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);
        App::new(settings, human_name, players)?
    };

    app.run(&mut terminal)
}
