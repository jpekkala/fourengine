use crate::engine::Engine;
use crate::position::Position;
use crate::score::Score;
use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use std::{env, io};

mod bitboard;
mod engine;
mod heuristic;
mod position;
mod score;
mod trans_table;

struct Benchmark {
    variation: String,
    score: Score,
    duration: Duration,
    work_count: usize,
}

impl Benchmark {
    fn print(&self) {
        println!("The score is {:?}", self.score);
        println!("Work count is {}", self.work_count);
        println!("Elapsed time is {:?}", self.duration);
        println!(
            "Nodes per second: {}",
            self.work_count as f64 / self.duration.as_secs_f64()
        );
    }
}

fn run_variation(variation: &str) -> Benchmark {
    let position = Position::from_variation(&variation);
    let mut engine = Engine::new(position);
    let start = Instant::now();
    let score = engine.solve();
    let duration = start.elapsed();

    Benchmark {
        variation: String::from(variation),
        score,
        duration,
        work_count: engine.work_count,
    }
}

fn read_from_stdin() {
    let mut variation = String::new();
    println!("Input variation:");
    io::stdin()
        .read_line(&mut variation)
        .expect("Failed to read line");

    println!("The board is\n{}", Position::from_variation(&variation));
    println!("Solving...");
    let benchmark = run_variation(&variation);
    benchmark.print();
}

fn run_test_file(filename: &str) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut benchmarks = Vec::new();
    for line in reader.lines() {
        if let Some((variation, score)) = parse_line(line?) {
            println!(
                "Expecting score {:<4} for variation {}",
                format!("{:?}", score),
                variation
            );
            let benchmark = run_variation(&variation);
            assert_eq!(benchmark.score, score, "Invalid score");
            benchmarks.push(benchmark);
        }
    }

    let mut total_work_count = 0;
    let mut total_elapsed_time = 0.0;
    for b in &benchmarks {
        total_work_count += b.work_count;
        total_elapsed_time += b.duration.as_secs_f64();
    }

    println!("Total elapsed time: {}", total_elapsed_time);
    println!(
        "Average elapsed time: {}",
        total_elapsed_time / benchmarks.len() as f64
    );
    println!(
        "Average work count: {}",
        total_work_count / benchmarks.len()
    );
    println!(
        "Nodes per second: {}",
        total_work_count as f64 / total_elapsed_time
    );
    Ok(())
}

fn parse_line(line: String) -> Option<(String, Score)> {
    let mut iter = line.split_whitespace();
    let variation = String::from(iter.next()?);
    let score_value = iter.next()?.parse::<i32>().unwrap();

    let score = if score_value == 0 {
        Score::Draw
    } else if score_value < 0 {
        Score::Loss
    } else {
        Score::Win
    };

    Some((variation, score))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = App::new("Fourengine")
        .version("1.0")
        .about("Connect-4 engine")
        .author("Jukka Pekkala, Johan Nordlund")
        .arg(
            Arg::new("test_file")
                .long("test")
                .short('t')
                .about("Runs a test set from a file")
                .takes_value(true),
        )
        .get_matches();

    if let Some(test_file) = matches.value_of("test_file") {
        run_test_file(test_file);
    } else {
        read_from_stdin();
    }
}
