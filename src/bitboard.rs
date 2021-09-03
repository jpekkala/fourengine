#![allow(dead_code)]
#![allow(clippy::precedence)]

use crate::score::Score;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::mem;

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

/// The board state of a particular position but not how the position was arrived at.
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Position {
    pub current: Bitboard,
    pub other: Bitboard,
}

/// Available moves as a bitmap where each column can have max 1 bit set.
#[derive(Copy, Clone)]
pub struct MoveBitmap(pub BoardInteger);

pub enum Disc {
    White,
    Red,
    Empty,
}

// the column height including the gutter cell
pub const BIT_HEIGHT: u32 = BOARD_HEIGHT + 1;

const ALL_BITS: BoardInteger = (1 << (BIT_HEIGHT * BOARD_WIDTH)) - 1;
pub const FIRST_COLUMN: BoardInteger = (1 << BIT_HEIGHT) - 1;
const BOTTOM_ROW: BoardInteger = ALL_BITS / FIRST_COLUMN;
const GUTTER_ROW: BoardInteger = BOTTOM_ROW << BOARD_HEIGHT;
const FULL_BOARD: BoardInteger = ALL_BITS ^ GUTTER_ROW;
const LEFT_HALF: BoardInteger = FIRST_COLUMN
    | (FIRST_COLUMN << BIT_HEIGHT)
    | (FIRST_COLUMN << 2 * BIT_HEIGHT)
    | (FIRST_COLUMN << 3 * BIT_HEIGHT);

const ODD_ROWS: BoardInteger = BOTTOM_ROW * 0b010101;
const EVEN_ROWS: BoardInteger = BOTTOM_ROW * 0b101010;

impl Bitboard {
    pub fn empty() -> Bitboard {
        Bitboard(0)
    }

    /// The reverse of to_string
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
    fn get_threat_cells(&self) -> BoardInteger {
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
    fn keep_lowest_or_gutter(&self) -> BoardInteger {
        // The formula for finding the least significant bit in a number is `v & (!v + 1)`
        // which for a bitboard can be generalized to `board & (!board + BOTTOM_ROW)`

        // prevent overflow by always having at least the gutter set
        let helper = self.0 | GUTTER_ROW;
        helper & (!helper + BOTTOM_ROW)
    }

    /// Finds the highest set bit for each column and then sets all cells below the highest bit as
    /// well.
    /// ```
    /// use fourengine::bitboard::*;
    /// use fourengine::*;
    ///
    /// let position = position!(
    ///     "......."
    ///     "......."
    ///     "......."
    ///     "....O.."
    ///     "..X.XO."
    ///     "..OXOXX"
    /// );
    ///
    /// let silhouette = bitboard!(
    ///     "0000000"
    ///     "0000000"
    ///     "0000000"
    ///     "0000000"
    ///     "0000100"
    ///     "0010110"
    ///     "0011111"
    /// );
    /// assert_eq!(position.get_silhouette().get_silhouette(), silhouette);
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

impl Position {
    pub fn empty() -> Position {
        Position {
            current: Bitboard::empty(),
            other: Bitboard::empty(),
        }
    }

    pub fn new(current: Bitboard, other: Bitboard) -> Position {
        Position { current, other }
    }

    pub fn from_position_code(code: BoardInteger) -> Option<Position> {
        let silhouette = Bitboard(code).get_silhouette();
        // every column must have at least one bit set that indicates the height
        if (silhouette.0 & BOTTOM_ROW) != BOTTOM_ROW {
            return None;
        }

        let both = (silhouette.0 >> 1) & FULL_BOARD;
        let current = Bitboard(code & both);
        let other = Bitboard(!code & both);
        Some(Position { current, other })
    }

    pub fn as_bitboard(&self) -> Bitboard {
        Bitboard(self.to_position_code())
    }

    pub fn get_silhouette(&self) -> Bitboard {
        let board = (self.to_position_code() >> 1) & FULL_BOARD;
        Bitboard(board).get_silhouette()
    }

    pub fn from_variation(variation: &str) -> Option<Position> {
        let mut position = Position::empty();
        for ch in variation.trim().chars() {
            let column = Position::char_to_column(ch)?;
            position = position.position_after_drop(column)?;
        }
        Some(position)
    }

    fn char_to_column(ch: char) -> Option<u32> {
        let column = if ch.is_ascii_digit() {
            ch.to_digit(10)? as i32 - 1
        } else if ch.is_ascii_alphabetic() {
            ch.to_ascii_uppercase() as i32 - 65
        } else {
            -1
        };

        if column >= 0 && column < BOARD_WIDTH as i32 {
            Some(column as u32)
        } else {
            None
        }
    }

    /// The reverse of to_string
    pub fn from_string(str: &str) -> Option<Position> {
        let mut first_player = Bitboard::empty();
        let mut second_player = Bitboard::empty();

        for (y, line) in str.split_whitespace().rev().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                match ch {
                    'X' => first_player = first_player.set_disc(x as u32, y as u32),
                    'O' => second_player = second_player.set_disc(x as u32, y as u32),
                    '.' => {}
                    _ => return None,
                }
            }
        }

        let ply = first_player.0.count_ones() + second_player.0.count_ones();
        if ply % 2 == 0 {
            Some(Position::new(first_player, second_player))
        } else {
            Some(Position::new(second_player, first_player))
        }
    }

    pub fn to_other_perspective(&self) -> Position {
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

    pub fn has_anyone_won(&self) -> bool {
        self.current.has_won() || self.other.has_won()
    }

    fn is_white_moves(&self) -> bool {
        self.get_ply() % 2 == 0
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

    pub fn normalize(&self) -> Position {
        let flipped = self.flip();
        let code1 = self.to_position_code();
        let code2 = flipped.to_position_code();
        if code1 < code2 {
            flipped
        } else {
            *self
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
        let threat_cells = self.current.get_threat_cells();
        let empty_cells = FULL_BOARD ^ self.both();
        Bitboard(threat_cells & empty_cells)
    }

    pub fn get_immediate_wins(&self) -> MoveBitmap {
        let threat_cells = self.current.get_threat_cells();
        MoveBitmap(threat_cells & self.get_height_cells())
    }

    pub fn count_threats(&self) -> u32 {
        self.get_threats().0.count_ones()
    }

    /// An integer that uniquely represents the position. For each column, the highest bit indicates
    /// the lowest empty cell and all cells below it are filled. For each filled cell, 1 is the
    /// current player and 0 is the other player.
    pub fn to_position_code(&self) -> BoardInteger {
        // Step 1: current + other creates a silhouette of the board
        // Step 2: after adding bottom row to the silhouette, each column has exactly one bit set
        //         which indicates the height of the column (plus one)
        // Step 3: adding current again is the same as bitwise OR because the result of step 2 has
        //         only zeroes below the height bit
        BOTTOM_ROW + self.current.0 + self.current.0 + self.other.0
    }

    pub fn as_hex_string(&self) -> String {
        format!("{:0>16X}", self.to_position_code())
    }

    pub fn from_hex_string(str: &str) -> Option<Position> {
        let str = str.trim();
        if str.len() == 2 * mem::size_of::<BoardInteger>() {
            let code = BoardInteger::from_str_radix(str, 16).ok()?;
            Position::from_position_code(code)
        } else {
            None
        }
    }

    pub fn to_normalized_position_code(&self) -> (BoardInteger, bool) {
        let flipped = self.flip();
        let code1 = self.to_position_code();
        let code2 = flipped.to_position_code();
        let symmetric = code1 == code2;
        if code1 < code2 {
            (code1, symmetric)
        } else {
            (code2, symmetric)
        }
    }

    pub fn get_legal_moves(&self) -> MoveBitmap {
        MoveBitmap(self.get_height_cells() & FULL_BOARD)
    }

    /// Returns all moves where the opponent does not have a threat directly above. Unblocked moves
    /// are a subset of legal moves.
    ///
    /// Note that this function does not check for forced moves so it is possible that a move
    /// returned here still leads to an immediate loss, just not in the same column.
    pub fn get_unblocked_moves(&self) -> MoveBitmap {
        let legal_moves = self.get_height_cells() & FULL_BOARD;
        let enemy_threats = self.to_other_perspective().get_threats();
        MoveBitmap(!(enemy_threats.0 >> 1) & legal_moves)
    }

    fn all_colums_even(&self) -> bool {
        let both = self.both();
        for x in 0..BOARD_WIDTH {
            let column = (both >> x * BIT_HEIGHT) & FIRST_COLUMN;
            if (column + 1).trailing_zeros() % 2 != 0 {
                return false;
            }
        }
        true
    }

    fn is_column_even(&self, x: u32) -> bool {
        let both = self.both();
        let column = (both >> x * BIT_HEIGHT) & FIRST_COLUMN;
        (column + 1).trailing_zeros() % 2 == 0
    }

    pub fn guess_variation(&self) -> Option<String> {
        fn recurse(target: &Position, variation: &mut String) -> bool {
            let current_position = Position::from_variation(variation).unwrap();
            if current_position == *target {
                return true;
            }
            let target_board = if current_position.is_white_moves() {
                target.get_ordered_boards().0
            } else {
                target.get_ordered_boards().1
            };
            for i in 0..BOARD_WIDTH {
                // Use order: 3,2,4,1,5,0,6
                let x = {
                    let middle = BOARD_WIDTH / 2;
                    if i % 2 == 0 {
                        middle + (i + 1) / 2
                    } else {
                        middle - (i + 1) / 2
                    }
                };
                let y = current_position.get_height(x);
                if target_board.has_disc(x, y) {
                    let ch = std::char::from_digit(x + 1, 10).unwrap();
                    variation.push(ch);
                    if recurse(target, variation) {
                        return true;
                    } else {
                        variation.pop();
                    }
                }
            }
            false
        }

        let mut variation = String::new();
        if recurse(self, &mut variation) {
            Some(variation)
        } else {
            None
        }
    }

    /// What happens if the other player always plays in the same column as the current player.
    /// The score is returned from the current player's perspective. If there are non-losing moves
    /// in an uneven column, the score cannot be determined and Unknown is returned.
    #[inline(always)]
    pub fn autofinish_score(&self, playable_moves: MoveBitmap) -> Score {
        let mut current = self.current.0;
        let mut other = self.other.0;
        let empty = !self.both();

        // the other player can imitate moves only if every playable column has an even number of
        // cells left
        if (playable_moves.0 & EVEN_ROWS) != 0 {
            return Score::Unknown;
        }

        // gutter bit set for each non-losing column
        let playable_gutter = (FULL_BOARD + playable_moves.0) & GUTTER_ROW;
        // gutter bit set for each losing or full column
        let unplayable_gutter = playable_gutter ^ GUTTER_ROW;

        let obtainable_cells = {
            // the same as FULL_BOARD except that unplayable columns are full zeroes
            let playable_columns =
                (playable_gutter | (unplayable_gutter >> BOARD_HEIGHT)) - BOTTOM_ROW;
            let playable_area = playable_columns & empty;

            let even_enemy_threats = Bitboard(other).get_threat_cells() & EVEN_ROWS & playable_area;
            let under_threats = Bitboard(even_enemy_threats).keep_lowest_or_gutter() - BOTTOM_ROW;

            let immediate_cells = self.get_height_cells() & FULL_BOARD;
            immediate_cells | (playable_area & under_threats & ODD_ROWS)
        };

        current |= obtainable_cells;
        other |= (obtainable_cells << 1) & FULL_BOARD;

        if Bitboard(current).has_won() {
            return Score::Unknown;
        }

        if Bitboard(other).has_won() {
            Score::Loss
        } else {
            Score::DrawOrLoss
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_position_code().cmp(&other.to_position_code())
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MoveBitmap {
    pub fn count_moves(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_left_half(&self) -> MoveBitmap {
        MoveBitmap(self.0 & LEFT_HALF)
    }

    #[inline]
    pub fn has_move(&self, column: u32) -> bool {
        let column_bits = (self.0 >> (column * BIT_HEIGHT)) & FIRST_COLUMN;
        column_bits != 0
    }

    /// Initializes the moves represented by this bitmap into an array with a compile-time size.
    /// Creating move arrays is one of the performance bottlenecks which means that something like
    /// a Vec is not an option. It is better to allocate an array on the stack and pass its
    /// reference to this function.
    ///
    /// The return value is a slice of the given array where each item corresponds to a valid move.
    #[inline(always)]
    pub fn init_array<'a, T, F>(
        &self,
        move_array: &'a mut [T; BOARD_WIDTH as usize],
        f: F,
    ) -> &'a mut [T]
    where
        F: Fn(u32) -> T,
    {
        let mut move_count = 0;
        for x in 0..BOARD_WIDTH {
            if self.has_move(x) {
                move_array[move_count] = f(x);
                move_count += 1;
            }
        }
        &mut move_array[0..move_count]
    }

    pub fn as_bitboard(&self) -> Bitboard {
        Bitboard(self.0)
    }
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

/// These macros allow tests to be written in a somewhat more concise way without formatting being
/// affected by cargo fmt.
#[macro_export]
macro_rules! position {
    ($($x:literal)+) => {
        Position::from_string(concat!($($x,"\n",)+)).expect("Invalid position representation");
    };
}

#[macro_export]
macro_rules! bitboard {
    ($($x:literal)+) => {
        Bitboard::from_string(concat!($($x,"\n",)+)).expect("Invalid bitboard representation");
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_macro() {
        let pos = position!(
            "...O..."
            "...X..."
            "...O..."
            "...X..."
            "...O..."
            "...X..."
        );

        let expected = "\
             ...O...\n\
             ...X...\n\
             ...O...\n\
             ...X...\n\
             ...O...\n\
             ...X...\n";

        assert_eq!(pos.to_string(), expected);
    }

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

    #[test]
    fn from_variation() {
        let position = Position::from_variation("444444").unwrap();
        assert_eq!(
            position,
            position!(
                 "...O..."
                 "...X..."
                 "...O..."
                 "...X..."
                 "...O..."
                 "...X..."
            )
        );

        let position = Position::from_variation("436675553").unwrap();
        assert_eq!(
            position,
            position!(
                 "......."
                 "......."
                 "......."
                 "....O.."
                 "..X.XO."
                 "..OXOXX"
            )
        );
    }

    #[test]
    fn height() {
        let position = Position::from_variation("436675553").unwrap();
        assert_eq!(
            position,
            position!(
                 "......."
                 "......."
                 "......."
                 "....O.."
                 "..X.XO."
                 "..OXOXX"
            )
        );
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
        let position = Position::from_variation("436675553").unwrap();
        assert_eq!(
            position,
            position!(
                 "......."
                 "......."
                 "......."
                 "....O.."
                 "..X.XO."
                 "..OXOXX"
            )
        );

        let flipped = position.flip();
        assert_eq!(
            flipped,
            position!(
                 "......."
                 "......."
                 "......."
                 "..O...."
                 ".OX.X.."
                 "XXOXO.."
            )
        );
    }

    #[test]
    fn invalid_move() {
        let position = Position::from_variation("444444").unwrap();
        assert!(position.position_after_drop(3).is_none());
        assert!(position.position_after_drop(0).is_some());
    }

    #[test]
    fn win_checking() {
        // horizontal
        {
            let position = Position::from_variation("4455667").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0000000"
                    "0000000"
                    "0000000"
                    "0000000"
                    "0000000"
                    "0001111"
                )
            );
        }

        // vertical
        {
            let position = Position::from_variation("4343434").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0000000"
                    "0000000"
                    "0001000"
                    "0001000"
                    "0001000"
                    "0001000"
                )
            );
        }

        // slash (/)
        {
            let position = Position::from_variation("45567667677").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0000000"
                    "0000000"
                    "0000001"
                    "0000010"
                    "0000100"
                    "0001000"
                )
            );
        }

        // backslash (\)
        {
            let position = Position::from_variation("76654554544").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0000000"
                    "0000000"
                    "0001000"
                    "0000100"
                    "0000010"
                    "0000001"
                )
            );
        }

        // five in a row
        {
            let position = Position::from_variation("112233554").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0000000"
                    "0000000"
                    "0000000"
                    "0000000"
                    "0000000"
                    "1111100"
                )
            );
        }

        // star
        {
            let position =
                Position::from_variation("431223323133222116756556575566677741414").unwrap();
            let (white_board, red_board) = position.get_ordered_boards();
            assert_eq!(white_board.has_won(), true);
            assert_eq!(red_board.has_won(), false);
            assert_eq!(
                Bitboard(white_board.get_won_cells()),
                bitboard!(
                    "0100010"
                    "0010100"
                    "1111111"
                    "0011100"
                    "0101010"
                    "1001001"
                )
            );
        }
    }

    #[test]
    fn threat_counting() {
        let position = Position::from_variation("43443555").unwrap();
        assert_eq!(position.count_threats(), 2);
        assert_eq!(position.to_other_perspective().count_threats(), 0);
    }

    #[test]
    fn position_code() {
        let position1 = Position::from_variation("43443555").unwrap();
        let position_code = position1.to_position_code();
        let position2 = Position::from_position_code(position_code).unwrap();
        assert_eq!(position1, position2);
    }

    #[test]
    fn even_columns() {
        let position = Position::from_variation("4455").unwrap();
        assert!(position.all_colums_even());
    }
}
