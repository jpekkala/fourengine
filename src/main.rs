use crate::bitboard::Position;
use crate::engine::Engine;
use crate::score::Score;
use clap::{App, Arg};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

pub mod bitboard;
pub mod engine;
pub mod heuristic;
pub mod score;
pub mod trans_table;

struct Benchmark {
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

fn run_variation(engine: &mut Engine, variation: &str) -> Result<Benchmark, String> {
    let position = Position::from_variation(&variation).ok_or("Invalid variation")?;
    engine.set_position(position);
    let start = Instant::now();
    let score = engine.solve();
    let duration = start.elapsed();

    Ok(Benchmark {
        score,
        duration,
        work_count: engine.work_count,
    })
}

fn run_test_file(filename: &str) -> Result<(), String> {
    let file = File::open(filename).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut benchmarks = Vec::new();
    let mut engine = Engine::new();
    for line in reader.lines() {
        if let Some((variation, score)) = parse_line(line.map_err(|e| e.to_string())?) {
            println!(
                "Expecting score {:<4} for variation {}",
                format!("{:?}", score),
                variation
            );
            //engine.reset();
            engine.work_count = 0;
            let benchmark = run_variation(&mut engine, &variation)?;
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
        .arg(
            Arg::new("variation")
                .long("variation")
                .about("Runs a specific variation")
                .takes_value(true),
        )
        .get_matches();

    if let Some(test_file) = matches.value_of("test_file") {
        run_test_file(test_file).expect("Cannot read file");
    } else {
        let variation = match matches.value_of("variation") {
            Some(variation) => String::from(variation),
            None => {
                let mut str = String::new();
                println!("Input variation:");
                io::stdin()
                    .read_line(&mut str)
                    .expect("Failed to read line");
                str
            }
        };

        println!(
            "The board is\n{}",
            Position::from_variation(&variation).unwrap()
        );
        println!("Solving...");
        let mut engine = Engine::new();
        let benchmark = run_variation(&mut engine, &variation);
        match benchmark {
            Ok(benchmark) => benchmark.print(),
            Err(str) => eprintln!("{}", str),
        }
    }
}
