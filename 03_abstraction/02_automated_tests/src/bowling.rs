//! Ten-pin bowling scores, the classic test-driven-development kata.
//!
//! The rules:
//! - A game has 10 frames. In each frame you get up to two rolls to knock
//!   down 10 pins.
//! - **Strike**: all 10 with the first roll. Score 10 + the next TWO rolls.
//! - **Spare**: all 10 with both rolls. Score 10 + the next ONE roll.
//! - **Open frame**: fewer than 10. Score the pins knocked down.
//! - A strike or spare in the 10th frame earns the bonus rolls needed to
//!   score it (2 for a strike, 1 for a spare), and nothing more.
//!
//! ```
//! use automated_tests::bowling::score;
//!
//! assert_eq!(score(&[10; 12]), Ok(300)); // a perfect game: 12 strikes
//! assert_eq!(score(&[0; 20]), Ok(0)); // a gutter game
//! ```

use std::fmt;

/// Why a list of rolls is not a valid, complete game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BowlingError {
    /// A roll, or a frame, knocked down more than 10 pins.
    TooManyPins { frame: usize },
    /// The rolls stop before the game is over.
    Incomplete,
    /// There are rolls left over after the game is over.
    TooManyRolls,
}

impl fmt::Display for BowlingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BowlingError::TooManyPins { frame } => {
                write!(f, "frame {frame} knocks down more than 10 pins")
            }
            BowlingError::Incomplete => write!(f, "the game is not finished"),
            BowlingError::TooManyRolls => write!(f, "there are rolls after the end of the game"),
        }
    }
}

impl std::error::Error for BowlingError {}

/// Scores a complete game.
///
/// # Errors
///
/// Returns a [`BowlingError`] if a frame knocks down more than 10 pins, the
/// rolls stop early, or there are rolls after the game has ended.
///
/// ```
/// use automated_tests::bowling::{score, BowlingError};
///
/// // A spare (5 + 5) earns the next roll (3) as a bonus: 10 + 3, then 3 + 0.
/// let mut rolls = vec![5, 5, 3, 0];
/// rolls.extend([0; 16]);
/// assert_eq!(score(&rolls), Ok(16));
///
/// assert_eq!(score(&[1, 2]), Err(BowlingError::Incomplete));
/// ```
pub fn score(rolls: &[u32]) -> Result<u32, BowlingError> {
    let roll = |i: usize| rolls.get(i).copied().ok_or(BowlingError::Incomplete);

    let mut total = 0;
    let mut i = 0; // index of the first roll of the current frame

    for frame in 1..=10 {
        let first = roll(i)?;
        if first > 10 {
            return Err(BowlingError::TooManyPins { frame });
        }

        if first == 10 {
            // Strike: the next two rolls are the bonus.
            let (b1, b2) = (roll(i + 1)?, roll(i + 2)?);
            if frame == 10 {
                // Earlier frames' bonus rolls are checked again as their own
                // frames, which reports the right frame number. The 10th
                // frame's bonus rolls belong to no other frame.
                check_bonus_pair(b1, b2, frame)?;
            }
            total += 10 + b1 + b2;
            i += 1;
        } else {
            let second = roll(i + 1)?;
            if first + second > 10 {
                return Err(BowlingError::TooManyPins { frame });
            }
            if first + second == 10 {
                // Spare: the next roll is the bonus.
                let bonus = roll(i + 2)?;
                if bonus > 10 {
                    return Err(BowlingError::TooManyPins { frame });
                }
                total += 10 + bonus;
            } else {
                total += first + second;
            }
            i += 2;
        }

        if frame == 10 {
            let bonus_rolls = if first == 10 {
                2
            } else if first + rolls[i - 1] == 10 {
                1
            } else {
                0
            };
            if rolls.len() > i + bonus_rolls {
                return Err(BowlingError::TooManyRolls);
            }
        }
    }
    Ok(total)
}

/// The two rolls after a strike: each at most 10, and if the first of them
/// is not a strike, together at most 10 (they share one rack of pins).
fn check_bonus_pair(b1: u32, b2: u32, frame: usize) -> Result<(), BowlingError> {
    let fresh_rack_after_b1 = b1 == 10;
    if b1 > 10 || b2 > 10 || (!fresh_rack_after_b1 && b1 + b2 > 10) {
        return Err(BowlingError::TooManyPins { frame });
    }
    Ok(())
}

// UNIT TESTS live next to the code, in a `tests` module that is only compiled
// for `cargo test` (#[cfg(test)]). Being a child module, it can test private
// functions too, like `check_bonus_pair`.
#[cfg(test)]
mod tests {
    use super::*;

    fn game(rolls: &[u32]) -> Vec<u32> {
        // Pads a short list of rolls with gutter balls up to 20 rolls.
        let mut all = rolls.to_vec();
        all.resize(20, 0);
        all
    }

    #[test]
    fn gutter_game_scores_zero() {
        assert_eq!(score(&[0; 20]), Ok(0));
    }

    #[test]
    fn all_ones_scores_twenty() {
        assert_eq!(score(&[1; 20]), Ok(20));
    }

    #[test]
    fn spare_adds_the_next_roll() {
        assert_eq!(score(&game(&[5, 5, 3])), Ok(16));
    }

    #[test]
    fn strike_adds_the_next_two_rolls() {
        let mut rolls = vec![10, 3, 4];
        rolls.resize(19, 0);
        assert_eq!(score(&rolls), Ok(24));
    }

    #[test]
    fn perfect_game() {
        assert_eq!(score(&[10; 12]), Ok(300));
    }

    #[test]
    fn tenth_frame_spare_gets_one_bonus_roll() {
        let mut rolls = vec![0; 18];
        rolls.extend([7, 3, 10]);
        assert_eq!(score(&rolls), Ok(20));
    }

    #[test]
    fn private_helper_can_be_tested_directly() {
        assert!(check_bonus_pair(10, 10, 10).is_ok());
        assert!(check_bonus_pair(3, 7, 10).is_ok());
        assert_eq!(
            check_bonus_pair(3, 8, 10),
            Err(BowlingError::TooManyPins { frame: 10 })
        );
    }

    #[test]
    fn blames_the_frame_that_is_wrong() {
        // The strike in frame 1 is fine; frame 2 (6 + 5) is impossible.
        assert_eq!(
            score(&game(&[10, 6, 5])),
            Err(BowlingError::TooManyPins { frame: 2 })
        );
    }

    #[test]
    fn rejects_impossible_frames() {
        assert_eq!(
            score(&game(&[6, 5])),
            Err(BowlingError::TooManyPins { frame: 1 })
        );
        assert_eq!(
            score(&game(&[11])),
            Err(BowlingError::TooManyPins { frame: 1 })
        );
    }

    #[test]
    fn rejects_incomplete_and_overlong_games() {
        assert_eq!(score(&[]), Err(BowlingError::Incomplete));
        assert_eq!(score(&[0; 19]), Err(BowlingError::Incomplete));
        assert_eq!(score(&[0; 21]), Err(BowlingError::TooManyRolls));
        assert_eq!(score(&[10; 13]), Err(BowlingError::TooManyRolls));
        assert_eq!(score(&[10; 11]), Err(BowlingError::Incomplete));
    }
}
