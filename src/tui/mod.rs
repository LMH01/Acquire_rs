use crossterm::event::{Event, self, KeyCode};
use ratatui::{backend::Backend, Terminal};
use anyhow::Result;

use self::ui::draw;

/// Drawing of the ui
mod ui;

/// App holds the state of the application
pub struct App {

}

impl App {

    pub fn new() -> Self {
        Self {}
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| draw(f, self))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    },
                    _ => (),
                }
            }
        }
    }
}