<!-- hand-written lesson: gen_course must not overwrite -->
# Clap

**Docs:** [clap on docs.rs](https://docs.rs/clap/4) · [derive tutorial](https://docs.rs/clap/4/clap/_derive/_tutorial/index.html) · [derive reference](https://docs.rs/clap/4/clap/_derive/index.html) · [cookbook](https://docs.rs/clap/4/clap/_cookbook/index.html)
· Completions: [`clap_complete`](https://docs.rs/clap_complete/4)
· Background: [Command Line Interface Guidelines](https://clig.dev/), [Rust CLI book](https://rust-cli.github.io/book/)

Run it: `cargo run -p clap_cli -- --help`  ·  Test it: `cargo test -p clap_cli`

In [03/03 minigrep](../../03_abstraction/03_minigrep_cli_project/README.md) you parsed `std::env::args()` by hand: count the arguments, pick them out by position, write the usage message yourself. That's fine for two arguments and painful for twenty. **Clap** (Command Line Argument Parser) turns a struct into a full command line: typed values, defaults, environment variables, validation, subcommands, `--help`, `--version`, friendly errors with the right exit code, and shell completions. After this lesson you can:

- describe a command line with the derive API (`Parser`, `Args`, `Subcommand`, `ValueEnum`) and know what each attribute does;
- use positional arguments, options and flags, defaults, environment variables, and validated custom values;
- build a git-style tool with (nested) subcommands, good help text and correct exit codes;
- generate shell completions, and test a CLI both in-process and as a real program.

This lesson uses clap 4.6 (4.6.7 in `Cargo.lock`) and clap_complete 4.6.

> 💡 **The big picture: the order form at a sandwich shop.** Shouting your order at a busy counter goes wrong ("did you say ham or jam?"). So the shop has an **order form** (your `#[derive(Parser)]` struct): a **box** for each thing (a field), some boxes **required** (the bread), some **optional** with a usual choice already printed in (defaults), **tick boxes** (flags), and lists where you **circle one** (`ValueEnum`). Different services have **their own pages**: sandwiches, drinks, catering (subcommands). A **clerk** checks the form before the kitchen sees it and points at mistakes ("there is no bread called 'brick'; choose white, brown or rye"): those are clap's errors. A **leaflet on the wall** explains the form (`--help`), regulars can have **"the usual" on file** (environment variables), and a helpful clerk **finishes your words** as you start them (shell completions).

## 1. From `std::env::args` to `#[derive(Parser)]`

```rust
use clap::Parser;

/// Greets someone, as many times as you like       <- the doc comment is the "about" text
#[derive(Parser, Debug)]
#[command(name = "greet", version, about)]
struct Greet {
    /// Who to greet
    name: String,                                   // positional, required

    /// How many times
    #[arg(short, long, default_value_t = 1)]
    times: u8,                                      // an option: -t 3, --times 3, --times=3

    /// Use capital letters
    #[arg(short, long)]
    shout: bool,                                    // a flag: present = true

    /// Sign the greeting
    #[arg(long)]
    from: Option<String>,                           // optional

    /// Also greet these people
    #[arg(long = "and", value_name = "NAME")]
    others: Vec<String>,                            // repeatable: --and Bo --and Cy
}

fn main() {
    let greet = Greet::parse();   // parse std::env::args(); on bad input print the error and exit
}
```

The **field's type decides the kind of argument**: a plain type is required, `Option<T>` optional, `bool` a flag, `Vec<T>` repeatable; `#[arg(short, long)]` turns a positional into an option named after the field (`log_level` becomes `--log-level`). The **doc comments become the help**. Real output of the example:

```text
$ greet Ada --times 2 -s
  Greet { name: "Ada", times: 2, shout: true, from: None, others: [] }
    HELLO, ADA!
    HELLO, ADA!
$ greet --times two Ada
error: invalid value 'two' for '--times <TIMES>': invalid digit found in string

For more information, try '--help'.
$ greet --help
Greets someone, as many times as you like

Usage: greet [OPTIONS] <NAME>

Arguments:
  <NAME>  Who to greet

Options:
  -t, --times <TIMES>  How many times [default: 1]
  -s, --shout          Use capital letters
      --from <FROM>    Sign the greeting
      --and <NAME>     Also greet these people
  -h, --help           Print help
  -V, --version        Print version
```

`parse` is a method of the **`Parser` trait** that the derive implements, so the trait must be in scope to call it. The real error from exercise 1, where the derive is written as `#[derive(clap::Parser)]` and nothing imports the trait:

```text
error[E0599]: no associated function or constant named `parse` found for struct `Cli` in the current scope
  --> 06_framework_mastery\04_clap\exercises\ex01_fix_parser_not_in_scope.rs:28:20
   |
18 | struct Cli {
   | ---------- associated function or constant `parse` not found for this struct
...
28 |     let cli = Cli::parse();
   |                    ^^^^^ associated function or constant not found in `Cli`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `Parser` which provides `parse` is implemented but not in scope; perhaps you want to import it
   |
15 + use clap::Parser;
```

The four derives: **`Parser`** for the top-level struct (it can `parse`), **`Args`** for a group of arguments used inside another struct (`#[command(flatten)]`), **`Subcommand`** for an enum of subcommands, **`ValueEnum`** for an enum of allowed values.

> 💡 **Analogy: printing the form.** You design the order form once (the struct), and clap prints it, reads what the customer filled in, and hands you a neat, checked copy (the parsed struct). A plain box must be filled in; a box marked "optional" may stay empty; a tick box is ticked or not; a box with "and more..." lines takes several answers. The notes beside each box become the leaflet on the wall (`--help`).

▶ `cargo run -p clap_cli --example derive_basics` (or `-- Ada --times 2 --shout`)

## 2. Arguments, options and flags

| Attribute | Gives | Example |
|---|---|---|
| (none) | positional argument | `copy a.txt b.txt` |
| `#[arg(short, long)]` | option or flag with `-x` and `--name` | `-o out.txt`, `--output out.txt` |
| `#[arg(short = 'n', long = "lines")]` | choose the names yourself | `-n 5` |
| `#[arg(action = ArgAction::Count)]` on a `u8` | counts repeats | `-vvv` is 3 |
| `#[arg(value_delimiter = ',')]` on a `Vec` | splits one value | `--skip tmp,bak` |
| `#[arg(required = true)]` on a `Vec` | at least one value | |
| `#[arg(conflicts_with = "verbose")]` | can't be used together | `-v -q` is an error |
| `#[arg(requires = "log_level")]` | needs another argument too | `--log` needs `--log-level` |
| `#[group(required = true, multiple = false)]` on an `Args` struct | exactly one of its fields | `--to DIR` **or** `--clipboard` |

Real output:

```text
$ copy a.txt -vvv --skip tmp,bak --skip log --clipboard
  Copy { files: ["a.txt"], verbose: 3, quiet: false, skip: ["tmp", "bak", "log"], log: None, log_level: None, target: Target { to: None, clipboard: true } }
$ copy a.txt -v -q --clipboard
error: the argument '--verbose...' cannot be used with '--quiet'
$ copy a.txt
error: the following required arguments were not provided:
  <--to <TO>|--clipboard>
$ copy a.txt --to x --log copy.log
error: the following required arguments were not provided:
  --log-level <LOG_LEVEL>
```

> 💡 **Analogy: kinds of boxes.** A **positional** argument is a box everyone fills in the same place ("first line: your name"). An **option** is a labelled box ("Drink: ____"). A **flag** is a tick box. **Count** is a row of tick boxes for "extra cheese": tick it three times for triple. A **conflict** is "you can't tick both *no sauce* and *extra sauce*", **requires** is "if you tick *delivery*, fill in the address", and a **group** is "circle exactly one: eat in / take away".

▶ `cargo run -p clap_cli --example args_options_flags`

## 3. Defaults and environment variables

```rust
#[arg(long, env = "DEMO_PORT", default_value_t = 8080)]
port: u16,
#[arg(long, env = "DEMO_THEME", default_value = "light")]
theme: String,
```

A value comes from the **command line**, else the **environment variable**, else the **default** (the `env` attribute needs clap's `env` feature). Real output (the example runs itself as a child process with different variables):

```text
$ server
  -> port=8080 theme=light debug=false
$ DEMO_PORT=3000 DEMO_THEME=dark server
  -> port=3000 theme=dark debug=false
$ DEMO_PORT=3000 server --port 9000
  -> port=9000 theme=light debug=false
$ DEMO_DEBUG=true server
  -> port=8080 theme=light debug=true
$ DEMO_PORT=lots server
error: invalid value 'lots' for '--port <PORT>': invalid digit found in string
  (exit code 2)
```

Values from the environment are **validated** exactly like typed ones, and `--help` shows the variable: `[env: TASKS_STORE=]`.

**Two kinds of default.** `default_value = "..."` is **text**, parsed as if the user had typed it. `default_value_t = expr` is a **value of the field's type** (shown in the help through `Display`). Mixing them up is a type error. The real error from exercise 3:

```text
error[E0308]: mismatched types
  --> 06_framework_mastery\04_clap\exercises\ex03_fix_default_value_type.rs:25:35
   |
25 |     #[arg(long, default_value_t = "localhost")]
   |                                   ^^^^^^^^^^^ expected `String`, found `&str`
26 |     host: String,
   |           ------ expected due to this
```

**Testing environment variables:** don't call `std::env::set_var` in a test. In Rust 2024 it is `unsafe`, because another thread (tests run in parallel) may read the environment at the same moment. Set variables on a **child process** instead: `Command::new(exe).env("TASKS_STORE", path)`. That's what the example and `tests/cli.rs` do.

> 💡 **Analogy: "the usual".** If you write your order on the form, that's what you get. If you leave the box empty, the shop checks whether you have **"the usual" on file** (the environment variable). If not, it gives you **the house choice** printed on the form (the default). And "the usual" is checked like any order: if the file says "a brick sandwich", you still get told off.

▶ `cargo run -p clap_cli --example env_and_defaults`

## 4. Validation and value parsers

Every argument has a **value parser** that turns text into the field's type: numbers, `bool`, `String`, `PathBuf` and any `FromStr` type work out of the box. Three ways to check more:

```rust
#[derive(Clone, ValueEnum)]            // allowed values: fast, normal, best
enum Level { Fast, Normal, #[value(alias = "max")] Best }

#[arg(long, value_parser = clap::value_parser!(u8).range(1..=16))]   // a numeric range
jobs: u8,

#[arg(long, value_parser = parse_duration)]   // your own fn(&str) -> Result<T, E>
every: Option<Duration>,
```

Real output:

```text
$ backup --level max --jobs 8 --every 15m --notify slack
  Backup { level: Best, jobs: 8, every: Some(900s), notify: "slack" }
$ backup --level turbo
error: invalid value 'turbo' for '--level <LEVEL>'
  [possible values: fast, normal, best]
$ backup --jobs 64
error: invalid value '64' for '--jobs <JOBS>': 64 is not in 1..=16
$ backup --every 2d
error: invalid value '2d' for '--every <EVERY>': unknown unit "d": use s, m or h
```

- **`ValueEnum`** variants become lower-case values (kebab-case for longer names: `DryRun` is `dry-run`); doc comments on the variants describe them in `--help`.
- A custom parser's error text appears after clap's "invalid value ..." part. Validate as early as possible: a wrong value is then a usage error (exit code 2) with the argument's name in the message, not a confusing failure later.
- Forget `ValueEnum` on an enum field and clap has no way to parse it. The real errors from exercise 2 (abridged; the second is one of clap's least friendly):

```text
error[E0277]: the trait bound `Format: Clone` is not satisfied
   --> 06_framework_mastery\04_clap\exercises\ex02_fix_missing_value_enum.rs:33:13
    |
 33 |     format: Format,
    |             ^^^^^^ the trait `Clone` is not implemented for `Format`

error[E0599]: the method `value_parser` exists for reference `&&&&&&clap::builder::_infer_ValueParser_for<Format>`, but its trait bounds were not satisfied
    --> 06_framework_mastery\04_clap\exercises\ex02_fix_missing_value_enum.rs:31:5
     |
     = note: the following trait bounds were not satisfied:
             `Format: ValueEnum`
             ...
             `Format: ValueParserFactory`
```

  Read the `note:` list: `Format: ValueEnum` is the fix. (Clap tries several ways to parse a type in turn, hence all the `&&&&&&`.)

> 💡 **Analogy: the clerk with a checklist.** "Circle one: white, brown, rye" (`ValueEnum`): anything else is sent back with the list of choices. "Number of sandwiches, 1 to 16" (a range). And a special check you wrote yourself, "pickup time must look like 15m or 2h" (a custom parser). Mistakes are caught at the counter, with a note saying exactly which box was wrong, before the kitchen starts cooking.

▶ `cargo run -p clap_cli --example value_parsers`

## 5. Subcommands

```rust
#[derive(Parser)]
struct Pkg {
    #[arg(long, global = true)]           // allowed before or after the subcommand
    dry_run: bool,
    #[command(subcommand)]
    command: Option<Cmd>,                 // Option: `pkg` alone is allowed
}

#[derive(Subcommand)]
enum Cmd {
    #[command(visible_alias = "i")]
    Install { #[arg(required = true)] names: Vec<String>, #[command(flatten)] net: NetworkOptions },
    Search(SearchArgs),                   // a tuple variant wrapping an `Args` struct
    #[command(subcommand)]
    Source(SourceCmd),                    // nested: `pkg source add ...`
    Version,                              // no arguments
}
```

Real output:

```text
$ pkg
  -> no subcommand: show a summary (the default action)
$ pkg i clap --dry-run
  -> [dry run] install ["clap"] (timeout 30s, offline false)
$ pkg source add work https://example.com/pkgs
  -> add source work = https://example.com/pkgs
$ pkg instal serde
error: unrecognized subcommand 'instal'

  tip: some similar subcommands exist: 'i', 'install'
```

- Variant names become subcommand names (`Install` is `install`); the doc comment is its help line; `#[command(subcommand)]` on a field or variant says "a subcommand goes here".
- **`#[command(flatten)]`** reuses an `Args` struct in several places (shared network options); **`global = true`** makes an option work at every level; **aliases** give short names.
- A missing required subcommand (`pkg source`) prints that subcommand's help with exit code 2.
- Handle the parsed result with one `match`: each variant gives you its fields by name (exercise 5).

> 💡 **Analogy: pages of the form.** The front page asks "which service?": sandwiches, drinks or catering. Each service has **its own page** of boxes, and catering even has sub-pages (nested subcommands). Some boxes appear on every page, like "your name" (`global`), and some pages share a printed block, like the delivery address (`flatten`). Ask for "sandwhich" and the clerk says "did you mean sandwich?".

▶ `cargo run -p clap_cli --example subcommands`

## 6. Help and version text

| Source | Becomes |
|---|---|
| First paragraph of a doc comment | the short help: `-h`, and `about` |
| The rest of the doc comment | the long help: only `--help` (`long_about`, `long_help`) |
| `#[command(version)]` | `-V`/`--version` from `Cargo.toml`'s version (or `version = "2.1.0"`) |
| `#[arg(help_heading = "Size")]` | a section of its own in the help |
| `#[arg(hide = true)]` | accepted, but not shown |
| `#[command(after_help = "...")]` | text after the options, e.g. examples |
| `#[command(bin_name = "clap_cli")]` | the name in `Usage:` (otherwise from how it was started: `clap_cli.exe` on Windows) |

Real output: `-h` gives the summary and says `Print help (see more with '--help')`; `--help` adds the extra paragraphs and puts each description on its own line:

```text
$ resize -h
Resize images

Usage: resize [OPTIONS] <INPUT>
...
Size:
      --width <WIDTH>    New width in pixels
      --height <HEIGHT>  New height in pixels

Example: resize photo.jpg --width 800

$ resize --help
Resize images.

Everything after the first paragraph of the doc comment is the LONG about: shown by --help, but not by -h. Use it for details and caveats.
...
      --width <WIDTH>
          New width in pixels.

          The height follows, keeping the proportions, unless you give --height as well. (This paragraph only appears in --help.)

$ resize --version
resize 2.1.0
```

> 💡 **Analogy: the leaflet.** `-h` is the **small card** by the till: one line per box. `--help` is the **full leaflet** with the small print. The version is the **date printed on the menu**, and hidden options are the **secret menu**: it works if you know to ask, but it isn't printed.

▶ `cargo run -p clap_cli --example help_and_version`

## 7. Error messages and exit codes

`Cli::parse()` prints the message and **exits the process**; `Cli::try_parse_from(args)` returns `Result<Cli, clap::Error>` so you can look first. Some "errors" are really requests (real output):

```text
convert a.png                   Ok                          exit code 0 (your program decides)
convert --help                  DisplayHelp                 exit code 0, printed to stdout
convert --version               DisplayVersion              exit code 0, printed to stdout
convert                         MissingRequiredArgument     exit code 2, printed to stderr
convert a.png --quality 300     ValueValidation             exit code 2, printed to stderr
convert a.png --colour          UnknownArgument             exit code 2, printed to stderr
```

The convention this lesson follows (`src/main.rs`):

| Exit code | Meaning | Who decides |
|---|---|---|
| 0 | it worked; also `--help` and `--version` | you / clap |
| 1 | the command failed: no such task, unreadable file | you: `ExitCode::FAILURE` |
| 2 | the arguments were wrong | clap |

- Results go to **stdout**, messages to **stderr**, so `clap_cli list -f json > tasks.json` captures only the JSON. Exercise 6 implements exactly this routing.
- For rules clap can't express (the output file must differ from the input), raise a clap-style error after parsing: `Cli::command().error(ErrorKind::ArgumentConflict, "...").exit()`.
- `e.kind()`, `e.exit_code()`, `e.use_stderr()` and `e.render()` let tests and wrappers inspect an error.

> 💡 **Analogy: three kinds of reply.** "Here's your sandwich" (0). "Sorry, we've run out of rye" (1: the order was fine, the kitchen couldn't do it). "This form isn't filled in properly" (2: the order itself was wrong). Asking "what's on the menu?" isn't a mistake at all, so the leaflet comes back with a smile (help is exit 0). Sandwiches go **on the tray** (stdout); apologies are **said out loud** (stderr), so they never end up in the sandwich bag.

▶ `cargo run -p clap_cli --example errors_and_exit_codes`

## 8. Shell completions

Clap knows every subcommand, option and allowed value, so `clap_complete` can write the script that makes Tab complete them:

```rust
Command::Completions { shell } => {
    clap_complete::generate(shell, &mut Cli::command(), "clap_cli", &mut io::stdout());
}
```

Real output of the example (lines per shell for the mini project's CLI):

```text
bash         561 lines
zsh          471 lines
fish         128 lines
powershell   252 lines
elvish       221 lines
```

`clap_complete::Shell` is itself a `ValueEnum`, so `completions <SHELL>` validates the shell name for free. To install one: `source <(clap_cli completions bash)` in `~/.bashrc`, or `clap_cli completions powershell | Out-String | Invoke-Expression` in your PowerShell `$PROFILE`.

> 💡 **Analogy: the helpful clerk.** You start saying "sand..." and the clerk says "sandwich?". Completions are that clerk living inside your terminal: press Tab and it offers what the form allows. Clap writes the clerk's script because it already knows the whole form.

▶ `cargo run -p clap_cli --example completions`

## 9. Testing command-line programs

Three layers, all used in this lesson:

1. **Parse from a list.** `Cli::try_parse_from(["clap_cli", "add", "x", "-p", "urgent"])` needs no process and no real arguments; check the parsed struct, or `unwrap_err().kind()` (`ErrorKind::InvalidValue`, `ValueValidation`, `MissingRequiredArgument`, `ArgumentConflict`...). Add one test calling **`Cli::command().debug_assert()`**: it catches definition mistakes that clap otherwise only finds at run time (two arguments with the same `-v`, a default that isn't an allowed value).
2. **Run the logic in-process.** `run(cli, out: &mut dyn Write, err: &mut dyn Write)` writes to any `Write`; tests pass a `Vec<u8>` and compare text (the minigrep trick from 03/03).
3. **Run the real binary.** Integration tests get its path from **`env!("CARGO_BIN_EXE_clap_cli")`** (Cargo builds it first), run it with `std::process::Command`, and check stdout, stderr and the exit code. Give each test its own folder with `tempfile::TempDir`, and set or remove environment variables **on the child** (`.env(..)`, `.env_remove(..)`), so the developer's own `TASKS_STORE` can't change the results.

> 💡 **Analogy: practising the shop.** First you check the **form** itself, filling it in on paper (`try_parse_from`). Then you check the **kitchen** with pretend orders handed straight through the hatch (`run` with a `Vec<u8>`). Finally a **mystery shopper** walks in from the street and orders like a real customer (the binary in a separate process).

## 10. The builder API, briefly

The derive API generates **builder** code. You can write it directly:

```rust
let matches = Command::new("greet")
    .arg(Arg::new("name").required(true).help("Who to greet"))
    .arg(Arg::new("times").short('t').long("times").value_parser(value_parser!(u8)).default_value("1"))
    .arg(Arg::new("shout").short('s').long("shout").action(ArgAction::SetTrue))
    .get_matches();
let name = matches.get_one::<String>("name").unwrap();
let times = *matches.get_one::<u8>("times").unwrap();
let shout = matches.get_flag("shout");
```

Values come back **by name and type**, so a typo in `"tiems"` or asking for a `u32` instead of a `u8` is a panic at run time, not a compile error. Prefer derive; reach for the builder when the arguments aren't known until run time (plugins, a config file) or to adjust a derived command (`Cli::command().mut_arg("store", |a| a.hide(true))`).

> 💡 **Analogy:** the derive API is a **printed form**; the builder is **writing the form by hand** with a pen. Handy when today's form depends on what's in stock, but nothing stops you from misspelling a box.

▶ `cargo run -p clap_cli --example builder_api`

## The mini project: `clap_cli`, a git-style task tracker

| File | What's in it |
|---|---|
| [`src/cli.rs`](src/cli.rs) | `Cli`: global `--store` (with `env = "TASKS_STORE"` and a default) and `-v` (counted); `Command`: `add`, `list`/`ls`, `done`, `remove`/`rm` (an argument group: ids **or** `--done`), `tag add/remove` (nested), `completions <SHELL>`; `Priority`, `Status`, `Format` (`ValueEnum`s); `parse_tag` and `parse_date` (custom parsers, leap years included) |
| [`src/store.rs`](src/store.rs) | `Task`, `Store` (JSON with serde, saved through a temporary file and a rename), `AppError` |
| [`src/lib.rs`](src/lib.rs) | `run(cli, out, err)`: one `match` over the subcommands; table, plain and JSON output |
| [`src/main.rs`](src/main.rs) | parse, run, exit code 0/1 (clap does 2) |
| [`tests/cli.rs`](tests/cli.rs) | 8 tests that run the real binary |

A real session (in an empty folder, so the default `tasks.json` is used):

```text
$ clap_cli add Buy milk -p high -t home -d 2026-11-20
added task 1: Buy milk
$ clap_cli add Call Ada -t work
added task 2: Call Ada
$ clap_cli add Water the plants
added task 3: Water the plants
$ clap_cli done 3
done: 3
$ clap_cli list
 ID  PRI     DONE  DUE         TAGS            TITLE
  1  high          2026-11-20  home            Buy milk
  2  normal        -           work            Call Ada
$ clap_cli tag add 2 urgent
task 2 tags: work, urgent
$ clap_cli -v rm --done
note: using tasks.json (3 tasks)
removed 1 task(s)
$ clap_cli done 7
error: no task with id 7                          (exit code 1)
$ clap_cli rm
error: the following required arguments were not provided:
  <IDS|--done>

Usage: clap_cli remove <IDS|--done>               (exit code 2)
```

> 💡 **Analogy:** you've built the whole sandwich shop's paperwork: one form with a page per service, boxes that check themselves, "the usual" on file, a leaflet in two sizes, a clerk who finishes your words, and three clear kinds of reply.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `Cli::parse()` without `use clap::Parser` | E0599 no associated function or constant named `parse` | `use clap::Parser;` (or `CommandFactory` for `Cli::command()`) |
| An enum field without `#[derive(ValueEnum)]` | E0277 `Format: Clone` not satisfied, E0599 `value_parser` ... trait bounds were not satisfied | `#[derive(Clone, ValueEnum)]` |
| `default_value_t = "text"` on a `String` | E0308 mismatched types | `default_value = "text"` (text) or `default_value_t = String::from(..)` |
| `default_value_t` on a type without `Display` | E0277 the trait bound `Point: ToString` is not satisfied | `default_value = "..."`, implement `Display`, or for an enum derive `ValueEnum` and add `value_enum` |
| Two arguments with the same `short` | a panic at run time (debug builds): Short option names must be unique for each argument | Name one explicitly: `#[arg(short = 'V')]`; test with `Cli::command().debug_assert()` |
| A subcommand enum field without `#[command(subcommand)]` | clap tries to parse it as a value: E0277/E0599 | `#[command(subcommand)]` on the field |
| `std::env::set_var` in a test | `unsafe` in Rust 2024; tests race | `.env(..)` on a child `Command` |
| Errors printed to stdout | they end up in `> file` and pipes | `eprintln!`; data only on stdout |
| Every failure exits with 1 (or 0) | scripts can't tell usage errors apart | 0 ok, 1 failed, 2 bad arguments |
| `Cli::parse()` in a test | exits the test process | `Cli::try_parse_from([...])` |
| Help shows `Usage: tool.exe` on Windows | the name comes from `argv[0]` | `#[command(bin_name = "tool")]` |
| Reading `matches.get_one::<u32>("times")` for a `u8` arg (builder) | panic: Mismatch between definition and access of `times` | Use the type the value parser produces, or the derive API |
| An optional positional (`Vec`, `Option`) before a required one | panic: Found non-required positional argument with a lower index than a required positional argument | Make it `required = true` (then `cp a b DEST` style works), or reorder |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p clap_cli --example derive_basics
cargo run -p clap_cli --example args_options_flags
cargo run -p clap_cli --example env_and_defaults
cargo run -p clap_cli --example value_parsers
cargo run -p clap_cli --example subcommands
cargo run -p clap_cli --example help_and_version
cargo run -p clap_cli --example errors_and_exit_codes
cargo run -p clap_cli --example completions
cargo run -p clap_cli --example builder_api
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_parser_not_in_scope` | Make it compile (E0599), a trait not in scope | `cargo run -p clap_cli --features exercises --bin ex01_fix_parser_not_in_scope -- hello world` |
| 2 | `ex02_fix_missing_value_enum` | Make it compile (E0277 + E0599), the missing derive | `cargo run -p clap_cli --features exercises --bin ex02_fix_missing_value_enum -- --format json` |
| 3 | `ex03_fix_default_value_type` | Make it compile (E0308), `default_value` vs `default_value_t` | `cargo run -p clap_cli --features exercises --bin ex03_fix_default_value_type` |
| 4 | `ex04_value_parsers` | Two custom value parsers and a range, pass 3 tests | `cargo test -p clap_cli --features exercises --bin ex04_value_parsers` |
| 5 | `ex05_subcommand_dispatch` | Carry out four subcommands, pass 3 tests | `cargo test -p clap_cli --features exercises --bin ex05_subcommand_dispatch` |
| 6 | `ex06_exit_codes` | stdout vs stderr, exit codes 0/1/2, pass 3 tests | `cargo test -p clap_cli --features exercises --bin ex06_exit_codes` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p clap_cli --features solutions --bins`, or `cargo run -p clap_cli --features solutions --bin sol01_fix_parser_not_in_scope -- hello world` and so on).

**4. Use the mini project** (it writes `tasks.json` in the current folder unless you pass `--store` or set `TASKS_STORE`):

```bash
cargo run -p clap_cli -- --help
cargo run -p clap_cli -- add Buy milk -p high -t home -d 2026-11-20
cargo run -p clap_cli -- list
cargo run -p clap_cli -- list --status all --format json
cargo run -p clap_cli -- done 1
cargo run -p clap_cli -- completions powershell
cargo test -p clap_cli          # 13 unit tests + 8 tests that run the binary
```

Then try: add an `edit <ID> [--title ...] [--priority ...]` subcommand, with a test in `tests/cli.rs`.

## Checklist

- [ ] Why a parser crate beats hand-parsing `std::env::args`
- [ ] The derives: `Parser`, `Args`, `Subcommand`, `ValueEnum`; doc comments as help; `Parser` in scope to call `parse`
- [ ] Positional arguments, options, flags; `Option`, `Vec`, `bool`, counted flags, `value_delimiter`
- [ ] `conflicts_with`, `requires`, argument groups
- [ ] `default_value` vs `default_value_t`; `env = "..."` and its precedence
- [ ] Value parsers: built-in types, `ValueEnum`, `value_parser!(..).range(..)`, custom functions
- [ ] Subcommands: variants, nested, `flatten`, `global`, aliases, optional subcommand
- [ ] Help and version: about/long_about, `-h` vs `--help`, headings, `hide`, `after_help`, `bin_name`
- [ ] Errors and exit codes: `ErrorKind`, 0/1/2, stdout vs stderr, `Command::error`
- [ ] Shell completions with `clap_complete`
- [ ] Testing: `try_parse_from`, `debug_assert`, `run` with `Write`, `CARGO_BIN_EXE_<name>`, env on child processes
- [ ] The builder API and when to use it
- [ ] Exercises 1–6 and the task tracker done without looking at the solutions

## Recap

- A struct with `#[derive(Parser)]` is the command line: the field types decide required/optional/flag/repeated, attributes add names, defaults, env vars and checks, and doc comments become the help.
- Values are parsed and validated by value parsers: built-ins, `ValueEnum`, ranges and your own functions, so bad input is rejected at the door with a precise message.
- Subcommands are enums with `#[derive(Subcommand)]`; `flatten` and `global` share options; one `match` carries them out.
- Clap exits 0 for help/version and 2 for bad arguments; your program adds 1 for failures, prints data to stdout and messages to stderr.
- `clap_complete` writes completion scripts; test with `try_parse_from`, an in-process `run`, and the real binary via `CARGO_BIN_EXE_<name>`.

**In one sentence for a child:** clap is a clever order form that checks every box before the kitchen starts cooking, explains itself when you ask, and politely tells you exactly what you got wrong.

**Next:** [05 Actix Web](../05_actix_web/README.md): the other popular web framework, compared with Axum.
