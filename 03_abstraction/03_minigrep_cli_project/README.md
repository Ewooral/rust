<!-- hand-written lesson: gen_course must not overwrite -->
# An I/O Project: minigrep

**The Rust Book, chapter 12:** [An I/O Project: Building a Command Line Program](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)
· [`std::io`](https://doc.rust-lang.org/std/io/index.html) · [`std::fs`](https://doc.rust-lang.org/std/fs/index.html) · [`std::process`](https://doc.rust-lang.org/std/process/index.html)

This chapter builds a small version of `grep` ("**g**lobally search a **r**egular **e**xpression and **p**rint"): give it a word and a file, and it prints every line containing the word. It's the first real program of the course, and it uses nearly everything so far: ownership, structs, error handling, modules, lifetimes and tests.

> 💡 **The big picture: a restaurant.** `main.rs` is the **waiter**: it takes the order (the command-line arguments), passes it to the kitchen, and tells the customer if something went wrong. `lib.rs` is the **kitchen**, where the real work happens, and where an inspector (the tests) can check every dish without a customer in the room.

## How to run it

The project reads `poem.txt` from its own folder, so run it **from this folder**:

```bash
cd 03_abstraction/03_minigrep_cli_project
cargo run -- to poem.txt                 # Are you nobody, too? / How dreary to be somebody!
cargo run -- body poem.txt
cargo run                                # Problem parsing arguments: not enough arguments
cargo run -- to poem.txt > output.txt    # results to the file, errors still on screen
```

Case-insensitive search uses an environment variable:

```bash
IGNORE_CASE=1 cargo run -- to poem.txt                                       # bash
$env:IGNORE_CASE=1; cargo run -- to poem.txt; Remove-Item Env:IGNORE_CASE    # PowerShell
```

Everything after `--` goes to *your* program, not to Cargo.

## The steps (one example each)

| Step | Book | Example | What changes |
|---|---|---|---|
| 1 | 12.1 | `step1_arguments` | read the arguments with `env::args()` |
| 2 | 12.2 | `step2_read_file` | read the file with `fs::read_to_string` |
| 3 | 12.3 | `step3_refactor` | a `Config` struct, `Config::build` returning `Result`, `run`, `process::exit` |
| 4 | 12.3 | **`src/`** | split into `lib.rs` (logic) and `main.rs` (thin) |
| 5 | 12.4 | `src/lib.rs` tests | write `search` test-first |
| 6 | 12.5 | `src/lib.rs` | `IGNORE_CASE` and `search_case_insensitive` |
| 7 | 12.6 | `src/main.rs` | errors to **stderr** with `eprintln!` |

```bash
cargo run --example step1_arguments -- the poem.txt
cargo run --example step2_read_file -- the poem.txt
cargo run --example step3_refactor -- the poem.txt
cargo run --example step3_refactor        # no arguments: a friendly error, not a panic
```

## 1. Accepting command-line arguments

```rust
let args: Vec<String> = env::args().collect();
let query = &args[1];       // args[0] is the program's own name
let file_path = &args[2];
```

`env::args()` returns an **iterator**, and `collect()` turns it into a `Vec`. `collect` can build many kinds of collection, so the type annotation tells it which. `env::args` panics if an argument isn't valid Unicode; `env::args_os` returns `OsString`s instead.

> 💡 **Analogy: the order slip.** `args` is the order the customer wrote down. Line 0 is always the name of the restaurant (the program), so the actual order starts at line 1. Step 1 reads `args[1]` without checking, so a blank slip makes the waiter panic.

## 2. Reading a file

```rust
let contents = fs::read_to_string(file_path)?;
```

This reads the **whole** file into one `String` in one go. That's simple and right for files that fit comfortably in memory. For huge files or endless streams, see *buffered I/O* below.

> 💡 **Analogy:** pouring the whole pot into one bowl. That's fine for soup for four, but for a reservoir you use a ladle (a `BufReader`).

## 3. Refactoring: modularity and error handling

Step 2's `main` does everything and panics on every problem. The Book's guidelines for a binary project are:
- **Split** the program into `main.rs` and `lib.rs`, and put the logic in `lib.rs`.
- **Keep `main` small.** Its only jobs are to collect the arguments, build the config, call `run`, and report any error.

```rust
pub struct Config { pub query: String, pub file_path: String, pub ignore_case: bool }

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        Ok(Config { query: args[1].clone(), file_path: args[2].clone(), ignore_case: /* … */ })
    }
}
```

- **A struct, not loose values.** Values that belong together get one name. Each field says what it *means*, which a tuple like `(String, String)` can't.
- **`build`, not `new`.** By convention `new` never fails, and this can.
- **`Result`, not `panic!`.** A user typing too little is *expected*, not a bug (lesson 02/05). Handle it in `main` with `unwrap_or_else`, and leave with `process::exit(1)`.
- **`.clone()` is fine here.** It's a few bytes, once. Chapter 13 rewrites `build` with iterators and drops it.
- **`run` returns `Result<(), Box<dyn Error>>`.** Any error goes back to `main`, which reports it.

If you forget that `build` returns a `Result` (exercise 2), the compiler's error even names the type you're holding:

```text
error[E0609]: no field `query` on type `Result<Config, &str>`
   |
30 |     println!("Searching for {} in {}", config.query, config.file_path);
   |                                               ^^^^^ unknown field
```

> 💡 **Analogy:** `Config` is the **order ticket**: all the customer's choices on one card. `build` is the waiter checking the ticket. If something is missing, the waiter politely tells the customer ("not enough arguments"); they don't set the kitchen on fire (`panic!`). `process::exit(1)` is the **stamp on the receipt**: 0 means "served", anything else means "there was a problem", so scripts can tell.

## 4. Writing `search` test-first, and its lifetime

The Book builds `search` with **TDD** (lesson 03/02): first a failing test, then just enough code to pass it.

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
```

The returned lines are **slices of `contents`**, so `'a` ties the result to `contents` and **not** to `query`. Leave the lifetimes out, and the compiler asks (exercise 1):

```text
error[E0106]: missing lifetime specifier
   |
10 | fn search(query: &str, contents: &str) -> Vec<&str> {
   |                  ----            ----         ^ expected named lifetime parameter
help: consider introducing a named lifetime parameter
   |
10 | fn search<'a>(query: &'a str, contents: &'a str) -> Vec<&'a str> {
```

Notice the compiler's suggestion puts `'a` on **both** parameters. That's more than needed, and it would make the results depend on `query` staying alive too. In exercise 1 the query is dropped early, so following the suggestion turns E0106 into E0597.

> 💡 **Analogy: highlighting a book.** The results are lines **highlighted in the book** (`contents`). They're only valid while the book exists. The query is a sticky note saying what to look for, and you can throw it away once you've finished highlighting.

## 5. Environment variables

```rust
let ignore_case = env::var("IGNORE_CASE").is_ok();   // set to anything = on
```

Use environment variables for settings that should apply to *every* run in a shell session, and command-line flags for one run at a time. Many programs accept both, and a flag overrides the environment. Exercise 5 adds `-i` and `-n` flags.

In the 2024 edition, **`env::set_var` is `unsafe`**, because another thread could be reading the environment at the same moment. Tests shouldn't change their own environment. The integration test in `tests/cli.rs` instead sets `IGNORE_CASE` only for the **child process** it starts.

> 💡 **Analogy:** an environment variable is a **house rule posted on the kitchen wall** ("today, ignore capital letters"). It applies to every order until someone takes it down. A flag is a note on **one** order ticket.

## 6. Errors to standard error

Programs have two output streams: **stdout** for results, and **stderr** for errors and diagnostics. `println!` writes to stdout and `eprintln!` to stderr. When the user redirects output with `cargo run -- to poem.txt > output.txt`, only stdout goes into the file, and errors still appear on screen, where a person will see them.

> 💡 **Analogy: two trays.** One tray carries the **meal** (results) to the table. The other carries **complaints** to the manager (errors). Redirecting with `>` sends only the meal tray to the next table, and complaints never end up in someone's dinner.

Exercise 3 fixes a program that prints errors to stdout and exits with "success". Its `report` function writes to whatever streams it's *given*, so the test can pass in byte buffers. Passing dependencies in like this is how you test I/O without running a process.

## Beyond the Book: buffered I/O

| Tool | Use it to |
|---|---|
| `BufReader::new(file).lines()` | read a file **one line at a time**, whatever its size |
| `io::stdin().lock().lines()` | read piped input: `cat file \| minigrep …` |
| `BufWriter::new(file)` | collect many small writes into a few big ones, then **`flush()`** |
| `io::stdout().lock()` | take the lock once instead of on every `println!` |
| `impl BufRead` parameter | accept a file, stdin, or a byte slice in tests (exercise 6) |

> 💡 **Analogy:** a `BufWriter` is a **mailbag**. Instead of walking to the post box with every letter, you fill the bag and make one trip. `flush` is that trip: do it yourself, because if the bag is only dropped, a failed delivery goes unnoticed. A `BufReader` is reading a book **one page at a time** instead of photocopying the whole library first.

▶ `cargo run -p minigrep --example buffered_io` (it writes and searches a 100,000-line file)

## Beyond the Book: the file system and other processes

- `std::fs` provides `create_dir_all` (like `mkdir -p`), `write`, `copy`, `rename`, `read_dir`, `metadata` and `remove_dir_all`.
- `std::process::Command` runs another program. `.output()` waits for it and collects stdout, stderr and the exit status.
- **A program that fails** still gives you `Ok(output)`, so check `output.status.success()`. **A program that can't be started** gives `Err` (`NotFound`).
- `tests/cli.rs` uses `Command` to run the real `minigrep` binary end to end. Cargo tells integration tests where the binary is through `env!("CARGO_BIN_EXE_minigrep")`.

> 💡 **Analogy:** `Command` is **hiring a contractor**: you give them instructions (arguments, environment), wait for the job, then read their report (stdout), their complaints (stderr) and whether they say the job is done (exit status).

▶ `cargo run -p minigrep --example fs_and_process`

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `&args[1]` with no arguments | panic: index out of bounds | Check the length in `Config::build` and return `Err`. |
| Used the `Result` from `build` as a `Config` | E0609 no field | `unwrap_or_else(\|err\| { eprintln!(…); process::exit(1) })` |
| `fn search(q: &str, c: &str) -> Vec<&str>` | E0106 | `'a` on `contents` and the result only |
| `.expect()` on a file read in `run` | ugly panic for a missing file | `?` and `Box<dyn Error>` |
| `println!` for errors | errors vanish into redirected files | `eprintln!` and a non-zero exit code |
| `cargo run to poem.txt` | Cargo complains about `to` | `cargo run -- to poem.txt` |
| Running from the repo root | `poem.txt` not found | `cd` into the lesson folder, or give the full path. |
| `env::set_var` in tests | `unsafe` in 2024, and tests race | Set the variable for a child `Command` instead. |
| `BufWriter` without `flush()` | write errors silently lost on drop | `flush()?` when you're done. |

## Practise

**1. Follow the steps** with the examples (from this folder):

```bash
cargo run --example step1_arguments -- the poem.txt
cargo run --example step2_read_file -- the poem.txt
cargo run --example step3_refactor -- the poem.txt
cargo run -- to poem.txt
cargo run --example buffered_io
cargo run --example fs_and_process
```

**2. Do the exercises** in `exercises/`. Numbers 4 to 6 extend minigrep towards real `grep`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_search_lifetime` | Make it compile (E0106), `'a` on the right parameter only | `cargo run -p minigrep --features exercises --bin ex01_fix_search_lifetime` |
| 2 | `ex02_fix_unhandled_build` | Make it compile (E0609), errors to stderr | `cargo run -p minigrep --features exercises --bin ex02_fix_unhandled_build -- to poem.txt` |
| 3 | `ex03_fix_stdout_errors` | stderr and exit code 1, testable I/O, pass 2 tests | `cargo test -p minigrep --features exercises --bin ex03_fix_stdout_errors` |
| 4 | `ex04_search_with_line_numbers` | Like `grep -n`, pass 3 tests | `cargo test -p minigrep --features exercises --bin ex04_search_with_line_numbers` |
| 5 | `ex05_parse_flags` | `-i`/`--ignore-case`, `-n`/`--line-number`, pass 3 tests | `cargo test -p minigrep --features exercises --bin ex05_parse_flags` |
| 6 | `ex06_search_stdin` | Search any `BufRead`: files, stdin, test bytes, pass 3 tests | `cargo test -p minigrep --features exercises --bin ex06_search_stdin` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p minigrep --features solutions --bins`).

**4. Read the finished program:** `src/main.rs` (15 lines of code) and `src/lib.rs`. Then run all of its tests:

```bash
cargo test -p minigrep    # 5 unit tests, 5 end-to-end tests running the real binary, 1 doc test
```

## Checklist

- [ ] [12.1 Accepting Command Line Arguments](https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html)
- [ ] [12.2 Reading a File](https://doc.rust-lang.org/book/ch12-02-reading-a-file.html)
- [ ] [12.3 Refactoring to Improve Modularity and Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html)
- [ ] [12.4 Developing the Library's Functionality with Test Driven Development](https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html)
- [ ] [12.5 Working with Environment Variables](https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html)
- [ ] [12.6 Writing Error Messages to Standard Error Instead of Standard Output](https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html)
- [ ] Beyond the Book: buffered I/O: `BufReader`, `BufWriter`, `lines()`
- [ ] Beyond the Book: file system and processes: `std::fs`, `std::process::Command`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `env::args().collect()` gives you the arguments, with `args[0]` the program name. `fs::read_to_string` reads a whole file.
- Keep `main` thin and put the logic in `lib.rs`, where it can be tested. Gather settings in a `Config` built by a fallible `build`.
- Report expected problems with `Result`, then `eprintln!` and `process::exit(1)` in `main`, never with a panic.
- `search` returns slices of `contents`, so the lifetime goes on `contents` alone.
- Environment variables set defaults for a whole session, results go to stdout and errors to stderr, and buffered I/O handles large or endless input.

**In one sentence for a child:** the waiter (`main`) takes your order and tells you about problems, the kitchen (`lib`) does the cooking and gets inspected, and complaints go on a different tray from the food.

**Next:** [04 Iterators and Closures](../04_iterators_and_closures/README.md) (Book chapter 13, which also improves minigrep)
