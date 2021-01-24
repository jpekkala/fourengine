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

    /// A slightly faster version of FromPrimitive::from_u64 that also returns Score::Unknown
    /// instead of Option::None.
    #[inline]
    pub fn from_u64_fast(number: u64) -> Score {
        match number {
            1 => Score::Loss,
            2 => Score::DrawOrLoss,
            3 => Score::Draw,
            4 => Score::DrawOrWin,
            5 => Score::Win,
            _ => Score::Unknown,
        }
    }

    pub fn from_char(ch: char) -> Score {
        match ch {
            '-' => Score::Loss,
            '<' => Score::DrawOrLoss,
            '=' => Score::Draw,
            '>' => Score::DrawOrWin,
            '+' => Score::Win,
            _ => Score::Unknown,
        }
    }

    pub fn from_string(score_str: &str) -> Score {
        if score_str.len() == 1 {
            return Score::from_char(score_str.chars().next().unwrap());
        }

        match score_str.to_lowercase().as_ref() {
            "win" => Score::Win,
            "loss" => Score::Loss,
            "draw" => Score::Draw,
            _ => Score::Unknown,
        }
    }

    #[allow(dead_code)]
    pub fn is_compatible(self, other: Score) -> bool {
        if self == other || self == Score::Unknown || other == Score::Unknown {
            true
        } else if self.is_exact() {
            if other.is_exact() {
                false
            } else {
                other.is_compatible(self)
            }
        } else {
            match self {
                Score::DrawOrLoss => {
                    other == Score::Draw || other == Score::Loss || other == Score::DrawOrWin
                }
                Score::DrawOrWin => {
                    other == Score::Draw || other == Score::Win || other == Score::DrawOrLoss
                }
                _ => false,
            }
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Score::Loss => '-',
            Score::DrawOrLoss => '<',
            Score::Draw => '=',
            Score::DrawOrWin => '>',
            Score::Win => '+',
            Score::Unknown => '?',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare() {
        assert!(Score::Win > Score::Loss);
    }

    #[test]
    fn flip() {
        assert_eq!(Score::Win.flip(), Score::Loss);
        assert_eq!(Score::Unknown.flip(), Score::Unknown);
    }

    #[test]
    fn compatible_scores() {
        assert!(Score::Draw.is_compatible(Score::DrawOrWin));
    }
}
