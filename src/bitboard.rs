pub type Bitboard = u64;

// board dimensions
const WIDTH: u32 = 7;
const HEIGHT: u32 = 6;

const H1: u32 = HEIGHT + 1;
const H2: u32 = HEIGHT + 2;

const COL1: Bitboard = (1 << H1) - 1;

pub fn has_won(board: Bitboard) -> bool {
    let vertical = board & (board >> 1);
    let horizontal = board & (board >> H1);
    let slash = board & (board >> HEIGHT);
    let backslash = board & (board >> H2);

    let result = (vertical & (vertical >> 2)) | (horizontal & (horizontal >> 2 * H1)) | (slash & (slash >> 2 * HEIGHT)) | (backslash & (backslash >> 2 * H2));
    result != 0
}

fn get_height_bit(p1: Bitboard, p2: Bitboard, column: u32) -> Bitboard {
    let columnMask = COL1 << (H1 * column);
    //TODO: check x86 BSR instruction
    let mut both = p1 | p2;
    both &= columnMask;
    both += 1 << (H1 * column);
    both
}

pub fn drop(current: Bitboard, other: Bitboard, column: u32) -> Bitboard {
    let new_board = get_height_bit(current, other, column);
    // TODO: check legality
    new_board
}
