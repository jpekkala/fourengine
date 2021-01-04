use std::fmt;

use crate::bitboard;
use crate::bitboard::Bitboard;
use std::fmt::Formatter;

pub struct GameState {
    pub ply: u32,

    pub current: Bitboard,
    pub other: Bitboard,
}

enum Disc {
    White,
    Red,
    Empty,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            current: 0,
            other: 0,
        }
    }

    pub fn drop(&mut self, column: u32) {
        let new_board = self.current | bitboard::drop(self.current, self.other, column);
        if !bitboard::is_legal(new_board) {
            panic!("Invalid move");
        }
        self.current = self.other;
        self.other = new_board;
        self.ply += 1;
    }

    pub fn has_won(&self) -> bool {
        bitboard::has_won(self.current) || bitboard::has_won(self.other)
    }

    pub fn get_disc_at(&self, x: u32, y: u32) -> Disc {
        let cell: Bitboard = 1 << (bitboard::WIDTH * x + y);

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

        if white_board & cell != 0 {
            Disc::White
        } else if red_board & cell != 0 {
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
            writeln!(f);
        }
        Ok(())
    }
}
