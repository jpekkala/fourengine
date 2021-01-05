use crate::engine::Engine;
use crate::position::Position;
use std::io;
use std::time::Instant;

mod bitboard;
mod engine;
mod history_heuristic;
mod position;
mod score;
mod trans_table;

fn main() {
    let mut variation = String::new();
    io::stdin()
        .read_line(&mut variation)
        .expect("Failed to read line");

    let position = Position::from_variation(&variation);
    println!("The board is\n{}", position);
    println!("Solving...");
    let mut engine = Engine::new(position);
    let start = Instant::now();
    let result = engine.solve();
    let duration = start.elapsed();
    println!("The result is {:?}", result);
    println!("Work count is {}", engine.work_count);
    println!("Elapsed time is {:?}", duration);
    println!(
        "Nodes per second: {}",
        engine.work_count as f64 / duration.as_secs_f64()
    );
}
