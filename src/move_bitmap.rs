use crate::bitboard::{BIT_HEIGHT, Bitboard, BOARD_WIDTH, BoardInteger, FIRST_COLUMN};

/// Available moves as a bitmap where each column can have max 1 bit set.
#[derive(Copy, Clone)]
pub struct MoveBitmap(pub BoardInteger);

impl MoveBitmap {
    pub fn count_moves(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_left_half(&self) -> MoveBitmap {
        MoveBitmap(self.0 & crate::bitboard::LEFT_HALF)
    }

    #[inline]
    pub fn has_move(&self, column: u32) -> bool {
        let column_bits = (self.0 >> (column * BIT_HEIGHT)) & FIRST_COLUMN;
        column_bits != 0
    }

    #[inline]
    pub fn unset_move(&self, column: u32) -> MoveBitmap {
        let mask = !(FIRST_COLUMN << (column * BIT_HEIGHT));
        return MoveBitmap(self.0 & mask);
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

    pub fn from_bitboard_string(s: &str) -> Option<MoveBitmap> {
        let bitboard = Bitboard::from_string(s)?;
        Some(MoveBitmap(bitboard.0))
    }

    pub fn to_bitboard_string(&self) -> String {
        self.as_bitboard().to_string()
    }
}

// Initialize MoveBitmap from a visual string representation
#[macro_export]
macro_rules! move_bitmap {
    ($($x:literal)+) => {
        MoveBitmap::from_bitboard_string(concat!($($x,"\n",)+)).expect("Invalid move bitmap representation")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unset_move() {
        let bitmap = move_bitmap!(
            "0000000"
            "0001000"
            "0000000"
            "0001000"
            "0000000"
            "0001000"
        );

        let new_bitmap = bitmap.unset_move(3);
        let expected = move_bitmap!(
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
        );

        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());
    }

    #[test]
    fn test_unset_move_on_full_bitmap() {
        let bitmap = move_bitmap!(
            "1111111"
            "1111111"
            "1111111"
            "1111111"
            "1111111"
            "1111111"
        );

        // Unset the move in the first column (index 0)
        let new_bitmap = bitmap.unset_move(0);
        let expected = move_bitmap!(
            "0111111"
            "0111111"
            "0111111"
            "0111111"
            "0111111"
            "0111111"
        );

        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());

        // Unset the move in the last column (index 6)
        let new_bitmap = new_bitmap.unset_move(6);
        let expected = move_bitmap!(
            "0111110"
            "0111110"
            "0111110"
            "0111110"
            "0111110"
            "0111110"
        );

        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());
    }

    #[test]
    fn test_unset_move_on_empty_bitmap() {
        let bitmap = move_bitmap!(
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
        );

        let new_bitmap = bitmap.unset_move(0);
        let expected = move_bitmap!(
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
        );

        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());
    }

    #[test]
    fn test_unset_move_in_all_columns() {
        let bitmap = move_bitmap!(
            "1111111"
            "1111111"
            "1111111"
            "1111111"
            "1111111"
            "1111111"
        );

        // Unset moves in all columns
        let mut new_bitmap = bitmap;
        for i in 0..BOARD_WIDTH {
            new_bitmap = new_bitmap.unset_move(i);
        }
        let expected = move_bitmap!(
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
            "0000000"
        );

        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());
    }

    #[test]
    fn test_unset_move_does_not_affect_other_columns() {
        let bitmap = move_bitmap!(
            "1010101"
            "1010101"
            "1010101"
            "1010101"
            "1010101"
            "1010101"
        );

        let new_bitmap = bitmap.unset_move(2);
        let expected = move_bitmap!(
            "1000101"
            "1000101"
            "1000101"
            "1000101"
            "1000101"
            "1000101"
        );
        assert_eq!(new_bitmap.to_bitboard_string(), expected.to_bitboard_string());
    }
}
