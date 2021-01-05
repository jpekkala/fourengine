use std::fmt;

use crate::bitboard;
use crate::bitboard::{Bitboard, PositionCode};
use crate::constants::*;
use std::fmt::Formatter;

/// Represents the board state of a particular position but not how the position was arrived at.
/// Contains the same information as PositionCode but in a format that is easier to manipulate.
pub struct Position {
    pub ply: u32,

    pub current: Bitboard,
    pub other: Bitboard,
}

pub enum Disc {
    White,
    Red,
    Empty,
}

impl Position {
    pub fn empty() -> Position {
        Position {
            ply: 0,
            current: Bitboard::empty(),
            other: Bitboard::empty(),
        }
    }

    pub fn from_variation(variation: &str) -> Position {
        let mut position = Position::empty();
        for ch in variation.trim().chars() {
            let column: u32 = ch.to_digit(10).expect("Expected digit") - 1;
            position = position.drop(column);
        }
        position
    }

    pub fn drop(&self, column: u32) -> Position {
        let new_board = self.current.drop(self.other, column);
        if !new_board.is_legal() {
            panic!("Invalid move");
        }
        Position {
            current: self.other,
            other: new_board,
            ply: self.ply + 1,
        }
    }

    pub fn has_won(&self) -> bool {
        return self.current.has_won() || self.other.has_won();
    }

    fn get_ordered_boards(&self) -> (Bitboard, Bitboard) {
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

        (white_board, red_board)
    }

    pub fn get_disc_at(&self, x: u32, y: u32) -> Disc {
        let (white_board, red_board) = self.get_ordered_boards();

        if white_board.has_disc(x, y) {
            Disc::White
        } else if red_board.has_disc(x, y) {
            Disc::Red
        } else {
            Disc::Empty
        }
    }

    pub fn to_position_code(&self) -> PositionCode {
        let (white_board, red_board) = self.get_ordered_boards();
        PositionCode::new(white_board, red_board)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_from_variation() {
        let position = Position::from_variation("444444");
        let expected = "\
             ...O...\n\
             ...X...\n\
             ...O...\n\
             ...X...\n\
             ...O...\n\
             ...X...\n";
        assert_eq!(position.to_string(), expected);

        let position = Position::from_variation("436675553");
        let expected = "\
             .......\n\
             .......\n\
             .......\n\
             ....O..\n\
             ..X.XO.\n\
             ..OXOXX\n";
        assert_eq!(position.to_string(), expected);
    }

    #[test]
    fn win_checking() {
        // horizontal
        {
            let position = Position::from_variation("4455667");
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
        }

        // vertical
        {
            let position = Position::from_variation("4343434");
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
        }

        // slash (/)
        {
            let position = Position::from_variation("45567667677");
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
        }

        // backslash (\)
        {
            let position = Position::from_variation("76654554544");
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
        }
    }
}
