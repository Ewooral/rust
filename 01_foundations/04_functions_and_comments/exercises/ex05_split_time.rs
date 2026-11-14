//! Exercise 5: return several values with a tuple.
//!
//! - `split_seconds`: turn a number of seconds into `(hours, minutes,
//!   seconds)`. Use `/` and `%`: 3725 seconds is 1 hour, 2 minutes, 5 seconds.
//! - `clock`: format seconds as `"HH:MM:SS"` by CALLING `split_seconds` and
//!   destructuring its tuple. `{:02}` pads a number to two digits.
//!
//! Test: cargo test -p functions_and_comments --features exercises --bin ex05_split_time

#![allow(unused_variables)]

fn split_seconds(total: u32) -> (u32, u32, u32) {
    todo!("hours, minutes and seconds in {total}")
}

fn clock(total: u32) -> String {
    todo!("{total} as HH:MM:SS")
}

fn main() {
    println!("{}", clock(3725));
    println!("{}", clock(86_399));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_into_hours_minutes_seconds() {
        assert_eq!(split_seconds(3725), (1, 2, 5));
        assert_eq!(split_seconds(59), (0, 0, 59));
        assert_eq!(split_seconds(3600), (1, 0, 0));
    }

    #[test]
    fn formats_a_clock() {
        assert_eq!(clock(3725), "01:02:05");
        assert_eq!(clock(0), "00:00:00");
        assert_eq!(clock(86_399), "23:59:59");
    }

    #[test]
    fn hours_can_go_past_a_day() {
        assert_eq!(clock(90_000), "25:00:00");
    }
}
