use core::fmt;
use miette::{miette, Result};
use std::{fmt::Display, slice::Iter};

use self::Letter::*;
use colored::*;

pub struct Piece {
    chain: Option<Hotel>, // Stores what hotel chain this piece belongs to
    position: Position,   // Stores the position on the board of this piece
    piece_set: bool,      // Stores if the piece has been set yet
}

impl Piece {
    fn print_text(&self) -> ColoredString {
        if self.piece_set {
            if self.chain.is_some() {
                let mut string = String::from(" ");
                string.push(self.chain.as_ref().unwrap().identifier());
                string.push(' ');
                string.color(Hotel::color(&self.chain.as_ref().unwrap()))
            } else {
                String::from("XXX").bright_white()
            }
        } else {
            String::from(format!(
                "{}{:2}",
                self.position.letter, self.position.number
            ))
            .white()
        }
    }
}

#[derive(Clone, Copy)]
pub enum Hotel {
    Airport,
    Continental,
    Festival,
    Imperial,
    Luxor,
    Oriental,
    Prestige,
}

impl Hotel {
    fn identifier(&self) -> char {
        match *self {
            Hotel::Airport => 'A',
            Hotel::Continental => 'C',
            Hotel::Festival => 'F',
            Hotel::Imperial => 'I',
            Hotel::Luxor => 'L',
            Hotel::Oriental => 'O',
            Hotel::Prestige => 'P',
        }
    }

    /// Returns the specific color for the hotel
    fn color(&self) -> Color {
        match *self {
            Hotel::Airport => Color::TrueColor {
                r: 107,
                g: 141,
                b: 165,
            },
            Hotel::Continental => Color::TrueColor {
                r: 32,
                g: 64,
                b: 136,
            },
            Hotel::Festival => Color::TrueColor {
                r: 12,
                g: 106,
                b: 88,
            },
            Hotel::Imperial => Color::TrueColor {
                r: 198,
                g: 83,
                b: 80,
            },
            Hotel::Luxor => Color::TrueColor {
                r: 231,
                g: 219,
                b: 0,
            },
            Hotel::Oriental => Color::TrueColor {
                r: 184,
                g: 96,
                b: 20,
            },
            Hotel::Prestige => Color::TrueColor {
                r: 99,
                g: 47,
                b: 107,
            },
        }
    }

    pub fn iterator() -> Iter<'static, Hotel> {
        static HOTELS: [Hotel; 7] = [
            Hotel::Airport,
            Hotel::Continental,
            Hotel::Festival,
            Hotel::Imperial,
            Hotel::Luxor,
            Hotel::Oriental,
            Hotel::Prestige,
        ];
        HOTELS.iter()
    }
}

/// Symbolizes a position on the board
#[derive(Clone, Copy)]
pub struct Position {
    pub letter: Letter,
    pub number: u8,
}

impl Position {
    /// Creates a new position
    pub fn new(letter: Letter, number: u8) -> Self {
        Self { letter, number }
    }
}

/// This enum contains all letters of the board
#[derive(Clone, Copy, PartialEq)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
}

impl Letter {
    pub fn iterator() -> Iter<'static, Letter> {
        static LETTERS: [Letter; 9] = [A, B, C, D, E, F, G, H, I];
        LETTERS.iter()
    }

    pub fn letter(&self) -> char {
        match *self {
            Letter::A => 'A',
            Letter::B => 'B',
            Letter::C => 'C',
            Letter::D => 'D',
            Letter::E => 'E',
            Letter::F => 'F',
            Letter::G => 'G',
            Letter::H => 'H',
            Letter::I => 'I',
        }
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter())
    }
}

pub struct Board {
    pieces: Vec<Vec<Piece>>,
}

impl Board {
    pub fn new() -> Self {
        let mut pieces: Vec<Vec<Piece>> = Vec::new();
        // initialize pieces
        for c in Letter::iterator() {
            let mut x_pieces: Vec<Piece> = Vec::new();
            for i in 1..=12 {
                x_pieces.push(Piece {
                    chain: None,
                    position: Position::new(*c, i),
                    piece_set: false,
                })
            }
            pieces.push(x_pieces);
        }
        Self { pieces }
    }

    pub fn print(&self) {
        println!();
        for x in &self.pieces {
            for y in x {
                print!("[{}] ", y.print_text());
            }
            println!()
        }
    }

    /// Places a hotel at the designated coordinates
    /// # Return
    /// Ok when the hotel was placed correctly
    /// Error when the hotel was already placed
    pub fn place_hotel(&mut self, position: Position) -> Result<()> {
        for x in self.pieces.iter_mut() {
            for y in x.iter_mut() {
                if y.position.number.eq(&position.number) && y.position.letter == position.letter {
                    if y.piece_set {
                        return Err(miette!("Unable to set hotel at [{}{:2}] active: The hotel has already been placed!", position.letter, position.number));
                    } else {
                        y.piece_set = true;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn place_hotel_debug(&mut self, position: Position, chain: Hotel) -> Result<()> {
        'outer: for x in self.pieces.iter_mut() {
            for y in x.iter_mut() {
                if y.position.number.eq(&position.number) && y.position.letter == position.letter {
                    if y.piece_set {
                        return Err(miette!("Unable to set hotel at [{}{:2}] active: The hotel has already been placed!", position.letter, position.number));
                    } else {
                        y.piece_set = true;
                        y.chain = Some(chain);
                        break 'outer;
                    }
                }
            }
        }
        Ok(())
    }
}
