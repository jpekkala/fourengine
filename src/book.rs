use crate::benchmark::{format_large_number, Benchmark};
use crate::bitboard::{Bitboard, BoardInteger, Position, BOARD_HEIGHT, BOARD_WIDTH};
use crate::engine::Engine;
use crate::score::{Score, SCORE_BITS};
use core::mem;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::{cmp, io};

pub const DEFAULT_BOOK_PLY: u32 = 8;
pub const BOOK_FOLDER: &str = "books";

pub fn get_path_for_ply(ply: u32) -> PathBuf {
    PathBuf::from(BOOK_FOLDER).join(format!("{}x{}-ply{}.txt", BOARD_WIDTH, BOARD_HEIGHT, ply))
}

/// Packs a position code and its score in one value
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BookEntry(BoardInteger);

impl BookEntry {
    // Score is saved in the most significant bits by shifting left and the position can be derived
    // with a simple mask. An alternative design would have been to shift position left to make room
    // for score, which would have had the benefit that entries are numerically sorted in the file.
    // The current design was chosen in case we want to encode more information in the empty space
    // between score and position.
    const BYTE_COUNT: usize = mem::size_of::<BoardInteger>();
    const SCORE_SHIFT: u32 = (Self::BYTE_COUNT * 8) as u32 - SCORE_BITS;
    const POSITION_MASK: BoardInteger = (1 << Self::SCORE_SHIFT) - 1;

    pub fn new(position: &Position, score: Score) -> Self {
        let code = position.normalize().to_position_code();
        let score_bits = (score as u64) << Self::SCORE_SHIFT;
        BookEntry(code | score_bits)
    }

    pub fn get_position(&self) -> Position {
        Position::from_position_code(self.get_position_code()).unwrap()
    }

    pub fn get_position_code(&self) -> BoardInteger {
        self.0 & Self::POSITION_MASK
    }

    pub fn get_score(&self) -> Score {
        Score::from_u64_fast(self.0 >> Self::SCORE_SHIFT)
    }

    fn to_hex_string(&self) -> String {
        format!(
            "{}{}",
            self.get_position().as_hex_string(),
            self.get_score().to_char()
        )
    }

    fn from_hex_string(line: &str) -> Option<BookEntry> {
        const HEX_LENGTH: usize = mem::size_of::<BoardInteger>() * 2;
        if line.len() != HEX_LENGTH + 1 {
            return None;
        }

        let position_str = &line[0..HEX_LENGTH];
        let position_code = BoardInteger::from_str_radix(position_str, 16).ok()?;
        let position = Position::from_position_code(position_code)?;

        let score = Score::from_string(&line[HEX_LENGTH..]);
        if score == Score::Unknown {
            None
        } else {
            Some(BookEntry::new(&position, score))
        }
    }

    fn from_verbose_string(line: &str) -> Option<BookEntry> {
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
            Some(BookEntry::new(&position, score))
        }
    }

    fn autodetect_parse(line: &str) -> Option<BookEntry> {
        Self::from_hex_string(line).or_else(|| Self::from_verbose_string(line))
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_COUNT] {
        self.0.to_be_bytes()
    }

    pub fn from_bytes(bytes: &[u8; Self::BYTE_COUNT]) -> Option<BookEntry> {
        let mut board: BoardInteger = 0;
        for byte in bytes {
            board <<= 8;
            board |= *byte as u64;
        }
        Some(BookEntry(board))
    }
}

impl Ord for BookEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_position_code().cmp(&other.get_position_code())
    }
}

impl PartialOrd for BookEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Book {
    entries: Vec<BookEntry>,
    /// A bitwise union of the plies of positions stored in this book. It is recommended that books
    /// contain plies that are powers of two (e.g. 4 and 8) so that the ply mask works efficiently.
    ply_mask: u32,
}

impl Book {
    pub fn empty() -> Book {
        Book {
            entries: vec![],
            ply_mask: 0,
        }
    }

    pub fn standard() -> Book {
        let mut book = Book::empty();
        book.include_book(&Self::ply(4));
        book.include_book(&Self::ply(8));
        book
    }

    pub fn ply(ply: u32) -> Book {
        let path = get_path_for_ply(ply);
        Self::open(path.as_path()).unwrap_or_else(|_| Book::empty())
    }

    pub fn include_book(&mut self, another_book: &Book) {
        for entry in another_book.iter() {
            self.add_entry(*entry)
        }
        self.sort_and_shrink()
    }

    pub fn open(file_path: &Path) -> Result<Book, std::io::Error> {
        let file = File::open(file_path)?;
        let mut buf = BufReader::new(file);
        match Self::read_text_book(&mut buf) {
            Ok(book) => Ok(book),
            Err(err) => {
                buf.seek(SeekFrom::Start(0))?;
                if let Ok(book) = Self::read_binary_book(&mut buf) {
                    Ok(book)
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Creates a new book from a string that contains one position per line
    /// ```
    /// use fourengine::book::Book;
    ///
    /// let mut str = String::new();
    /// str.push_str("0000040812A04081+\n"); // variation 4444
    /// str.push_str("000004081040C103-\n"); // variation 1234
    ///
    /// let book = Book::from_lines(&str).unwrap();
    /// assert_eq!(book.len(), 2);
    /// ```
    pub fn from_lines(data: &str) -> Result<Book, std::io::Error> {
        let mut reader = BufReader::new(data.as_bytes());
        Self::read_text_book(&mut reader)
    }

    fn read_text_book<R: Read>(reader: &mut BufReader<R>) -> Result<Book, std::io::Error> {
        let mut book = Book::empty();
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                if let Some(entry) = BookEntry::autodetect_parse(&line) {
                    book.add_entry(entry);
                } else {
                    let err = std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid position: {}", line),
                    );
                    return Err(err);
                }
            }
        }
        book.sort_and_shrink();
        Ok(book)
    }

    fn read_binary_book<R: Read>(reader: &mut BufReader<R>) -> Result<Book, std::io::Error> {
        let mut book = Book::empty();
        let mut buffer = [0; BookEntry::BYTE_COUNT];

        loop {
            match reader.read_exact(&mut buffer) {
                Ok(_) => {
                    let entry = BookEntry::from_bytes(&buffer).ok_or_else(|| {
                        std::io::Error::new(ErrorKind::InvalidData, "Invalid position")
                    })?;
                    book.add_entry(entry);
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }

        book.sort_and_shrink();
        Ok(book)
    }

    fn add_entry(&mut self, entry: BookEntry) {
        let ply = entry.get_position().get_ply();
        self.ply_mask |= ply;
        self.entries.push(entry);
    }

    fn sort_and_shrink(&mut self) {
        self.entries.sort();
        self.entries.shrink_to_fit();
    }

    /// A fast check if there are any positions of the given ply in this book. This check is in the
    /// hot path of the engine so it must be kept as simple as possible.
    pub fn contains_ply(&self, ply: u32) -> bool {
        self.ply_mask & ply != 0
    }

    pub fn get(&self, position: &Position) -> Score {
        let entry = BookEntry::new(position, Score::Unknown);
        match self.entries.binary_search(&entry) {
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

    pub fn iter(&self) -> impl Iterator<Item = &BookEntry> {
        self.entries.iter()
    }

    pub fn to_position_set(&self) -> HashSet<Position> {
        let mut set = HashSet::new();
        for book_entry in self.iter() {
            let position = book_entry.get_position();
            let score = book_entry.get_score();
            if score != Score::Unknown {
                set.insert(position);
            }
        }
        set
    }
}

pub enum BookFormat {
    Hex,
    Binary,
}

pub struct BookWriter<W: Write> {
    format: BookFormat,
    writer: W,
}

impl<W: Write> BookWriter<W> {
    pub fn create(writer: W, format: BookFormat) -> BookWriter<W> {
        BookWriter { format, writer }
    }

    pub fn write_entry(&mut self, entry: &BookEntry) -> io::Result<()> {
        match &self.format {
            BookFormat::Hex => {
                let line = entry.to_hex_string();
                self.writer.write_all(line.as_bytes())?;
                self.writer.write_all(b"\n")
            }
            BookFormat::Binary => self.writer.write_all(&entry.to_bytes()),
        }
    }
}

pub fn generate_book(ply: u32, use_book: Option<&Path>) -> Result<(), std::io::Error> {
    create_dir_all(BOOK_FOLDER)?;
    let book_path = get_path_for_ply(ply);

    let set = find_positions_to_solve(ply);
    let total_count = set.len();
    println!(
        "There are {} positions to solve. Saving book as {}",
        total_count,
        book_path.display()
    );

    let existing_book = Book::open(book_path.as_path()).unwrap_or_else(|_| Book::empty());
    if !existing_book.is_empty() {
        println!("Found {} existing positions", existing_book.len());
    }

    let mut total_benchmark = Benchmark::empty();
    let mut solved = 0;

    let mut engine = Engine::new();
    if let Some(another_book_path) = use_book {
        let another_book = Box::new(Book::open(another_book_path)?);
        engine.set_book(another_book);
    }
    let file = File::create(book_path.as_path())?;
    let mut book_writer = BookWriter::create(file, BookFormat::Hex);

    for (count, pos) in set.into_iter().enumerate() {
        let existing_score = existing_book.get(&pos);
        if existing_score != Score::Unknown {
            book_writer.write_entry(&BookEntry::new(&pos, existing_score))?;
            continue;
        }

        engine.set_position(pos);
        let benchmark = Benchmark::run(&mut engine);
        book_writer.write_entry(&BookEntry::new(&pos, benchmark.score))?;

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

pub fn verify_book(book1_path: &Path, book2_path: &Path) -> Result<(), std::io::Error> {
    let book1 = Book::open(book1_path)?;
    let book2 = Book::open(book2_path)?;

    let positions1 = book1.to_position_set();
    let positions2 = book2.to_position_set();

    let shared = positions1.intersection(&positions2);
    let conflict_count = shared
        .filter(|pos| book1.get(pos) != book2.get(pos))
        .count();

    if conflict_count > 0 {
        panic!("{} positions with conflicting scores", conflict_count);
    }

    let count1 = positions1.len();
    let count2 = positions2.len();
    let width = cmp::max(count1.to_string().len(), count2.to_string().len());
    println!(
        "There are {:>width$} positions in {}",
        count1,
        book1_path.display(),
        width = width
    );
    println!(
        "There are {:>width$} positions in {}",
        count2,
        book2_path.display(),
        width = width
    );
    println!();

    let diff_count = positions1.symmetric_difference(&positions2).count();
    if diff_count == 0 {
        println!("The books match exactly");
    } else {
        let shared_count = positions1.intersection(&positions2).count();
        println!(
            "The books have matching scores but they share only {} positions",
            shared_count
        );
    }

    Ok(())
}

fn find_positions_to_solve(ply: u32) -> BTreeSet<Position> {
    let mut set = BTreeSet::new();
    explore_tree(Position::empty(), ply, &mut |pos| {
        let pos = pos.normalize();
        set.insert(pos);
    });
    set
}

/// Explores the game tree up to a certain depth and calls the function for each leaf node. This
/// function skips leaf nodes that already contain a win/loss.
pub fn explore_tree<F>(position: Position, max_depth: u32, f: &mut F)
where
    F: FnMut(Position),
{
    if position.has_anyone_won() {
        return;
    }

    if max_depth == 0 {
        f(position);
        return;
    }

    let mut move_bitmap = position.get_unblocked_moves();
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
