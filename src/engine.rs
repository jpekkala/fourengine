use crate::bitboard::{Bitboard, BOARD_HEIGHT, BOARD_WIDTH};
use crate::history_heuristic::HistoryHeuristic;
use crate::position::Position;
use crate::score::Score;
use crate::trans_table::TransTable;

pub struct Engine {
    pub position: Position,
    trans_table: TransTable,
    pub history: HistoryHeuristic,
    pub work_count: usize,
}

#[derive(Clone)]
struct Move {
    /// The column where the disc is dropped
    x: u32,
    y: u32,
    old_board: Bitboard,
    new_board: Bitboard,
    enemy_board: Bitboard,
}

impl Move {
    pub fn new(position: &Position, column: u32) -> Move {
        Move {
            x: column,
            y: position.get_height(column),
            old_board: position.current,
            new_board: position.current.drop(position.other, column),
            enemy_board: position.other,
        }
    }

    pub fn is_legal(&self) -> bool {
        self.new_board.is_legal()
    }

    pub fn has_won(&self) -> bool {
        self.new_board.has_won()
    }

    pub fn is_forced_move(&self) -> bool {
        // must be legal for the opponent if it was legal for the current player
        self.enemy_board.drop(self.old_board, self.x).has_won()
    }
}

impl Engine {
    pub fn new(position: Position) -> Engine {
        Engine {
            position,
            trans_table: TransTable::new(67108859),
            work_count: 0,
            history: HistoryHeuristic::new(),
        }
    }

    pub fn solve(&mut self) -> Score {
        self.negamax(Score::Loss, Score::Win, 42)
    }

    fn negamax(&mut self, alpha: Score, beta: Score, max_depth: u32) -> Score {
        if self.position.has_won() {
            panic!("Already won");
        }

        self.work_count += 1;

        if self.position.ply == BOARD_WIDTH * BOARD_HEIGHT {
            return Score::Draw;
        }

        if max_depth == 0 {
            return Score::Unknown;
        }

        if self.work_count % 1_000_000 == 0 {
            //println!("{}", self.position);
        }

        let mut best_score = Score::Loss;
        let mut new_alpha = alpha;
        let mut new_beta = beta;

        let position = self.position.to_position_code();
        let trans_score = self.trans_table.fetch(position);
        if trans_score.is_exact() {
            return trans_score;
        }

        if trans_score != Score::Unknown {
            if trans_score == Score::DrawOrLoss {
                new_beta = Score::Draw;
            } else if trans_score == Score::DrawOrWin {
                new_alpha = Score::Draw;
            }

            if new_alpha == new_beta {
                return trans_score;
            }
        }

        let mut possible_moves = get_possible_moves(&self.position);
        for m in &possible_moves {
            if m.has_won() {
                return Score::Win;
            }
        }

        let mut forced_move = None;
        for m in &possible_moves {
            if m.is_forced_move() {
                if forced_move.is_some() {
                    // double threat
                    return Score::Loss;
                }
                forced_move = Some(m);
            }
        }
        if let Some(drop) = forced_move {
            possible_moves = vec![drop.clone()];
        }

        // TODO: Order moves
        possible_moves.sort_by(|a, b| {
            self.history
                .get_score(b.x, b.y)
                .cmp(&self.history.get_score(a.x, a.y))
        });

        // alpha-beta
        let old_position = self.position;
        let original_interior_count = self.work_count;
        let mut unknown_count = 0;
        for (index, m) in possible_moves.iter().enumerate() {
            self.position = old_position.drop(m.x).unwrap();

            let score = self
                .negamax(new_beta.flip(), new_alpha.flip(), max_depth - 1)
                .flip();

            if score == Score::Unknown {
                unknown_count += 1;
                continue;
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
                self.history.increase_score(m.x, m.y, 1);
                for i in 0..index {
                    self.history.increase_score(possible_moves[i].x, possible_moves[i].y, -1);
                }
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

        self.trans_table
            .store(self.position.to_position_code(), best_score, work as u32);
        best_score
    }
}

fn get_possible_moves(position: &Position) -> Vec<Move> {
    let mut possible_moves = Vec::new();
    for column in 0..BOARD_WIDTH {
        let m = Move::new(position, column);
        if m.is_legal() {
            possible_moves.push(m)
        }
    }
    possible_moves
}
