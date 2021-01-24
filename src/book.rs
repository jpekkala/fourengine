use crate::benchmark::{format_large_number, Benchmark};
use crate::bitboard::{Bitboard, BoardInteger, Position, BOARD_HEIGHT, BOARD_WIDTH};
use crate::engine::Engine;
use crate::score::Score;
use core::mem;
use std::collections::{BTreeSet, HashMap};
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, LineWriter, Write};

const DEFAULT_PLY: u32 = 8;
const BOOK_FOLDER: &str = "books";

fn get_book_file(ply: u32) -> String {
    format!("{}/{}-ply.txt", BOOK_FOLDER, ply)
}

pub struct Book {
    map: HashMap<Position, Score>,
}

impl Book {
    pub fn empty() -> Book {
        Book {
            map: HashMap::new(),
        }
    }

    pub fn open_for_ply(ply: u32) -> Result<Book, std::io::Error> {
        Book::open(&get_book_file(ply))
    }

    pub fn open(file_name: &str) -> Result<Book, std::io::Error> {
        let mut book = Book {
            map: HashMap::new(),
        };
        let file = File::open(file_name)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                book.include_line(&line);
            }
        }
        Ok(book)
    }

    fn include_line(&mut self, line: &str) {
        if let Some((position, score)) = parse_hex_line(line).or_else(|| parse_verbose_line(line)) {
            let (position, _symmetric) = position.normalize();
            self.map.insert(position, score);
        } else {
            panic!("Unknown line: {}", line);
        }
    }

    pub fn get(&self, position: &Position) -> Score {
        match self.map.get(position) {
            Some(score) => *score,
            None => Score::Unknown,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

fn format_hex_line(pos: Position, score: Score) -> String {
    format!("{:0>16X}{}", pos.to_position_code(), score.to_char())
}

fn parse_hex_line(line: &str) -> Option<(Position, Score)> {
    const HEX_LENGTH: usize = mem::size_of::<BoardInteger>() * 2;
    if line.len() != HEX_LENGTH + 1 {
        return None;
    }

    let position_str = &line[0..HEX_LENGTH];
    let position_code = BoardInteger::from_str_radix(position_str, 16).ok()?;
    let position = Position::from_position_code(position_code);

    let score = Score::from_string(&line[HEX_LENGTH..]);
    if score == Score::Unknown {
        None
    } else {
        Some((position, score))
    }
}

fn parse_verbose_line(line: &str) -> Option<(Position, Score)> {
    const CELL_COUNT: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;
    let line: String = line.chars().filter(|x| *x != ',').collect();
    if line.len() < CELL_COUNT + 1 {
        return None;
    }
    let position_str = &line[0..CELL_COUNT];
    let score_str = &line[CELL_COUNT..];

    let mut current = Bitboard::empty();
    let mut other = Bitboard::empty();
    for (i, ch) in position_str.chars().enumerate() {
        let y = i as u32 % BOARD_HEIGHT;
        let x = i as u32 / BOARD_HEIGHT;
        match ch {
            'X' | 'x' => current = current.set_disc(x, y),
            'O' | 'o' => other = other.set_disc(x, y),
            ' ' | 'b' => {}
            _ => return None,
        }
    }

    let position = Position::new(current, other);
    let score = Score::from_string(score_str);
    if score == Score::Unknown {
        None
    } else {
        Some((position, score))
    }
}

struct BookWriter {
    file: LineWriter<File>,
    engine: Engine,
}

impl BookWriter {
    fn create_for_ply(ply: u32) -> Result<BookWriter, std::io::Error> {
        BookWriter::create(&get_book_file(ply))
    }

    fn create(file_name: &str) -> Result<BookWriter, std::io::Error> {
        let file = File::create(file_name)?;
        Ok(BookWriter {
            file: LineWriter::new(file),
            engine: Engine::new(),
        })
    }

    fn solve_position(&mut self, pos: Position) -> Result<Benchmark, std::io::Error> {
        self.engine.set_position(pos);
        let benchmark = Benchmark::run(&mut self.engine);
        self.write_entry(pos, benchmark.score)?;
        Ok(benchmark)
    }

    fn write_entry(&mut self, pos: Position, score: Score) -> Result<(), std::io::Error> {
        let line = format_hex_line(pos, score);
        self.file.write_all(line.as_bytes())?;
        self.file.write_all(b"\n")
    }
}

pub fn generate_book() -> Result<(), std::io::Error> {
    create_dir_all(BOOK_FOLDER)?;

    let set = find_positions_to_solve();
    let total_count = set.len();
    println!("There are {} positions to solve", total_count);

    let existing_book = Book::open_for_ply(DEFAULT_PLY).unwrap_or_else(|_err| Book::empty());
    if !existing_book.is_empty() {
        println!("Found {} existing positions", existing_book.len());
    }

    let mut total_benchmark = Benchmark::empty();
    let mut count = 0;
    let mut solved = 0;
    let mut book_writer = BookWriter::create_for_ply(DEFAULT_PLY)?;
    for pos in set {
        count += 1;
        if let Some(score) = existing_book.map.get(&pos) {
            book_writer.write_entry(pos, *score)?;
            continue;
        }

        let benchmark = book_writer.solve_position(pos)?;
        total_benchmark = total_benchmark.add(benchmark);
        solved += 1;

        if count % 20 == 0 {
            let average_work = total_benchmark.work_count as f64 / solved as f64;
            println!(
                "Solved {} out of {}. Speed is {} nodes per second. Average work per position: {}",
                count,
                total_count,
                format_large_number(total_benchmark.get_speed(), 0),
                format_large_number(average_work, 0),
            );
            total_benchmark = Benchmark::empty();
            solved = 0;
        }
    }
    Ok(())
}

pub fn verify_book(reference_book: &str) -> Result<(), std::io::Error> {
    let book = Book::open_for_ply(DEFAULT_PLY)?;
    let reference_book = Book::open(reference_book)?;

    let mut missing_count = 0;
    let mut invalid_count = 0;
    for (position, reference_score) in reference_book.map.iter() {
        let score = book.get(position);
        if score == Score::Unknown {
            missing_count += 1;
        } else if *reference_score != score {
            invalid_count += 1;
        }
    }

    if missing_count > 0 {
        println!(
            "Warning: {} entries missing from the generated book",
            missing_count
        );
    }
    if invalid_count > 0 {
        panic!("{} invalid positions", invalid_count);
    } else {
        println!("The generated book is OK");
    }

    Ok(())
}

fn find_positions_to_solve() -> BTreeSet<Position> {
    let mut set = BTreeSet::new();
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
