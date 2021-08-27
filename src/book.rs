use crate::benchmark::{format_large_number, Benchmark};
use crate::bitboard::{Bitboard, BoardInteger, Position, BOARD_HEIGHT, BOARD_WIDTH};
use crate::engine::Engine;
use crate::score::{Score, SCORE_BITS};
use core::mem;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::{Path, PathBuf};

pub const DEFAULT_BOOK_PLY: u32 = 8;
pub const BOOK_FOLDER: &str = "books";

pub fn get_path_for_ply(ply: u32) -> PathBuf {
    PathBuf::from(BOOK_FOLDER).join(format!("{}-ply.txt", ply))
}

/// Packs a position code and its score in one value
#[derive(PartialEq, Eq)]
pub struct PackedPositionScore(BoardInteger);

impl PackedPositionScore {
    // Score is saved in the most significant bits by shifting left
    const SCORE_SHIFT: u32 = (mem::size_of::<BoardInteger>() * 8) as u32 - SCORE_BITS;
    const POSITION_MASK: BoardInteger = (1 << Self::SCORE_SHIFT) - 1;

    pub fn new(position: &Position, score: Score) -> Self {
        let code = position.to_position_code();
        let score_bits = (score as u64) << Self::SCORE_SHIFT;
        PackedPositionScore(code | score_bits)
    }

    pub fn get_position(&self) -> Position {
        Position::from_position_code(self.get_position_code())
    }

    pub fn get_position_code(&self) -> BoardInteger {
        self.0 & Self::POSITION_MASK
    }

    pub fn get_score(&self) -> Score {
        Score::from_u64_fast(self.0 >> Self::SCORE_SHIFT)
    }
}

impl Ord for PackedPositionScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_position_code().cmp(&other.get_position_code())
    }
}

impl PartialOrd for PackedPositionScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Book {
    entries: Vec<PackedPositionScore>,
}

impl Book {
    pub fn empty() -> Book {
        Book { entries: vec![] }
    }

    pub fn open_for_ply(ply: u32) -> Result<Book, std::io::Error> {
        Book::open(&get_path_for_ply(ply))
    }

    pub fn open_for_ply_or_empty(ply: u32) -> Book {
        Book::open(&get_path_for_ply(ply)).unwrap_or_else(|_| Book::empty())
    }

    pub fn from_lines(data: &str) -> Book {
        let mut book = Book::empty();
        for line in data.lines() {
            book.include_line(line);
        }
        book.sort_and_shrink();
        book
    }

    pub fn open(file_path: &Path) -> Result<Book, std::io::Error> {
        let mut book = Book::empty();
        let file = File::open(file_path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                book.include_line(&line);
            }
        }
        book.sort_and_shrink();
        Ok(book)
    }

    fn include_line(&mut self, line: &str) {
        if let Some((position, score)) = parse_hex_line(line).or_else(|| parse_verbose_line(line)) {
            let (position, _symmetric) = position.normalize();
            self.entries
                .push(PackedPositionScore::new(&position, score));
        } else {
            panic!("Unknown line: {}", line);
        }
    }

    fn sort_and_shrink(&mut self) {
        self.entries.sort();
        self.entries.shrink_to_fit();
    }

    pub fn get(&self, position: &Position) -> Score {
        let s = PackedPositionScore::new(position, Score::Unknown);
        match self.entries.binary_search(&s) {
            Ok(index) => self.entries[index].get_score(),
            Err(_) => Score::Unknown,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl IntoIterator for Book {
    type Item = PackedPositionScore;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

fn format_hex_line(pos: Position, score: Score) -> String {
    format!("{}{}", pos.as_hex_string(), score.to_char())
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
        BookWriter::create(&get_path_for_ply(ply))
    }

    fn create(file_path: &Path) -> Result<BookWriter, std::io::Error> {
        let file = File::create(file_path)?;
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

    let existing_book = Book::open_for_ply_or_empty(DEFAULT_BOOK_PLY);
    if !existing_book.is_empty() {
        println!("Found {} existing positions", existing_book.len());
    }

    let mut total_benchmark = Benchmark::empty();
    let mut solved = 0;
    let mut book_writer = BookWriter::create_for_ply(DEFAULT_BOOK_PLY)?;
    for (count, pos) in set.into_iter().enumerate() {
        let existing_score = existing_book.get(&pos);
        if existing_score != Score::Unknown {
            book_writer.write_entry(pos, existing_score)?;
            continue;
        }

        let benchmark = book_writer.solve_position(pos)?;
        total_benchmark = total_benchmark.add(&benchmark);
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

pub fn verify_book(book: &Path, reference_book: &Path) -> Result<(), std::io::Error> {
    let book = Book::open(book)?;
    let reference_book = Book::open(reference_book)?;

    let mut missing_count = 0;
    let mut invalid_count = 0;
    for p in reference_book.into_iter() {
        let position = p.get_position();
        let reference_score = p.get_score();
        let score = book.get(&position);
        if score == Score::Unknown {
            missing_count += 1;
        } else if reference_score != score {
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
