//! The game board: pieces, positions and analyzed positions.
//!
//! Rendering of the board is the host's (TUI's) job. This module only holds
//! the state and the pure helpers.

use miette::{miette, Result};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use super::chains::HotelChain;
use super::rules::{analyze_position, PlaceHotelCase};

use self::letter::{next_letter, prev_letter, LETTERS};

/// The board object that contains all information about the current state of the board.
#[derive(Clone, Debug)]
pub struct Board {
    pub pieces: Vec<Vec<Piece>>,
}

impl Board {
    /// Creates a new board and initializes it.
    pub fn new() -> Self {
        let mut pieces: Vec<Vec<Piece>> = Vec::new();
        // initialize pieces
        for c in LETTERS {
            let mut x_pieces: Vec<Piece> = Vec::new();
            for i in 1..=12 {
                x_pieces.push(Piece {
                    chain: None,
                    position: Position::new(c, i),
                    piece_set: false,
                })
            }
            pieces.push(x_pieces);
        }
        Self { pieces }
    }

    /// Returns the number of hotels that have been placed on the board.
    #[allow(dead_code)]
    pub fn placed_count(&self) -> u32 {
        self.pieces
            .iter()
            .flat_map(|line| line.iter())
            .filter(|p| p.piece_set)
            .count() as u32
    }

    /// Places a hotel at the designated coordinates. Does not check if this placement is valid
    /// according to the game rules.
    /// # Return
    /// Ok when the hotel was placed correctly, Error when the hotel was already placed.
    pub fn place_hotel(&mut self, position: &Position) -> Result<()> {
        for x in self.pieces.iter_mut() {
            for y in x.iter_mut() {
                if y.position.number.eq(&position.number) && y.position.letter == position.letter {
                    if y.piece_set {
                        return Err(miette!(
                            "Unable to set hotel at [{}{:2}]: The hotel has already been placed!",
                            position.letter,
                            position.number
                        ));
                    } else {
                        y.piece_set = true;
                    }
                }
            }
        }
        Ok(())
    }

    /// Updates the hotel of the piece at the specified position.
    /// Will overwrite any chain that stands there.
    pub fn update_hotel(&mut self, hotel_chain: HotelChain, position: &Position) -> Result<()> {
        for line in self.pieces.iter_mut() {
            for piece in line {
                if piece.position.eq(position) && piece.piece_set {
                    piece.chain = Some(hotel_chain);
                    return Ok(());
                }
            }
        }
        Err(miette!(
            "Unable to update hotel at position {} to chain {}: Hotel has not been placed yet",
            position,
            hotel_chain
        ))
    }

    /// Checks if a hotel has been placed at the position.
    /// # Returns
    /// * `None` - Hotel has not been placed
    /// * `Some(None)` - When the hotel has been placed but it does not belong to any chain
    /// * `Some(HotelChain)` - When the hotel has been placed and belongs to a chain
    pub fn is_hotel_placed(&self, position: &Position) -> Option<Option<HotelChain>> {
        for line in &self.pieces {
            for piece in line {
                if piece.position.eq(position) {
                    if !piece.piece_set {
                        return None;
                    }
                    return Some(piece.chain);
                }
            }
        }
        None
    }
}

/// Functions related to the letter.
pub mod letter {
    pub const LETTERS: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'];

    /// Returns the index for this letter in the `LETTERS` array.
    fn letter_to_index(letter: char) -> Option<usize> {
        for (index, l) in LETTERS.iter().enumerate() {
            if letter == *l {
                return Some(index);
            }
        }
        None
    }

    /// Returns the next letter if there is one. B would return C.
    pub fn next_letter(letter: char) -> Option<char> {
        let index = letter_to_index(letter)?;
        if index == 8 {
            return None;
        }
        Some(*LETTERS.get(index + 1).unwrap())
    }

    /// Returns the previous letter if there is one. B would return A.
    pub fn prev_letter(letter: char) -> Option<char> {
        let index = letter_to_index(letter)?;
        if index == 0 {
            return None;
        }
        Some(*LETTERS.get(index - 1).unwrap())
    }
}

/// Symbolizes a position on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub letter: char,
    pub number: u32,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match self.letter.cmp(&other.letter) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => Ordering::Equal,
            },
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Position {
    /// Creates a new position.
    pub fn new(letter: char, number: u32) -> Self {
        Self { letter, number }
    }

    /// Returns the next position. Input B3 would return B4.
    pub fn next(&self) -> Option<Position> {
        if self.number > 12 {
            return None;
        }
        Some(Position::new(self.letter, self.number + 1))
    }

    /// Returns the previous position. Input B3 would return B2.
    pub fn prev(&self) -> Option<Position> {
        if self.number < 1 {
            return None;
        }
        Some(Position::new(self.letter, self.number - 1))
    }

    /// Returns the position that is above this position. Input B3 would return A3.
    pub fn up(&self) -> Option<Position> {
        prev_letter(self.letter).map(|letter| Position::new(letter, self.number))
    }

    /// Returns the position that is below this position. Input B3 would return C3.
    pub fn down(&self) -> Option<Position> {
        next_letter(self.letter).map(|letter| Position::new(letter, self.number))
    }

    /// Returns the neighbouring positions.
    pub fn neighbours(&self) -> Vec<Position> {
        let mut neighbours = Vec::new();
        if let Some(next) = self.next() {
            neighbours.push(next);
        }
        if let Some(down) = self.down() {
            neighbours.push(down);
        }
        if let Some(prev) = self.prev() {
            neighbours.push(prev);
        }
        if let Some(up) = self.up() {
            neighbours.push(up);
        }
        neighbours
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{:2}", self.letter, self.number)
    }
}

/// Symbolizes a position on the board that has been analyzed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyzedPosition {
    pub position: Position,
    pub place_hotel_case: PlaceHotelCase,
}

impl PartialOrd for AnalyzedPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.position.cmp(&other.position))
    }
}

impl Ord for AnalyzedPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

impl AnalyzedPosition {
    /// Analyzes the position to return a new analyzed position.
    pub fn new(
        position: Position,
        board: &Board,
        hotel_chain_manager: &super::chains_mgr::HotelChainManager,
    ) -> Self {
        let place_hotel_case = analyze_position(&position, board, hotel_chain_manager);
        Self {
            position,
            place_hotel_case,
        }
    }

    /// Creates a new analyzed position without analyzing the position.
    /// Should only be used when the players' cards are initialized for the first time.
    /// The place hotel case will be set to single hotel.
    pub fn new_unchecked(position: Position) -> Self {
        Self {
            position,
            place_hotel_case: PlaceHotelCase::SingleHotel,
        }
    }

    /// Analyzes the position again and updates the place hotel case value.
    pub fn check(
        &mut self,
        board: &Board,
        hotel_chain_manager: &super::chains_mgr::HotelChainManager,
    ) {
        self.place_hotel_case = analyze_position(&self.position, board, hotel_chain_manager);
    }

    /// Checks if this position is illegal.
    pub fn is_illegal(&self) -> bool {
        matches!(&self.place_hotel_case, PlaceHotelCase::Illegal(_reason))
    }
}

/// Symbolizes a single piece that can be placed on the board.
#[derive(Clone, Debug)]
pub struct Piece {
    /// Stores what hotel chain this piece belongs to.
    pub chain: Option<HotelChain>,
    /// Stores the position on the board of this piece.
    pub position: Position,
    /// Stores if the piece has been set yet.
    pub piece_set: bool,
}

#[cfg(test)]
mod tests {
    use miette::{miette, Result};

    use super::{Board, Position};
    use crate::core::chains::HotelChain;

    #[test]
    fn surrounding_positions_correct() {
        let position = Position::new('B', 3);
        let position_prev = Position::new('B', 2);
        let position_next = Position::new('B', 4);
        let position_up = Position::new('A', 3);
        let position_down = Position::new('C', 3);
        assert_eq!(position_prev, position.prev().unwrap());
        assert_eq!(position_next, position.next().unwrap());
        assert_eq!(position_up, position.up().unwrap());
        assert_eq!(position_down, position.down().unwrap());
    }

    #[test]
    fn is_hotel_placed() -> Result<()> {
        let mut board = Board::new();
        let position = Position::new('H', 5);
        board.place_hotel(&position)?;
        assert!(board.is_hotel_placed(&position).is_some());
        let position2 = Position::new('G', 3);
        place_hotel_debug(&mut board, position2, HotelChain::Luxor)?;
        assert!(board.is_hotel_placed(&position2).unwrap().is_some());
        assert!(board.is_hotel_placed(&Position::new('F', 4)).is_none());
        Ok(())
    }

    /// Place a hotel on the board without abiding by the game rules.
    pub fn place_hotel_debug(
        board: &mut Board,
        position: Position,
        chain: HotelChain,
    ) -> Result<()> {
        for x in board.pieces.iter_mut() {
            for y in x.iter_mut() {
                if y.position.number.eq(&position.number) && y.position.letter == position.letter {
                    if y.piece_set {
                        return Err(miette!(
                            "Unable to set hotel at [{}{:2}]: The hotel has already been placed!",
                            position.letter,
                            position.number
                        ));
                    } else {
                        y.piece_set = true;
                        y.chain = Some(chain);
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}
