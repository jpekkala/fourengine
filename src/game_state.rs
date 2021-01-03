use std::fmt;

use crate::bitboard;
use crate::bitboard::Bitboard;
use std::fmt::Formatter;

pub struct GameState {
    pub ply: u32,

    pub current: Bitboard,
    pub other: Bitboard,
}

enum Player {
    WHITE,
    RED,
    EMPTY,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            current: 0,
            other: 0,
        }
    }

    pub fn drop(&mut self, column: u32) {
        let new_board = self.current | bitboard::drop(self.current, self.other, column);
        if !bitboard::is_legal(new_board) {
            panic!("Invalid move");
        }
        self.current = self.other;
        self.other = new_board;
        self.ply += 1;
    }

    pub fn has_won(&self) -> bool {
        bitboard::has_won(self.current) || bitboard::has_won(self.other)
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}
