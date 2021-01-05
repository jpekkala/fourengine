use num_derive::FromPrimitive;

#[derive(FromPrimitive, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Score {
    Loss = 1,
    DrawOrLoss,
    Draw,
    DrawOrWin,
    Win,
    Unknown = 0,
}

impl Score {
    pub fn is_exact(self) -> bool {
        self == Score::Loss || self == Score::Draw || self == Score::Win
    }

    /// Returns the score from the other player's perspective
    pub fn flip(self) -> Score {
        match self {
            Score::Unknown => Score::Unknown,
            Score::Draw => Score::Draw,
            Score::Loss => Score::Win,
            Score::Win => Score::Loss,
            Score::DrawOrLoss => Score::DrawOrWin,
            Score::DrawOrWin => Score::DrawOrLoss,
        }
    }
}
