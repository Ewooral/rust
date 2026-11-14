//! Beyond the Book: buffered input and output.
//! Run: cargo run -p minigrep --example buffered_io
//! Pipe text in: echo "one two`nthree" | cargo run -p minigrep --example buffered_io -- -
//!
//! `fs::read_to_string` loads a whole file at once. For big files, or for
//! input that never ends (stdin from another program), read it LINE BY LINE
//! with a BufReader instead. Writing many small pieces? Buffer that too.

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

fn main() -> io::Result<()> {
    let path = env::temp_dir().join("minigrep_big_file.txt");

    // BufWriter collects writes in memory and sends them to the OS in big
    // chunks. Without it, each writeln! below would be a separate system call.
    {
        let mut out = BufWriter::new(File::create(&path)?);
        for i in 1..=100_000 {
            writeln!(
                out,
                "line {i}: {}",
                if i % 25_000 == 0 { "needle" } else { "hay" }
            )?;
        }
        out.flush()?; // also happens on drop, but then errors are silently lost
    }

    // BufReader + lines(): one line in memory at a time, however big the file.
    let reader = BufReader::new(File::open(&path)?);
    let mut found = 0;
    for (number, line) in reader.lines().enumerate() {
        let line = line?; // each line can fail (bad UTF-8, disk error)
        if line.contains("needle") {
            println!("{:>6}: {line}", number + 1);
            found += 1;
        }
    }
    println!("{found} matches in a 100,000-line file, read line by line");
    std::fs::remove_file(&path)?;

    // The same code works on standard input: `-` as the argument.
    if env::args().nth(1).as_deref() == Some("-") {
        let stdin = io::stdin();
        let mut count = 0;
        for line in stdin.lock().lines() {
            let line = line?;
            count += line.split_whitespace().count();
        }
        println!("stdin had {count} words");
    }

    // Many println! calls each lock stdout. Lock once and buffer for speed:
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());
    for i in 1..=3 {
        writeln!(handle, "fast output line {i}")?;
    }
    Ok(())
}
