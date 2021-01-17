use crate::bitboard::{Bitboard, Position, BIT_HEIGHT, BOARD_HEIGHT, BOARD_WIDTH, FIRST_COLUMN};
use crate::heuristic::{FixedHeuristic, Heuristic};
use crate::score::Score;
use crate::trans_table::TransTable;

pub struct Engine {
    pub position: Position,
    trans_table: TransTable,
    pub work_count: usize,
    pub heuristic: FixedHeuristic,
    ply: u32,
}

#[derive(Clone)]
struct Move {
    /// The column where the disc is dropped
    x: u32,
    y: u32,
    new_position: Position,
    priority: i32,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            position: Position::empty(),
            /// Bigger is not necessarily better because it can lead to more cache misses. The
            /// transposition table is a bottleneck and can easily take half of the execution time.
            trans_table: TransTable::new(101501),
            work_count: 0,
            heuristic: FixedHeuristic {},
            ply: 0,
        }
    }

    pub fn reset(&mut self) {
        self.work_count = 0;
        self.trans_table.reset();
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        self.ply = position.get_ply();
    }

    pub fn solve(&mut self) -> Score {
        for x in 0..BOARD_WIDTH {
            let board = self.position.drop(x);
            if board.is_legal() && board.has_won() {
                return Score::Win;
            }
        }
        self.negamax(Score::Loss, Score::Win, 42)
    }

    fn negamax(&mut self, alpha: Score, beta: Score, max_depth: u32) -> Score {
        debug_assert!(!self.position.has_won(), "Already won");

        self.work_count += 1;

        if self.ply == BOARD_WIDTH * BOARD_HEIGHT - 1 {
            return Score::Draw;
        }

        if max_depth == 0 {
            return Score::Unknown;
        }

        let mut nonlosing_moves = self.position.get_nonlosing_moves();
        if nonlosing_moves.0 == 0 {
            return Score::Loss;
        }

        let immediate_enemy_threats = self.position.to_other_perspective().get_immediate_threats();

        let forced_move_count = immediate_enemy_threats.0.count_ones();
        if forced_move_count > 1 {
            return Score::Loss;
        } else if forced_move_count == 1 {
            if immediate_enemy_threats.0 & nonlosing_moves.0 == 0 {
                return Score::Loss;
            }

            let old_position = self.position;
            let new_board = Bitboard(old_position.current.0 | immediate_enemy_threats.0);
            self.position = Position::new(old_position.other, new_board);
            self.ply += 1;
            let score = self
                .negamax(beta.flip(), alpha.flip(), max_depth - 1)
                .flip();
            self.ply -= 1;
            self.position = old_position;
            return score;
        }

        let auto_score = self.position.autofinish_score(nonlosing_moves);
        if auto_score == Score::Loss {
            return Score::Loss;
        }
        if auto_score == Score::Draw && alpha == Score::Draw {
            return Score::Draw;
        }

        let (position_code, symmetric) = self.position.to_normalized_position_code();
        if symmetric {
            nonlosing_moves = nonlosing_moves.get_left_half();
        }

        let mut best_score = Score::Loss;
        let mut new_alpha = alpha;
        let mut new_beta = beta;

        let trans_score = self.trans_table.fetch(position_code);
        if trans_score.is_exact() {
            return trans_score;
        }

        if trans_score != Score::Unknown {
            if trans_score == Score::DrawOrWin {
                new_alpha = Score::Draw;
                best_score = Score::Draw;
            } else if trans_score == Score::DrawOrLoss {
                new_beta = Score::Draw;
            }

            if new_alpha == new_beta {
                return trans_score;
            }
        }

        let mut possible_moves = Vec::with_capacity(BOARD_WIDTH as usize);
        for x in 0..BOARD_WIDTH {
            let column = (nonlosing_moves.0 >> (x * BIT_HEIGHT)) & FIRST_COLUMN;
            if column != 0 {
                possible_moves.push(self.create_move(x));
            }
        }

        insertion_sort(&mut possible_moves);
        // Timsort:
        // possible_moves.sort_by(|a, b| { b.priority.cmp(&a.priority) });
        let old_position = self.position;
        let original_interior_count = self.work_count;
        // If any of the children remains unknown, we may not have an exact score. This can happen
        // alpha-beta cutoffs and depth limits.
        let mut unknown_count = possible_moves.len();
        for m in possible_moves {
            self.position = m.new_position;
            self.ply += 1;

            let score = self
                .negamax(new_beta.flip(), new_alpha.flip(), max_depth - 1)
                .flip();

            self.ply -= 1;

            if score != Score::Unknown {
                unknown_count -= 1;
            }

            if score > best_score {
                if score == Score::Win {
                    new_alpha = Score::Win
                } else if score == Score::Draw || score == Score::DrawOrWin {
                    new_alpha = Score::Draw;
                }

                best_score = score;
            }

            if new_alpha >= new_beta {
                break;
            }

            if best_score == Score::Win {
                break;
            }
        }
        self.position = old_position;
        let work = self.work_count - original_interior_count;

        if unknown_count > 0 {
            if best_score == Score::Draw {
                best_score = Score::DrawOrWin;
            } else if best_score < Score::Draw {
                best_score = Score::Unknown;
            }
        }

        if trans_score == Score::DrawOrLoss && best_score >= Score::Draw {
            debug_assert!(best_score != Score::Win);
            // we have an exact value
            best_score = Score::Draw;
        }

        self.trans_table
            .store(position_code, best_score, work as u32);
        best_score
    }

    fn create_move(&self, x: u32) -> Move {
        let new_position = Position::new(self.position.other, self.position.drop(x));
        let y = self.position.get_height(x);

        let threats = new_position.to_other_perspective().count_threats() as i32;
        let mut priority: i32 = threats * 1000000;
        if self.ply > 19 {
            priority += 1000 * y as i32;
        }
        priority += self.heuristic.get_value(x, y);

        Move {
            x,
            y,
            new_position,
            priority,
        }
    }
}

/// Insertion sort is good when an array is small, which is the case for us because the number of
/// possible moves is max BOARD_WIDTH. This marginally outperforms the Timsort that Vec::sort uses
/// internally (but not by much).
fn insertion_sort(moves: &mut Vec<Move>) {
    for i in 1..moves.len() {
        let mut j = i;
        while j > 0 && moves[j - 1].priority < moves[j].priority {
            moves.swap(j - 1, j);
            j -= 1;
        }
    }
}
