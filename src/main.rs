use crate::bitboard::Position;
use crate::engine::{explore_tree, Engine};
use crate::score::Score;
use clap::{App, Arg};
use std::collections::HashSet;
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
    fn run(engine: &mut Engine) -> Benchmark {
        let start_work = engine.work_count;
        let start_time = Instant::now();
        let score = engine.solve();
        let duration = start_time.elapsed();
        let work_count = engine.work_count - start_work;

        Benchmark {
            score,
            duration,
            work_count,
        }
    }

    fn empty() -> Benchmark {
        Benchmark {
            score: Score::Unknown,
            duration: Duration::from_secs(0),
            work_count: 0,
        }
    }

    fn add(&self, other: Benchmark) -> Benchmark {
        Benchmark {
            score: self.score,
            duration: self.duration + other.duration,
            work_count: self.work_count + other.work_count,
        }
    }

    fn get_speed(&self) -> f64 {
        self.work_count as f64 / self.duration.as_secs_f64()
    }

    fn print(&self) {
        let width = 8;
        println!("The score is {:?}", self.score);
        println!("Work: {}", format_large_number(self.work_count as f64, width));
        println!("Time: {:>width$.3} s", self.duration.as_secs_f64(), width = width);
        println!("Speed: {}/s", format_large_number(self.get_speed(), width - 1));

        fn format_large_number(n: f64, width: usize) -> String {
            if n < 100_000.0 {
                format!("{:>width$}", n, width = width)
            } else {
                format!("{:>width$.3} M", n / 1_000_000.0, width = width)
            }
        }
    }
}

fn run_variation(engine: &mut Engine, variation: &str) -> Result<Benchmark, String> {
    let position = Position::from_variation(&variation).ok_or("Invalid variation")?;
    engine.set_position(position);
    Ok(Benchmark::run(engine))
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

fn generate() {
    let mut set = HashSet::new();
    explore_tree(Position::empty(), 8, &mut |pos| {
        let (pos, _symmetric) = pos.normalize();
        let code = pos.to_position_code();
        let is_immediate_win = pos.get_immediate_wins().count_moves() > 0;
        let is_forced = pos
            .to_other_perspective()
            .get_immediate_wins()
            .count_moves()
            > 0;
        if !pos.has_won() && !is_immediate_win && !is_forced {
            set.insert(code);
        }
    });
    let total_count = set.len();
    println!("{} total positions", total_count);

    let mut total_benchmark = Benchmark::empty();
    let mut engine = Engine::new();
    let mut count = 0;
    explore_tree(Position::empty(), 8, &mut |pos| {
        count += 1;
        engine.set_position(pos);
        let benchmark = Benchmark::run(&mut engine);
        total_benchmark = total_benchmark.add(benchmark);
        if count % 10 == 0 {
            println!(
                "Solved {} out of {}. Speed is {} nodes per second",
                count,
                total_count,
                total_benchmark.get_speed()
            );
        }
    });
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
        .arg(Arg::new("generate").long("generate"))
        .get_matches();

    if let Some(test_file) = matches.value_of("test_file") {
        run_test_file(test_file).expect("Cannot read file");
    } else if matches.is_present("generate") {
        generate();
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
