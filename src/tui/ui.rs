use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

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
}
