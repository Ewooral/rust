<!-- hand-written lesson: gen_course must not overwrite -->
# Error Handling

**The Rust Book, chapter 9:** [Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
· Rust by Example: [Error handling](https://doc.rust-lang.org/rust-by-example/error.html)
· Crates: [`thiserror`](https://docs.rs/thiserror), [`anyhow`](https://docs.rs/anyhow)

Things go wrong: files are missing, users type "seventy" where you wanted 70, networks drop. Rust has no exceptions. It splits errors into two kinds and makes you choose, in the types, how each is handled:

| Kind | Example | Rust's tool |
|---|---|---|
| **Unrecoverable**: a bug, a broken promise in the code | reading past the end of a Vec | `panic!`, which stops the program |
| **Recoverable**: expected trouble the program should handle | file not found, bad user input | `Result<T, E>`, a value you must deal with |

> 💡 **The big picture: a parcel delivery.** Every risky operation is a delivery. It comes back either with your **parcel** (`Ok(value)`) or with a **note** explaining why not (`Err(error)`). You can't use the parcel without first looking at which one you got. A **panic** is different: it's the fire alarm. Everyone stops, leaves the building, and the program ends.

## 1. Unrecoverable errors with `panic!`

A panic prints a message, **unwinds** the stack (cleaning up every value on the way out) and exits with an error code. It happens when you call `panic!` yourself, or when code hits a bug, like an out-of-bounds index:

```text
thread 'main' (16228) panicked at 02_core_types_and_errors\05_error_handling\examples\panic.rs:25:18:
index out of bounds: the len is 3 but the index is 99
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- **Backtrace:** set `RUST_BACKTRACE=1` to see the chain of function calls that led to the panic. Read it from the top until you reach a file you wrote. In PowerShell: `$env:RUST_BACKTRACE=1; cargo run …`
- **Why Rust panics instead of reading past the end:** in C, reading index 99 of a 3-element array returns whatever happens to be in that memory, a *buffer over-read*, which is a classic security hole. Rust stops instead.
- **Unwind or abort:** `panic = 'abort'` under `[profile.release]` in `Cargo.toml` skips the clean-up and makes the binary smaller.

> 💡 **Analogy: the fire alarm.** Pulling it doesn't solve anything; it stops everything safely. Everyone files out in order (unwinding drops every value), and the building is closed. You pull it when continuing would be dangerous, not because the kitchen ran out of milk.

▶ `cargo run -p error_handling --example panic` (it panics on purpose, and `-- explicit` shows a plain `panic!`)

## 2. Recoverable errors with `Result`

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Like `Option`, it's an ordinary enum, and `Ok` and `Err` need no prefix. `File::open` returns `Result<File, io::Error>`, so you must look inside before you have a `File`:

```rust
let greeting_file = match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => match error.kind() {
        ErrorKind::NotFound => File::create("hello.txt").expect("create failed"),
        other => panic!("Problem opening the file: {other:?}"),
    },
};
```

You can react differently to different errors: `error.kind()` returns an `ErrorKind` such as `NotFound` or `PermissionDenied`.

> 💡 **Analogy:** the delivery comes back with a note. "Address not found"? Fine, build the house (create the file). "Access denied"? That's serious: pull the alarm.

### Shortcuts: `unwrap` and `expect`

| Call | On `Ok(v)` | On `Err(e)` |
|---|---|---|
| `.unwrap()` | gives `v` | panics with a generic message |
| `.expect("why it should work")` | gives `v` | panics with **your** message |
| `.unwrap_or(default)` / `.unwrap_or_else(f)` | gives `v` | gives the default, no panic |

Use `expect` in tests, examples and prototypes, and when you know something the compiler doesn't. Its message should say *why you expected success*: `.expect("hello.txt should be included in this project")`.

> 💡 **Analogy:** `unwrap` is tearing the parcel open while saying "it's definitely fine". If it turns out to be a failure note, you pull the fire alarm. `expect` is the same, but you leave a note for the firefighters saying what you were sure of.

▶ `cargo run -p error_handling --example result_basics`

## 3. Propagating errors and `?`

Often the function that hits an error isn't the right place to decide what to do about it. Return the error to the caller, which is called **propagating** it:

```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut username = String::new();
    File::open("hello.txt")?.read_to_string(&mut username)?;
    Ok(username)
}
```

`expr?` means: if this is `Err(e)`, **return** `Err(From::from(e))` from the current function immediately. If it's `Ok(v)`, the whole expression is `v`, so carry on. The example shows the same function four ways, from a hand-written `match` down to `fs::read_to_string`.

> 💡 **Analogy: passing the note up.** `?` is an assistant who opens each delivery. A parcel goes straight on to the next step. A failure note goes straight up to your boss (the caller), and today's work stops there. You only write code for the happy path; `?` handles every early exit.

Rules for `?`:
- It only works inside a function whose return type can carry the error: `Result`, or `Option` for `?` on an `Option`. The real error from exercise 1:

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
   |
10 | fn main() {
   | --------- this function should return `Result` or `Option` to accept `?`
11 |     let answer: i32 = "42".parse()?;
   |                                   ^ cannot use the `?` operator in a function that returns `()`
```

- The error **converts** through `From`, and only if a conversion exists. The real error from exercise 2:

```text
error[E0277]: `?` couldn't convert the error to `ParseIntError`
   |
14 | fn read_number(path: &Path) -> Result<i64, ParseIntError> {
   |                                -------------------------- expected `ParseIntError` because of this
15 |     let text = fs::read_to_string(path)?;
   |                ------------------------^ the trait `From<std::io::Error>` is not implemented for `ParseIntError`
```

- `?` on an `Option` returns `None` early. You can't mix `?` on a `Result` and on an `Option` in one function without converting: `.ok()` turns a `Result` into an `Option`, and `.ok_or(err)` goes the other way.

▶ `cargo run -p error_handling --example propagating`

### `main` can return a `Result`

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let n: i64 = "42".parse()?;
    Ok(())
}
```

`Box<dyn Error>` means "any kind of error" (trait objects, lesson 03/07). Every error type converts into it, so `?` works on all of them. If `main` returns `Err`, Rust prints `Error: …` and the process exits with a **non-zero code**, which scripts and CI can check.

> 💡 **Analogy:** `Box<dyn Error>` is a universal envelope. Any kind of failure note fits inside, so whoever receives it only needs to handle "an envelope".

▶ `cargo run -p error_handling --example main_returns_result -- 3 four 5`

### Results must be used

`Result` is marked `#[must_use]`, so ignoring one gives a **warning**. Exercise 3 turns the warning into an error with `#![deny(unused_must_use)]`, as many teams do in CI:

```text
error: unused `Result` that must be used
   |
21 |     save_score("Ama", 90);
   |     ^^^^^^^^^^^^^^^^^^^^^
help: use `let _ = ...` to ignore the resulting value
```

Notice the compiler's suggestion, `let _ =`. That *silences* the warning while still ignoring the failure. Only use it when ignoring the error really is the right decision.

> 💡 **Analogy:** a letter stamped **"reply required"**. You may decide not to reply, but you have to decide out loud. `let _ =` is putting the letter in the bin unread.

## 4. To `panic!` or not to `panic!`

Default to **`Result`**: returning it lets the caller choose, and the caller can always turn an `Err` into a panic. A panic can't be turned back into a decision.

**Panicking is fine when:**
- **Examples, prototypes and tests.** In tests, a panic is how a test fails.
- **You know more than the compiler:** `"127.0.0.1".parse::<IpAddr>().expect("hardcoded IP should be valid")`
- **The code would otherwise be in a *bad state*:** a broken promise or invariant (a bug), not something that's *expected* to happen sometimes, and continuing could be insecure or harmful.
- **A caller broke the function's contract.** Document it with a `# Panics` section in the doc comment.

**Return a `Result` when** failure is *expected*: bad input, a missing file, a network timeout, a parser fed garbage.

**Validate once with types.** Make a type that can *only* hold valid values. Check in the constructor (`Guess::new`), keep the field private, and every function that receives a `Guess` knows it's between 1 and 100 without checking again. The Book's `Guess::new` panics, because passing 0 is a bug. The example adds `Guess::parse`, which returns a `Result` for user input.

> 💡 **Analogy: the smoke alarm vs the shopping list.** A smoke alarm (panic) is for "something is badly wrong right now". "We're out of milk" is not a smoke-alarm event; it's a note on the fridge (a `Result`) that someone deals with. And a **wristband** at a festival is a validated type: you're checked once at the gate, and inside nobody asks for your ticket again.

▶ `cargo run -p error_handling --example validation_types`

## Beyond the Book: custom error types

A professional error type is usually an **enum**, with one variant for each kind of failure. It implements:
- **`Display`:** the message for people.
- **`Debug`:** detail for programmers. You get it with `derive`.
- **`std::error::Error`:** so it works with `?`, `Box<dyn Error>` and other libraries. Optionally `source()` returns the lower-level error that caused it.
- **`From<OtherError>`:** so `?` converts lower-level errors automatically.

```rust
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}
let retries: u32 = text.parse()?;  // ParseIntError → AppError, by itself
```

`map_err(|e| …)` converts one error by hand when no `From` exists, or when you want to add detail.

> 💡 **Analogy:** a custom error enum is a **form with a tick box for every known problem**, so whoever receives it can react to each one precisely. `From` is a **translator at the border**: a note written in "parse-error language" gets rewritten into your "app-error language" as it passes through `?`, and `source()` keeps the original note stapled behind it.

▶ `cargo run -p error_handling --example custom_errors`

## Beyond the Book: `thiserror` and `anyhow`

Writing `Display`, `Error` and `From` by hand is repetitive. Two crates handle almost every real project:

| Crate | Use in | Gives you |
|---|---|---|
| `thiserror` | **libraries** | `#[derive(Error)]`: messages with `#[error("…")]` and `From` with `#[from]`. Callers can still `match` each variant. |
| `anyhow` | **applications** | one `anyhow::Error` that holds anything, `.context("what I was doing")`, `bail!("message")`. Print the whole chain with `{:#}`. |

```rust
#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("missing key `{0}`")]
    MissingKey(&'static str),
    #[error("not a whole number")]
    BadNumber(#[from] ParseIntError),
}

fn load(name: &str, text: &str) -> anyhow::Result<u32> {
    let threads = read_threads(text).with_context(|| format!("while loading config `{name}`"))?;
    Ok(threads)
}
// error: while loading config `db`: not a whole number: invalid digit found in string
```

> 💡 **Analogy:** `thiserror` is a **label maker** for your tick-box forms: same forms, much less writing. `anyhow` is a single **in-tray** that accepts any note, where everyone the note passes adds a sticky note saying what they were doing ("while loading config `db`"). By the time it reaches the top, it tells the whole story.

▶ `cargo run -p error_handling --example thiserror_and_anyhow`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `?` in a function returning `()` (often `main`) | E0277 | Return `Result<…>`, e.g. `fn main() -> Result<(), Box<dyn Error>>`. |
| `?` on two different error types | E0277 "couldn't convert the error" | `Box<dyn Error>`, an enum with `From` impls, or `map_err`. |
| Used a `Result<T, E>` as if it were a `T` | E0308 / E0369 | `match`, `?`, `unwrap_or`, … |
| Called a fallible function and ignored it | warning: unused `Result` | Handle it: `?`, `if let Err(e)`, `match`. |
| `?` on an `Option` in a `Result` function | E0277 | `.ok_or(MyError::…)?` |
| `.unwrap()` everywhere | panics in production | `?` and a real error type. Use `expect("why")` where a panic is truly right. |
| Error type without `Display` / `Error` | E0277 when boxing it | Implement both, or `#[derive(thiserror::Error)]`. |
| `panic!` on bad user input | crashes on expected trouble | Return a `Result`, and let the caller report it. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p error_handling --example panic
cargo run -p error_handling --example result_basics
cargo run -p error_handling --example propagating
cargo run -p error_handling --example main_returns_result -- 3 four 5
cargo run -p error_handling --example validation_types
cargo run -p error_handling --example custom_errors
cargo run -p error_handling --example thiserror_and_anyhow
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_question_mark_in_main` | Make it compile (E0277) | `cargo run -p error_handling --features exercises --bin ex01_fix_question_mark_in_main` |
| 2 | `ex02_fix_mismatched_errors` | Make it compile (E0277), signature only | `cargo run -p error_handling --features exercises --bin ex02_fix_mismatched_errors` |
| 3 | `ex03_fix_ignored_result` | Make it compile (unused_must_use), no `let _` | `cargo run -p error_handling --features exercises --bin ex03_fix_ignored_result` |
| 4 | `ex04_parse_temperature` | Custom error enum + `Display`, pass 3 tests | `cargo test -p error_handling --features exercises --bin ex04_parse_temperature` |
| 5 | `ex05_validated_username` | A type that is always valid, pass 3 tests | `cargo test -p error_handling --features exercises --bin ex05_validated_username` |
| 6 | `ex06_question_mark_with_from` | `impl From` so `?` converts, pass 3 tests | `cargo test -p error_handling --features exercises --bin ex06_question_mark_with_from` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p error_handling --features solutions`).

**4. The mini project** is a **grade-book loader** that never crashes on bad data. It reads `data/students.csv`, which has 6 good lines and 6 broken ones, on purpose.

```bash
cargo run -p error_handling                   # loads 6 students, reports lines 7-12 and why
cargo run -p error_handling -- missing.csv    # a fatal error: message + exit code 1, no panic
cargo test -p error_handling
```

- **Two levels of error:** a missing *file* stops the program, because `main` returns `Err`. A bad *line* is reported with its line number, and loading continues.
- `LineError` is a hand-written custom error with `Display`, `Error::source` and `From<ParseIntError>`, so `grade.parse()?` just works.
- It finishes what `01_structs/examples/hogwarts_preview.rs` started: the same houses, now loaded from a file and fully error-handled.

> 💡 **Analogy:** the loader is a teacher marking a stack of forms. A form with a smudged grade goes on the "come and see me" pile with a note saying which line was wrong. The rest still get marked. Only if the whole stack is missing does the teacher stop and say so.

## Checklist

- [ ] [9.1 Unrecoverable Errors with panic!](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html)
- [ ] [9.2 Recoverable Errors with Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [ ] [9.3 To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)
- [ ] Beyond the Book: custom error enums implementing `Display` and `Error`
- [ ] Beyond the Book: `thiserror` (libraries) and `anyhow` (applications)
- [ ] Beyond the Book: `?` with automatic `From` conversions
- [ ] Exercises 1–6 and the grade-book loader done

## Recap

- `panic!` is for bugs and broken invariants: it stops the program. `Result<T, E>` is for expected failure: the caller decides.
- Look inside a `Result` with `match`, `if let`, `unwrap_or`, or (when a panic is right) `expect("why")`.
- `?` returns the error early and converts it with `From`. It works in functions returning `Result` or `Option`, including `main`.
- Validate once in a constructor, and let the type carry the guarantee.
- Professional code uses error enums: `thiserror` in libraries, `anyhow` with `.context()` in applications.

**In one sentence for a child:** every risky job comes back with either the thing you wanted or a note saying what went wrong, you must read which one you got, and the fire alarm (`panic!`) is only for real emergencies.

**Next:** [03 Abstraction / 01 Generics, Traits and Lifetimes](../../03_abstraction/01_generics_traits_lifetimes/README.md) (Book chapter 10)
