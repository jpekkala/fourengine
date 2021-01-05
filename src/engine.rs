use crate::bitboard::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::position::Position;
use crate::score::Score;
use crate::trans_table::TransTable;

pub struct Engine {
    pub position: Position,
    trans_table: TransTable,
    interior_count: usize,
}

impl Engine {
    pub fn new(position: Position) -> Engine {
        Engine {
            position,
            trans_table: TransTable::new(67108859),
            interior_count: 0,
        }
    }

    pub fn negamax(&mut self, alpha: Score, beta: Score, max_depth: u32) -> Score {
        self.interior_count += 1;

        if self.position.ply == BOARD_WIDTH * BOARD_HEIGHT {
            return Score::Draw;
        }

        if max_depth == 0 {
            return Score::Unknown;
        }

        if self.interior_count % 1_000_000 == 0 {
            println!("{}", self.position);
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

        let mut children = expand_children(&self.position);
        for child in &children {
            if child.other.has_won() {
                return Score::Win;
            }
        }

        let mut enemy_threat_at = None;
        for column in 0..BOARD_WIDTH {
            let enemy_board = self.position.other.drop(self.position.current, column);
            if enemy_board.is_legal() && enemy_board.has_won() {
                if enemy_threat_at.is_some() {
                    return Score::Loss;
                }
                enemy_threat_at = Some(column);
            }
        }
        if let Some(column) = enemy_threat_at {
            // drop must be valid if it was valid for the opponent
            let child = self.position.drop(column).unwrap();
            children = vec![child];
        }

        // TODO: Order moves

        // alpha-beta
        let old_position = self.position;
        let original_interior_count = self.interior_count;
        let mut unknown_count = 0;
        for child in children {
            self.position = child;

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
                // TODO: update history score
                break;
            }

            if best_score == Score::Win {
                break;
            }
        }
        self.position = old_position;
        let work = self.interior_count - original_interior_count;

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

fn expand_children(position: &Position) -> Vec<Position> {
    let mut possible_moves = Vec::new();
    for column in 0..BOARD_WIDTH {
        match position.drop(column) {
            Some(child) => possible_moves.push(child),
            None => {}
        }
    }
    possible_moves
}
