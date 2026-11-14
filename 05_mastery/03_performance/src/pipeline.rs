//! The mini project: one log report, four implementations.
//!
//! Each version gives **exactly the same report** (the tests check that), and
//! each one fixes the biggest cost the benchmarks found in the one before:
//!
//! | Version | Change | What it saves |
//! |---|---|---|
//! | [`analyse_naive`] | copies every line and field into `String`s | (the starting point) |
//! | [`analyse_borrowed`] | borrows `&str` slices of the input | ~7 allocations per line |
//! | [`analyse_fast`] | no `Vec` per line, a pre-sized map, a faster hasher | the last allocation per line, hashing time |
//! | [`analyse_parallel`] | the fast version on every core with rayon | wall-clock time |
//!
//! A line is valid when it has exactly six fields (timestamp, IP, method,
//! path, status, latency), the status is a number from 100 to 599 and the
//! latency fits in a `u32`. Anything else is counted as malformed.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use rayon::prelude::*;

/// Totals for one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathStats {
    /// The request path, e.g. `/api/orders`.
    pub path: String,
    /// How many requests.
    pub requests: u64,
    /// How many failed with a 5xx status.
    pub errors: u64,
    /// Sum of all latencies, in milliseconds.
    pub total_ms: u64,
    /// The slowest single request, in milliseconds.
    pub max_ms: u32,
}

/// The finished report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// Lines read, including malformed ones.
    pub lines: u64,
    /// Lines that did not parse.
    pub malformed: u64,
    /// One entry per path, most total time first (ties: by path).
    pub paths: Vec<PathStats>,
}

impl Report {
    /// The `top` paths that cost the most time, as a small text table.
    pub fn render(&self, top: usize) -> String {
        let mut out = format!(
            "{} lines, {} malformed, {} paths\n{:<22} {:>8} {:>7} {:>10} {:>8} {:>7}\n",
            self.lines,
            self.malformed,
            self.paths.len(),
            "path",
            "requests",
            "errors",
            "total ms",
            "avg ms",
            "max ms"
        );
        for p in self.paths.iter().take(top) {
            let avg = p.total_ms as f64 / p.requests as f64;
            out.push_str(&format!(
                "{:<22} {:>8} {:>7} {:>10} {:>8.1} {:>7}\n",
                p.path, p.requests, p.errors, p.total_ms, avg, p.max_ms
            ));
        }
        out
    }
}

/// Running totals while reading (no path: the map key holds it).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Totals {
    requests: u64,
    errors: u64,
    total_ms: u64,
    max_ms: u32,
}

impl Totals {
    #[inline]
    fn record(&mut self, status: u16, ms: u32) {
        self.requests += 1;
        if status >= 500 {
            self.errors += 1;
        }
        self.total_ms += u64::from(ms);
        self.max_ms = self.max_ms.max(ms);
    }

    fn merge(&mut self, other: Totals) {
        self.requests += other.requests;
        self.errors += other.errors;
        self.total_ms += other.total_ms;
        self.max_ms = self.max_ms.max(other.max_ms);
    }
}

fn parse_status(field: &str) -> Option<u16> {
    field.parse().ok().filter(|s| (100..=599).contains(s))
}

/// Most total time first; ties broken by path so the order is always the same.
fn sort_paths(paths: &mut [PathStats]) {
    // Paths are unique, so an unstable sort gives the same order as a stable
    // one, and it sorts in place without allocating a buffer.
    paths.sort_unstable_by(|a, b| {
        b.total_ms
            .cmp(&a.total_ms)
            .then_with(|| a.path.cmp(&b.path))
    });
}

fn into_report<'a>(
    lines: u64,
    malformed: u64,
    totals: impl IntoIterator<Item = (&'a str, Totals)>,
) -> Report {
    let mut paths: Vec<PathStats> = totals
        .into_iter()
        .map(|(path, t)| PathStats {
            path: path.to_string(), // one String per DISTINCT path, not per line
            requests: t.requests,
            errors: t.errors,
            total_ms: t.total_ms,
            max_ms: t.max_ms,
        })
        .collect();
    sort_paths(&mut paths);
    Report {
        lines,
        malformed,
        paths,
    }
}

// ---------------------------------------------------------------------------
// Version 1: the way most of us write it the first time.
// ---------------------------------------------------------------------------

/// Version 1: correct, readable, and allocates for every line and field.
pub fn analyse_naive(input: &str) -> Report {
    // A copy of every line...
    let lines: Vec<String> = input.lines().map(|line| line.to_string()).collect();
    let mut stats: HashMap<String, PathStats> = HashMap::new();
    let mut malformed = 0;

    for line in &lines {
        // ...and a copy of every field: 6 Strings and a Vec per line.
        let fields: Vec<String> = line
            .split_ascii_whitespace()
            .map(|f| f.to_string())
            .collect();
        let parsed = if fields.len() == 6 {
            parse_status(&fields[4]).zip(fields[5].parse::<u32>().ok())
        } else {
            None
        };
        let Some((status, ms)) = parsed else {
            malformed += 1;
            continue;
        };
        let path = fields[3].clone(); // one more copy, "to be safe"
        let entry = stats.entry(path.clone()).or_insert(PathStats {
            path,
            requests: 0,
            errors: 0,
            total_ms: 0,
            max_ms: 0,
        });
        entry.requests += 1;
        if status >= 500 {
            entry.errors += 1;
        }
        entry.total_ms += u64::from(ms);
        entry.max_ms = entry.max_ms.max(ms);
    }

    let mut paths: Vec<PathStats> = stats.values().cloned().collect();
    sort_paths(&mut paths);
    Report {
        lines: lines.len() as u64,
        malformed,
        paths,
    }
}

// ---------------------------------------------------------------------------
// Version 2: borrow instead of copying.
// ---------------------------------------------------------------------------

/// Version 2: every line, field and map key is a `&str` slice of `input`.
/// The only allocation left per line is the `Vec<&str>` of fields.
pub fn analyse_borrowed(input: &str) -> Report {
    let mut stats: HashMap<&str, Totals> = HashMap::new();
    let (mut lines, mut malformed) = (0, 0);

    for line in input.lines() {
        lines += 1;
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        let parsed = if fields.len() == 6 {
            parse_status(fields[4]).zip(fields[5].parse::<u32>().ok())
        } else {
            None
        };
        match parsed {
            Some((status, ms)) => stats.entry(fields[3]).or_default().record(status, ms),
            None => malformed += 1,
        }
    }
    into_report(lines, malformed, stats)
}

// ---------------------------------------------------------------------------
// Version 3: no allocation per line, and less hashing work.
// ---------------------------------------------------------------------------

/// FNV-1a: a very simple, fast hash for short keys.
///
/// `HashMap`'s default hasher (SipHash) is designed to resist "HashDoS"
/// attacks, where someone sends keys chosen to collide. Our keys come from
/// our own logs, so a simpler hash is a fair trade. Never make that trade for
/// keys an attacker controls.
#[derive(Debug, Clone, Copy)]
pub struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for FnvHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// A `HashMap` that hashes with [`FnvHasher`].
pub type FnvMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

/// One parsed line, borrowing from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry<'a> {
    /// The request path.
    pub path: &'a str,
    /// The HTTP status code.
    pub status: u16,
    /// The latency in milliseconds.
    pub ms: u32,
}

/// Parses one line without allocating: the fields are taken straight from
/// the iterator, so there is no `Vec` to fill.
#[inline]
pub fn parse_line(line: &str) -> Option<Entry<'_>> {
    let mut fields = line.split_ascii_whitespace();
    let _timestamp = fields.next()?;
    let _ip = fields.next()?;
    let _method = fields.next()?;
    let path = fields.next()?;
    let status = parse_status(fields.next()?)?;
    let ms = fields.next()?.parse().ok()?;
    if fields.next().is_some() {
        return None; // more than six fields
    }
    Some(Entry { path, status, ms })
}

/// Totals for part of the input. The parallel version makes one per chunk
/// and merges them; the sequential version uses just one.
#[derive(Debug, Default)]
struct Partial<'a> {
    lines: u64,
    malformed: u64,
    stats: FnvMap<&'a str, Totals>,
}

impl<'a> Partial<'a> {
    fn with_capacity(paths: usize) -> Self {
        Partial {
            lines: 0,
            malformed: 0,
            stats: FnvMap::with_capacity_and_hasher(paths, Default::default()),
        }
    }

    #[inline]
    fn add_line(&mut self, line: &'a str) {
        self.lines += 1;
        match parse_line(line) {
            Some(e) => self.stats.entry(e.path).or_default().record(e.status, e.ms),
            None => self.malformed += 1,
        }
    }

    fn merge(mut self, other: Partial<'a>) -> Self {
        self.lines += other.lines;
        self.malformed += other.malformed;
        for (path, totals) in other.stats {
            self.stats.entry(path).or_default().merge(totals);
        }
        self
    }

    fn finish(self) -> Report {
        into_report(self.lines, self.malformed, self.stats)
    }
}

/// How many distinct paths to make room for up front. A guess is fine: the
/// map still grows if the guess is too small.
const EXPECTED_PATHS: usize = 64;

/// Version 3: zero allocations per line, a pre-sized map, FNV hashing.
pub fn analyse_fast(input: &str) -> Report {
    let mut partial = Partial::with_capacity(EXPECTED_PATHS);
    for line in input.lines() {
        partial.add_line(line);
    }
    partial.finish()
}

// ---------------------------------------------------------------------------
// Version 4: the same work, on every core.
// ---------------------------------------------------------------------------

/// Version 4: version 3 split across all cores with rayon. Each worker
/// `fold`s its lines into its own `Partial` (no locks, no sharing), then the
/// partials are merged pairwise with `reduce`.
pub fn analyse_parallel(input: &str) -> Report {
    input
        .par_lines()
        .fold(
            || Partial::with_capacity(EXPECTED_PATHS),
            |mut partial, line| {
                partial.add_line(line);
                partial
            },
        )
        .reduce(Partial::default, Partial::merge)
        .finish()
}

/// The signature every version shares.
pub type Analyse = fn(&str) -> Report;

/// Every version, with a short name, for the program and the benchmarks.
pub const VERSIONS: [(&str, Analyse); 4] = [
    ("v1_naive", analyse_naive),
    ("v2_borrowed", analyse_borrowed),
    ("v3_fast", analyse_fast),
    ("v4_parallel", analyse_parallel),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc_counter::count_allocations;
    use crate::logs;

    const SAMPLE: &str = "\
2026-11-14T10:00:00Z 203.0.113.1 GET /a 200 10
2026-11-14T10:00:01Z 203.0.113.2 GET /b 500 30
not a log line
2026-11-14T10:00:02Z 203.0.113.3 GET /a 404 50
2026-11-14T10:00:03Z 203.0.113.4 GET /a 200 seven
2026-11-14T10:00:04Z 203.0.113.5 GET /a 200 5 extra
2026-11-14T10:00:05Z 203.0.113.6 GET /c 700 1
";

    #[test]
    fn naive_version_counts_correctly() {
        let report = analyse_naive(SAMPLE);
        assert_eq!(report.lines, 7);
        assert_eq!(report.malformed, 4);
        let a = &report.paths[0];
        assert_eq!(
            (a.path.as_str(), a.requests, a.errors, a.total_ms, a.max_ms),
            ("/a", 2, 0, 60, 50)
        );
        let b = &report.paths[1];
        assert_eq!((b.path.as_str(), b.errors), ("/b", 1));
        assert_eq!(report.paths.len(), 2);
    }

    #[test]
    fn every_version_agrees_on_the_sample() {
        let expected = analyse_naive(SAMPLE);
        for (name, analyse) in VERSIONS {
            assert_eq!(analyse(SAMPLE), expected, "{name} disagrees");
        }
    }

    #[test]
    fn every_version_agrees_on_a_generated_log() {
        let log = logs::generate(5_000, 42);
        let expected = analyse_naive(&log);
        assert!(expected.malformed > 0, "the generator adds some bad lines");
        for (name, analyse) in VERSIONS {
            assert_eq!(analyse(&log), expected, "{name} disagrees");
        }
    }

    #[test]
    fn every_version_handles_empty_input() {
        for (name, analyse) in VERSIONS {
            assert_eq!(analyse(""), Report::default(), "{name}");
        }
    }

    #[test]
    fn parse_line_takes_slices_of_the_line() {
        let line = "t ip GET /x 503 12";
        assert_eq!(
            parse_line(line),
            Some(Entry {
                path: "/x",
                status: 503,
                ms: 12
            })
        );
        assert_eq!(parse_line("t ip GET /x 503"), None);
        assert_eq!(parse_line("t ip GET /x 99 1"), None);
    }

    // Allocation budgets turn "it's faster" into a test that can't flake:
    // counts are exact, timings never are.
    #[test]
    fn naive_allocates_per_line_but_fast_does_not() {
        let log = logs::generate(1_000, 7);
        let (_, naive) = count_allocations(|| analyse_naive(&log));
        let (_, borrowed) = count_allocations(|| analyse_borrowed(&log));
        let (_, fast) = count_allocations(|| analyse_fast(&log));
        assert!(naive.allocations > 7_000, "{naive:?}");
        assert!(borrowed.allocations >= 1_000, "{borrowed:?}");
        // The map, the report's Vec and one String per distinct path (16).
        assert!(fast.allocations <= 20, "{fast:?}");
    }

    #[test]
    fn fnv_hasher_matches_the_reference_value() {
        // The FNV-1a 64-bit hash of "a" is 0xaf63dc4c8601ec8c.
        let mut h = FnvHasher::default();
        h.write(b"a");
        assert_eq!(h.finish(), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn render_shows_the_top_paths() {
        let text = analyse_fast(SAMPLE).render(1);
        assert!(text.starts_with("7 lines, 4 malformed, 2 paths"));
        assert!(text.contains("/a"));
        assert!(!text.contains("/b"), "only the top 1 path:\n{text}");
    }
}
