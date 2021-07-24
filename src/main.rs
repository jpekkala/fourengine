use crate::benchmark::Benchmark;
use crate::bitboard::Position;
use crate::book::{Book, DEFAULT_BOOK_PLY, generate_book, get_path_for_ply, verify_book};
use crate::engine::Engine;
use crate::score::Score;
use clap::{crate_version, App, Arg, ArgMatches};
use std::cmp::Ordering;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub mod benchmark;
pub mod bitboard;
pub mod book;
pub mod engine;
pub mod heuristic;
pub mod score;
pub mod trans_table;

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

    let score = match score_value.cmp(&0) {
        Ordering::Less => Score::Loss,
        Ordering::Equal => Score::Draw,
        Ordering::Greater => Score::Win,
    };

    Some((variation, score))
}

fn play(matches: &ArgMatches) -> Result<(), String> {
    let use_book = !matches.is_present("no-book");
    if use_book {
        let path_buf = get_path_for_ply(DEFAULT_BOOK_PLY);
        let book_exists = path_buf.as_path().exists();
        if !book_exists {
            println!("The book file {} does not exist. You can generate it with --generate-book", path_buf.display());
        }
    }

    let variation = {
        let mut str = String::new();
        println!("Input variation:");
        io::stdin()
            .read_line(&mut str)
            .expect("Failed to read line");
        str
    };

    solve(&variation, use_book)
}

fn solve(variation: &str, use_book: bool) -> Result<(), String> {
    let position = Position::from_variation(&variation).unwrap();
    println!(
        "The board is ({} moves next)\n{}",
        if position.get_ply() % 2 == 0 {
            "white"
        } else {
            "red"
        },
        position,
    );
    if use_book {
        println!("Solving (book enabled)...");
    } else {
        println!("Solving (book disabled)...")
    }

    let mut engine = Engine::new();
    if use_book {
        let book = Box::new(Book::open_for_ply_or_empty(DEFAULT_BOOK_PLY));
        engine.set_book(book);
    }
    match run_variation(&mut engine, &variation) {
        Ok(benchmark) => {
            benchmark.print();
            Ok(())
        },
        Err(str) => Err(str),
    }
}

fn main() {
    let matches = App::new("Fourengine")
        .version(crate_version!())
        .about("Connect-4 engine")
        .author("Jukka Pekkala, Johan Nordlund")
        .arg(
            Arg::new("no-book")
                .long("no-book")
                .about("Disables opening book")
        )
        .subcommand(App::new("format-book")
            .about("Converts a book to another format")
            .arg(
                Arg::new("book-file")
                    .required(true)
                    .index(1)
            )
        )
        .subcommand(App::new("generate-book")
            .about("Generates and saves an opening book")
            .arg(
                Arg::new("out")
                    .long("out")
                    .takes_value(true)
            )
        )
        .subcommand(App::new("solve")
            .about("Solves a position")
            .arg(
                Arg::new("variation")
                    .required(false)
                    .index(1)
            )
        )
        .subcommand(App::new("test")
            .about("Runs a test set from a file")
            .arg(
                Arg::new("file")
                    .required(true)
                    .index(1)
            )
        )
        .subcommand(App::new("verify-book")
            .about("Compares and verifies a book against a reference book")
            .arg(
                Arg::new("book")
                    .index(1)
                    .required(true)
            )
            .arg(
                Arg::new("reference_book")
                    .index(2)
                    .required(true)
            )
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("generate-book", sub_matches)) => {
            generate_book().or_else(|err| Err(err.to_string()))
        },
        Some(("solve", sub_matches)) => {
            let variation = sub_matches.value_of("variation").unwrap_or("");
            solve(variation, false)
        },
        Some(("test", sub_matches)) => {
            let file = sub_matches.value_of("file").unwrap();
            run_test_file(file)
        },
        Some(("verify-book", sub_matches)) => {
            let book = Path::new(sub_matches.value_of("book").unwrap());
            let reference_book = Path::new(sub_matches.value_of("reference_book").unwrap());
            verify_book(book, reference_book).or_else(|err| Err(err.to_string()))
        }
        _ => play(&matches)
    };

    if let Err(str) = result {
        eprintln!("{}", str);
    }
}
