//! Testing time: inject a clock instead of calling `SystemTime::now()`.
//!
//! Run:  cargo run -p advanced_testing --example injected_clock
//! Test: cargo test -p advanced_testing --example injected_clock

use advanced_testing::clock::{Clock, SystemClock, Timestamp};

const THIRTY_MINUTES: u64 = 30 * 60;

struct Session {
    last_seen: Timestamp,
}

impl Session {
    // Hard to test: the answer depends on when the test runs. To see it
    // expire you'd have to sleep for 30 minutes.
    //
    //     fn is_expired(&self) -> bool {
    //         let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    //         now - self.last_seen.0 >= THIRTY_MINUTES
    //     }

    // Easy to test: "now" is a parameter, so the test decides what time it is.
    fn is_expired(&self, clock: &impl Clock) -> bool {
        clock.now().0 - self.last_seen.0 >= THIRTY_MINUTES
    }

    fn touch(&mut self, clock: &impl Clock) {
        self.last_seen = clock.now();
    }
}

fn main() {
    let clock = SystemClock;
    let mut session = Session {
        last_seen: Timestamp(0),
    };
    println!("now: {}", clock.now());
    println!(
        "a session last seen in 1970 is expired: {}",
        session.is_expired(&clock)
    );
    session.touch(&clock);
    println!("after touch(): expired: {}", session.is_expired(&clock));
}

#[cfg(test)]
mod tests {
    use super::*;
    use advanced_testing::clock::FixedClock;

    fn session_started_at(clock: &FixedClock) -> Session {
        Session {
            last_seen: clock.now(),
        }
    }

    #[test]
    fn a_session_lasts_just_under_thirty_minutes() {
        let clock = FixedClock::at(Timestamp::from_utc(2026, 11, 16, 9, 0));
        let session = session_started_at(&clock);

        clock.advance(THIRTY_MINUTES - 1); // 29:59 later, instantly
        assert!(!session.is_expired(&clock));

        clock.advance(1); // exactly 30:00
        assert!(session.is_expired(&clock));
    }

    #[test]
    fn activity_keeps_a_session_alive() {
        let clock = FixedClock::at(Timestamp::from_utc(2026, 11, 16, 9, 0));
        let mut session = session_started_at(&clock);

        clock.advance(20 * 60);
        session.touch(&clock);
        clock.advance(20 * 60); // 40 minutes after the start

        assert!(!session.is_expired(&clock));
    }

    #[test]
    fn works_across_midnight_on_new_years_eve() {
        let clock = FixedClock::at(Timestamp::from_utc(2026, 12, 31, 23, 45));
        let session = session_started_at(&clock);

        clock.set(Timestamp::from_utc(2027, 1, 1, 0, 15));

        assert!(session.is_expired(&clock));
    }
}
