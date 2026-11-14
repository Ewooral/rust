//! Solution to exercise 6: tuples, arrays and conversions.
//!
//! Test: cargo test -p variables_and_data_types --features solutions --bin sol06_tuples_arrays_casts

fn swap(pair: (i32, char)) -> (char, i32) {
    let (number, letter) = pair;
    (letter, number)
}

fn weekend(days: [&str; 7]) -> (&str, &str) {
    (days[5], days[6])
}

fn average(scores: [u32; 4]) -> f64 {
    // `f64::from` can't lose information, and converting BEFORE adding means
    // four huge scores can't overflow a u32.
    let total =
        f64::from(scores[0]) + f64::from(scores[1]) + f64::from(scores[2]) + f64::from(scores[3]);
    total / 4.0
}

fn to_byte(n: i32) -> Option<u8> {
    // `n as u8` would turn 300 into 44 and -1 into 255 without a word.
    u8::try_from(n).ok()
}

fn main() {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    println!("{:?}", swap((1, 'a')));
    println!("{:?}", weekend(days));
    println!("{}", average([7, 8, 8, 8]));
    println!("{:?} {:?}", to_byte(200), to_byte(300));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_a_pair() {
        assert_eq!(swap((1, 'a')), ('a', 1));
        assert_eq!(swap((-7, 'z')), ('z', -7));
    }

    #[test]
    fn finds_the_weekend() {
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        assert_eq!(weekend(days), ("Sat", "Sun"));
    }

    #[test]
    fn average_keeps_the_fraction() {
        assert_eq!(average([7, 8, 8, 8]), 7.75);
        assert_eq!(average([1, 2, 3, 4]), 2.5);
        assert_eq!(average([0, 0, 0, 0]), 0.0);
    }

    #[test]
    fn average_of_big_scores_does_not_overflow() {
        let big = u32::MAX;
        assert_eq!(average([big, big, big, big]), big as f64);
    }

    #[test]
    fn to_byte_only_when_it_fits() {
        assert_eq!(to_byte(0), Some(0));
        assert_eq!(to_byte(255), Some(255));
        assert_eq!(to_byte(256), None);
        assert_eq!(to_byte(-1), None);
    }
}
