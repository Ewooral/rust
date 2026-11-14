//! Solution to exercise 5: return several values with a tuple.
//!
//! Test: cargo test -p functions_and_comments --features solutions --bin sol05_split_time

fn split_seconds(total: u32) -> (u32, u32, u32) {
    (total / 3600, total % 3600 / 60, total % 60)
}

fn clock(total: u32) -> String {
    let (hours, minutes, seconds) = split_seconds(total);
    format!("{hours:02}:{minutes:02}:{seconds:02}")
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
