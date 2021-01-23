use crate::benchmark::{format_large_number, Benchmark};
use crate::bitboard::{Position, BOARD_WIDTH};
use crate::engine::Engine;
use crate::score::Score;
use std::collections::HashSet;
use std::fs::{create_dir, File};
use std::io::{LineWriter, Write};
use std::time::Instant;

struct BookWriter {
    file: LineWriter<File>,
}

impl BookWriter {
    fn create(file_name: &str) -> Result<BookWriter, std::io::Error> {
        let file = File::create(file_name)?;
        Ok(BookWriter {
            file: LineWriter::new(file),
        })
    }

    fn write_entry(&mut self, pos: Position, score: Score) -> Result<(), std::io::Error> {
        let line = format_entry(pos, score);
        println!("{}", line);
        self.file.write_all(line.as_bytes())?;
        self.file.write_all(b"\n")
    }
}

fn format_entry(pos: Position, score: Score) -> String {
    format!("{:0>16X}{}", pos.to_position_code(), score.to_char())
}

pub fn generate() -> Result<(), std::io::Error> {
    let set = find_positions_to_solve();
    let total_count = set.len();
    println!("There are {} positions to solve", total_count);

    let start_time = Instant::now();
    let mut total_benchmark = Benchmark::empty();
    let mut engine = Engine::new();
    let mut count = 0;
    create_dir("books")?;
    let mut book_writer = BookWriter::create("books/8-ply.txt")?;
    for pos in set {
        count += 1;
        engine.set_position(pos);
        let benchmark = Benchmark::run(&mut engine);
        book_writer.write_entry(pos, benchmark.score)?;
        total_benchmark = total_benchmark.add(benchmark);
        if count % 10 == 0 {
            let duration = start_time.elapsed();
            let speed = count as f64 / duration.as_secs_f64();
            let left_secs = (total_count - count) as f64 / speed;
            println!(
                "Solved {} out of {}. Speed is {} nodes per second. Estimated minutes left: {:.2}",
                count,
                total_count,
                format_large_number(total_benchmark.get_speed(), 0),
                left_secs / 60.0,
            );
        }
    }
    Ok(())
}

fn find_positions_to_solve() -> HashSet<Position> {
    let mut set = HashSet::new();
    explore_tree(Position::empty(), 8, &mut |pos| {
        let (pos, _symmetric) = pos.normalize();
        set.insert(pos);
    });
    set
}

pub fn explore_tree<F>(position: Position, max_depth: u32, f: &mut F)
where
    F: FnMut(Position),
{
    if max_depth == 0 {
        f(position);
        return;
    }

    let mut move_bitmap = position.get_nonlosing_moves();
    let (_position_code, symmetric) = position.to_normalized_position_code();
    if symmetric {
        move_bitmap = move_bitmap.get_left_half();
    }

    let mut move_array = [Position::empty(); BOARD_WIDTH as usize];
    let possible_moves = move_bitmap.init_array(&mut move_array, |x| {
        position.position_after_drop(x).unwrap()
    });

    for m in possible_moves {
        explore_tree(*m, max_depth - 1, f);
    }
}
