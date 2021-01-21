use fourengine::bitboard::{Bitboard, Disc, Position};
use fourengine::engine::Engine;
use fourengine::score::Score;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Position)]
pub struct JsPosition {
    position: Position,
}

#[wasm_bindgen(js_class = Position)]
impl JsPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(variation: &str) -> JsPosition {
        JsPosition {
            position: Position::from_variation(variation).unwrap(),
        }
    }

    #[wasm_bindgen(js_name = getCell)]
    pub fn get_cell(&self, x: u32, y: u32) -> u32 {
        match self.position.get_disc_at(x, y) {
            Disc::White => 1,
            Disc::Red => 2,
            Disc::Empty => 0,
        }
    }

    #[wasm_bindgen(js_name = hasWon)]
    pub fn has_won(&self) -> bool {
        self.position.has_won()
    }

    #[wasm_bindgen(js_name = isWinningCell)]
    pub fn is_winning_cell(&self, x: u32, y: u32) -> bool {
        let board = Bitboard(self.position.other.get_won_cells());
        // return board.to_string()
        board.has_disc(x, y)
    }

    #[wasm_bindgen(js_name = canDrop)]
    pub fn can_drop(&self, x: u32) -> bool {
        self.position.drop(x).is_legal()
    }

    #[wasm_bindgen]
    pub fn drop(&self, x: u32) -> Option<JsPosition> {
        let new_position = self.position.position_after_drop(x);
        match new_position {
            Some(position) => Some(JsPosition {
                position,
            }),
            None => None,
        }
    }

    #[wasm_bindgen(js_name = getHeight)]
    pub fn get_height(&self, x: u32) -> u32 {
        self.position.get_height(x)
    }
}

#[wasm_bindgen(js_name = Engine)]
pub struct JsEngine {
    engine: Engine,
}

#[wasm_bindgen(js_class = Engine)]
impl JsEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsEngine {
        JsEngine {
            engine: Engine::new(),
        }
    }

    #[wasm_bindgen]
    pub fn solve(&mut self, variation: &str) -> Solution {
        let mut engine = &mut self.engine;
        let position = Position::from_variation(variation).unwrap();
        engine.set_position(position);
        engine.work_count = 0;
        let score = engine.solve();
        Solution {
            score,
            work_count: engine.work_count,
        }
    }
}

#[wasm_bindgen]
pub struct Solution {
    score: Score,
    work_count: usize,
}

#[wasm_bindgen]
impl Solution {
    #[wasm_bindgen(js_name = getScore)]
    pub fn get_score(&self) -> String {
        format!("{:?}", self.score)
    }

    #[wasm_bindgen(js_name = getWorkCount)]
    pub fn get_work_count(&self) -> usize {
        self.work_count
    }
}
