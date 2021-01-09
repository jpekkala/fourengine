use wasm_bindgen::prelude::*;
use fourengine::engine::Engine;
use fourengine::bitboard::Position;
use fourengine::score::Score;
use std::time::Duration;

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
