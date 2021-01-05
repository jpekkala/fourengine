use std::fmt;

use crate::bitboard::{Bitboard, PositionCode, BOARD_HEIGHT, BOARD_WIDTH};
use std::fmt::Formatter;

/// Represents the board state of a particular position but not how the position was arrived at.
/// Contains the same information as PositionCode but in a format that is easier to manipulate.
#[derive(Copy, Clone, PartialEq, Debug)]
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
            position = position.drop(column).expect("Invalid move");
        }
        position
    }

    pub fn drop(&self, column: u32) -> Option<Position> {
        let new_board = self.current.drop(self.other, column);
        if !new_board.is_legal() {
            return None;
        }
        Some(Position {
            current: self.other,
            other: new_board,
            ply: self.ply + 1,
        })
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
        PositionCode::new(self.current, self.other)
    }

    pub fn get_height(&self, column: u32) -> u32 {
        self.current.get_height(self.other, column)
    }

    pub fn normalize(&self) -> (Position, bool) {
        let flipped_current = self.current.flip();
        let flipped_other = self.other.flip();
        let code1 = PositionCode::new(self.current, self.other);
        let code2 = PositionCode::new(flipped_current, flipped_other);
        let symmetric = code1 == code2;
        if code1 < code2 {
            (Position {
                ply: self.ply,
                current: flipped_current,
                other: flipped_other,
            }, symmetric)
        } else {
            (*self, symmetric)
        }
    }

    pub fn flip(&self) -> Position {
        Position {
            ply: self.ply,
            current: self.current.flip(),
            other: self.other.flip(),
        }
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
    fn height() {
        let position = Position::from_variation("436675553");
        assert_eq!(position.get_height(0), 0);
        assert_eq!(position.get_height(1), 0);
        assert_eq!(position.get_height(2), 2);
        assert_eq!(position.get_height(3), 1);
        assert_eq!(position.get_height(4), 3);
        assert_eq!(position.get_height(5), 2);
        assert_eq!(position.get_height(6), 1);
    }

    #[test]
    fn flip() {
        let position = Position::from_variation("436675553");
        let expected = "\
             .......\n\
             .......\n\
             .......\n\
             ....O..\n\
             ..X.XO.\n\
             ..OXOXX\n";
        assert_eq!(position.to_string(), expected);

        let flipped = position.flip();
        let expected = "\
             .......\n\
             .......\n\
             .......\n\
             ..O....\n\
             .OX.X..\n\
             XXOXO..\n";
    }

    #[test]
    fn invalid_move() {
        let position = Position::from_variation("444444");
        assert!(position.drop(3).is_none());
        assert!(position.drop(0).is_some());
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

    #[test]
    fn threat_counting() {
        let position = Position::from_variation("43443555");
        assert_eq!(position.current.count_threats(position.other), 2);
        assert_eq!(position.other.count_threats(position.current), 0);
    }
}
