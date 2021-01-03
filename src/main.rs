use std::io;
use crate::bitboard::Bitboard;

mod bitboard;

fn main() {
    let mut variation = String::new();
    io::stdin()
        .read_line(&mut variation)
        .expect("Failed to read line");

    let won = check_variation(variation);
    println!("The variation contains a win: {}", won);
}

fn check_variation(variation: String) -> bool {
    let won = bitboard::has_won(0);
    let mut current: Bitboard = 0;
    let mut other: Bitboard = 0;

    let mut ply = 0;
    for ch in variation.trim().chars() {
        let column: u32 = ch.to_digit(10).expect("Expected digit") - 1;
        let temp = current | bitboard::drop(current, other, column);
        current = other;
        other = temp;
        ply += 1;
    }

    println!("Current board is {:#066b}", current);
    println!("Other board is   {:#066b}", other);
    bitboard::has_won(current) || bitboard::has_won(other)
}
