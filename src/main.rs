use crate::game_state::GameState;
use std::io;

mod bitboard;
mod game_state;
mod trans_table;

fn main() {
    let mut variation = String::new();
    io::stdin()
        .read_line(&mut variation)
        .expect("Failed to read line");

    let won = check_variation(variation);
    println!("The variation contains a win: {}", won);
}

fn check_variation(variation: String) -> bool {
    let mut game_state = GameState::new();

    for ch in variation.trim().chars() {
        let column: u32 = ch.to_digit(10).expect("Expected digit") - 1;
        game_state.drop(column);
    }

    println!("The board is\n{}", game_state);

    return game_state.has_won();
}
