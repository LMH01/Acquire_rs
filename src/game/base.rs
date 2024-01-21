use anyhow::Result;
use ratatui::style::Color;

/// Contains all variables required to play a game.
pub struct Game {
    board: Board,
}

/// The board on which the hotels are placed.
///
/// Stores currently placed hotels.
pub struct Board {
    /// The pieces on the board
    ///
    /// pieces[0] is x coordinate
    ///
    /// pieces[1] is y coordinate
    pieces: Vec<Vec<Piece>>,
}

impl Board {
    /// Create a new board with the specified dimensions.
    ///
    /// (u8, u8) are the (x,y) dimensions.
    ///
    /// Max y coordinate is 26 because Z is the last letter in the alphabet.
    fn new(dimensions: (u8, u8)) -> Result<Self> {
        // limitation of alphabet is caused by using a char in the Card struct, this might be changed later
        // shuffle dimensions around so that x and y are aligned correctly
        let dimensions = (dimensions.1, dimensions.0);
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
}

impl Default for Board {
    fn default() -> Self {
        Self::new((12, 9)).unwrap()
    }
}

/// A piece on the board
#[derive(Debug, PartialEq, Eq)]
pub struct Piece {
    /// True when the piece is placed on the board.
    placed: bool,
    chain: Option<HotelChain>,
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
#[derive(Debug, PartialEq, Eq)]
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
    use crate::game::base::Piece;

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
            board.pieces[0][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[0][2],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[0][3],
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
            board.pieces[1][1],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[1][2],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(
            board.pieces[1][3],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(board.pieces.get(2), None);
        assert_eq!(board.pieces[0].get(4), None);
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
            board.pieces[11][8],
            Piece {
                placed: false,
                chain: None
            }
        );
        assert_eq!(board.pieces.get(12), None);
        assert_eq!(board.pieces[11].get(9), None);
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
}
