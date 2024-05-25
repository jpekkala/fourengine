use clap::{crate_version, Arg, ArgMatches, Command, ArgAction};
use fourengine::benchmark::Benchmark;
use fourengine::bitboard::{Bitboard};
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
use fourengine::position::Position;

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

fn run_test_files(filenames: &[String]) -> Result<(), String> {
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
    matches.get_one::<String>("book_file").map(|s| s.as_str());
    let book_file = get_path_arg(&matches, "book-file").unwrap();
    let book = Book::open(book_file)?;

    let book_format = match get_string_arg(&matches, "format").unwrap() {
        "binary" => BookFormat::Binary,
        "hex" => BookFormat::Hex,
        &_ => panic!("Invalid format"),
    };

    let omit_won = matches.get_flag("omit-won");
    let omit_forced = matches.get_flag("omit-forced");
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

    if matches.get_flag("count-only") {
        println!("{}", filtered_entries.count());
        return Ok(());
    }

    let writer: Box<dyn Write> = match get_path_arg(&matches, "out") {
        None => Box::new(io::stdout()),
        Some(path) => {
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
    let use_book = !matches.get_flag("no-book");
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
    let variation = get_string_arg(&matches, "variation").unwrap_or("");
    let position = if matches.get_flag("hex") {
        PositionInput::Hex(String::from(variation))
    } else {
        PositionInput::Variation(String::from(variation))
    }
    .parse()?;

    print_board(position);

    if matches.get_flag("technical") {
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

fn get_string_arg<'a>(matches: &'a ArgMatches, name: &str) -> Option<&'a str> {
    matches.get_one::<String>(name)
        .map(|s| s.as_str())
}

fn get_path_arg<'a>(matches: &'a ArgMatches, name: &str) -> Option<&'a Path> {
    get_string_arg(matches, name).map(|s| Path::new(s))
}

fn main() {
    let matches = Command::new("Fourengine")
        .version(crate_version!())
        .about("Connect-4 engine")
        .author("Jukka Pekkala, Johan Nordlund")
        .arg(
            Arg::new("no-book")
                .long("no-book")
                .help("Disables opening book")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("format-book")
                .about("Converts a book to another format")
                .arg(Arg::new("book-file").required(true).index(1))
                .arg(Arg::new("out").long("out").value_name("FILE").num_args(1))
                .arg(Arg::new("count-only").long("count-only"))
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["hex", "binary"])
                        .default_value("hex"),
                )
                .arg(Arg::new("omit-forced").long("omit-forced").action(ArgAction::SetTrue))
                .arg(Arg::new("omit-won").long("omit-won").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("generate-book")
                .about("Generates and saves an opening book")
                .arg(Arg::new("out").long("out").value_name("FILE").num_args(1))
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
                        .value_name("FILE")
                        .num_args(1)
                ),
        )
        .subcommand(
            Command::new("print")
                .about("Prints a position as ASCII text")
                .alias("draw")
                .arg(Arg::new("variation").required(false).index(1))
                .arg(
                    Arg::new("hex")
                        .long("hex")
                        .help("Interpret the variation as a hexadecimal 64-bit position code")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("technical")
                        .short('t')
                        .long("technical")
                        .help("Include technical details")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("solve")
                .about("Solves a position")
                .arg(Arg::new("variation").required(false).index(1))
                .arg(
                    Arg::new("hex")
                        .long("hex")
                        .help("Interpret the variation as a hexadecimal 64-bit position code")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("test")
                .about("Runs a test set from a file (or several files)")
                .arg(
                    Arg::new("files")
                        .required(true)
                        .index(1)
                        .num_args(1..)
                ),
        )
        .subcommand(
            Command::new("verify-book")
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
            let ply = get_string_arg(&sub_matches, "ply").unwrap().parse().unwrap();
            let use_book = get_path_arg(&sub_matches, "use-book");
            generate_book(ply, use_book).map_err(|err| err.to_string())
        }
        Some(("print", sub_matches)) => print_subcommand(sub_matches),
        Some(("solve", sub_matches)) => {
            let variation = get_string_arg(&sub_matches, "variation").unwrap_or("");
            let pos_input = if sub_matches.get_flag("hex") {
                PositionInput::Hex(String::from(variation))
            } else {
                PositionInput::Variation(String::from(variation))
            };
            solve(pos_input, false)
        }
        Some(("test", sub_matches)) => {
            let files: Vec<String> = sub_matches.get_many::<String>("files")
                .expect("Files expected")
                .cloned()
                .collect();
            run_test_files(&files)
        }
        Some(("verify-book", sub_matches)) => {
            let book = get_path_arg(&sub_matches, "book").unwrap();
            let reference_book = get_path_arg(&sub_matches, "reference_book").unwrap();
            verify_book(book, reference_book).map_err(|err| err.to_string())
        }
        _ => play(&matches),
    };

    if let Err(str) = result {
        eprintln!("{}", str);
    }
}
