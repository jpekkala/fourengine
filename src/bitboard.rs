#![allow(dead_code)]
#![allow(clippy::precedence)]

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

/// The discs of a single player.
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Bitboard(pub BoardInteger);

// the column height including the gutter cell
pub const BIT_HEIGHT: u32 = BOARD_HEIGHT + 1;

pub const ALL_BITS: BoardInteger = (1 << (BIT_HEIGHT * BOARD_WIDTH)) - 1;
pub const FIRST_COLUMN: BoardInteger = (1 << BIT_HEIGHT) - 1;
pub const BOTTOM_ROW: BoardInteger = ALL_BITS / FIRST_COLUMN;
pub const GUTTER_ROW: BoardInteger = BOTTOM_ROW << BOARD_HEIGHT;
pub const FULL_BOARD: BoardInteger = ALL_BITS ^ GUTTER_ROW;
pub const LEFT_HALF: BoardInteger = FIRST_COLUMN
    | (FIRST_COLUMN << BIT_HEIGHT)
    | (FIRST_COLUMN << 2 * BIT_HEIGHT)
    | (FIRST_COLUMN << 3 * BIT_HEIGHT);

pub const ODD_ROWS: BoardInteger = BOTTOM_ROW * 0b010101;
pub const EVEN_ROWS: BoardInteger = BOTTOM_ROW * 0b101010;

impl Bitboard {
    pub fn empty() -> Bitboard {
        Bitboard(0)
    }

    /// The reverse of to_string. The gutter row (seventh row) is optional but to_string always
    /// includes it.
    ///
    /// There is also a "bitboard!" macro which is somewhat more ergonomic to use especially when
    /// writing tests.
    /// ```
    /// use fourengine::bitboard::*;
    /// use fourengine::*;
    ///
    /// let string = concat!(
    ///     "0000000\n",
    ///     "0000000\n",
    ///     "0000000\n",
    ///     "0000000\n",
    ///     "0000100\n",
    ///     "0000010\n",
    ///     "0010100\n",
    /// );
    /// let board = Bitboard::from_string(string).unwrap();
    /// assert_eq!(board.to_string(), string);
    ///
    /// // the same but with a macro
    /// let board_with_macro = bitboard!(
    ///     "0000000"
    ///     "0000000"
    ///     "0000000"
    ///     "0000100"
    ///     "0000010"
    ///     "0010100"
    /// );
    /// assert_eq!(board_with_macro, board);
    /// ```
    pub fn from_string(str: &str) -> Option<Bitboard> {
        let mut bitboard = Bitboard::empty();

        for (y, line) in str.split_whitespace().rev().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                match ch {
                    '1' => bitboard = bitboard.set_disc(x as u32, y as u32),
                    '0' => {}
                    _ => return None,
                }
            }
        }

        Some(bitboard)
    }

    pub fn has_won(&self) -> bool {
        let board = self.0;
        let vertical = board & (board >> 1);
        let horizontal = board & (board >> BIT_HEIGHT);

        const SLASH_SHIFT: u32 = BIT_HEIGHT + 1;
        const BACKSLASH_SHIFT: u32 = BIT_HEIGHT - 1;
        let slash = board & (board >> SLASH_SHIFT);
        let backslash = board & (board >> BACKSLASH_SHIFT);

        let result = (vertical & (vertical >> 2))
            | (horizontal & (horizontal >> 2 * BIT_HEIGHT))
            | (slash & (slash >> 2 * SLASH_SHIFT))
            | (backslash & (backslash >> 2 * BACKSLASH_SHIFT));

        result != 0
    }

    pub fn get_won_cells(&self) -> BoardInteger {
        fn win_line(board: BoardInteger, shift_amount: u32) -> BoardInteger {
            let half = board & (board >> shift_amount);
            let fourth = half & (half >> 2 * shift_amount);
            let helper = fourth | (fourth << shift_amount);
            helper | (helper << 2 * shift_amount)
        }

        let board = self.0;
        let vertical = win_line(board, 1);
        let horizontal = win_line(board, BIT_HEIGHT);
        let slash = win_line(board, BIT_HEIGHT + 1);
        let backslash = win_line(board, BIT_HEIGHT - 1);

        vertical | horizontal | slash | backslash
    }

    pub fn is_legal(&self) -> bool {
        (GUTTER_ROW & self.0) == 0
    }

    pub fn has_disc(&self, x: u32, y: u32) -> bool {
        let bit = 1 << (BOARD_WIDTH * x + y);
        (self.0 & bit) != 0
    }

    pub fn set_disc(&self, x: u32, y: u32) -> Bitboard {
        let bit = 1 << (BOARD_WIDTH * x + y);
        Bitboard(self.0 | bit)
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
    pub fn get_threat_cells(&self) -> BoardInteger {
        let board = self.0;

        let vertical = (board << 1) & (board << 2) & (board << 3);
        let horizontal = threat_line(board, BIT_HEIGHT);
        let diagonal1 = threat_line(board, BIT_HEIGHT + 1);
        let diagonal2 = threat_line(board, BIT_HEIGHT - 1);

        (vertical | horizontal | diagonal1 | diagonal2) & FULL_BOARD
    }

    fn get_column_as_first(&self, x: u32) -> BoardInteger {
        (self.0 >> (x * BIT_HEIGHT)) & FIRST_COLUMN
    }

    fn get_column_in_place(&self, x: u32) -> BoardInteger {
        self.0 & (FIRST_COLUMN << x * BIT_HEIGHT)
    }

    /// Returns a bitmap where only the lowest 1 bit in each column is kept set. If a column has no
    /// bits set, the gutter bit is set for that column instead.
    /// ```
    /// use fourengine::bitboard::*;
    /// use fourengine::*;
    ///
    /// let board = bitboard!(
    ///     "0000000"
    ///     "0000000"
    ///     "0000001"
    ///     "0000000"
    ///     "0000100"
    ///     "0000010"
    ///     "0010100"
    /// );
    ///
    /// assert_eq!(board.keep_lowest_or_gutter(), bitboard!(
    ///     "1101000"
    ///     "0000000"
    ///     "0000001"
    ///     "0000000"
    ///     "0000000"
    ///     "0000010"
    ///     "0010100"
    /// ));
    /// ```
    pub fn keep_lowest_or_gutter(&self) -> Bitboard {
        // The formula for finding the least significant bit in a number is `v & (!v + 1)`
        // which for a bitboard can be generalized to `board & (!board + BOTTOM_ROW)`

        // prevent overflow by always having at least the gutter set
        let helper = self.0 | GUTTER_ROW;
        Bitboard(helper & (!helper + BOTTOM_ROW))
    }

    /// Finds the highest set bit for each column and then sets all cells below the highest bit as
    /// well.
    /// ```
    /// use fourengine::bitboard::*;
    /// use fourengine::*;
    ///
    /// let board = bitboard!(
    ///     "0000000"
    ///     "0000000"
    ///     "0000001"
    ///     "0000000"
    ///     "0000100"
    ///     "0000010"
    ///     "0010100"
    /// );
    ///
    /// assert_eq!(board.get_silhouette(), bitboard!(
    ///     "0000000"
    ///     "0000000"
    ///     "0000001"
    ///     "0000001"
    ///     "0000101"
    ///     "0000111"
    ///     "0010111"
    /// ));
    /// ```
    pub fn get_silhouette(&self) -> Bitboard {
        let mut tmp = self.0;
        for _ in 0..(BOARD_HEIGHT - 1) {
            tmp |= (tmp >> 1) & FULL_BOARD;
        }
        Bitboard(tmp)
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

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..BOARD_HEIGHT + 1).rev() {
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

#[macro_export]
macro_rules! bitboard {
    ($($x:literal)+) => {
        Bitboard::from_string(concat!($($x,"\n",)+)).expect("Invalid bitboard representation")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_macro() {
        // gutter row is optional
        let bitboard = bitboard!(
            "0000000"
            "0001000"
            "0000000"
            "0001000"
            "0000000"
            "0001000"
        );

        let expected = "\
             0000000\n\
             0000000\n\
             0001000\n\
             0000000\n\
             0001000\n\
             0000000\n\
             0001000\n";

        assert_eq!(bitboard.to_string(), expected);
    }
}
