use wasm_bindgen::prelude::*;
use fourengine::engine::Engine;
use fourengine::bitboard::{Disc, Position};
use fourengine::score::Score;
use std::time::Duration;

#[wasm_bindgen(js_name = Position)]
pub struct JsPosition {
    position: Position
}

#[wasm_bindgen(js_class = Position)]
impl JsPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(variation: &str) -> JsPosition {
        JsPosition {
            position: Position::from_variation(variation)
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

    #[wasm_bindgen(js_name = canDrop)]
    pub fn can_drop(&self, x: u32) -> bool {
        self.position.drop(x).is_legal()
    }
}

/// This wrapper exists only for annotating with wasm_bindgen
#[wasm_bindgen]
pub struct EngineWrapper {
    engine: Engine
}

#[wasm_bindgen]
pub struct Solution {
    score: Score,
    work_count: usize,
}

#[wasm_bindgen]
impl Solution {
    pub fn get_score(&self) -> String {
        format!("{:?}", self.score)
    }

    pub fn get_work_count(&self) -> usize {
        self.work_count
    }
}

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
pub fn init_engine() -> EngineWrapper {
    EngineWrapper {
        engine: Engine::new()
    }
}

#[wasm_bindgen]
pub fn solve(engine_wrapper: &mut EngineWrapper, variation: &str) -> Solution {
    let mut engine = &mut engine_wrapper.engine;
    let position = Position::from_variation(variation);
    engine.set_position(position);
    engine.work_count = 0;
    let score = engine.solve();
    Solution {
        score,
        work_count: engine.work_count,
    }
}
