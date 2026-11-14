//! Time as a dependency: a `Clock` trait the code asks for "now".
//!
//! Code that calls `SystemTime::now()` directly can only be tested at the
//! time the test happens to run. Code that asks a `Clock` can be tested at
//! any time we like: Friday at 16:59, the second a coupon expires, a leap
//! day. Production passes [`SystemClock`]; tests pass a [`FixedClock`].

use std::cell::Cell;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A moment in time: whole seconds since 1970-01-01 00:00:00 UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub u64);

/// A day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

const DAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

impl Timestamp {
    /// A UTC date and time. Panics if the date is before 1970.
    ///
    /// ```
    /// use advanced_testing::clock::{Timestamp, Weekday};
    /// let t = Timestamp::from_utc(2026, 11, 16, 15, 30);
    /// assert_eq!(t.weekday(), Weekday::Monday);
    /// assert_eq!(t.to_string(), "2026-11-16 15:30 UTC");
    /// ```
    pub fn from_utc(year: i64, month: u32, day: u32, hour: u32, minute: u32) -> Timestamp {
        let days = days_from_civil(year, month, day);
        let seconds = days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60;
        Timestamp(u64::try_from(seconds).expect("dates before 1970 are not supported"))
    }

    /// The day of the week (1970-01-01 was a Thursday).
    pub fn weekday(self) -> Weekday {
        DAYS[((self.0 / 86_400 + 3) % 7) as usize]
    }

    /// The hour of the day, 0 to 23 (UTC).
    pub fn hour(self) -> u32 {
        (self.0 / 3_600 % 24) as u32
    }

    /// The same moment plus some seconds.
    pub fn plus_seconds(self, seconds: u64) -> Timestamp {
        Timestamp(self.0 + seconds)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = civil_from_days((self.0 / 86_400) as i64);
        let minutes = self.0 / 60 % 60;
        write!(
            f,
            "{y:04}-{m:02}-{d:02} {:02}:{minutes:02} UTC",
            self.hour()
        )
    }
}

// Howard Hinnant's well-known algorithms for converting between a date and
// a day count, valid for every date in the proleptic Gregorian calendar.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Something that knows what time it is.
pub trait Clock {
    /// The current time.
    fn now(&self) -> Timestamp;
}

/// The real clock, for production.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is set before 1970");
        Timestamp(since_epoch.as_secs())
    }
}

/// A clock that says whatever the test tells it to. A **fake**: a working,
/// simplified implementation of the real thing.
#[derive(Debug)]
pub struct FixedClock(Cell<Timestamp>);

impl FixedClock {
    /// A clock stopped at `now`.
    pub fn at(now: Timestamp) -> FixedClock {
        FixedClock(Cell::new(now))
    }

    /// Moves the clock forward, like waiting without the wait.
    pub fn advance(&self, seconds: u64) {
        self.0.set(self.0.get().plus_seconds(seconds));
    }

    /// Sets the clock to another time.
    pub fn set(&self, now: Timestamp) {
        self.0.set(now);
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0.get()
    }
}

// A reference to a clock is a clock too, so a test can keep the FixedClock
// (to advance it) and lend it to the code under test.
impl<C: Clock + ?Sized> Clock for &C {
    fn now(&self) -> Timestamp {
        (**self).now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_was_a_thursday() {
        assert_eq!(Timestamp(0).weekday(), Weekday::Thursday);
        assert_eq!(Timestamp(0).to_string(), "1970-01-01 00:00 UTC");
    }

    #[test]
    fn dates_round_trip_through_display() {
        for (y, m, d) in [(2000, 2, 29), (2024, 12, 31), (2026, 1, 1), (2100, 3, 1)] {
            let t = Timestamp::from_utc(y, m, d, 23, 59);
            assert_eq!(t.to_string(), format!("{y:04}-{m:02}-{d:02} 23:59 UTC"));
        }
    }

    #[test]
    fn fixed_clock_only_moves_when_told() {
        let clock = FixedClock::at(Timestamp(100));
        assert_eq!(clock.now(), Timestamp(100));
        clock.advance(60);
        assert_eq!(clock.now(), Timestamp(160));
        // A borrowed clock is a clock too (the blanket impl for &C).
        fn read(clock: impl Clock) -> Timestamp {
            clock.now()
        }
        assert_eq!(read(&clock), Timestamp(160));
    }

    #[test]
    fn system_clock_is_after_this_lesson_was_written() {
        assert!(SystemClock.now() > Timestamp::from_utc(2026, 1, 1, 0, 0));
    }
}
