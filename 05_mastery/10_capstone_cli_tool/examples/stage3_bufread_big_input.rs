//! Stage 3: reading input of any size. Writes a ~12 MB log to the temp
//! folder, reads it three ways, and deletes it again.
//! Run: cargo run -p capstone_cli_tool --example stage3_bufread_big_input

use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

use capstone_cli_tool::for_each_line;

const LINES: usize = 200_000;

fn main() -> io::Result<()> {
    let path = env::temp_dir().join("logsum_stage3_big.log");
    {
        let mut out = BufWriter::new(File::create(&path)?);
        for i in 0..LINES {
            let level = ["INFO", "INFO", "INFO", "WARN", "ERROR"][i % 5];
            writeln!(
                out,
                "2026-05-01T09:{:02}:{:02}Z {level} request {i} handled in {}ms",
                i / 60 % 60,
                i % 60,
                i % 997
            )?;
        }
        // One line with a byte that isn't UTF-8, like real logs sometimes have.
        out.write_all(b"2026-05-01T10:00:00Z ERROR bad byte \xFF in a message\n")?;
        out.flush()?;
    }
    let size = fs::metadata(&path)?.len();
    println!("wrote {} lines, {:.1} MB", LINES + 1, size as f64 / 1e6);

    // 1. read_to_string: the whole file in memory at once, AND it refuses the
    //    file because of that one bad byte.
    let start = Instant::now();
    match fs::read_to_string(&path) {
        Ok(text) => println!("read_to_string: {} lines", text.lines().count()),
        Err(e) => println!("read_to_string: failed after {:?}: {e}", start.elapsed()),
    }

    // 2. lines(): one line at a time (good), a new String per line (slower),
    //    and an Err for the bad line (we count it and carry on).
    let start = Instant::now();
    let (mut ok, mut bad) = (0, 0);
    for line in BufReader::new(File::open(&path)?).lines() {
        match line {
            Ok(_) => ok += 1,
            Err(_) => bad += 1,
        }
    }
    println!(
        "lines():        {ok} lines, {bad} error(s), {:?}",
        start.elapsed()
    );

    // 3. for_each_line (logsum's way): read_until into ONE reused buffer,
    //    bad bytes replaced with U+FFFD, nothing lost.
    let start = Instant::now();
    let mut count = 0;
    let mut errors = 0;
    let mut last_error = String::new();
    let reader = BufReader::with_capacity(64 * 1024, File::open(&path)?);
    let bytes = for_each_line(reader, |line, _| {
        count += 1;
        if line.contains(" ERROR ") {
            errors += 1;
            last_error.clear();
            last_error.push_str(line);
        }
    })?;
    println!(
        "for_each_line:  {count} lines, {bytes} bytes, {errors} errors, {:?}",
        start.elapsed()
    );
    println!("  last error line: {last_error}");

    fs::remove_file(&path)?;
    Ok(())
}
