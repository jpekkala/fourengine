use wasm_bindgen::prelude::*;
use crate::engine::Engine;
use crate::bitboard::Position;

mod bitboard;
mod engine;
mod heuristic;
mod score;
mod trans_table;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn show_position(variation: &str) {
    let position = Position::from_variation(variation);
    alert(&format!("{}", position));
}

#[wasm_bindgen]
pub fn solve(variation: &str) -> usize {
    let position = Position::from_variation(variation);
    let mut engine = Engine::new();
    engine.set_position(position);
    let score = engine.solve();
    //alert(&format!("Score: {:?}", score));
    engine.work_count
}
