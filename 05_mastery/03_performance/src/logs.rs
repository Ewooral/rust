//! A deterministic generator of web-server access logs.
//!
//! Benchmarks need realistic input that is **the same every run**, or two
//! measurements can't be compared. So instead of a random-number crate this
//! uses a tiny seeded generator (xorshift64*): the same seed always gives
//! the same log.
//!
//! A line looks like this (timestamp, client IP, method, path, status,
//! latency in milliseconds):
//!
//! ```text
//! 2026-11-14T10:00:07Z 203.0.113.42 GET /api/orders 200 37
//! ```

use std::fmt::Write as _;

/// Endpoints with a typical latency in milliseconds. A few are slow, as in
/// real life, so the report has something to find.
const ENDPOINTS: [(&str, &str, u32); 16] = [
    ("GET", "/", 3),
    ("GET", "/login", 8),
    ("POST", "/login", 40),
    ("GET", "/api/orders", 35),
    ("POST", "/api/orders", 90),
    ("GET", "/api/orders/recent", 25),
    ("GET", "/api/products", 20),
    ("GET", "/api/products/search", 120),
    ("GET", "/api/cart", 12),
    ("POST", "/api/cart", 18),
    ("POST", "/api/checkout", 250),
    ("GET", "/api/users/me", 6),
    ("GET", "/static/app.js", 2),
    ("GET", "/static/style.css", 2),
    ("GET", "/reports/monthly", 900),
    ("GET", "/health", 1),
];

/// A small, fast, seeded pseudo-random generator (xorshift64*).
/// Good enough for test data; never use it for anything secret.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// A generator that always produces the same sequence for the same seed.
    pub fn new(seed: u64) -> Self {
        // Zero would get stuck at zero forever, so mix the seed first.
        Rng((seed ^ 0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// The next pseudo-random number.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `0..n` (n must be greater than 0).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Generates `lines` log lines, about 1 in 500 of them malformed.
///
/// ```
/// let log = performance::logs::generate(3, 7);
/// assert_eq!(log.lines().count(), 3);
/// assert_eq!(log, performance::logs::generate(3, 7)); // same seed, same log
/// ```
pub fn generate(lines: usize, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    // Roughly 55 bytes per line: one allocation up front instead of ~20
    // doublings (this module practises what the lesson preaches).
    let mut out = String::with_capacity(lines * 60);
    for i in 0..lines {
        if rng.below(500) == 0 {
            out.push_str("-- log rotated --\n");
            continue;
        }
        let (method, path, typical) = ENDPOINTS[rng.below(ENDPOINTS.len() as u64) as usize];
        let status = match rng.below(100) {
            0..=1 => 500,
            2 => 503,
            3..=6 => 404,
            _ => 200,
        };
        // Latency: around the typical value, with an occasional slow outlier.
        let jitter = rng.below(u64::from(typical) + 1) as u32;
        let outlier = if rng.below(50) == 0 { typical * 10 } else { 0 };
        let latency = typical / 2 + jitter + outlier;
        let seconds = i as u64;
        let _ = writeln!(
            out,
            "2026-11-14T{:02}:{:02}:{:02}Z 203.0.113.{} {method} {path} {status} {latency}",
            10 + seconds / 3600 % 14,
            seconds / 60 % 60,
            seconds % 60,
            rng.below(254) + 1,
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_the_same_log() {
        assert_eq!(generate(200, 1), generate(200, 1));
        assert_ne!(generate(200, 1), generate(200, 2));
    }

    #[test]
    fn lines_have_six_fields_or_are_rotation_markers() {
        for line in generate(2_000, 3).lines() {
            let fields = line.split_ascii_whitespace().count();
            assert!(fields == 6 || line.starts_with("--"), "odd line: {line}");
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(0);
        assert!((0..1_000).all(|_| rng.below(7) < 7));
    }
}
