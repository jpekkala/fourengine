pub type Bitboard = u64;

// board dimensions
pub const WIDTH: u32 = 7;
pub const HEIGHT: u32 = 6;

// the column height including the buffer cell
pub const BIT_HEIGHT: u32 = HEIGHT + 1;

const ALL_BITS: Bitboard = (1 << (BIT_HEIGHT * WIDTH)) - 1;
const FIRST_COLUMN: Bitboard = (1 << BIT_HEIGHT) - 1;
const BOTTOM_ROW: Bitboard = ALL_BITS / FIRST_COLUMN;
const TOP_ROW: Bitboard = BOTTOM_ROW << HEIGHT;
const FULL_BOARD: Bitboard = ALL_BITS ^ TOP_ROW;

pub fn has_won(board: Bitboard) -> bool {
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

fn get_height_bit(p1: Bitboard, p2: Bitboard, column: u32) -> Bitboard {
    let columnMask = FIRST_COLUMN << (BIT_HEIGHT * column);
    //TODO: check x86 BSR instruction
    let mut both = p1 | p2;
    both &= columnMask;
    both += 1 << (BIT_HEIGHT * column);
    both
}

pub fn drop(current: Bitboard, other: Bitboard, column: u32) -> Bitboard {
    let new_board = get_height_bit(current, other, column);
    // TODO: check legality
    new_board
}

pub fn is_legal(board: Bitboard) -> bool {
    (TOP_ROW & board) == 0
}

pub fn get_position_code(p1: Bitboard, p2: Bitboard) -> Bitboard {
    BOTTOM_ROW + p1 + p1 + p2
}
