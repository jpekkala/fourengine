use std::fmt;

use crate::bitboard::{BOARD_HEIGHT, BOARD_WIDTH};
use std::cmp::min;
use std::fmt::Formatter;

pub struct HistoryHeuristic {
    table: [i32; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
}

fn get_index(x: u32, y: u32) -> usize {
    (x * BOARD_HEIGHT + y) as usize
}

impl HistoryHeuristic {
    pub fn new() -> HistoryHeuristic {
        let mut history_heuristic = HistoryHeuristic {
            table: [0; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
        };

        for x in 0..BOARD_WIDTH {
            // give middle cells a slightly better score so they are tried first in absence of everything else
            let value = min(x, BOARD_WIDTH - x - 1) as i32;
            for y in 0..BOARD_HEIGHT {
                history_heuristic.set_score(x, y, value);
            }
        }

        history_heuristic
    }

    pub fn get_score(&self, x: u32, y: u32) -> i32 {
        self.table[get_index(x, y)]
    }

    pub fn set_score(&mut self, x: u32, y: u32, score: i32) {
        self.table[get_index(x, y)] = score;
    }

    pub fn increase_score(&mut self, x: u32, y: u32, score: i32) {
        self.table[get_index(x, y)] += score;
    }
}

impl fmt::Display for HistoryHeuristic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
                write!(f, "{:>10} ", self.get_score(x,y))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
