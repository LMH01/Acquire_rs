use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::Backend, Terminal};

use crate::game::base::{Board, Game, HotelChain};

use self::ui::draw;

/// Drawing of the ui
mod ui;

/// App holds the state of the application
pub struct App {
    game: Game,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: Game {
                board: Board::default(),
            },
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| draw(f, self))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('n') => {
                        self.game.board.pieces[1][2].chain = Some(HotelChain::Airport);
                    }
                    _ => (),
                }
            }
        }
    }
}
