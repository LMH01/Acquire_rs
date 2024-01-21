use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::game::base::{BoardSize, self};

use super::App;

/// Draw the current ui
pub fn draw(f: &mut Frame, app: &mut App) {
    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(99), Constraint::Percentage(1)])
        .split(f.size());

    // everything except for the keybind hints
    let base_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(global_chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(base_chunks[0]);
    
    let right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(5), Constraint::Percentage(20), Constraint::Percentage(45)])
        .split(base_chunks[1]);

    let key_hints = Tabs::new(vec!["a", "b", "c"])
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));

    f.render_widget(key_hints, global_chunks[1]);

    let board_block = Block::new().title("Board").borders(Borders::ALL);
    
    //f.render_widget(app.game.board.to_paragraph(BoardSize::Large).block(board_block), left_chunks[0])
    
    if left_chunks[0].height < 20 || left_chunks[0].width < 60 {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Small).block(board_block), left_chunks[0])
    } else if left_chunks[0].height >= 20 && left_chunks[0].height <= 37 {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Medium).block(board_block), left_chunks[0])
    } else {
        f.render_widget(app.game.board.to_paragraph(BoardSize::Large).block(board_block), left_chunks[0])
    }
}
