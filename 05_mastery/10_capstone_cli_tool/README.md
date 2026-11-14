<!-- hand-written lesson: gen_course must not overwrite -->
# Capstone: a Production CLI Tool

**Guides:** [Command Line Applications in Rust](https://rust-cli.github.io/book/) (the official CLI book) · [Command Line Interface Guidelines](https://clig.dev) · [no-color.org](https://no-color.org)
· **Crates:** [clap](https://docs.rs/clap) ([derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)) · [anyhow](https://docs.rs/anyhow) · [thiserror](https://docs.rs/thiserror) · [serde_json](https://docs.rs/serde_json) · [toml](https://docs.rs/toml) · [dist (cargo-dist)](https://axodotdev.github.io/cargo-dist/book/)
· **std:** [`io::IsTerminal`](https://doc.rust-lang.org/std/io/trait.IsTerminal.html) · [`io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) · [`process::ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html)

You'll build **logsum**, a tool that reads log files (or a pipe), counts the lines at each level, and lists the most frequent warnings and errors, with the numbers blanked out so that similar messages are counted together. It's the tool you reach for during an incident ("what is this service complaining about?") and in CI ("fail the build if the test run logged an error").

minigrep ([lesson 03/03](../../03_abstraction/03_minigrep_cli_project/README.md)) was a first CLI. This one is **production quality**: the difference isn't the main feature, it's everything around it. Real argument parsing and `--help`, settings from a config file, the environment and flags, precise errors, input of any size, output for people *and* for programs, colours that know when to stay off, exit codes scripts can trust, tests that run the real binary, and a way to ship it. Each stage below adds one of those, and says which earlier lessons it draws on.

> 💡 **The big picture: a sorting office.** Letters (log lines) pour in from a sack or down a chute (files or a pipe). A machine reads the stamp on each letter (its level), drops it into the right pigeonhole and clicks a counter. At the end it prints a neat report for the manager (the table) or a punched card for another machine (JSON). The day's instructions come from the rule book, then the notice board, then the manager's sticky note (config file, environment, flags). The progress light only shines when someone is in the room (a terminal). And when the machine stops, it raises a flag: green, amber or red (the exit code).

## What you'll build

```text
$ cargo run -- data/app.log
logsum: 1 input, 29 lines (1.5 KiB), 2 without a level
time:   2026-05-01T09:00:00Z .. 2026-05-01T09:00:30Z

LEVEL  COUNT   SHARE
fatal      1    3.7%
error      5   18.5%
warn       8   29.6%
info      11   40.7%
debug      2    7.4%
trace      0    0.0%

TOP MESSAGES (warn and above)
COUNT  LEVEL  MESSAGE
    4  warn   [db] slow query took #ms
    3  error  payment # failed: card declined
    3  warn   retry # of # for mail.example.com
    1  fatal  out of memory, shutting down
    1  error  could not send email to user #
```

`payment 991 failed`, `payment 17 failed` and `payment 1204 failed` became one row, `payment # failed`, with a count of 3. That grouping is what makes the tool useful on a log with a million lines.

## How to run it

Run these **from this folder** (`05_mastery/10_capstone_cli_tool`), so `data/` is found:

```bash
cargo run -- data/app.log                                   # the table above
cargo run -- data/app.log --json                            # JSON for scripts
cargo run -- data/app.log -l error -n 3                     # errors and above, top 3
cat data/app.log | cargo run                                # standard input
cargo run -- data/app.log --config data/logsum.toml --show-config
cargo run -- data/app.log --fail-on error; echo $?          # exit code 1: errors found
cargo run -- --help
cargo test -p capstone_cli_tool                             # 68 tests, including the real binary
```

The binary is called `capstone_cli_tool` (after the package) but calls itself `logsum` in its help; [stage 12](#12-release-builds-and-distribution) explains how to ship it as `logsum`.

## The stages

| # | Stage | Draws on | Example |
|---|---|---|---|
| 1 | Project layout: a library and a thin `main` | 02/03 modules, 03/03 minigrep | |
| 2 | Arguments with clap's derive API | 02/01 structs, 05/02 macros | `stage1_clap_basics` |
| 3 | Errors: `thiserror` in the library, `anyhow` in the app | 02/05 errors, 05/04 API design | `stage2_errors` |
| 4 | Reading big inputs through `BufRead` | 03/03 buffered I/O, 03/04 closures | `stage3_bufread_big_input` |
| 5 | The core: parsing lines and counting | 02/04 collections, 03/01 lifetimes, 03/08 patterns | |
| 6 | Layered configuration: defaults, file, environment, flags | 05/07 serde, `Option` | `stage4_layered_config` |
| 7 | Output for people and programs: a table and `--json` | 05/07 serde, 03/01 traits | `stage7_json_output` |
| 8 | A good terminal citizen: `IsTerminal`, colours, progress | 03/01 traits, generics | `stage5_terminal_detection`, `stage6_progress_and_colors` |
| 9 | Exit codes, stderr and pipes | 03/03 stderr | `stage8_exit_codes_and_pipes` |
| 10 | Integration tests that run the real binary | 03/02 tests, 05/05 testing | `generate_log` |
| 11 | Documentation: `--help`, README, doc comments | 03/05 docs | |
| 12 | Release builds and distribution | 03/05 Cargo | |

```bash
cargo run -p capstone_cli_tool --example stage1_clap_basics      # and so on, see Practise
```

## 1. Project layout: a library and a thin `main`

```text
src/
  lib.rs       the library's front door: `pub mod` + `pub use`
  level.rs     Level enum: trace … fatal, FromStr, ordering by severity
  record.rs    parse_line: "timestamp LEVEL message" -> Record; normalize
  summary.rs   Summarizer: counts while lines stream past -> Summary
  config.rs    the four settings layers and ConfigError        (thiserror)
  input.rs     for_each_line: any BufRead, any size
  render.rs    write_table and write_json
  term.rs      ColorWhen, use_color, Palette (ANSI), Progress
  main.rs      thin: parse args, run, turn the result into an exit code
  cli.rs       the clap struct                                  (binary only)
  app.rs       glue: files, stdin, env, terminal               (anyhow, binary only)
tests/cli.rs   end-to-end tests of the real binary
data/          app.log (a sample log), logsum.toml (a sample config)
```

The rule is minigrep's, made stricter: **the library never touches the outside world.** It doesn't read `std::env`, doesn't ask whether stdout is a terminal, doesn't print, and doesn't know clap exists. Anything it needs from outside (the environment, the terminal answer, where to write) is passed in. That makes every piece testable with plain values, and it means a different front end (a web service, a GUI) could reuse it unchanged.

`main.rs`, `cli.rs` and `app.rs` are modules of the **binary** crate (`mod app; mod cli;` in `main.rs`), not of the library: they're the application layer.

> 💡 **Analogy: the machine and its control panel.** The sorting machine (the library) only knows about letters and pigeonholes. The control panel (the binary) has the buttons, the warning lights and the plug. You can test the machine on a workbench by feeding it letters by hand, and you can bolt a different control panel onto the same machine.

Draws on: [02/03 packages, crates, modules](../../02_core_types_and_errors/03_packages_crates_modules/README.md), [03/03 minigrep](../../03_abstraction/03_minigrep_cli_project/README.md) (the lib/main split).

## 2. Arguments with clap's derive API

minigrep parsed `env::args()` by hand. Real tools use [clap](https://docs.rs/clap): you describe the command line as a **struct**, and `#[derive(Parser)]` writes the parser, the `--help` text, `--version`, and the error messages. [Lesson 06/04](../../06_framework_mastery/04_clap/README.md) teaches clap in depth (subcommands, validation, shell completions); this capstone keeps to the straightforward parts.

```rust
/// Summarise log files: count the lines at each level and list the most
/// frequent warnings and errors.
#[derive(Debug, Parser)]
#[command(name = "logsum", bin_name = "logsum", version, after_help = AFTER_HELP)]
pub struct Cli {
    /// Log files to read; - means standard input [default: -]
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// How many top messages to list [default: 5]
    #[arg(short = 'n', long, value_name = "N")]
    pub top: Option<usize>,

    /// When to use colours: auto, always, never [default: auto]
    #[arg(long, value_name = "WHEN")]
    pub color: Option<ColorWhen>,

    /// Print JSON instead of a table
    #[arg(long)]
    pub json: bool,
    // … min_level, fail_on, config, show_config, quiet (src/cli.rs)
}
```

- **The field's type is the parsing rule.** `Vec<PathBuf>` with no `#[arg(long)]` is a positional list. `bool` is a flag. `Option<usize>` is an optional value that must parse as a number. Our own types work too: `Level` and `ColorWhen` implement `FromStr`, and clap uses it, so the library needs no clap code.
- **Doc comments become the help text.** The struct's comment is the description, and each field's comment is that option's line.
- **`Option`, not `default_value`, for settings.** With `default_value = "5"`, clap would fill in `top` even when the user didn't type `--top`, and the config file and environment ([stage 6](#6-layered-configuration-defaults-file-environment-flags)) could never apply. `None` means "not given". The help still shows `[default: 5]`, written in the doc comment.
- **`bin_name = "logsum"`.** Without it, clap names the program after the file it was started from, and the usage line says `capstone_cli_tool.exe`.
- **`Cli::parse()`** prints any error and exits with code **2**, or prints the help/version and exits with **0**. **`Cli::try_parse_from([...])`** returns a `Result` instead, which is how `src/cli.rs` tests the parser. `Cli::command().debug_assert()` catches definition mistakes, such as two options with the same short letter, in a test instead of at run time.

A bad value is reported before any of our code runs. Notice our own `ParseLevelError` message inside clap's:

```text
$ cargo run -q -- --min-level loud
error: invalid value 'loud' for '--min-level <LEVEL>': unknown level "loud" (expected trace, debug, info, warn, error or fatal)

For more information, try '--help'.
```

> 💡 **Analogy: an order form with labelled boxes.** Instead of reading a scribbled note, you hand customers a form: a box for files, a tick-box for "JSON please", a box for "how many". The form's own instructions are printed on it (`--help`), and the clerk (clap) sends back any form with "lots" in the number box before the kitchen sees it. Leaving a box empty (`None`) is different from writing the usual answer in it.

▶ `cargo run -p capstone_cli_tool --example stage1_clap_basics`

Draws on: [02/01 structs](../../02_core_types_and_errors/01_structs/README.md), derive macros ([05/02](../02_macros_in_depth/README.md)), `FromStr`.

## 3. Errors: `thiserror` in the library, `anyhow` in the app

Two kinds of code need two kinds of error ([02/05](../../02_core_types_and_errors/05_error_handling/README.md), [05/04](../04_api_design/README.md)):

| | Library (`config.rs`, `level.rs`…) | Application (`app.rs`, `main.rs`) |
|---|---|---|
| Who reads the error | other **code**, which may `match` on it | a **person**, in a terminal |
| What it needs | a precise type with variants | any error, plus "what were we doing?" |
| Crate | [`thiserror`](https://docs.rs/thiserror): derives `Display` and `Error` | [`anyhow`](https://docs.rs/anyhow): one `anyhow::Error` type, `.context()` |

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read config file {}", .path.display())]
    Read {
        path: PathBuf,
        #[source] // the io::Error underneath is kept, not flattened into text
        source: io::Error,
    },
    #[error("invalid config file {}", .path.display())]
    Parse { path: PathBuf, #[source] source: toml::de::Error },
    #[error("invalid value {value:?} in environment variable {var}: {reason}")]
    Env { var: &'static str, value: String, reason: String },
}
```

- `#[error("...")]` writes `Display`. `#[source]` makes the inner error available through `Error::source()`, so the **chain** survives.
- **`#[from]` vs building the variant yourself.** `#[from]` writes `impl From<io::Error> for ConfigError`, so `?` converts automatically, but then the error can't say *which* file. `ConfigError::Read` needs the path, so it's built by hand: `.map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })`. (thiserror 2 also accepts `{path}` for a `PathBuf` field; logsum spells out `.display()`.)
- In the app, `?` turns any `std::error::Error` into `anyhow::Error`, and `.with_context(|| format!("cannot open {}", path.display()))` adds a layer on top.
- `main` prints the chain itself: the outer message, then every `source()`:

```text
$ cargo run -q -- nope.log
logsum: error: cannot open nope.log
  caused by: The system cannot find the file specified. (os error 2)

$ logsum data/app.log        (with a logsum.toml that says top = "three")
logsum: error: invalid config file …\logsum_demo\logsum.toml
  caused by: TOML parse error at line 2, column 7
  |
2 | top = "three"
  |       ^^^^^^^
invalid type: string "three", expected usize
```

Forget the `From` impl and `?` can't convert. The real error from exercise 2:

```text
error[E0277]: `?` couldn't convert the error to `ConfigError`
  --> 05_mastery\10_capstone_cli_tool\exercises\ex02_fix_error_conversion.rs:34:40
   |
33 | fn load_top(path: &Path) -> Result<usize, ConfigError> {
   |                             -------------------------- expected `ConfigError` because of this
34 |     let text = fs::read_to_string(path)?;
   |                ------------------------^ the trait `From<std::io::Error>` is not implemented for `ConfigError`
   |                |
   |                this can't be annotated with `?` because it has type `Result<_, std::io::Error>`
```

> 💡 **Analogy: fault codes and the note to the customer.** The machine's display shows exact fault codes, "E12: paper jam in tray 2", because the engineer (calling code) needs to know precisely what to fix. The shop assistant (the app) turns that into a note for the customer that explains the whole story, outside in: "Your parcel is late, *because* the van broke down, *because* a tyre burst." Each "because" is one link in the chain.

▶ `cargo run -p capstone_cli_tool --example stage2_errors` (prints `{}`, `{:#}`, the chain, and `{:?}` side by side)

## 4. Reading big inputs through `BufRead`

A log can be 10 GB, or an endless stream from `tail -f`. `fs::read_to_string` would try to hold it all in memory. logsum reads **one line at a time**:

```rust
pub fn for_each_line<R: BufRead>(mut reader: R, mut on_line: impl FnMut(&str, u64)) -> io::Result<u64> {
    let mut buf = Vec::with_capacity(256);
    let mut total = 0u64;
    loop {
        buf.clear();                                  // reuse ONE buffer for every line
        let n = match reader.read_until(b'\n', &mut buf) {
            Ok(0) => return Ok(total),                // end of input
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        total += n as u64;
        let line = trim_line_ending(&buf);            // drop "\n" or "\r\n"
        on_line(&String::from_utf8_lossy(line), total);
    }
}
```

- **Why not `lines()`?** It allocates a new `String` for every line, and it returns an `Err` for any line that isn't valid UTF-8, so the usual `let line = line?;` stops the whole program at the first bad byte. Real logs often contain some. `read_until` into one reused `Vec<u8>`, then `String::from_utf8_lossy`, replaces bad bytes with `�` and carries on. (`from_utf8_lossy` returns a `Cow`: it only allocates when it had to replace something.)
- **`R: BufRead` accepts anything.** A `BufReader<File>` (logsum gives it a 64 KiB buffer instead of the default 8 KiB), `io::stdin().lock()`, or `&b"bytes"[..]` in a test.
- **Standard input.** No files, or a file named `-`, means stdin: `cat app.log | logsum`. If stdin is a terminal, nobody piped anything in, so logsum prints a hint on stderr instead of silently waiting.
- **The callback** gets the running byte count too, for the progress line in [stage 8](#8-a-good-terminal-citizen-isterminal-colours-progress).

The example writes an 11.5 MB log with one bad byte, then reads it three ways (real output; times vary):

```text
wrote 200001 lines, 11.5 MB
read_to_string: failed after 33.7971ms: stream did not contain valid UTF-8
lines():        200000 lines, 1 error(s), 127.1118ms
for_each_line:  200001 lines, 11506831 bytes, 40001 errors, 142.1758ms
  last error line: 2026-05-01T10:00:00Z ERROR bad byte � in a message
```

> 💡 **Analogy: a straw and one cup.** You don't pour a whole swimming pool into your mouth (`read_to_string`); you drink through a straw, a sip at a time (`BufRead`). And you keep using the **same cup** for every sip instead of fetching a new one each time (the reused buffer). If one sip tastes funny (a bad byte), you note it and keep drinking, instead of throwing the pool away.

▶ `cargo run -p capstone_cli_tool --example stage3_bufread_big_input`

Draws on: [03/03 buffered I/O](../../03_abstraction/03_minigrep_cli_project/README.md#beyond-the-book-buffered-io), [03/04 closures](../../03_abstraction/04_iterators_and_closures/README.md) (`FnMut`), generics with trait bounds ([03/01](../../03_abstraction/01_generics_traits_lifetimes/README.md)).

## 5. The core: parsing lines and counting

**Parsing** finds the level among the first four words, in any of the shapes real logs use:

```text
2026-05-01T09:00:00Z INFO server started             timestamp, INFO, message
2026-05-01 09:00:03 WARN [db] slow query             a two-word timestamp
[2026-05-01 09:00:05] [error] payment 991 failed     brackets, lowercase
ERROR: disk full                                     no timestamp, a colon
    at billing::charge (src/billing.rs:88)           no level: "unparsed"
```

```rust
pub struct Record<'a> {
    pub timestamp: Option<&'a str>,
    pub level: Level,
    pub message: &'a str,
}
pub fn parse_line(line: &str) -> Option<Record<'_>>
```

`Record` **borrows** from the line ([03/01 lifetimes](../../03_abstraction/01_generics_traits_lifetimes/README.md)), so parsing a million lines allocates nothing. `Level` derives `Ord`, and because the variants are declared from `Trace` to `Fatal`, `Level::Warn < Level::Error` compares by severity. It parses from any spelling real logs use (`WARN`, `warning`, `Err`, `CRITICAL`).

**Grouping** turns each message into a *pattern* by replacing every run of digits with `#`, so `slow query took 1532ms` and `slow query took 87ms` both become `slow query took #ms`.

**Counting** happens while the lines stream past. A `Summarizer` keeps only numbers and one example per pattern:

```rust
groups: HashMap<(Level, String), (u64, String)>,   // (level, pattern) -> (count, first example)
```

so memory depends on how many *different* patterns there are, not on the size of the log. `finish(top)` sorts by count, then severity, then text. The last two matter: `HashMap` iterates in a random order, and a tool whose output changes between identical runs is a tool nobody trusts.

> 💡 **Analogy: reading the stamp and the pigeonholes.** The machine glances at the first few marks on each envelope to find the stamp (the level). Then it files the letter by what it says with the details blurred out: "Lost dog called Max, 3 years old" and "Lost dog called Rex, 5 years old" both go in the "lost dog, # years old" pigeonhole, and a counter on each pigeonhole clicks. It keeps one example letter per pigeonhole and recycles the rest, so the sorting office never fills up.

Draws on: [02/04 collections](../../02_core_types_and_errors/04_collections/README.md) (`HashMap::entry`), [02/02 enums](../../02_core_types_and_errors/02_enums_and_pattern_matching/README.md), [03/08 patterns](../../03_abstraction/08_patterns_and_matching/README.md) (`let … else`), [03/02 tests](../../03_abstraction/02_automated_tests/README.md) (every function here has unit tests).

## 6. Layered configuration: defaults, file, environment, flags

Every setting can come from four places. Each layer overrides the ones before it, **setting by setting**:

| Priority | Layer | Example | Good for |
|---|---|---|---|
| 1 (lowest) | built-in default | `top = 5` | sensible behaviour with no set-up |
| 2 | config file | `logsum.toml`: `top = 3` | a team's or project's preferences, committed to git |
| 3 | environment | `LOGSUM_TOP=10` | one shell session, a CI job, a container |
| 4 (highest) | flag | `--top 7` | this one run |

Each layer is the same struct with every field optional:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]          // `min_levle = "warn"` is an error, not ignored
pub struct PartialSettings {
    pub min_level: Option<Level>,
    pub top: Option<usize>,
    pub color: Option<ColorWhen>,
}

pub fn resolve(layers: &[(Source, PartialSettings)]) -> Resolved   // later layers win
```

- **The file** is TOML, read with the `toml` crate and serde ([05/07](../07_serialization/README.md)). logsum looks for `--config <PATH>`, then `LOGSUM_CONFIG`, then `logsum.toml` in the current folder. A file you *named* must exist; the default file is only used if it's there. (Real tools often also look in the user's config folder; the [`directories`](https://docs.rs/directories) crate knows where that is on each platform.)
- **The environment** is read through a closure, `PartialSettings::from_env(|key| std::env::var(key).ok())`. Tests pass a closure over a `HashMap` instead, because changing the real environment is `unsafe` in the 2024 edition and races with other tests. Empty values count as unset.
- **The flags** are the clap struct's `Option` fields from [stage 2](#2-arguments-with-claps-derive-api).
- `resolve` also records **where** each value came from, so `--show-config` can explain it. When a user asks "why is it doing that?", this is the answer:

```text
$ LOGSUM_TOP=7 cargo run -q -- data/app.log --config data/logsum.toml --color always --show-config
config file: data/logsum.toml
min_level = error    from config file
top       = 7        from environment (LOGSUM_TOP)
color     = always   from flag
```

clap can read environment variables itself (`#[arg(env = "LOGSUM_TOP")]`, with its `env` feature), but then it can't slot the config file *between* the environment and the default. Doing the layering in our own code keeps the order explicit and testable.

> 💡 **Analogy: who decides what you wear to school.** The school has a rule (default: uniform). Your class has its own rule (the config file: "PE kit on Fridays"). Today there's a notice on the board (the environment: "it's raining, wellies allowed"). And the teacher tells you something right now (the flag: "take your coat off"). The most recent, most specific instruction wins, but only for the thing it talks about: the notice about wellies doesn't cancel the PE kit.

▶ `cargo run -p capstone_cli_tool --example stage4_layered_config`

## 7. Output for people and programs: a table and `--json`

**For people:** an aligned table. Every column is as wide as its widest value, big numbers get separators (`202,028`), and long messages are cut at 100 characters with `…` (counting characters, not bytes, so a letter is never cut in half). The renderers write to any `impl Write`, so tests render into a `Vec<u8>` and compare text.

**For programs:** `--json`. The `Summary` struct derives `Serialize`, so this is all it takes:

```rust
pub fn write_json(out: &mut impl Write, summary: &Summary) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, summary)?; // serde_json::Error converts into io::Error
    writeln!(out)
}
```

```text
$ cat data/app.log | cargo run -q -- --json --top 2
{
  "inputs": [
    "-"
  ],
  "lines": 29,
  "bytes": 1531,
  "unparsed": 2,
  "levels": {
    "trace": 0,
    "debug": 2,
    "info": 11,
    "warn": 8,
    "error": 5,
    "fatal": 1
  },
  "first_timestamp": "2026-05-01T09:00:00Z",
  "last_timestamp": "2026-05-01T09:00:30Z",
  "min_level": "warn",
  "top": [
    {
      "level": "warn",
      "count": 4,
      "pattern": "[db] slow query took #ms",
      "example": "[db] slow query took 1532ms"
    },
    …
```

- **The JSON is a promise.** Scripts will read `.levels.error`. Renaming a field breaks them, exactly like changing a `pub fn` ([05/04 API design](../04_api_design/README.md)). That's why `levels` is a struct (all six keys, always, in the same order) rather than a map that leaves out zeros.
- **Pretty-print for people** with `to_writer_pretty`; `to_writer` gives one compact line.
- **Write through one locked, buffered stdout:** `BufWriter::new(io::stdout().lock())`, then `flush()?`. A `BufWriter` that is just dropped flushes too, but it **ignores** any error, such as a full disk.

> 💡 **Analogy: a report for the manager and a punched card for a machine.** The manager wants a tidy page they can read at a glance, with the scary numbers in red. The next machine down the line wants a card with holes in exactly the same places every day. Move one hole and that machine jams, however nice the card looks.

▶ `cargo run -p capstone_cli_tool --example stage7_json_output`

## 8. A good terminal citizen: `IsTerminal`, colours, progress

The same program is used by a person at a terminal and by scripts, pipes and CI logs. `std::io::IsTerminal` tells them apart:

| Question | Asked of | Because |
|---|---|---|
| Colour the results? | **stdout** | that's where the results go: escape codes in a file or a `grep` would be junk |
| Draw a progress line? | **stderr** | that's where progress goes: `\r` redraws in a CI log would be junk |
| Print "waiting for input"? | **stdin** | a terminal on stdin means nobody piped anything in |

`is_terminal` is a **trait method**, so the trait must be in scope. Forget the `use` (exercise 1):

```text
error[E0599]: no method named `is_terminal` found for struct `Stdout` in the current scope
    --> 05_mastery\10_capstone_cli_tool\exercises\ex01_fix_is_terminal.rs:23:31
     |
  23 |     let colour = io::stdout().is_terminal() && !no_color;
     |                               ^^^^^^^^^^^ method not found in `Stdout`
…
     = help: items from traits can only be used if the trait is in scope
help: trait `IsTerminal` which provides `is_terminal` is implemented but not in scope; perhaps you want to import it
     |
  16 + use std::io::IsTerminal;
```

**The colour decision** is a pure function, so every combination is tested without a real terminal:

```rust
pub fn use_color(when: ColorWhen, is_terminal: bool, no_color: Option<&str>, term: Option<&str>) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => is_terminal && no_color.is_none_or(str::is_empty) && term != Some("dumb"),
    }
}
```

- [`NO_COLOR`](https://no-color.org) set to anything non-empty turns colour off; `TERM=dumb` too. `--color always` overrides both (for `logsum … | less -R`).
- **Colours are ANSI escape codes**: a few invisible bytes such as `ESC[31m` ("red from here") and `ESC[0m` ("back to normal"). `Palette` writes them by hand so you can see there's no magic. Real programs use [`anstream`](https://docs.rs/anstream) or [`owo-colors`](https://docs.rs/owo-colors); clap itself colours its help and errors this way. Windows Terminal and VS Code's terminal understand these codes; an older console may print them raw unless the program switches on "virtual terminal processing", which crates like `anstream` do for you.
- **Pad first, then paint.** `format!("{:<5}", painted)` counts the invisible escape bytes as characters, so the column comes out too short. `Palette::level(level, &format!("{level:<5}"))` puts the spaces inside the colour (a unit test in `term.rs` shows both).
- **The progress line** (`Progress<W: Write>`) writes `\r` (back to the start of the line), `ESC[2K` (clear the line) and the new text, at most every 100 ms, because printing is slow and a progress bar must not slow the work down. `finish()` clears it so the real output starts on a clean line. [`indicatif`](https://docs.rs/indicatif) is the crate for real progress bars.

To see the progress line, give logsum a big file at a real terminal:

```bash
cargo run -q -p capstone_cli_tool --example generate_log -- 2000000 > big.log
cargo run -q -p capstone_cli_tool -- big.log        # watch stderr, then delete big.log
```

> 💡 **Analogy: the light in the room.** The sorting machine only switches on its "42% done" light if someone is in the room to see it (stderr is a terminal), and it only uses coloured ink on reports going to a person (stdout is a terminal). It never puts crayon on a card that's going into another machine. And if the room has a "no colours please" sign on the door (`NO_COLOR`), it respects it.

▶ `cargo run -p capstone_cli_tool --example stage5_terminal_detection` (then again with `| cat`)
▶ `cargo run -p capstone_cli_tool --example stage6_progress_and_colors`

## 9. Exit codes, stderr and pipes

The exit code is how scripts and CI learn what happened. logsum follows the convention of `grep` and `diff`:

| Code | Meaning | Where it comes from |
|---|---|---|
| `0` | success | `ExitCode::SUCCESS` |
| `1` | a "finding": `--fail-on LEVEL` and such a line was found | `ExitCode::from(1)` |
| `2` | trouble: bad usage, unreadable file or config | clap's own code for usage errors, and ours for the rest |
| `101` | a panic (a bug) | Rust's default for a panicking `main` |

```rust
fn main() -> ExitCode {
    let cli = Cli::parse();                     // usage errors: exits with 2 by itself
    match app::run(cli) {
        Ok(Outcome::Clean) => ExitCode::SUCCESS,
        Ok(Outcome::ThresholdReached) => ExitCode::from(1),
        Err(err) if is_broken_pipe(&err) => ExitCode::SUCCESS,
        Err(err) => { /* print the chain to stderr */ ExitCode::from(2) }
    }
}
```

- **Return `ExitCode` from `main`** instead of calling `process::exit` deep inside the code: `process::exit` ends the process at once, without running destructors, so a `BufWriter` that hasn't been flushed loses its output.
- `ExitCode::from` takes a **`u8`**: 0 to 255 is the range that works the same everywhere (Unix keeps only the lowest byte of an exit code). Pass an `i32` and the compiler refuses: E0277, the trait bound `ExitCode: From<i32>` is not satisfied (the `help:` adds that `From<u8>` is implemented).
- **Results to stdout, everything else to stderr** (errors, the progress line, the "waiting for input" hint), so `logsum app.log > report.txt` puts only the report in the file.
- **Broken pipes.** In `logsum … | head -n 3`, `head` closes the pipe after three lines, and the next write fails with `BrokenPipe`. That's the reader saying "enough", not a failure. `println!` **panics** on it; the example shows both (real output, Git Bash):

```text
$ cargo run -q -p capstone_cli_tool --example stage8_exit_codes_and_pipes | head -n 3
line 1
line 2
line 3
the reader closed the pipe early; that's fine: exit code 0

$ cargo run -q -p capstone_cli_tool --example stage8_exit_codes_and_pipes -- --panic | head -n 3
line 1
line 2
line 3

thread 'main' (33776) panicked at …\library/std\src\io\stdio.rs:1209:9:
failed printing to stdout: The pipe is being closed. (os error 232)
```

In CI: `logsum test-output.log --fail-on error` makes the job fail when the tests logged an error, and still prints the summary for whoever looks at the job.

> 💡 **Analogy: the flag on the roof.** When the machine stops, it raises a flag that someone across the yard can see without reading any reports: green (0) "all fine", amber (1) "done, but I found what you told me to look for", red (2) "I couldn't do the job". If the person collecting the reports walks away early (a closed pipe), the machine just stops quietly; it doesn't set off the fire alarm.

▶ `cargo run -p capstone_cli_tool --example stage8_exit_codes_and_pipes | head -n 3`

Draws on: [03/03 minigrep](../../03_abstraction/03_minigrep_cli_project/README.md) (stderr, exit code 1).

## 10. Integration tests that run the real binary

Unit tests cover the library's pieces. `tests/cli.rs` runs the **finished program** as a separate process, the way a shell or CI would, and checks stdout, stderr and the exit code (the pattern from [minigrep's tests](../../03_abstraction/03_minigrep_cli_project/tests/cli.rs)):

```rust
fn logsum(dir: &PathBuf) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_capstone_cli_tool"));
    for var in ["LOGSUM_CONFIG", "LOGSUM_MIN_LEVEL", "LOGSUM_TOP", "LOGSUM_COLOR", "NO_COLOR"] {
        cmd.env_remove(var);            // the developer's own shell can't change the result
    }
    cmd.current_dir(dir).stdin(Stdio::null());   // an empty folder: no stray logsum.toml
    cmd
}
```

- **`env!("CARGO_BIN_EXE_<name>")`** is the path of the binary, which Cargo builds before running integration tests.
- **`env!("CARGO_TARGET_TMPDIR")`** is a scratch folder Cargo provides for integration tests: each test makes its own sub-folder there for config files.
- **Isolation:** remove the variables the program reads, and set new ones only on the **child** `Command` (never `env::set_var`). Run in an empty folder so a `logsum.toml` lying around can't interfere.
- **Pipe stdin** by spawning with `Stdio::piped()`, writing the bytes, and dropping the handle (end of input). That tests `cat file | logsum`, invalid UTF-8 and `\r\n` endings for real. `Stdio::null()` everywhere else means a test can never hang waiting for the keyboard.
- **Parse `--json` output with `serde_json::Value`** instead of comparing strings: the test then checks meaning, not spacing.
- Because the tests' stdout and stderr are pipes, they also prove the terminal rules: no colour codes, and nothing at all on stderr.

```text
$ cargo test -p capstone_cli_tool
     Running unittests src\lib.rs …        test result: ok. 40 passed
     Running unittests src\main.rs …       test result: ok. 5 passed
     Running tests\cli.rs …                test result: ok. 12 passed
   Doc-tests capstone_cli_tool             test result: ok. 11 passed
```

> 💡 **Analogy: the bench and the test track.** Each part of a car is checked on a workbench (unit tests): the brakes squeeze, the lights light. Then someone drives the finished car round a test track (integration tests) and checks it actually stops at the line. Both matter: a car made of perfect parts can still have the brake pedal wired to the horn.

Draws on: [03/02 automated tests](../../03_abstraction/02_automated_tests/README.md), [05/05 advanced testing](../05_advanced_testing/README.md).

## 11. Documentation: `--help`, README usage, doc comments

A CLI's documentation lives in three places, and the first is the one people actually read:

1. **`--help`**, generated from the doc comments. clap gives two sizes: `-h` shows the first line of each comment; `--help` shows them in full. `after_help` adds examples, the settings order and the exit codes, which are the questions people ask most:

```text
$ cargo run -q -- -h
Summarise log files: count the lines at each level and list the most frequent warnings and errors

Usage: logsum [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Log files to read; - means standard input [default: -]

Options:
  -l, --min-level <LEVEL>  Lowest level to list under top messages: trace, debug, info, warn, error, fatal [default: warn]
  -n, --top <N>            How many top messages to list [default: 5]
      --color <WHEN>       When to use colours: auto, always, never [default: auto]
      --json               Print JSON instead of a table
      --fail-on <LEVEL>    Exit with code 1 if any line is at LEVEL or above
      --config <PATH>      Read settings from this TOML file [env: LOGSUM_CONFIG] [default: ./logsum.toml if present]
      --show-config        Print the settings and where each one came from, then exit
  -q, --quiet              Never show the progress line
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version

Examples:
  logsum app.log                       summarise one file
  logsum app.log old.log --top 10      several files, ten top messages
  cat app.log | logsum                 read standard input
  logsum app.log --json                JSON for scripts (try: | jq .levels)
  logsum app.log --fail-on error       exit code 1 if there are errors (CI)

Settings, lowest priority first: built-in defaults, the config file
(logsum.toml), environment variables (LOGSUM_MIN_LEVEL, LOGSUM_TOP,
LOGSUM_COLOR), then flags. NO_COLOR=1 turns colours off.

Exit codes: 0 ok, 1 --fail-on level found, 2 error (usage, file or config).
```

2. **The README's usage section** (like "How to run it" above): install, the three commands people need first, and a sample of the output.
3. **Doc comments on the library** with `# Examples` and `# Errors` ([03/05](../../03_abstraction/05_cargo_and_crates_io/README.md)). The 11 doc tests run with `cargo test`, so the examples can't go stale. `cargo doc -p capstone_cli_tool --no-deps --open` shows them.

`--version` prints `logsum 0.1.0`, from `CARGO_PKG_VERSION`: bump the version in `Cargo.toml` and the binary follows. (For man pages and shell completions, see `clap_mangen` and `clap_complete` in 06/04.)

> 💡 **Analogy: the label on the machine.** Nobody reads the thick manual in the drawer before pressing a button; they read the label stuck on the front (`--help`). So the label says what each button does, gives a few examples, and explains what the coloured flags on the roof mean. The manual (README and API docs) is there for the details.

## 12. Release builds and distribution

> These steps are **explained, not run** in this course, to save disk space: a release build compiles every dependency again with optimisations, and `cargo install` builds in its own temporary folder.

**A release build:**

```bash
cargo build --release -p capstone_cli_tool
# -> target/release/capstone_cli_tool.exe (Windows) or target/release/capstone_cli_tool (Linux)
```

Profiles can make the shipped binary smaller or faster. In a workspace they go in the **root** `Cargo.toml` ([03/05](../../03_abstraction/05_cargo_and_crates_io/README.md#1-release-profiles) explains why), so this lesson only shows them:

```toml
[profile.release]
lto = true            # optimise across crates: smaller, faster, slower to build
codegen-units = 1     # one unit: better optimisation, slower to build
strip = true          # remove symbols: much smaller file, worse backtraces
panic = "abort"       # no unwinding: smaller, but no catch_unwind
```

**Installing it on your own machine:**

```bash
cargo install --path 05_mastery/10_capstone_cli_tool --locked   # builds --release, copies to ~/.cargo/bin
capstone_cli_tool data/app.log                                   # now on your PATH
cargo uninstall capstone_cli_tool
```

The installed command is named after the **binary target**, which defaults to the package name. To ship it as `logsum`, name the package `logsum`, or name the binary: `[[bin]] name = "logsum"`, `path = "src/main.rs"` (and update `default-run`, and `CARGO_BIN_EXE_logsum` in the tests).

**Sharing it:**

| Way | Users run | Needs |
|---|---|---|
| crates.io | `cargo install logsum` | a Rust toolchain on their machine; `cargo publish` from you ([03/05](../../03_abstraction/05_cargo_and_crates_io/README.md)) |
| prebuilt binaries | download from a GitHub Release, or run an installer script | you build once per platform (CI does it) |
| package managers | `winget`, `apt`, Homebrew, … | a package per manager, usually generated |

**[dist](https://axodotdev.github.io/cargo-dist/book/) (formerly cargo-dist)** automates the prebuilt-binaries row. You install it once (`cargo install cargo-dist --locked`; the package still has the old name, the command is `dist`), then:

```bash
dist init      # asks which platforms and installers you want; adds its settings to Cargo.toml
               # and writes .github/workflows/release.yml
dist plan      # shows what a release would build, without building
dist build     # builds this platform's archive locally, to check
git tag v0.1.0 && git push --tags     # CI builds every platform and creates a GitHub Release
```

The release then has an archive per platform (for example `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`) and, if you chose them, installers: a `curl … | sh` script for Linux, a PowerShell script for Windows, an MSI. The builds run in GitHub's CI, so you don't need every platform on your own desk. (Building for another platform yourself needs `rustup target add <target>` and often that platform's linker; a fully static Linux binary uses the `x86_64-unknown-linux-musl` target.)

> 💡 **Analogy: shipping the machine.** `cargo build --release` is building the machine properly, with the factory tools, instead of the quick prototype. `cargo install` is wheeling it into your own office. `cargo publish` is putting the blueprints in the town library so anyone with a workshop can build their own. `dist` is a delivery company: you ring the bell (push a tag), and it builds a ready-made machine for every kind of office (Windows, Linux), packs each in a crate with instructions, and leaves them at the depot (a GitHub Release).

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `#[arg(long, default_value = "5")] top: usize` | the config file and env can never change `top` | `top: Option<usize>`, and default it when resolving |
| `io::stdout().is_terminal()` without the import | E0599 no method named `is_terminal` | `use std::io::IsTerminal;` |
| `?` on an `io::Error` in a fn returning your error | E0277 `?` couldn't convert the error | `#[from]`, or `.map_err(...)` to add context such as the path |
| `ExitCode::from(code)` with `code: i32` | E0277 `ExitCode: From<i32>` is not satisfied | Exit codes are `u8`: `ExitCode::from(2u8)`. |
| `process::exit(1)` after writing through a `BufWriter` | output lost: no destructors, no flush | Return an `ExitCode` from `main`; `flush()?` first. |
| `println!` in a tool people pipe into `head` | panic: failed printing to stdout | `writeln!` + `?`, and treat `BrokenPipe` as success. |
| `for line in reader.lines() { let line = line?; … }` on logs | stops at the first non-UTF-8 byte | `read_until` + `String::from_utf8_lossy` |
| `fs::read_to_string` on a log of unknown size | memory grows with the file | Stream it through `BufRead`. |
| Colour decided by `stderr().is_terminal()` | escape codes in `logsum … > report.txt` | Ask the stream you're writing to: stdout for results. |
| `format!("{:<5}", painted)` | misaligned table columns | Pad first, then paint. |
| Ignoring `NO_COLOR` | users who asked for no colour get it anyway | Treat a non-empty `NO_COLOR` as `--color never` (unless `always`). |
| Errors printed with `println!` | errors end up in the report file | `eprintln!`, and a non-zero exit code. |
| Iterating a `HashMap` straight into the output | the order changes between runs | Sort, with a tie-breaker. |
| `#[serde(...)]` struct without `deny_unknown_fields` for config | `min_levle = "warn"` is silently ignored | Add `deny_unknown_fields`. |
| `env::set_var` in a test | `unsafe` in edition 2024, and tests race | `Command::env` on the child, or inject a `get` closure. |
| An integration test run from the repo root | a stray `logsum.toml` or `LOGSUM_TOP` changes results | `current_dir(empty_dir)` and `env_remove`. |
| Renaming a JSON field "to tidy up" | every script reading it breaks | Treat the JSON shape like a public API: semver major. |

## Practise

**1. Run the examples**, in order (from anywhere in the workspace):

```bash
cargo run -p capstone_cli_tool --example stage1_clap_basics
cargo run -p capstone_cli_tool --example stage1_clap_basics -- app.log -n 3 --json
cargo run -p capstone_cli_tool --example stage2_errors
cargo run -p capstone_cli_tool --example stage3_bufread_big_input
cargo run -p capstone_cli_tool --example stage4_layered_config
cargo run -p capstone_cli_tool --example stage5_terminal_detection
cargo run -p capstone_cli_tool --example stage5_terminal_detection | cat
cargo run -p capstone_cli_tool --example stage6_progress_and_colors
cargo run -p capstone_cli_tool --example stage7_json_output
cargo run -p capstone_cli_tool --example stage8_exit_codes_and_pipes | head -n 3
cargo run -p capstone_cli_tool --example generate_log -- 20
cargo run -q -p capstone_cli_tool --example generate_log -- 200000 | cargo run -q -p capstone_cli_tool
```

**2. Do the exercises** in `exercises/`. Numbers 3 to 6 extend logsum, reusing its library:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_is_terminal` | Make it compile (E0599): a trait in scope | `cargo run -p capstone_cli_tool --features exercises --bin ex01_fix_is_terminal` |
| 2 | `ex02_fix_error_conversion` | Make it compile (E0277) with `#[from]` | `cargo run -p capstone_cli_tool --features exercises --bin ex02_fix_error_conversion` |
| 3 | `ex03_csv_output` | A CSV output format with proper quoting, pass 3 tests | `cargo test -p capstone_cli_tool --features exercises --bin ex03_csv_output` |
| 4 | `ex04_json_lines_input` | Read JSON-lines logs with serde aliases, pass 4 tests | `cargo test -p capstone_cli_tool --features exercises --bin ex04_json_lines_input` |
| 5 | `ex05_setting_precedence` | A new setting through all four layers, pass 4 tests | `cargo test -p capstone_cli_tool --features exercises --bin ex05_setting_precedence` |
| 6 | `ex06_better_normalize` | Group UUIDs, hex and quoted text too, pass 4 tests | `cargo test -p capstone_cli_tool --features exercises --bin ex06_better_normalize` |

For exercise 1, also run it piped (`| cat`) and compare the answers.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p capstone_cli_tool --features solutions --bins`, and `cargo run -p capstone_cli_tool --features solutions --bin sol01_fix_is_terminal`, etc.).

**4. The project.** Read `src/lib.rs`, then the modules in the stage order above, then `src/main.rs` → `src/app.rs`. Then make it yours:
- Wire exercise 3 into the real tool as `--format table|json|csv` (a `ValueEnum` is the clap way; see 06/04), and add an integration test.
- Wire exercise 4 in: detect lines that start with `{` and parse them as JSON.
- Add `--since <TIMESTAMP>` to ignore older lines (ISO 8601 timestamps sort correctly as plain strings).
- Run `logsum` over a real log from your machine, and see which pattern tops the list.

## Checklist

- [ ] The library/binary split: no I/O, environment or terminal access in the library
- [ ] Arguments with clap derive: positional, flags, `Option` values, `FromStr` types, `try_parse_from` in tests
- [ ] `thiserror` for library errors (`#[source]`, `#[from]`), `anyhow` and `.context()` in the app, printing the chain
- [ ] Reading any size of input through `BufRead`, from files and stdin, surviving bad UTF-8
- [ ] Layered settings: defaults < config file < environment < flags, with sources reported
- [ ] Output for people (aligned table) and for programs (`--json` with serde)
- [ ] `IsTerminal`, `NO_COLOR`, `--color`, and a progress line on stderr
- [ ] Exit codes 0/1/2, results on stdout and the rest on stderr, broken pipes handled
- [ ] Integration tests with `CARGO_BIN_EXE_<name>`, isolated environment and folder, piped stdin
- [ ] `--help` with examples and exit codes; doc comments with tested examples
- [ ] Release profiles, `cargo install --path`, crates.io and `dist` explained
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A production CLI is mostly the parts around the main feature: arguments, configuration, errors, input, output, terminal manners, exit codes, tests, docs and shipping.
- Keep the **library** pure and precise (`thiserror`, values passed in) and the **application** thin and forgiving (`clap`, `anyhow`, the real environment).
- Settings are layers of `Option`s: default < file < environment < flag, and say where each value came from.
- Stream input with `BufRead` and one reused buffer; write output through a locked `BufWriter`, as a table for people or JSON for programs.
- Colour and progress only when a person is watching the right stream, and never against `NO_COLOR`.
- Exit with 0, 1 or 2 from `main`, keep errors on stderr, treat a closed pipe as success, and test the real binary as a separate process.

**In one sentence for a child:** logsum is a sorting machine for messages that counts them into pigeonholes, prints a tidy report for people or a punched card for other machines, only switches its lights on when someone's watching, and raises a green, amber or red flag when it's done.

**Next:** [11 Capstone: an Interpreter](../11_capstone_interpreter/README.md)
