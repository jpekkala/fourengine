use std::io;
use crate::bitboard::Bitboard;
use crate::game_state::GameState;

mod bitboard;
mod game_state;

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

    println!("Current board is {:#066b}", game_state.current);
    println!("Other board is   {:#066b}", game_state.other);

    return game_state.has_won();
}
