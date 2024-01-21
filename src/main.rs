use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::tui::App;

/// All data types and logic of the game
mod game;
/// Terminal user interface
mod tui;

fn main() {
    // tui
    // setup terminal
    println!("Ready to run, launching tui");
    enable_raw_mode().expect("Unable to enable raw mode!");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
    let mut app = App::new();
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode().expect("Unable to disable raw mode!");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
    res.unwrap();
}
