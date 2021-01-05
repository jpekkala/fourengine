use crate::engine::Engine;
use crate::position::Position;
use crate::score::Score;
use std::io;

mod bitboard;
mod engine;
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
    let result = engine.negamax(Score::Loss, Score::Win, 20);
    println!("The result is {:?}", result);
}
