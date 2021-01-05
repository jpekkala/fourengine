use crate::constants::Score;
use crate::engine::Engine;
use crate::position::Position;
use std::io;

mod bitboard;
mod constants;
mod engine;
mod position;
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
    let result = engine.negamax(Score::Loss, Score::Win, 20);
    println!("The result is {:?}", result);
}
