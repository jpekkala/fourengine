#![allow(dead_code)]

use std::fmt;
use std::fmt::Formatter;

/// board dimensions
pub const BOARD_WIDTH: u32 = 7;
pub const BOARD_HEIGHT: u32 = 6;

/// The number of bits needed to encode a position
pub const POSITION_BITS: u32 = (BOARD_HEIGHT + 1) * BOARD_WIDTH;

/// The underlying unsigned integer used to represent the board. This type should have at least
/// board_width * (board_height + 1) bits. Generally you should use the other types which have a
/// semantic meaning. This type exists just so that it is easier to change the underlying type if
/// bigger board sizes are used.
pub type BoardInteger = u64;

/// Represents the discs of a single player.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Bitboard(BoardInteger);

/// Represents the board state of a particular position but not how the position was arrived at.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
    pub current: Bitboard,
    pub other: Bitboard,
}

pub enum Disc {
    White,
    Red,
    Empty,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct PositionCode(BoardInteger);

// the column height including the buffer cell
pub const BIT_HEIGHT: u32 = BOARD_HEIGHT + 1;

const ALL_BITS: BoardInteger = (1 << (BIT_HEIGHT * BOARD_WIDTH)) - 1;
const FIRST_COLUMN: BoardInteger = (1 << BIT_HEIGHT) - 1;
const BOTTOM_ROW: BoardInteger = ALL_BITS / FIRST_COLUMN;
const TOP_ROW: BoardInteger = BOTTOM_ROW << BOARD_HEIGHT;
const FULL_BOARD: BoardInteger = ALL_BITS ^ TOP_ROW;

impl Bitboard {
    pub fn empty() -> Bitboard {
        Bitboard(0)
    }

    pub fn one() -> Bitboard {
        Bitboard(1)
    }

    pub fn has_won(&self) -> bool {
        let board = self.0;
        let vertical = board & (board >> 1);
        let horizontal = board & (board >> BIT_HEIGHT);

        const SLASH_SHIFT: u32 = BIT_HEIGHT - 1;
        const BACKSLASH_SHIFT: u32 = BIT_HEIGHT + 1;
        let slash = board & (board >> SLASH_SHIFT);
        let backslash = board & (board >> BACKSLASH_SHIFT);

        let result = (vertical & (vertical >> 2))
            | (horizontal & (horizontal >> 2 * BIT_HEIGHT))
            | (slash & (slash >> 2 * SLASH_SHIFT))
            | (backslash & (backslash >> 2 * BACKSLASH_SHIFT));
        result != 0
    }

    pub fn is_legal(&self) -> bool {
        (TOP_ROW & self.0) == 0
    }

    pub fn has_disc(&self, x: u32, y: u32) -> bool {
        let bit = 1 << (BOARD_WIDTH * x + y);
        (self.0 & bit) != 0
    }

    pub fn flip(&self) -> Bitboard {
        let mut pos = self.0;
        let mut mirror: BoardInteger = 0;
        for _ in 0..BOARD_WIDTH {
            mirror = (mirror << BIT_HEIGHT) | (pos & FIRST_COLUMN);
            pos >>= BIT_HEIGHT;
        }
        Bitboard(mirror)
    }

    /// Returns the cells where a four-in-line would be created if the player had a disc there.
    /// NOTE: This does not check if the other player already has a disc in the cell.
    fn get_threat_cells(&self) -> BoardInteger {
        let board = self.0;

        let vertical = (board << 1) & (board << 2) & (board << 3);
        let horizontal = threat_line(board, BIT_HEIGHT);
        let diagonal1 = threat_line(board, BIT_HEIGHT + 1);
        let diagonal2 = threat_line(board, BIT_HEIGHT - 1);

        (vertical | horizontal | diagonal1 | diagonal2) & FULL_BOARD
    }
}

#[inline]
fn threat_line(board: BoardInteger, shift_amount: u32) -> BoardInteger {
    let right_helper = (board >> shift_amount) & (board >> 2 * shift_amount);
    let right_triple = right_helper & (board >> 3 * shift_amount);
    let right_hole = right_helper & (board << shift_amount);

    let left_helper = (board << shift_amount) & (board << 2 * shift_amount);
    let left_triple = left_helper & (board << 3 * shift_amount);
    let left_hole = left_helper & (board >> shift_amount);

    right_triple | right_hole | left_triple | left_hole
}

impl Position {
    pub fn empty() -> Position {
        Position {
            current: Bitboard::empty(),
            other: Bitboard::empty(),
        }
    }

    pub fn new(current: Bitboard, other: Bitboard) -> Position {
        Position {
            current,
            other,
        }
    }

    pub fn from_variation(variation: &str) -> Position {
        let mut position = Position::empty();
        for ch in variation.trim().chars() {
            let column: u32 = ch.to_digit(10).expect("Expected digit") - 1;
            position = position.position_after_drop(column).expect("Invalid move");
        }
        position
    }

    pub fn change_perspective(&self) -> Position {
        Position {
            current: self.other,
            other: self.current,
        }
    }

    fn both(&self) -> BoardInteger {
        self.current.0 | self.other.0
    }

    fn get_height_bit(&self, column: u32) -> BoardInteger {
        let column_mask = FIRST_COLUMN << (BIT_HEIGHT * column);
        let both = self.both() + BOTTOM_ROW;
        both & column_mask
    }

    pub fn get_height(&self, column: u32) -> u32 {
        let both = self.both();
        ((both >> (BIT_HEIGHT * column)) + 1).trailing_zeros()
    }

    fn get_height_cells(&self) -> BoardInteger {
        self.both() + BOTTOM_ROW
    }

    pub fn drop(&self, column: u32) -> Bitboard {
        let bit = self.get_height_bit(column);
        Bitboard(self.current.0 | bit)
    }

    pub fn position_after_drop(&self, column: u32) -> Option<Position> {
        let new_board = self.drop(column);
        if !new_board.is_legal() {
            return None;
        }
        Some(Position {
            current: self.other,
            other: new_board,
        })
    }

    pub fn has_won(&self) -> bool {
        return self.current.has_won() || self.other.has_won();
    }

    fn get_ordered_boards(&self) -> (Bitboard, Bitboard) {
        let white_moves = self.get_ply() % 2 == 0;
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

    pub fn normalize(&self) -> (Position, bool) {
        let flipped_current = self.current.flip();
        let flipped_other = self.other.flip();
        let code1 = PositionCode::new(self.current, self.other);
        let code2 = PositionCode::new(flipped_current, flipped_other);
        let symmetric = code1 == code2;
        if code1 < code2 {
            (
                Position {
                    current: flipped_current,
                    other: flipped_other,
                },
                symmetric,
            )
        } else {
            (*self, symmetric)
        }
    }

    #[allow(dead_code)]
    pub fn flip(&self) -> Position {
        Position {
            current: self.current.flip(),
            other: self.other.flip(),
        }
    }

    pub fn get_ply(&self) -> u32 {
        (self.current.0 | self.other.0).count_ones()
    }

    pub fn get_threats(&self) -> Bitboard {
        let threat_cells =  self.current.get_threat_cells();
        let empty_cells = FULL_BOARD ^ self.both();
        Bitboard(threat_cells & empty_cells)
    }

    pub fn count_threats(&self) -> u32 {
        self.get_threats().0.count_ones()
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
                if self.has_disc(x, y) {
                    write!(f, "1")?;
                } else {
                    write!(f, "0")?
                }
            }
            writeln!(f)?;
        }
        Ok(())
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

impl PositionCode {
    pub fn new(p1: Bitboard, p2: Bitboard) -> PositionCode {
        PositionCode(BOTTOM_ROW + p1.0 + p1.0 + p2.0)
    }

    pub fn from_integer(integer: BoardInteger) -> PositionCode {
        PositionCode(integer)
    }

    pub fn to_integer(&self) -> BoardInteger {
        self.0
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
