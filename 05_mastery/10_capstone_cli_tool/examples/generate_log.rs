//! A test-data generator: writes N realistic log lines to stdout, to pipe
//! into logsum (and to watch its progress line on a big file).
//! Run: cargo run -p capstone_cli_tool --example generate_log -- 20
//!      cargo run -q -p capstone_cli_tool --example generate_log -- 200000 | cargo run -q -p capstone_cli_tool
//!      cargo run -q -p capstone_cli_tool --example generate_log -- 2000000 > big.log
//!      cargo run -q -p capstone_cli_tool -- big.log        (then delete big.log)
//!
//! No `rand` crate: a tiny "linear congruential generator" makes the same
//! pseudo-random log every time, which is what test data should be.

use std::env;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

/// Numbers that look random but repeat exactly on every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn generate(lines: u64, out: &mut impl Write) -> io::Result<()> {
    let mut rng = Lcg(42);
    for i in 0..lines {
        let (h, m, s) = (8 + i / 3600 % 12, i / 60 % 60, i % 60);
        let ts = format!("2026-05-01T{h:02}:{m:02}:{s:02}Z");
        match rng.below(100) {
            0..=59 => writeln!(
                out,
                "{ts} INFO GET /orders/{} 200 {}ms",
                rng.below(5000),
                rng.below(80)
            )?,
            60..=74 => writeln!(out, "{ts} DEBUG cache miss for user {}", rng.below(9000))?,
            75..=86 => writeln!(
                out,
                "{ts} WARN [db] slow query took {}ms",
                1000 + rng.below(4000)
            )?,
            87..=92 => writeln!(
                out,
                "{ts} WARN retry {} of 3 for mail.example.com",
                1 + rng.below(3)
            )?,
            93..=97 => writeln!(
                out,
                "{ts} ERROR payment {} failed: card declined",
                rng.below(100_000)
            )?,
            98 => {
                writeln!(
                    out,
                    "{ts} ERROR unhandled exception in worker {}",
                    rng.below(8)
                )?;
                writeln!(out, "    at billing::charge (src/billing.rs:88)")?;
            }
            _ => writeln!(out, "{ts} FATAL out of memory, shutting down")?,
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let lines = match env::args().nth(1).map(|n| n.parse::<u64>()) {
        None => 1000,
        Some(Ok(n)) => n,
        Some(Err(e)) => {
            eprintln!("usage: generate_log [LINES]  ({e})");
            return ExitCode::from(2);
        }
    };
    let mut out = BufWriter::new(io::stdout().lock());
    match generate(lines, &mut out).and_then(|()| out.flush()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("generate_log: {e}");
            ExitCode::from(2)
        }
    }
}
