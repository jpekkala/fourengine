use crate::bitboard::{Bitboard, MoveBitmap, Position, BOARD_HEIGHT, BOARD_WIDTH};
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

#[derive(Clone, Copy)]
struct Move {
    new_position: Position,
    priority: i32,
}

enum QuickEvaluation {
    Score(Score),
    Moves(MoveBitmap),
}

#[derive(Clone, Copy)]
struct AlphaBeta {
    alpha: Score,
    beta: Score,
}

impl AlphaBeta {
    fn new() -> AlphaBeta {
        AlphaBeta {
            alpha: Score::Loss,
            beta: Score::Win,
        }
    }

    fn flip(&self) -> AlphaBeta {
        AlphaBeta {
            alpha: self.beta.flip(),
            beta: self.alpha.flip(),
        }
    }

    fn has_cutoff(&self) -> bool {
        self.alpha >= self.beta
    }

    fn narrow_alpha(&mut self, score: Score) {
        if score == Score::Win {
            self.alpha = Score::Win
        } else if score == Score::Draw || score == Score::DrawOrWin {
            self.alpha = Score::Draw;
        }
    }
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
        self.negamax(AlphaBeta::new(), 42)
    }

    #[inline(always)]
    fn quick_evaluate(&self, ab: &AlphaBeta) -> QuickEvaluation {
        if self.ply == BOARD_WIDTH * BOARD_HEIGHT - 1 {
            return QuickEvaluation::Score(Score::Draw);
        }

        let nonlosing_moves = self.position.get_nonlosing_moves();
        if nonlosing_moves.0 == 0 {
            return QuickEvaluation::Score(Score::Loss);
        }

        let immediate_enemy_threats = self.position.to_other_perspective().get_immediate_wins();

        let forced_move_count = immediate_enemy_threats.0.count_ones();
        if forced_move_count > 1 {
            return QuickEvaluation::Score(Score::Loss);
        } else if forced_move_count == 1 {
            if immediate_enemy_threats.0 & nonlosing_moves.0 == 0 {
                return QuickEvaluation::Score(Score::Loss);
            }
            return QuickEvaluation::Moves(immediate_enemy_threats);
        }

        let auto_score = self.position.autofinish_score(nonlosing_moves);
        if auto_score == Score::Loss {
            return QuickEvaluation::Score(Score::Loss);
        }
        if auto_score == Score::Draw && ab.alpha == Score::Draw {
            return QuickEvaluation::Score(Score::Draw);
        }

        QuickEvaluation::Moves(nonlosing_moves)
    }

    fn negamax(&mut self, ab: AlphaBeta, max_depth: u32) -> Score {
        debug_assert!(!self.position.has_won(), "Already won");

        if max_depth == 0 {
            return Score::Unknown;
        }

        self.work_count += 1;

        let mut move_bitmap = match self.quick_evaluate(&ab) {
            QuickEvaluation::Score(score) => return score,
            QuickEvaluation::Moves(board) => board,
        };

        if move_bitmap.has_only_one_move() {
            let old_position = self.position;
            let new_board = Bitboard(self.position.current.0 | move_bitmap.0);
            self.position = Position::new(old_position.other, new_board);
            self.ply += 1;
            let score = self.negamax(ab.flip(), max_depth - 1).flip();
            self.ply -= 1;
            self.position = old_position;
            return score;
        }

        let (position_code, symmetric) = self.position.to_normalized_position_code();
        if symmetric {
            move_bitmap = move_bitmap.get_left_half();
        }

        let mut ab = ab.clone();
        let mut best_score = Score::Loss;

        let trans_score = self.trans_table.fetch(position_code);
        if trans_score.is_exact() {
            return trans_score;
        }

        if trans_score != Score::Unknown {
            if trans_score == Score::DrawOrWin {
                ab.alpha = Score::Draw;
                best_score = Score::Draw;
            } else if trans_score == Score::DrawOrLoss {
                ab.beta = Score::Draw;
            }

            if ab.has_cutoff() {
                return trans_score;
            }
        }

        let mut move_array = [Move {
            new_position: Position::empty(),
            priority: 0,
        }; BOARD_WIDTH as usize];

        let mut possible_moves = move_bitmap.init_array(&mut move_array, |x| self.create_move(x));
        insertion_sort(&mut possible_moves);

        let old_position = self.position;
        let original_interior_count = self.work_count;
        // If any of the children remains unknown, we may not have an exact score. This can happen
        // alpha-beta cutoffs and depth limits.
        let mut unknown_count = possible_moves.len();
        for m in possible_moves {
            self.position = m.new_position;
            self.ply += 1;

            let score = self.negamax(ab.flip(), max_depth - 1).flip();

            self.ply -= 1;

            if score != Score::Unknown {
                unknown_count -= 1;
            }

            if score > best_score {
                ab.narrow_alpha(score);
                best_score = score;

                if ab.has_cutoff() {
                    break;
                }
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
            new_position,
            priority,
        }
    }
}

/// Insertion sort is good when an array is small, which is the case for us because the number of
/// possible moves is max BOARD_WIDTH.
fn insertion_sort(moves: &mut &mut [Move]) {
    for i in 1..moves.len() {
        let mut j = i;
        while j > 0 && moves[j - 1].priority < moves[j].priority {
            moves.swap(j - 1, j);
            j -= 1;
        }
    }
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
