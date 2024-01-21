use std::{cmp::Ordering, fmt::Display};

use anyhow::Result;
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Row, Table, Widget},
};

/// Contains all variables required to play a game.
pub struct Game {
    pub board: Board,
}

/// The board on which the hotels are placed.
///
/// Stores currently placed hotels.
pub struct Board {
    /// The pieces on the board
    ///
    /// pieces[0] is y coordinate
    ///
    /// pieces[1] is x coordinate
    pub pieces: Vec<Vec<Piece>>,
}

impl Board {
    /// Create a new board with the specified dimensions.
    ///
    /// (u8, u8) are the (y,x) dimensions.
    ///
    /// Max y coordinate is 26 because Z is the last letter in the alphabet.
    fn new(dimensions: (u8, u8)) -> Result<Self> {
        // limitation of alphabet is caused by using a char in the Card struct, this might be changed later
        let mut pieces = Vec::new();
        for i in 0..dimensions.1 {
            let mut inner = Vec::new();
            for j in 0..dimensions.0 {
                inner.push(Piece {
                    placed: false,
                    chain: None,
                });
            }
            pieces.push(inner);
        }
        Ok(Self { pieces })
    }

    // Some more colorization's could be implemented, for example that the background color of the surrounding empty squares is equal to the color of the hotel chain

    /// Creates a new paragraph out of the current state of the board.
    ///
    /// If `large` is true, the board will be printed in a larger fashion
    pub fn to_paragraph(&self, size: BoardSize) -> Paragraph {
        // TODO center board
        let mut text = Vec::new();
        for (y_idx, y) in self.pieces.iter().enumerate() {
            let mut line_components = Vec::new();
            if y_idx != 0 {
                extra_lines(&size, y.len(), &mut text);
                // insert row filled with dashes
                let mut hline = String::from("--");
                for i in 0..y.len() {
                    hline.push_str("+---");
                    if size >= BoardSize::Medium {
                        hline.push_str("--");
                    }
                }
                text.push(Line::from(hline));
                extra_lines(&size, y.len(), &mut text);
            } else {
                extra_lines(&size, y.len(), &mut text);
            }
            for (x_idx, x) in y.iter().enumerate() {
                if x_idx == 0 {
                    // insert row label
                    // determine line char with ascii magic
                    line_components.push(Span::from(format!("{} ", ((y_idx as u8) + 65) as char)))
                }
                // different amount of spaces depending on the size
                if size >= BoardSize::Medium {
                    line_components.push(Span::from("|  "));
                } else {
                    line_components.push(Span::from("| "));
                }
                line_components.push(Span::from(x));
                if size >= BoardSize::Medium {
                    line_components.push(Span::from("  "));
                } else {
                    line_components.push(Span::from(" "));
                }
            }
            text.push(Line::from(line_components));
            if y_idx == self.pieces.len() - 1 {
                extra_lines(&size, self.pieces[0].len(), &mut text)
            }
        }
        // add column labels
        let mut line = String::from("  ");
        for x in 0..self.pieces[0].len() {
            if size >= BoardSize::Medium {
                line.push_str(&format!("  {:2}  ", x));
            } else {
                line.push_str(&format!(" {:2} ", x));
            }
        }
        text.push(Line::from(line));
        Paragraph::new(text)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new((12, 9)).unwrap()
    }
}

/// Used to determine how large the board can be printed, depends on the canvas size.
#[derive(PartialEq)]
pub enum BoardSize {
    Small,
    Medium,
    Large,
}

impl PartialOrd for BoardSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            BoardSize::Small => {
                if other == &BoardSize::Small {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Less)
                }
            }
            BoardSize::Medium => match other {
                BoardSize::Small => Some(Ordering::Greater),
                BoardSize::Medium => Some(Ordering::Equal),
                BoardSize::Large => Some(Ordering::Less),
            },
            BoardSize::Large => {
                if other == &BoardSize::Large {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Greater)
                }
            }
        }
    }
}

/// Prints extra lines for the board when board size is equal to large
fn extra_lines(size: &BoardSize, len: usize, text: &mut Vec<Line>) {
    if size == &BoardSize::Large {
        let mut line = String::from("  ");
        for i in 0..len {
            line.push_str("|     ");
        }
        text.push(Line::from(line))
    }
}

/// A piece on the board
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Piece {
    /// True when the piece is placed on the board.
    pub placed: bool,
    pub chain: Option<HotelChain>,
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.chain {
            None => write!(f, " "),
            Some(chain) => write!(f, "{}", chain.identifier()),
        }
    }
}

impl From<&Piece> for Text<'_> {
    fn from(value: &Piece) -> Self {
        Text::from(Span::from(value))
    }
}

impl From<&Piece> for Span<'_> {
    fn from(value: &Piece) -> Self {
        match &value.chain {
            None => Span::raw("X"),
            Some(chain) => Span::styled(
                format!("{}", chain.identifier()),
                Style::default().fg(chain.color()),
            ),
        }
    }
}

/// A card that a player can play to place a hotel on the board.
pub struct Card {
    num: u8,
    letter: char,
}

impl Card {
    /// Returns the coordinates of this card in the vector
    fn to_vec_coordinates(&self) -> (u8, u8) {
        let x = self.num - 1;
        let y = (self.letter as u8) - 65;
        (x, y)
    }
}

/// All different hotel chains in the game
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HotelChain {
    Airport,
    Continental,
    Festival,
    Imperial,
    Luxor,
    Oriental,
    Prestige,
}

impl HotelChain {
    /// Character identifier of this hotel chain
    fn identifier(&self) -> char {
        match self {
            HotelChain::Airport => 'A',
            HotelChain::Continental => 'C',
            HotelChain::Festival => 'F',
            HotelChain::Imperial => 'I',
            HotelChain::Luxor => 'L',
            HotelChain::Oriental => 'O',
            HotelChain::Prestige => 'P',
        }
    }

    /// Color of that hotel chain
    fn color(&self) -> Color {
        // TODO maybe make rgb values configurable
        match self {
            HotelChain::Airport => Color::Rgb(107, 141, 165),
            HotelChain::Continental => Color::Rgb(32, 64, 136),
            HotelChain::Festival => Color::Rgb(12, 106, 88),
            HotelChain::Imperial => Color::Rgb(198, 83, 80),
            HotelChain::Luxor => Color::Rgb(231, 219, 0),
            HotelChain::Oriental => Color::Rgb(184, 96, 20),
            HotelChain::Prestige => Color::Rgb(99, 47, 107),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::game::base::{BoardSize, Piece};

    use super::{Board, Card};

    #[test]
    fn test_board_new() {
        let board = Board::new((2, 4)).unwrap();
        assert_eq!(
            board.pieces[0][0],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[1][0],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[2][0],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[3][0],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[0][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[1][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[2][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[3][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(board.pieces.get(4), None);
        assert_eq!(board.pieces[0].get(2), None);
    }

    #[test]
    fn test_board_default() {
        let board = Board::default();
        assert_eq!(
            board.pieces[0][0],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[8][11],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(board.pieces.get(10), None);
        assert_eq!(board.pieces[8].get(12), None);
    }

    #[test]
    fn test_card_to_vec_coordinates() {
        assert_eq!(
            Card {
                num: 1,
                letter: 'A'
            }
            .to_vec_coordinates(),
            (0, 0)
        );
        assert_eq!(
            Card {
                num: 5,
                letter: 'B'
            }
            .to_vec_coordinates(),
            (4, 1)
        );
        assert_eq!(
            Card {
                num: 20,
                letter: 'Z'
            }
            .to_vec_coordinates(),
            (19, 25)
        );
    }

    #[test]
    fn test_board_size_partial_cmp() {
        assert_eq!(
            BoardSize::Small.partial_cmp(&BoardSize::Small),
            Some(Ordering::Equal)
        );
        assert_eq!(
            BoardSize::Medium.partial_cmp(&BoardSize::Small),
            Some(Ordering::Greater)
        );
        assert_eq!(
            BoardSize::Large.partial_cmp(&BoardSize::Small),
            Some(Ordering::Greater)
        );
        assert_eq!(
            BoardSize::Small.partial_cmp(&BoardSize::Medium),
            Some(Ordering::Less)
        );
        assert_eq!(
            BoardSize::Medium.partial_cmp(&BoardSize::Medium),
            Some(Ordering::Equal)
        );
        assert_eq!(
            BoardSize::Large.partial_cmp(&BoardSize::Medium),
            Some(Ordering::Greater)
        );
        assert_eq!(
            BoardSize::Small.partial_cmp(&BoardSize::Large),
            Some(Ordering::Less)
        );
        assert_eq!(
            BoardSize::Medium.partial_cmp(&BoardSize::Large),
            Some(Ordering::Less)
        );
        assert_eq!(
            BoardSize::Large.partial_cmp(&BoardSize::Large),
            Some(Ordering::Equal)
        );
    }
}
