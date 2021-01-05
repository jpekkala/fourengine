use std::fmt;

use crate::bitboard;
use crate::bitboard::Bitboard;
use std::fmt::Formatter;

pub struct GameState {
    pub ply: u32,

    pub current: Bitboard,
    pub other: Bitboard,
}

pub enum Disc {
    White,
    Red,
    Empty,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            current: Bitboard::empty(),
            other: Bitboard::empty(),
        }
    }

    pub fn drop(&mut self, column: u32) {
        let new_board = self.current.drop(self.other, column);
        if !new_board.is_legal() {
            panic!("Invalid move");
        }
        self.current = self.other;
        self.other = new_board;
        self.ply += 1;
    }

    pub fn has_won(&self) -> bool {
        return self.current.has_won() || self.other.has_won();
    }

    pub fn get_disc_at(&self, x: u32, y: u32) -> Disc {
        let white_moves = self.ply % 2 == 0;
        let white_board = if white_moves {
            self.current
        } else {
            self.other
        };
        let red_board = if white_moves {
            self.other
        } else {
            self.current
        };

        if white_board.has_disc(x, y) {
            Disc::White
        } else if red_board.has_disc(x, y) {
            Disc::Red
        } else {
            Disc::Empty
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..bitboard::HEIGHT).rev() {
            for x in 0..bitboard::WIDTH {
                match self.get_disc_at(x, y) {
                    Disc::White => write!(f, "X")?,
                    Disc::Red => write!(f, "O")?,
                    Disc::Empty => write!(f, ".")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
