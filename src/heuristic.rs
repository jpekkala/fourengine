#![allow(dead_code)]

use std::fmt;

use crate::bitboard::{BOARD_HEIGHT, BOARD_WIDTH};
use std::cmp::min;
use std::fmt::Formatter;

pub trait Heuristic {
    fn get_value(&self, x: u32, y: u32) -> i32;
    fn increase_value(&self, x: u32, y: u32, amount: i32);
}

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
                history_heuristic.set_value(x, y, value);
            }
        }

        history_heuristic
    }

    pub fn get_value(&self, x: u32, y: u32) -> i32 {
        self.table[get_index(x, y)]
    }

    fn set_value(&mut self, x: u32, y: u32, score: i32) {
        self.table[get_index(x, y)] = score;
    }

    pub fn increase_value(&mut self, x: u32, y: u32, score: i32) {
        self.table[get_index(x, y)] += score;
    }
}

impl Default for HistoryHeuristic {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FixedHeuristic;

// https://www.scirp.org/html/1-9601415_90972.htm
// const TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
//     [3, 4, 5, 7, 5, 4, 3],
//     [4, 6, 8, 10, 8, 6, 4],
//     [5, 8, 11, 13, 11, 8, 5],
//     [5, 8, 11, 13, 11, 8, 5],
//     [4, 6, 8, 10, 8, 6, 4],
//     [3, 4, 5, 7, 5, 4, 3],
// ];

const TABLE: [[i32; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize] = [
    [3, 5, 7, 11, 7, 5, 3],
    [4, 6, 10, 12, 10, 6, 4],
    [5, 10, 11, 12, 11, 10, 5],
    [4, 8, 10, 11, 10, 8, 4],
    [3, 6, 8, 10, 8, 6, 3],
    [2, 3, 5, 7, 5, 3, 2],
];

// impl FixedHeuristic {
//     pub fn new() -> FixedHeuristic {
//         FixedHeuristic()
//     }
// }

impl Heuristic for FixedHeuristic {
    fn get_value(&self, x: u32, y: u32) -> i32 {
        TABLE[(BOARD_HEIGHT - y - 1) as usize][x as usize]
    }

    fn increase_value(&self, _x: u32, _y: u32, _amount: i32) {}
}

impl fmt::Display for HistoryHeuristic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
                write!(f, "{:>10} ", self.get_value(x, y))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
