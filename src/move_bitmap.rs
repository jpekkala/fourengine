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
}