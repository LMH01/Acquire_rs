use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::game::base::BoardSize;

use super::App;

/// Draw the current ui
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    let key_hints = Tabs::new(vec!["a", "b", "c"])
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));

    f.render_widget(key_hints, chunks[0]);
    
    if chunks[1].height < 20 || chunks[1].width < 60 {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Small), chunks[1])
    } else if chunks[1].height >= 20 && chunks[1].height <= 35 {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Medium), chunks[1])
    } else {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Large), chunks[1])
    }
}
