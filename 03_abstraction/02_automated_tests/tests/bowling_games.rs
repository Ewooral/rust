//! INTEGRATION TESTS: this file is a separate crate. It uses the library only
//! through its public API, exactly like a real user would, so it cannot see
//! private items such as `check_bonus_pair`.
//!
//! Run just this file: cargo test -p automated_tests --test bowling_games

mod common;

use automated_tests::bowling::{BowlingError, score};
use common::{from_notation, padded_game};

#[test]
fn a_real_game_from_score_sheet_notation() {
    // X | 7 / | 9 - | X | - 8 | 8 / | - 6 | X | X | X 8 1
    let rolls = from_notation("X 7 / 9 - X - 8 8 / - 6 X X X 8 1");
    assert_eq!(score(&rolls), Ok(167));
}

#[test]
fn all_spares_with_five_pins() {
    let rolls = from_notation(&"5 / ".repeat(10))
        .into_iter()
        .chain([5])
        .collect::<Vec<_>>();
    assert_eq!(score(&rolls), Ok(150));
}

#[test]
fn nine_strikes_then_an_open_frame() {
    let mut rolls = vec![10; 9];
    rolls.extend([3, 4]);
    // frames 1-7: 30 each (210); frame 8: 10+10+3 = 23; frame 9: 10+3+4 = 17;
    // frame 10: 7
    assert_eq!(score(&rolls), Ok(210 + 23 + 17 + 7));
}

#[test]
fn errors_are_reported_through_the_public_api() {
    assert_eq!(
        score(&padded_game(&[9, 9])),
        Err(BowlingError::TooManyPins { frame: 1 })
    );
    let message = score(&[3]).unwrap_err().to_string();
    assert_eq!(message, "the game is not finished");
}

// Tests can return Result; `?` then fails the test on Err.
#[test]
fn tests_can_use_the_question_mark() -> Result<(), BowlingError> {
    let total = score(&[10; 12])?;
    assert_eq!(total, 300);
    Ok(())
}
