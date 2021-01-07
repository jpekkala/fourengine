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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Bitboard(pub BoardInteger);

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

    fn get_height_bit(&self, other: Bitboard, column: u32) -> BoardInteger {
        let column_mask = FIRST_COLUMN << (BIT_HEIGHT * column);
        let both = (self.0 | other.0) + BOTTOM_ROW;
        both & column_mask
    }

    pub fn get_height(&self, other: Bitboard, column: u32) -> u32 {
        let both = self.0 | other.0;
        ((both >> (BIT_HEIGHT * column)) + 1).trailing_zeros()
    }

    fn get_height_cells(&self, other: Bitboard) -> BoardInteger {
        (self.0 | other.0) + BOTTOM_ROW
    }

    pub fn drop(&self, other: Bitboard, column: u32) -> Bitboard {
        let bit = self.get_height_bit(other, column);
        Bitboard(self.0 | bit)
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

        vertical | horizontal | diagonal1 | diagonal2
    }

    pub fn get_threat_board(&self, other: Bitboard) -> Bitboard {
        let threat_cells =  self.get_threat_cells();
        let empty_cells = FULL_BOARD ^ (self.0 | other.0);
        Bitboard(threat_cells & empty_cells)
    }

    pub fn has_immediate_win(&self, other: Bitboard) -> bool {
        let threat_cells = self.get_threat_cells();
        let height_cells = self.get_height_cells(other);
        let immediate_threats = threat_cells & height_cells;
        if immediate_threats.count_ones() > 1 {
            return true;
        }
        let double_threats = immediate_threats & (threat_cells >> 1);
        return double_threats != 0;
    }

    pub fn count_threats(&self, other: Bitboard) -> u32 {
        self.get_threat_board(other).0.count_ones()
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
