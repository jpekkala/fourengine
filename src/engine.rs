use crate::bitboard::{Bitboard, Position, BOARD_HEIGHT, BOARD_WIDTH, BIT_HEIGHT, FIRST_COLUMN};
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
}

impl Move {
    pub fn new(position: &Position, column: u32) -> Move {
        Move {
            x: column,
            y: position.get_height(column),
            new_position: Position::new(position.other, position.drop(column)),
        }
    }
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            position: Position::empty(),
            trans_table: TransTable::new(67108859),
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

        let immediate_enemy_threats = self.position.from_other_perspective().get_immediate_threats();

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
            let score = self.negamax(beta.flip(), alpha.flip(), max_depth - 1)
                .flip();
            self.ply -= 1;
            self.position = old_position;
            return score;
        }

        let (position_code, symmetric) = self.position.to_normalized_position_code();
        if symmetric {
            nonlosing_moves = nonlosing_moves.get_left_half();
        }

        let mut possible_moves = Vec::new();
        for x in 0..BOARD_WIDTH {
            let column = (nonlosing_moves.0 >> (x * BIT_HEIGHT)) & FIRST_COLUMN;
            if column != 0 {
                possible_moves.push(Move::new(&self.position, x));
            }
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

        possible_moves.sort_by(|a, b| {
            let threats1 = a.new_position.from_other_perspective().count_threats();
            let threats2 = b.new_position.from_other_perspective().count_threats();
            if threats1 != threats2 {
                threats2.cmp(&threats1)
            } else if a.y != b.y && a.new_position.get_ply() > 20 {
                b.y.cmp(&a.y)
            } else {
                self.heuristic
                    .get_value(b.x, b.y)
                    .cmp(&self.heuristic.get_value(a.x, a.y))
            }
        });

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
}
