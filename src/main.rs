use crate::position::Position;
use std::io;

mod bitboard;
mod constants;
mod position;
mod trans_table;

fn main() {
    let mut variation = String::new();
    io::stdin()
        .read_line(&mut variation)
        .expect("Failed to read line");

    let position = Position::from_variation(variation);

    println!("The board is\n{}", position);
    println!("The variation contains a win: {}", position.has_won());
}
