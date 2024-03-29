use clap::{crate_version, App, Arg, ArgMatches};
use fourengine::benchmark::Benchmark;
use fourengine::bitboard::{Bitboard, Position};
use fourengine::book::{
    generate_book, get_path_for_ply, verify_book, Book, BookFormat, BookWriter, DEFAULT_BOOK_PLY,
};
use fourengine::engine::Engine;
use fourengine::score::Score;
use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;

/// User input representing a position. The purpose of this is to be able to report errors using
/// the same string that the user gave. Using Position directly would lose that information.
enum PositionInput {
    Variation(String),
    Hex(String),
}

impl PositionInput {
    fn parse(&self) -> Result<Position, String> {
        match self {
            Self::Variation(str) => Position::from_variation(str)
                // Automatically try hex for convenience. In the standard board size, hex codes
                // always start with leading zeroes which cannot happen in variations. There might
                // be strings in other board sizes that are valid in both formats but for those
                // situations the user can explicitly use --hex
                .or_else(|| Position::from_hex_string(str))
                .ok_or(format!("Invalid variation: {}", str)),
            Self::Hex(str) => {
                Position::from_hex_string(str).ok_or(format!("Invalid hex code: {}", str))
            }
        }
    }
}

impl fmt::Display for PositionInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Variation(str) => write!(f, "{}", str),
            Self::Hex(str) => write!(f, "{}", str),
        }
    }
}

fn run_test_files<'a>(filenames: &mut impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut total_benchmark = Benchmark::empty();
    for filename in filenames {
        let benchmark = verify_and_benchmark_file(filename)?;
        total_benchmark = total_benchmark.add(&benchmark)
    }
    total_benchmark.print();
    println!("\nAll ok!");
    Ok(())
}

fn verify_and_benchmark_file(filename: &str) -> Result<Benchmark, String> {
    let file = File::open(filename).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut total_benchmark = Benchmark::empty();
    let mut engine = Engine::new();
    for line in reader.lines() {
        if let Some((pos_input, score)) = parse_line_with_score(line.map_err(|e| e.to_string())?) {
            println!(
                "Expecting score {:<4} for variation {}",
                format!("{:?}", score),
                pos_input
            );
            //engine.reset();
            engine.work_count = 0;
            engine.set_position(pos_input.parse()?);
            let benchmark = Benchmark::run(&mut engine);
            assert_eq!(benchmark.score, score, "Invalid score");
            total_benchmark = total_benchmark.add(&benchmark);
        }
    }

    Ok(total_benchmark)
}

fn parse_line_with_score(line: String) -> Option<(PositionInput, Score)> {
    let mut iter = line.split_whitespace();
    let variation = String::from(iter.next()?);
    let score_value = iter.next()?.parse::<i32>().unwrap();

    let score = match score_value.cmp(&0) {
        Ordering::Less => Score::Loss,
        Ordering::Equal => Score::Draw,
        Ordering::Greater => Score::Win,
    };

    Some((PositionInput::Variation(variation), score))
}

pub fn format_book(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let book_file = Path::new(matches.value_of("book-file").unwrap());
    let book = Book::open(book_file)?;

    let book_format = match matches.value_of("format").unwrap() {
        "binary" => BookFormat::Binary,
        "hex" => BookFormat::Hex,
        &_ => panic!("Invalid format"),
    };

    let omit_won = matches.is_present("omit-won");
    let omit_forced = matches.is_present("omit-forced");
    let filtered_entries = book.iter().filter(|entry| {
        let position = entry.get_position();

        if omit_won && position.has_anyone_won() {
            return false;
        }

        if omit_forced {
            let enemy_threats = position.to_other_perspective().get_immediate_wins();
            let is_forced_move = enemy_threats.0 != 0;
            if is_forced_move {
                return false;
            }
        }

        true
    });

    if matches.is_present("count-only") {
        println!("{}", filtered_entries.count());
        return Ok(());
    }

    let writer: Box<dyn Write> = match matches.value_of("out") {
        None => Box::new(io::stdout()),
        Some(path) => {
            let path = Path::new(path);
            let writer = LineWriter::new(File::create(path)?);
            Box::new(writer)
        }
    };

    let mut book_writer = BookWriter::create(writer, book_format);
    for entry in filtered_entries {
        let entry = entry;
        book_writer.write_entry(entry)?;
    }
    Ok(())
}

fn play(matches: &ArgMatches) -> Result<(), String> {
    let use_book = !matches.is_present("no-book");
    if use_book {
        let path_buf = get_path_for_ply(DEFAULT_BOOK_PLY);
        let book_exists = path_buf.as_path().exists();
        if !book_exists {
            println!(
                "The book file {} does not exist. You can generate it with --generate-book",
                path_buf.display()
            );
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

    let position_input = PositionInput::Variation(variation);
    solve(position_input, use_book)
}

fn print_subcommand(matches: &ArgMatches) -> Result<(), String> {
    let variation = matches.value_of("variation").unwrap_or("");
    let position = if matches.is_present("hex") {
        PositionInput::Hex(String::from(variation))
    } else {
        PositionInput::Variation(String::from(variation))
    }
    .parse()?;

    print_board(position);

    if matches.is_present("technical") {
        println!();
        println!("Hex code: {}", position.as_hex_string());
        let (normalized_code, symmetric) = position.to_normalized_position_code();
        println!(
            "Normalized code: {}",
            Position::from_position_code(normalized_code)
                .unwrap()
                .as_hex_string()
        );
        println!("Symmetric: {}", symmetric);
        println!(
            "Guessed variation: {}",
            position
                .guess_variation()
                .unwrap_or_else(|| "N/A".to_string())
        );
        let unblocked_moves = position.get_unblocked_moves();
        println!(
            "Autoscore: {:?}",
            position.autofinish_score(unblocked_moves)
        );
        println!();
        print_bitboard("Current", position.current);
        print_bitboard("Other", position.other);
        print_bitboard("Legal moves", position.get_legal_moves().as_bitboard());
        print_bitboard("Unblocked moves", unblocked_moves.as_bitboard());
        print_bitboard(
            "Immediate wins",
            position.get_immediate_wins().as_bitboard(),
        );
        print_bitboard(
            "Immediate threats",
            position
                .to_other_perspective()
                .get_immediate_wins()
                .as_bitboard(),
        );
    }
    Ok(())
}

fn print_bitboard(title: &str, bitboard: Bitboard) {
    println!(
        "{} ({} bits):\n{}",
        title,
        bitboard.0.count_ones(),
        bitboard
    );
}

fn print_board(position: Position) {
    println!(
        "The board is:\n{}\nPlayer {} moves next",
        position,
        if position.get_ply() % 2 == 0 {
            "X"
        } else {
            "O"
        }
    );
}

fn solve(pos_input: PositionInput, use_book: bool) -> Result<(), String> {
    let position = pos_input.parse()?;
    print_board(position);
    if use_book {
        println!("Solving (book enabled)...");
    } else {
        println!("Solving (book disabled)...")
    }

    let mut engine = Engine::new();
    if use_book {
        let book = Box::new(Book::standard());
        engine.set_book(book);
    }
    engine.set_position(position);
    let benchmark = Benchmark::run(&mut engine);
    println!();
    benchmark.print();
    Ok(())
}

fn main() {
    let matches = App::new("Fourengine")
        .version(crate_version!())
        .about("Connect-4 engine")
        .author("Jukka Pekkala, Johan Nordlund")
        .arg(
            Arg::new("no-book")
                .long("no-book")
                .help("Disables opening book"),
        )
        .subcommand(
            App::new("format-book")
                .about("Converts a book to another format")
                .arg(Arg::new("book-file").required(true).index(1))
                .arg(Arg::new("out").long("out").takes_value(true))
                .arg(Arg::new("count-only").long("count-only"))
                .arg(
                    Arg::new("format")
                        .long("format")
                        .possible_value("hex")
                        .possible_value("binary")
                        .default_value("hex"),
                )
                .arg(Arg::new("omit-forced").long("omit-forced"))
                .arg(Arg::new("omit-won").long("omit-won")),
        )
        .subcommand(
            App::new("generate-book")
                .about("Generates and saves an opening book")
                .arg(Arg::new("out").long("out").takes_value(true))
                .arg(
                    Arg::new("ply")
                        .long("ply")
                        .help("Solves and saves all positions that have the specified ply")
                        .default_value("8"),
                )
                .arg(
                    Arg::new("use-book")
                        .long("use-book")
                        .help("Uses another book when solving positions. Useful if generating a lower-ply book when a higher-ply book already exists.")
                        .takes_value(true)
                ),
        )
        .subcommand(
            App::new("print")
                .about("Prints a position as ASCII text")
                .alias("draw")
                .arg(Arg::new("variation").required(false).index(1))
                .arg(
                    Arg::new("hex")
                        .long("hex")
                        .help("Interpret the variation as a hexadecimal 64-bit position code"),
                )
                .arg(
                    Arg::new("technical")
                        .short('t')
                        .long("technical")
                        .help("Include technical details"),
                ),
        )
        .subcommand(
            App::new("solve")
                .about("Solves a position")
                .arg(Arg::new("variation").required(false).index(1))
                .arg(
                    Arg::new("hex")
                        .long("hex")
                        .help("Interpret the variation as a hexadecimal 64-bit position code"),
                ),
        )
        .subcommand(
            App::new("test")
                .about("Runs a test set from a file (or several files)")
                .arg(
                    Arg::new("files")
                        .required(true)
                        .index(1)
                        .multiple_values(true),
                ),
        )
        .subcommand(
            App::new("verify-book")
                .about("Compares and verifies a book against a reference book")
                .arg(Arg::new("book").index(1).required(true))
                .arg(Arg::new("reference_book").index(2).required(true)),
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("format-book", sub_matches)) => {
            format_book(sub_matches).map_err(|err| err.to_string())
        }
        Some(("generate-book", sub_matches)) => {
            let ply = sub_matches.value_of("ply").unwrap().parse().unwrap();
            let use_book = sub_matches.value_of("use-book").map(|str| Path::new(str));
            generate_book(ply, use_book).map_err(|err| err.to_string())
        }
        Some(("print", sub_matches)) => print_subcommand(sub_matches),
        Some(("solve", sub_matches)) => {
            let variation = sub_matches.value_of("variation").unwrap_or("");
            let pos_input = if sub_matches.is_present("hex") {
                PositionInput::Hex(String::from(variation))
            } else {
                PositionInput::Variation(String::from(variation))
            };
            solve(pos_input, false)
        }
        Some(("test", sub_matches)) => {
            let mut files = sub_matches.values_of("files").unwrap();
            run_test_files(&mut files)
        }
        Some(("verify-book", sub_matches)) => {
            let book = Path::new(sub_matches.value_of("book").unwrap());
            let reference_book = Path::new(sub_matches.value_of("reference_book").unwrap());
            verify_book(book, reference_book).map_err(|err| err.to_string())
        }
        _ => play(&matches),
    };

    if let Err(str) = result {
        eprintln!("{}", str);
    }
}
