<!-- hand-written lesson: gen_course must not overwrite -->
# Writing Automated Tests

**The Rust Book, chapter 11:** [Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
· Rust by Example: [Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
· [rustdoc: documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) · [proptest book](https://proptest-rs.github.io/proptest/)

The compiler proves a lot (types, ownership, lifetimes), but it can't know that `add_two(2)` should be `4` and not `5`. **Tests** check that code does what you *meant*. Cargo has testing built in, so writing a test is as easy as writing a function.

> 💡 **The big picture: a robot inspector with a checklist.** Every time you change something in the house, a robot walks through and checks every item on its list: lights on, taps running, doors locking. It takes seconds, so you can afford to run it after every change. If something you changed broke something you didn't touch, the robot tells you straight away, not your customers next month.

This lesson's crate is itself an example of a well-tested Rust project:

```text
03_automated_tests/ (package: automated_tests)
├── src/lib.rs            library root: the code worth testing lives in the LIBRARY
├── src/bowling.rs        mini project + its UNIT tests (#[cfg(test)] mod tests)
├── src/doc_tests.rs      every kind of DOC test
├── src/main.rs           thin binary around the library
├── tests/                INTEGRATION tests: each file is its own crate
│   ├── bowling_games.rs
│   ├── properties.rs     PROPERTY tests with proptest
│   └── common/mod.rs     helpers shared by test files
├── examples/             the chapter's ideas, as small test files
├── exercises/  solutions/
```

```bash
cargo test -p automated_tests     # runs ALL of it: unit, integration, property and doc tests
```

## 1. How to write a test

```rust
#[cfg(test)]                 // compile this module only for `cargo test`
mod tests {
    use super::*;            // the code under test

    #[test]                  // this function is a test
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }
}
```

A test **passes** if it returns without panicking and **fails** if anything in it panics: a failed `assert`, an `unwrap` on `None`, or a `panic!`. Each test runs in its own thread, so one failure doesn't stop the rest.

> 💡 **Analogy:** each `#[test]` is one line on the inspector's checklist. "Pass" means the robot reached the end of that line without an alarm going off.

▶ `cargo test -p automated_tests --example anatomy_of_a_test` (add `-- --ignored` to watch a deliberately failing test)

### Assertions

| Macro | Passes when | On failure prints |
|---|---|---|
| `assert!(cond)` | `cond` is `true` | the condition |
| `assert_eq!(left, right)` | `left == right` | **both values** |
| `assert_ne!(left, right)` | `left != right` | both values |

Any extra arguments form a custom message, `format!`-style: `assert!(result.contains("Carol"), "Greeting did not contain name, value was `{result}`")`. Here is a real failure, with its message:

```text
assertion `left == right` failed: add_two(3) should be 5, so this fails
  left: 5
 right: 4
```

`assert_eq!` needs your type to implement `PartialEq` (to compare) and `Debug` (to print). Without them you get the real errors from exercise 2:

```text
error[E0369]: binary operation `==` cannot be applied to type `Point`
help: consider annotating `Point` with `#[derive(PartialEq)]`
error[E0277]: `Point` doesn't implement `Debug`
help: consider annotating `Point` with `#[derive(Debug)]`
```

> 💡 **Analogy:** `assert!` asks the robot "is the light on? yes or no". `assert_eq!` asks it to compare the reading with the label on the box, and when they differ, it writes **both** numbers on its report. A custom message is a sticky note on that checklist line: "if this fails, here's what it means".

▶ `cargo test -p automated_tests --example assertions`

### `#[should_panic]` and tests that return `Result`

```rust
#[test]
#[should_panic(expected = "less than or equal to 100")]
fn greater_than_100() {
    Guess::new(200);
}

#[test]
fn parses() -> Result<(), ParseIntError> {
    let n: u32 = "42".parse()?;   // `?` works: Err fails the test
    assert_eq!(n, 42);
    Ok(())
}
```

- `#[should_panic]` passes only if the code **panics**. Always add `expected = "…"` (a piece of the panic message). Otherwise the test also passes if the code panics for a completely different reason. Exercise 3 is exactly that trap. When you add `expected`, the hidden truth comes out:

```text
note: panic did not contain expected string
      panic message: "amount must be positive"
```

- A test that returns `Result<(), E>` can use `?`. It can't also be `#[should_panic]`. To check that something returns `Err`, write `assert!(value.is_err())` rather than using `?`.

> 💡 **Analogy:** `#[should_panic]` is a **fire drill**: the test passes only if the alarm goes off. `expected` checks it was the **right** alarm. A smoke alarm going off during a flood drill isn't a pass.

▶ `cargo test -p automated_tests --example should_panic_and_result`

## 2. Controlling how tests run

Arguments **before** `--` go to Cargo, and arguments **after** `--` go to the test program:

| Command | Effect |
|---|---|
| `cargo test -- --show-output` | show `println!` output from **passing** tests (hidden by default) |
| `cargo test add` | run only tests whose name contains `add` |
| `cargo test -- --ignored` | run only `#[ignore]` tests |
| `cargo test -- --include-ignored` | run everything |
| `cargo test -- --test-threads=1` | run one at a time |
| `cargo test --test bowling_games` | one integration test file |
| `cargo test --doc` / `--lib` | only doc tests / only unit tests |
| `cargo test -- --list` | list tests without running them |

Tests run **in parallel** by default, so tests mustn't share things like the same file, the same environment variable or the same port. Give each test its own, or run with `--test-threads=1`.

> 💡 **Analogy:** parallel tests are **several inspectors working at once**. That's fast, but two of them must never test the same shower at the same time. `--show-output` asks every inspector to hand in their notes, not only the ones who found problems. A filter is "just check the kitchen today". `#[ignore]` is "skip the roof survey unless I ask; it takes all day".

▶ `cargo test -p automated_tests --example controlling_tests` (the file header lists every command to try)

## 3. Test organisation

| | Unit tests | Integration tests |
|---|---|---|
| Where | next to the code, in `#[cfg(test)] mod tests` | the `tests/` folder, one crate per file |
| Can see | **private** items too (the module is a child) | only the **public** API, like a real user |
| Checks | each piece on its own | the pieces working **together** |

- **Private functions can be tested:** `src/bowling.rs` tests the private `check_bonus_pair` directly.
- **Integration tests need a library crate.** Files in `tests/` can `use automated_tests::…` but can't reach into `src/main.rs`. That's why real logic lives in `src/lib.rs` and `main.rs` stays thin.
- **Shared helpers go in `tests/common/mod.rs`**, not `tests/common.rs`. Every file directly in `tests/` becomes its own test crate, so `common.rs` would show up as a test file with 0 tests.

> 💡 **Analogy:** unit tests check each part **on the workbench**: the brakes, the lights, the engine, with the bonnet open (private access). Integration tests **test-drive the finished car** from the driver's seat, using only the pedals and the wheel (the public API). Both find different problems. `tests/common/` is the shared toolbox in the garage.

## Beyond the Book: documentation tests

Every ```` ``` ```` Rust code block in a `///` or `//!` comment is **compiled and run** by `cargo test`, so the examples in your docs can't drift out of date. Block attributes:

| Write | Effect |
|---|---|
| ```` ``` ```` | compile and run, and pass if it doesn't panic |
| `# hidden line` | compiled but not shown in the docs (setup, `use`, asserts) |
| ```` ```should_panic ```` | pass only if it panics |
| ```` ```no_run ```` | compile, but never run (network, infinite loops, deleting files) |
| ```` ```compile_fail ```` | pass only if it **doesn't** compile, which proves a mistake is impossible |
| ```` ```text ```` | not Rust: not compiled |
| ending with `# Ok::<(), E>(())` | lets the example use `?` |

Doc tests only run for **library** crates. See `src/doc_tests.rs` for one of each.

> 💡 **Analogy:** the instruction manual's examples are actually **tried out** before every release. If the manual says "press the red button and the door opens", the robot presses it. A manual that doesn't match the product fails the build.

▶ `cargo test -p automated_tests --doc`

## Beyond the Book: property-based testing with `proptest`

Example-based tests check the cases **you** thought of. Property tests describe what *any* valid input looks like, state a rule that must **always** hold, and let a library generate hundreds of random cases:

```rust
proptest! {
    #[test]
    fn valid_games_score_between_0_and_300(rolls in valid_game()) {
        prop_assert!(score(&rolls)? <= 300);
    }
}
```

When a case fails, proptest **shrinks** it: it keeps simplifying the input until it finds the smallest one that still fails, then reports that. Good properties to look for:
- **Round trips:** `decode(encode(x)) == x`.
- **Invariants:** length is unchanged, a total stays within bounds.
- **Never panics:** any input gives `Ok` or `Err`, never a crash.
- **Agreement:** two implementations give the same answer.

> 💡 **Analogy:** instead of three test drives you planned yourself, a **robot driver tries 256 random routes**. When one ends in a crash, it keeps retrying shorter and shorter versions of that route until it finds the shortest drive that still crashes, and hands you that.

▶ `cargo test -p automated_tests --test properties`

## Beyond the Book: test-driven development

**TDD** turns testing into a way of *designing* code. Work in tiny loops:
1. **Red:** write a test for the next small piece of behaviour, and watch it fail.
2. **Green:** write the simplest code that makes it pass.
3. **Refactor:** tidy up, keeping every test green.

The bowling scorer in `src/bowling.rs` is the classic TDD kata. Its unit tests read like the steps it was built in: gutter game, all ones, a spare, a strike, a perfect game, then the 10th-frame rules. Exercise 5 has you do Roman numerals the same way.

> 💡 **Analogy:** TDD is **writing the exam question before you study for it**. You know exactly what "done" looks like before you start, and you stop studying when you can answer it.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `assert_eq!` on your own struct | E0369 / E0277 | `#[derive(Debug, PartialEq)]` |
| `#[should_panic]` with no `expected` | passes for the wrong reason | `#[should_panic(expected = "…")]` |
| A test with no assertions | always passes, proves nothing | Assert on the result. |
| Tests sharing a file or env var | random failures | Separate resources, or `--test-threads=1`. |
| `println!` "doesn't print" in tests | output is captured | `-- --show-output` (or `--nocapture`) |
| `tests/common.rs` for helpers | an empty "test file" in the output | `tests/common/mod.rs` |
| Integration test using a `main.rs` item | E0432 / E0433 | Move the logic into `src/lib.rs`. |
| Testing only the happy path | bugs hide at the edges | Test empty, zero, maximum and invalid input, or add a property test. |
| Changing the test to match the bug | the bug is now "tested in" | Fix the code. The test documents the intent. |

## Practise

**1. Run the examples** (all `cargo test`, not `cargo run`):

```bash
cargo test -p automated_tests --example anatomy_of_a_test
cargo test -p automated_tests --example anatomy_of_a_test -- --ignored
cargo test -p automated_tests --example assertions
cargo test -p automated_tests --example should_panic_and_result
cargo test -p automated_tests --example controlling_tests -- --show-output
cargo test -p automated_tests --doc
cargo test -p automated_tests --test properties
```

**2. Do the exercises** in `exercises/`. In this chapter you mostly **write tests**:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_the_bug` | A test fails: fix the **code** | `cargo test -p automated_tests --features exercises --bin ex01_fix_the_bug` |
| 2 | `ex02_fix_missing_derives` | Make the test compile (E0369, E0277) | `cargo test -p automated_tests --features exercises --bin ex02_fix_missing_derives` |
| 3 | `ex03_honest_should_panic` | A test passes for the wrong reason: make it honest | `cargo test -p automated_tests --features exercises --bin ex03_honest_should_panic` |
| 4 | `ex04_write_unit_tests` | Write 4 tests for working code | `cargo test -p automated_tests --features exercises --bin ex04_write_unit_tests` |
| 5 | `ex05_tdd_roman_numerals` | TDD: tests given, write the code | `cargo test -p automated_tests --features exercises --bin ex05_tdd_roman_numerals` |
| 6 | `ex06_property_tests` | Write 3 proptest properties (Beyond the Book) | `cargo test -p automated_tests --features exercises --bin ex06_property_tests` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p automated_tests --features solutions --bins`).

**4. The mini project** is a **bowling scorer**, tested every way this chapter teaches:

```bash
cargo run -p automated_tests                                   # a sample game: 167
cargo run -p automated_tests -- 10 10 10 10 10 10 10 10 10 10 10 10   # 300
cargo run -p automated_tests -- 5 6                            # not a valid game: exit code 1
cargo test -p automated_tests                                  # every kind of test
```

| Kind | Where | How many |
|---|---|---|
| Unit tests, including a private helper | `src/bowling.rs` | 10 |
| Integration tests, with shared helpers and score-sheet notation | `tests/bowling_games.rs` | 5 |
| Property tests: valid games score 0–300, never panic on any input, cutting a game short makes it incomplete | `tests/properties.rs` | 4 × 256 cases |
| Doc tests | `src/bowling.rs`, `src/doc_tests.rs` | 8 |

> 💡 **Analogy:** the bowling scorer is the house, and the four kinds of test are four inspectors. One checks each brick (unit), one walks through it like a visitor (integration), one reads the manual and tries every example (doc), and one throws random furniture at it to see if anything breaks (property).

## Checklist

- [ ] [11.1 How to Write Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
- [ ] [11.2 Controlling How Tests Are Run](https://doc.rust-lang.org/book/ch11-02-running-tests.html)
- [ ] [11.3 Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [ ] Beyond the Book: doc tests
- [ ] Beyond the Book: property testing with `proptest`
- [ ] Beyond the Book: test-driven development (red, green, refactor)
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A `#[test]` function passes if it doesn't panic. Use `assert!`, `assert_eq!` and `assert_ne!`, with custom messages.
- `#[should_panic(expected = "…")]` tests panics honestly. Tests can return `Result` and use `?`.
- `cargo test` takes filters, `--ignored`, `--show-output` and `--test-threads`. Tests run in parallel, so don't let them share resources.
- Unit tests sit beside the code and can test private items. Integration tests in `tests/` use only the public API of a **library** crate.
- Doc tests keep examples honest, property tests explore inputs you didn't think of, and TDD lets tests drive the design.

**In one sentence for a child:** tests are a robot inspector that re-checks the whole house every time you change anything, so you find out what broke straight away.

**Next:** [03 An I/O Project: minigrep](../03_minigrep_cli_project/README.md) (Book chapter 12)
