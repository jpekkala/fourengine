use num_derive::FromPrimitive;

/// board dimensions
pub const BOARD_WIDTH: u32 = 7;
pub const BOARD_HEIGHT: u32 = 6;

/// The number of bits needed to encode a position
pub const POSITION_BITS: u32 = (BOARD_HEIGHT + 1) * BOARD_WIDTH;

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum Score {
    Loss = 1,
    DrawOrLoss,
    Draw,
    DrawOrWin,
    Win,
    Unknown = 0,
}
