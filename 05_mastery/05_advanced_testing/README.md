<!-- hand-written lesson: gen_course must not overwrite -->
# Advanced Testing

**Read alongside:** [The Rust Book, chapter 11](https://doc.rust-lang.org/book/ch11-00-testing.html) (done in [03/02 Automated tests](../../03_abstraction/02_automated_tests/README.md))
· [rustc book: tests and their options](https://doc.rust-lang.org/rustc/tests/index.html) · Cargo: [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html), [test targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)
· Crates: [mockall](https://docs.rs/mockall), [insta](https://insta.rs/docs/), [proptest](https://proptest-rs.github.io/proptest/), [arbitrary](https://docs.rs/arbitrary)
· [Rust Fuzz Book](https://rust-fuzz.github.io/book/) · [cargo-nextest](https://nexte.st/) · [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

Run it: `cargo run -p advanced_testing`  ·  Test it: `cargo test -p advanced_testing`

[Lesson 03/02](../../03_abstraction/02_automated_tests/README.md) taught the basics: `#[test]`, `assert_eq!`, unit and integration tests, doc tests, `#[should_panic]`, a first property test and TDD. This lesson is about testing **real code**: code that reads the clock, rolls dice, calls a payment provider, prints long reports, and takes input from strangers. After it you'll be able to design tests that read like a specification, swap any dependency for a test double (by hand or with mockall), freeze time and randomness, check big outputs with snapshots, let property tests and a fuzzer hunt for the inputs you didn't think of, organise the tests of a big project, and know which tools (nextest, coverage) to reach for next.

> 💡 **The big picture: a flight simulator.** Pilots don't learn to handle an engine fire by waiting for one. In the simulator, **everything outside the cockpit is pretend**: the instructor sets the weather, the time of day and the other planes (test doubles), can **freeze the clock** or make the same storm happen twice (an injected clock, a seeded random generator), photographs the instrument panel to compare with the last run (snapshots), and throws thousands of random emergencies at the pilot (property tests and fuzzing). The real cockpit, your code, never knows the difference.

## 1. Test design: tests that read like a specification

A test suite is documentation that can't go out of date, if you write it that way:

- **Name the behaviour, not the function.** `orders_over_fifty_pounds_ship_free` says what's being promised; `test_shipping_3` says nothing. In the output, the names read like a list of rules.
- **Arrange, act, assert.** Set up only what this behaviour needs, call the code once, check one outcome. Blank lines between the three parts make the shape visible.
- **One behaviour per test**, so a failure points at one broken rule.
- **Test the edges.** Bugs live at boundaries: exactly £50, exactly 1 kg, empty input, the second a coupon expires.
- **Table-driven tests** check many cases in one loop. Give each case a name and put it in the assertion message, so you know *which* row failed.

```rust
#[test]
fn orders_over_fifty_pounds_ship_free() {
    // Arrange
    let order = 5_001;
    let heavy = 9_000;
    // Act
    let cost = shipping_pence(order, heavy, false);
    // Assert
    assert_eq!(cost, 0);
}

#[test]
fn weight_bands() {
    let cases = [("exactly 1 kg", 1_000, 350), ("just over 1 kg", 1_001, 600) /* … */];
    for (name, grams, expected) in cases {
        assert_eq!(shipping_pence(1_000, grams, false), expected, "case: {name}");
    }
}
```

> 💡 **Analogy: a recipe test card.** Each card says "**Given** these ingredients, **when** I bake for 20 minutes, **then** the cake has risen." One card per promise, with a name that says the promise. A card that just says "Test 3" is useless when it fails at midnight.

▶ `cargo test -p advanced_testing --example test_design` (and `cargo run` it too)

## 2. Test doubles and dependency injection, by hand

Code that calls the payment provider, the mail server or the clock **directly** can only be tested with the real thing. The fix is **dependency injection**: the code asks for the dependency through a **trait**, and whoever calls it decides which implementation to pass. Production passes the real one; a test passes a **test double**.

```rust
trait Notifier {
    fn notify(&self, user: &str, message: &str) -> Result<(), String>;
}

// The dependency comes IN; the function doesn't know or care which one it gets.
fn remind_overdue(invoices: &[Invoice], notifier: &dyn Notifier, audit: &dyn AuditLog) -> usize
```

Take it as `&dyn Trait` (one compiled version, a little dynamic dispatch) or as a generic `impl Trait` / `<N: Notifier>` (a version per type, no dispatch). The mini project uses generics: `Checkout<C: Clock, P: PaymentGateway>`.

The kinds of double, all written by hand in the example:

| Double | What it does | Example |
|---|---|---|
| **Dummy** | fills a parameter; must never be used | `DummyAudit` panics if called |
| **Stub** | gives canned answers | `AlwaysOk` returns `Ok(())` |
| **Spy** | a stub that also records its calls | `SpyNotifier` keeps a `RefCell<Vec<…>>` |
| **Fake** | a real, simplified, working implementation | `FakeNotifier` with an in-memory user list; the lesson's `FakeGateway` and `FixedClock` |
| **Mock** | knows which calls to **expect**, and fails the test if they don't happen | `MockNotifier` checks its list in `Drop` |

Trait methods usually take `&self`, so a spy or fake that must *remember* things keeps its record in a `RefCell` (lesson [03/06 Smart pointers](../../03_abstraction/06_smart_pointers/README.md)). Exercise 2 is exactly that.

> 💡 **Analogy: stand-ins at a rehearsal.** A **dummy** is a cardboard cut-out that fills a chair. A **stub** is a stand-in who reads one line, always the same. A **spy** reads the line and secretly writes down everything said to them. A **fake** is an understudy who can really play the part, just not as grandly. A **mock** is a stand-in with the script in hand, who stops the rehearsal if you skip their cue.

▶ `cargo test -p advanced_testing --example hand_rolled_doubles`

## 3. Mocking with mockall

Hand-written mocks get long. [mockall](https://docs.rs/mockall) generates them from the trait:

```rust
#[cfg_attr(test, mockall::automock)]   // adds MockWeatherApi, in test builds only
trait WeatherApi {
    fn temperature(&self, city: &str) -> Result<i32, String>;
}

let mut api = MockWeatherApi::new();
api.expect_temperature()          // expect a call to `temperature`...
    .with(eq("Accra"))            // ...with this argument...
    .times(1)                     // ...exactly once...
    .returning(|_| Ok(31));       // ...and answer this.
assert_eq!(advice(&api, "Accra"), "Accra: 31°C, take water");
// when `api` is dropped, mockall checks that times(1) was met
```

| Want | Write |
|---|---|
| check arguments | `.with(eq(x), always())`, or any test with `.withf(\|a, b\| …)` |
| how many calls | `.times(1)`, `.times(2..)`, `.never()` |
| what to answer | `.returning(\|args\| …)` (may use the arguments), `.return_const(v)` |
| calls in order | a `Sequence` and `.in_sequence(&mut seq)` on each expectation |

- `#[cfg_attr(test, automock)]` keeps mockall (a **dev-dependency**) out of the real program.
- **Integration tests can't see `cfg(test)` items**: they're a separate crate, compiled against the normal library. So `tests/checkout/support.rs` declares its own mock of the public trait with `mockall::mock! { pub Gateway {} impl PaymentGateway for Gateway { … } }`.
- A mock fails **loudly**. The real messages:

```text
MockWeatherApi::temperature: Expectation(<anything>) called 1 time(s) which is fewer than expected 2
```

```text
MockGateway::charge: Expectation(var == 915, var == "ORD-1") called 2 times which is more than the expected 1
```

  The second one is exercise 3: a retry loop with `is_err()` where it meant `is_ok()`, so every **successful** payment was charged twice. A stub that just returns `Ok` would never notice; a mock counts the calls.
- **Don't over-mock.** A mock pins down *how* the code talks to its dependency. Use one when the calls are the behaviour (charge exactly once, release stock on failure); prefer a fake when only the result matters, so refactoring doesn't break the tests.

> 💡 **Analogy: a robot stand-in with a checklist.** Instead of training a new stand-in for every rehearsal, you hand a robot a card: "when asked about Accra, say 31 degrees, and expect to be asked **exactly once**". If nobody asks, or someone asks twice, the robot raises its hand at the end of the scene.

▶ `cargo test -p advanced_testing --example mockall_basics` (add `-- --ignored` to see a mock fail)

## 4. Testing time: inject a clock

`SystemTime::now()` inside your logic means a test can only check "now". To test "the coupon stops working at midnight", you'd have to wait for midnight. Ask a `Clock` instead:

```rust
pub trait Clock {
    fn now(&self) -> Timestamp;
}

fn is_expired(&self, clock: &impl Clock) -> bool {
    clock.now().0 - self.last_seen.0 >= THIRTY_MINUTES
}

let clock = FixedClock::at(Timestamp::from_utc(2026, 11, 16, 9, 0));
clock.advance(THIRTY_MINUTES - 1);    // 29:59 later, instantly
assert!(!session.is_expired(&clock));
clock.advance(1);                      // exactly 30:00
assert!(session.is_expired(&clock));
```

- Production passes `SystemClock`; tests pass `FixedClock`, a fake that only moves when told (`advance`, `set`).
- `impl<C: Clock + ?Sized> Clock for &C` lets a test **keep** the clock (to move it) and **lend** it to the code under test: `Checkout::new(&clock, …)`.
- Freezing time makes the scary cases cheap: one second before expiry, exactly at expiry, Friday 16:59, New Year's Eve at midnight, a leap day.
- In async code, tokio's `time::pause()` (the `test-util` feature) does the same for timers and `sleep`.

Exercise 1 does this refactor. The compiler's first suggestion points the wrong way:

```text
error[E0061]: this method takes 0 arguments but 1 argument was supplied
  --> 05_mastery\05_advanced_testing\exercises\ex01_inject_the_clock.rs:47:16
   |
47 |         coupon.is_valid(&SystemClock)
   |                ^^^^^^^^ ------------ unexpected argument of type `&SystemClock`
   |
note: method defined here
  --> 05_mastery\05_advanced_testing\exercises\ex01_inject_the_clock.rs:26:8
   |
26 |     fn is_valid(&self) -> bool {
   |        ^^^^^^^^
help: remove the extra argument
```

"Remove the extra argument" would make it compile by going back to untestable code. The fix is the other way round: give `is_valid` the parameter.

> 💡 **Analogy: the simulator's clock.** In the simulator the instructor can say "it's now 16:59 on a Friday" or "fast-forward one hour", and the pilot's instruments believe it. Nobody waits for the real Friday.

▶ `cargo test -p advanced_testing --example injected_clock`

## 5. Testing randomness: inject the generator, seed it

Randomness is a dependency like time. Pass the generator in (`rng: &mut impl Rng`), use a truly random one in production and a **seeded** one in tests:

```rust
fn roll_dice(rng: &mut impl Rng, n: usize) -> Vec<u8> {
    (0..n).map(|_| rng.random_range(1..=6)).collect()
}

let mut random = StdRng::from_os_rng();          // production: different every run
let mut seeded = StdRng::seed_from_u64(42);      // tests: the same every run
```

```text
from the OS:  [6, 3, 6, 6, 4, 1, 2, 5]  (different every run)
seed 42:      [1, 4, 2, 4, 6, 4, 6, 3]  (the same every run)
```

- A seeded run is repeatable, so a statistical check ("each face about a sixth of the time in 60 000 rolls") can't flake.
- **Assert properties, not exact values.** `StdRng`'s documentation calls it "non-portable": any future version of rand may change the algorithm, and with it every number. Check "all dice are 1 to 6", "52 different cards", "same seed, same result". If you must pin exact values, use a generator with a fixed algorithm (the `rand_chacha` crate).
- The mini project's `OrderIds::seeded(2026)` makes every order number in the snapshot tests the same on every run.

> 💡 **Analogy: loaded dice for the rehearsal.** For the real game the dice must be fair and unpredictable. For the rehearsal you use dice that **always roll the same sequence**, so when something goes wrong you can play it again and watch exactly what happened.

▶ `cargo test -p advanced_testing --example seeded_randomness`

## 6. Snapshot testing with insta

Some outputs are big: a rendered receipt, an error report, a struct with twenty fields. Writing `assert_eq!` against a hand-typed copy is tedious. A **snapshot test** saves the output once, you **review** it, and from then on any change shows up as a diff:

```rust
use insta::{assert_debug_snapshot, assert_snapshot};

assert_snapshot!(receipt.render());          // saved in snapshots/<module>__<test>.snap
assert_debug_snapshot!(receipt);             // the {:#?} form of a value
assert_snapshot!(text, @"the expected text"); // inline: stored right in the source
```

The first time, there is no saved snapshot, so the test **fails** and insta writes the new output next to where it belongs (real output, shortened):

```text
stored new snapshot …\tests\checkout\snapshots\checkout__receipts__happy_hour_receipt.snap.new
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot: happy_hour_receipt
+new results
────────────┬──────────────────────────────────────────────────────────────────
          1 │+Receipt ORD-BNKD3Y
          2 │+2026-11-16 15:30 UTC
          …
          8 │+Happy hour (10%)                  -£0.91
          9 │+Total                              £8.24
────────────┴──────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review` or set `INSTA_UPDATE=always`
```

- **Review, then accept.** The usual tool is the `cargo insta` command (`cargo install cargo-insta`, not installed in this course): `cargo insta review` shows each pending snapshot and asks accept / reject / skip. Without it, read the `.snap.new` file, and if it's right, rename it to `.snap` (or run once with `INSTA_UPDATE=always`, which writes `.snap` files directly, then read them before committing). That's how this lesson's snapshots were made.
- **What `INSTA_UPDATE` does** (read from insta 1.49's source): the default `auto` writes `.snap.new` files (`.pending-snap` for inline snapshots) on your machine, and writes **nothing** when the `CI` environment variable is set; `always` overwrites the snapshots; `no` never writes.
- **Commit the `.snap` files.** They are the expected output, and reviewing their diff in a pull request is reviewing the change in behaviour.
- **Snapshots must be deterministic.** The receipts use a `FixedClock`, `OrderIds::seeded(2026)` and the fake gateway's predictable `pay_0001`. A real timestamp or random id would change on every run. (insta's `filters` and `redactions` features can also blank out such values.) And a future rand version could change the seeded order ids (section 5): then you review and accept the new snapshots.

When the output changes, the diff shows old (`-`) and new (`+`) lines. Exercise 5's real output:

```text
-old snapshot
+new results
────────────┬──────────────────────────────────────────────────────────────────
    1     1 │ Tea            £2.50
    2       │-Scone          £3.05
          2 │+Scone           £3.5
    3     3 │ Jam            £1.50
    4       │-TOTAL          £7.05
          4 │+TOTAL           £7.5
────────────┴──────────────────────────────────────────────────────────────────
```

> 💡 **Analogy: a photo of the stage set.** After the set is built, someone checks it and takes a **photo**. Before every show, a robot takes a new photo and holds it next to the old one. If a chair has moved, it circles the difference. If the move was on purpose, you file the new photo as the reference; if not, you put the chair back.

▶ `cargo test -p advanced_testing --example snapshot_testing` (the saved snapshots are in `examples/snapshots/`)

## 7. Property testing, recap

[Lesson 03/02](../../03_abstraction/02_automated_tests/README.md) introduced [proptest](https://proptest-rs.github.io/proptest/): describe *any* valid input, state a rule that must always hold, and let it try hundreds of cases. The tools you'll use most:

```rust
prop_compose! {                                   // build a strategy from parts
    fn money_text()(pounds in 0u64..1_000_000, pence in 0u64..100, symbol: bool) -> String {
        format!("{}{pounds}.{pence:02}", if symbol { "£" } else { "" })
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]   // default is 256

    #[test]
    fn shares_differ_by_at_most_a_penny(pence in 0u64..10_000_000, people in 0u32..50) {
        prop_assume!(people > 0);                 // throw this case away
        // …
    }

    #[test]
    fn bad_endings_never_parse(base in money_text(), tail in prop_oneof![Just(".".to_string()), "[a-z]{1,3}"]) {
        prop_assert!(parse_money(&format!("{base}{tail}")).is_err());
    }
}
```

When a property fails, proptest **shrinks** the input and reports a small failing case. The real output from the deliberately buggy bill splitter (it forgets the left-over pence):

```text
Test failed: assertion failed: `(left == right)`
  left: `54`,
 right: `55` at 05_mastery\05_advanced_testing\examples\property_tests.rs:95.
minimal failing input: pence = 55, people = 2
	successes: 0
```

Proptest picks new random cases every run, so your numbers will differ; an earlier run reported `pence = 256, people = 9`. Shrinking finds a **small** case, not always the smallest. Normally proptest also saves the failing case in a `proptest-regressions/` file next to the test, and tries it first next time; commit those files. (This demo turns that off with `failure_persistence: None`, because it fails on purpose.)

The mini project's properties: money always prints and parses back; parsing any text never panics; for any cart, time in 2026 and coupon choice, `total + discount == subtotal`, and the card is charged exactly the total.

> 💡 **Analogy: a hundred random test pilots.** Instead of the three emergencies you planned, a hundred random pilots each try a random flight. When one crashes, the instructor keeps replaying a **simpler** version of that flight (fewer passengers, less wind) until it's as simple as it can get while still crashing.

▶ `cargo test -p advanced_testing --example property_tests` (add `-- --ignored` to watch it shrink)

## 8. Fuzzing

A **fuzzer** goes further than random inputs: it runs your code millions of times, **watches which code each input reaches**, and keeps and mutates the inputs that reach something new. It's very good at crashing parsers: slicing past the end, overflowing arithmetic, `unwrap` on input from outside.

**cargo-fuzz** is the standard tool. It uses LLVM's libFuzzer, needs **nightly** Rust, and (its README says) only works on Unix-like systems, **not Windows**; on Windows run it inside WSL. It's not installed for this course; here's the shape of it:

```bash
cargo install cargo-fuzz        # once
cargo fuzz init                 # creates fuzz/ with a separate fuzz crate
cargo fuzz add parse_money      # creates fuzz/fuzz_targets/parse_money.rs
cargo +nightly fuzz run parse_money
```

```rust
// fuzz/fuzz_targets/parse_money.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    advanced_testing::money::check_parse_money(text);   // must never panic
});
```

**The harness pattern** makes the same check useful without nightly. The [`arbitrary`](https://docs.rs/arbitrary) crate turns raw bytes into typed values (`#[derive(Arbitrary)]` writes the code for your own types), and a typed `fuzz_target!` builds its input with `T::arbitrary_take_rest(Unstructured::new(bytes))`. So write the harness as a plain function of bytes, and call it from an ordinary test that feeds it a **seed corpus** and thousands of seeded random byte strings:

```rust
fn fuzz_money(data: &[u8]) {
    if let Ok(text) = <&str>::arbitrary_take_rest(Unstructured::new(data)) {
        check_parse_money(text);   // 1. never panics  2. what parses prints and parses back
    }
}

#[test]
fn harness_survives_ten_thousand_random_inputs() {
    let mut rng = StdRng::seed_from_u64(0xF022);   // seeded: a failure repeats
    for _ in 0..10_000 { /* random bytes */ fuzz_one(&buffer); }
}
```

(`<&str>::arbitrary(&mut u)`, without `_take_rest`, first reads a **length** from the bytes, so a seed such as `b"12.34"` would not become the text `"12.34"`. Exercise 6's first draft fell into exactly that hole, and its seed corpus "passed" while broken.)

Fuzzing found something real while this lesson was written. The `fuzz_harness` example checks a stock counter against a dead-simple model (an array), and its very first run failed:

```text
assertion `left == right` failed: [Take { item: 109, n: 0 }]
  left: false
 right: true
```

Taking **zero** of an item that was never stocked returned `false`, while the model said "0 ≤ 0, so yes". Neither was a typo: nobody had decided that case. Fuzzers are good at finding the questions nobody asked. (The decision: taking nothing always succeeds.)

> 💡 **Analogy: a toddler in the cockpit.** A test pilot follows a checklist. A toddler presses **every button in every order**, and remembers which presses made a new light come on, so they press around those more. They find the switch combination nobody wrote in the manual. The harness is a **padded cockpit** you can put the toddler in any time: in the real fuzzing lab (cargo-fuzz) or in an ordinary playroom (`cargo test`).

▶ `cargo test -p advanced_testing --example fuzz_harness` and `cargo test -p advanced_testing --test checkout fuzz::`

## 9. Organising the tests of a big project

```text
05_advanced_testing/
├── src/                     unit tests in #[cfg(test)] mod tests, next to the code (24)
│   └── lib.rs               plus doc tests in the /// comments (4)
├── tests/
│   └── checkout/            ONE integration-test binary, named "checkout"
│       ├── main.rs          just `mod` lines
│       ├── support.rs       fixtures, builders, the mock! declaration
│       ├── flows.rs         end-to-end through the public API
│       ├── receipts.rs      snapshot tests
│       ├── snapshots/       the reviewed .snap files
│       ├── properties.rs    proptest
│       └── fuzz.rs          the fuzz harnesses
└── examples/ exercises/ solutions/
```

- **One integration binary.** Every file directly in `tests/` is compiled and **linked as its own program**. With dozens of files, linking dominates. `tests/<name>/main.rs` with modules is one program; Cargo finds it by itself and names it after the folder (`--test checkout`). It also makes shared helpers plain modules, no `tests/common/mod.rs` trick needed.
- **Fixtures with meaningful names**: `monday_happy_hour()`, `coffee_and_cake()`, `welcome_coupon()`. Small **builders** (`Cart::new().with(…)`) keep the arrange step short.
- **Test-only code**: inside one crate, `#[cfg(test)]`. To share fakes with *other* crates' tests, either make them ordinary public items (as `FixedClock` and `FakeGateway` are here, because they're useful in examples too) or put them behind a feature such as `test-support` that test builds switch on.
- **Slow tests** get `#[ignore = "why"]` and run on purpose (`-- --include-ignored`, or a nightly CI job).
- **Test-only crates** go in `[dev-dependencies]`: they're never compiled into the program.

> 💡 **Analogy: one simulator building, many rooms.** You could build a separate building for each training exercise, each with its own power plant (a binary per test file, each linked). Or put all the rooms in **one building** with a shared equipment store (`support.rs`). Same training, far less building.

## 10. Running tests: the options worth knowing

Arguments before `--` go to Cargo; after `--` they go to the test program (libtest).

| Command | What it does |
|---|---|
| `cargo test -p advanced_testing --doc` | only the doc tests |
| `cargo test -p advanced_testing --lib` / `--bins` / `--examples` / `--test checkout` | only that kind of target |
| `cargo test -p advanced_testing receipts::` | only tests whose path contains `receipts::` |
| `cargo test … -- --exact flows::coupon_codes_are_case_insensitive` | exactly that one test |
| `cargo test … -- --skip fuzz --skip properties` | everything except those |
| `cargo test … -- --list` | list tests without running them |
| `cargo test … -- --no-capture` | print output **as it happens** (`--nocapture` still works) |
| `cargo test … -- --show-output` | print captured output of passing tests at the end |
| `cargo test … -- --test-threads=1` | one at a time (for tests that share something) |
| `cargo test … -- --include-ignored` / `--ignored` | run ignored tests too / only them |
| `cargo test --no-fail-fast` | keep going after a test **binary** fails, report all at the end |

Real output:

```text
$ cargo test -p advanced_testing --doc
running 4 tests
test 05_mastery\05_advanced_testing\src\lib.rs - (line 16) ... ok
test 05_mastery\05_advanced_testing\src\clock.rs - clock::Timestamp::from_utc (line 48) ... ok
test 05_mastery\05_advanced_testing\src\money.rs - money::parse_money (line 91) ... ok
test 05_mastery\05_advanced_testing\src\money.rs - money::Money::percent (line 44) ... ok

$ cargo test -p advanced_testing --test checkout receipts::
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.31s
```

`-- --format json` exists but isn't stable: "The "json" format is only accepted on the nightly compiler with -Z unstable-options". For machine-readable results on stable, use nextest.

> 💡 **Analogy: the simulator's control panel.** The same building can run every exercise, only the fire drills (`receipts::`), one named exercise (`--exact`), everything but the long-haul flights (`--skip`), or play the cockpit audio live (`--no-capture`).

## 11. cargo-nextest: a faster, stricter test runner (explained)

[cargo-nextest](https://nexte.st/) replaces the runner, not your tests: the same `#[test]` functions, run with `cargo nextest run`. Its site calls it "up to 3× faster than `cargo test`".

- **Per-test isolation**: each test runs in its own process, so one test's global state or crash can't affect another.
- **Retries** for flaky tests (`--retries 2`), reported as flaky rather than hidden.
- **JUnit XML** output for CI dashboards, and clearer summaries of slow and failing tests.
- **No doc tests**: the site says they're "currently not supported", so CI runs `cargo nextest run` **and** `cargo test --doc`.

It isn't installed for this course; installation instructions for Windows and Linux are on <https://nexte.st/>.

> 💡 **Analogy:** `cargo test` is one big classroom where all the pupils sit the exam together; one pupil fainting disturbs the room. nextest gives **every pupil their own little room**, and lets a pupil who had a nosebleed sit the exam again, while writing that down.

## 12. Coverage with cargo-llvm-cov (explained)

**Coverage** measures which lines your tests ran. [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) uses the compiler's own instrumentation (it needs the `llvm-tools-preview` rustup component):

```bash
cargo llvm-cov -p advanced_testing                 # a summary table in the terminal
cargo llvm-cov -p advanced_testing --open          # an HTML report, opened in the browser
cargo llvm-cov --workspace --lcov --output-path lcov.info   # for a CI coverage service
cargo llvm-cov nextest                             # coverage of a nextest run
```

Doc-test coverage needs nightly (`--doctests` is unstable). Use coverage to find **untested code**, especially untested error paths, not as a score to maximise: 100% coverage with weak assertions proves little, and a mutation-testing tool such as cargo-mutants checks whether your assertions would notice a change.

> 💡 **Analogy: footprints on the floor.** After the rehearsal you sprinkle chalk dust and see **where the actors walked**. Rooms with no footprints were never rehearsed. But footprints don't prove anyone *did* anything useful in the room: that's what the assertions are for.

## The mini project: a checkout service built to be tested

`src/` is a small shop checkout: carts, a happy hour (Monday to Friday 15:00-16:59 UTC, 10% off), coupons that expire, a payment provider, and a printed receipt. Every dependency comes in from outside:

```rust
pub struct Checkout<C, P> { clock: C, payments: P, ids: OrderIds, coupons: Vec<Coupon> }

let mut checkout = Checkout::new(FixedClock::at(monday_happy_hour()), FakeGateway::new(), OrderIds::seeded(2026));
```

| File | What it shows |
|---|---|
| [`src/checkout.rs`](src/checkout.rs) | the rules, `place_order`, `Receipt::render`; 9 unit tests with `MockPaymentGateway` (automock) and `FixedClock` |
| [`src/clock.rs`](src/clock.rs) | `Clock`, `SystemClock`, the `FixedClock` fake, the `&C` blanket impl |
| [`src/payment.rs`](src/payment.rs) | `PaymentGateway`, `#[cfg_attr(test, automock)]`, the `FakeGateway` fake and spy |
| [`src/ids.rs`](src/ids.rs) | order ids from a seedable `StdRng` |
| [`src/money.rs`](src/money.rs) | whole-pence `Money`, a parser that never panics, `check_parse_money` for fuzzers |
| [`tests/checkout/`](tests/checkout/main.rs) | 15 integration tests: flows with a `mock!` gateway and a `Sequence`, 5 snapshots, 3 properties, 2 fuzz harnesses |

```bash
cargo run -p advanced_testing        # production wiring: the real clock, random ids, a fake gateway
cargo test -p advanced_testing       # 24 unit + 15 integration + 4 doc tests
```

```text
--- card tok_visa_4242, coupon Some("welcome20")
Receipt ORD-QBW7K8
2026-11-14 13:35 UTC

2 x Flat white           £3.20     £6.40
1 x Banana bread         £2.75     £2.75

Subtotal                           £9.15
Coupon WELCOME20 (20%)            -£1.83
Total                              £7.32
Paid by card ending 4242 (pay_0002)
--- card tok_visa_4242, coupon Some("SPRING99")
order refused: no such coupon: SPRING99
--- card tok_broke_0000, coupon None
order refused: payment failed: card declined: insufficient funds
```

Things worth reading in the tests: `invalid_carts_never_reach_the_payment_gateway` (`never()`), `a_coupon_works_until_the_second_it_expires` (a borrowed `FixedClock` moved forward by one second), `the_order_id_is_the_idempotency_key` (`withf`), and `totals_are_never_more_than_the_subtotal` (a property over any cart, any time in 2026).

> 💡 **Analogy: the whole simulator, switched on.** The checkout is the cockpit. The instructor's desk has a clock dial (`FixedClock`), a pretend bank (`FakeGateway`), a robot bank clerk with a checklist (the mocks), loaded dice for order numbers (`OrderIds::seeded`), a camera on the receipt printer (snapshots) and a toddler let loose on the keypad (the fuzz harness). In production the same cockpit is wired to the real sky.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `SystemTime::now()` / `rand::rng()` inside the logic | tests can't control it; flaky or impossible | Inject a `Clock` / `&mut impl Rng`. |
| Added a parameter, then followed the compiler's "remove the extra argument" | E0061 goes away, the testability with it | Keep the parameter, update the callers. |
| A spy/fake with `&self` methods tries `self.calls.push(…)` | E0596 cannot borrow as mutable | `RefCell<Vec<…>>` and `borrow_mut()`. |
| Used `MockX` (from `#[cfg_attr(test, automock)]`) in `tests/` | E0432 unresolved import: it only exists in the library's own test build | `mockall::mock!` in the integration test. |
| `use mockall::automock;` in non-test code | E0432 unresolved import `mockall` (a dev-dependency isn't there in a normal build) | `#[cfg_attr(test, mockall::automock)]`, or `#[cfg(test)] use`. |
| Mocks for everything, including values | tests break on every refactor | Mock the calls that *are* the behaviour; fakes for the rest. |
| Exact values from a seeded `StdRng` | breaks when rand changes the algorithm | Assert properties, or use `rand_chacha`. |
| Edited a `.snap` file by hand to make it pass | the snapshot no longer reflects the code | Fix the code, or accept the new output after review. |
| Didn't commit `snapshots/*.snap` | every CI run fails "missing snapshot" | Commit them; review their diffs. |
| A timestamp or random id in a snapshot | different every run | `FixedClock`, a seed, or insta filters/redactions. |
| Left `.snap.new` / `.pending-snap` files lying around | clutter, confusing reviews | `cargo insta review`/`reject`, or delete them. |
| `<&str>::arbitrary(&mut u)` for a whole input | seeds don't map to the text you meant | `arbitrary_take_rest`, as `fuzz_target!` does. |
| `&text[dot + 1..dot + 3]` on outside input | panic: end byte index … out of bounds | `text.get(range)`, `split_once`, check lengths. |
| `pounds * 100` on outside input | panic in debug: attempt to multiply with overflow | `checked_mul` / `checked_add`. |
| `cargo fuzz run` on Windows | cargo-fuzz doesn't support Windows | WSL, or the harness-as-a-test pattern. |
| `cargo test -- --format json` | only accepted on the nightly compiler | nextest's JUnit output. |
| Only `cargo nextest run` in CI | doc tests silently not run | Also `cargo test --doc`. |
| Chasing 100% coverage | lines run, nothing checked | Coverage finds gaps; assertions find bugs. |

## Practise

**1. Run the examples.** Each has a `main` (`cargo run`) and tests (`cargo test`):

```bash
cargo test -p advanced_testing --example test_design
cargo test -p advanced_testing --example hand_rolled_doubles
cargo test -p advanced_testing --example mockall_basics
cargo test -p advanced_testing --example mockall_basics -- --ignored    # a mock failing
cargo test -p advanced_testing --example injected_clock
cargo test -p advanced_testing --example seeded_randomness
cargo test -p advanced_testing --example snapshot_testing
cargo test -p advanced_testing --example property_tests
cargo test -p advanced_testing --example property_tests -- --ignored    # shrinking
cargo test -p advanced_testing --example fuzz_harness
cargo run -p advanced_testing --example fuzz_harness
```

**2. Do the exercises** in `exercises/`. Several are "write the test code":

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_inject_the_clock` | Make it compile (E0061): inject a `Clock` | `cargo test -p advanced_testing --features exercises --bin ex01_inject_the_clock` |
| 2 | `ex02_hand_written_fake` | Write a fake mailer (`RefCell`), pass 3 tests | `cargo test -p advanced_testing --features exercises --bin ex02_hand_written_fake` |
| 3 | `ex03_mock_catches_double_charge` | Make it pass: mockall says "called 2 times"; fix the code | `cargo test -p advanced_testing --features exercises --bin ex03_mock_catches_double_charge` |
| 4 | `ex04_write_the_mocks` | Write 3 tests with mockall expectations | `cargo test -p advanced_testing --features exercises --bin ex04_write_the_mocks` |
| 5 | `ex05_snapshot_a_receipt` | Make it pass: read insta's diff, fix the formatting | `cargo test -p advanced_testing --features exercises --bin ex05_snapshot_a_receipt` |
| 6 | `ex06_fuzz_finds_a_crash` | Make it pass: the fuzz harness crashes the parser, twice | `cargo test -p advanced_testing --features exercises --bin ex06_fuzz_finds_a_crash` |

Exercise 6 starts like this (real output):

```text
thread 'tests::fuzz_seed_corpus' (1572) panicked at 05_mastery\05_advanced_testing\exercises\ex06_fuzz_finds_a_crash.rs:21:42:
end byte index 3 is out of bounds for string of length 1
…
thread 'tests::fuzz_ten_thousand_random_inputs' (22872) panicked at 05_mastery\05_advanced_testing\exercises\ex06_fuzz_finds_a_crash.rs:21:42:
end byte index 6 is out of bounds for string of length 5
```

Exercise 5 leaves a `.ex05_snapshot_a_receipt.rs.pending-snap` file in `exercises/` each time it fails (section 6); delete it when you're done.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p advanced_testing --features solutions --bins`).

**4. The mini project:** run it, run its tests, then extend it test-first: add a "3 for 2" offer on any line with quantity 3 or more. Write the unit test (with a mock gateway expecting the right amount) and a snapshot of the receipt first, watch them fail, then implement it. Finally, add the offer to the property test: `total + discount == subtotal` must still hold.

## Checklist

- [ ] Test design: names, arrange/act/assert, one behaviour per test, edges, table-driven tests
- [ ] Test doubles by hand: dummy, stub, spy, fake, mock; dependency injection with traits
- [ ] mockall: `automock`, `mock!`, `with`/`withf`, `times`/`never`, `returning`, `Sequence`
- [ ] Testing time: a `Clock` trait, `FixedClock`, the `&C` blanket impl
- [ ] Testing randomness: inject the `Rng`, seed it, assert properties not exact values
- [ ] Snapshot testing with insta: file and inline snapshots, `.snap.new`, `INSTA_UPDATE`, reviewing
- [ ] Property testing recap: `prop_compose!`, `prop_oneof!`, `prop_assume!`, config, shrinking, regression files
- [ ] Fuzzing: cargo-fuzz explained, `arbitrary`, `arbitrary_take_rest`, harnesses that also run as tests
- [ ] Test organisation: one integration binary, `support` module, fixtures, test-only code, `#[ignore]`
- [ ] `cargo test` options: `--doc`, filters, `--exact`, `--skip`, `--list`, `--no-capture`, `--show-output`, `--no-fail-fast`
- [ ] nextest and coverage with cargo-llvm-cov: what they do and when to reach for them
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Good tests read like a specification: behaviour names, arrange/act/assert, one rule each, the edges covered.
- Code is testable when its dependencies come **in** through traits. Then a test can pass a stub, spy, fake or mock, written by hand or generated by mockall.
- Time and randomness are dependencies too: inject a `Clock` and an `Rng`, and tests can freeze time and replay the dice.
- Snapshots check big outputs against a reviewed copy; properties and fuzzing hunt for inputs you didn't think of, and an `arbitrary` harness runs under cargo-fuzz and `cargo test` alike.
- Big projects keep integration tests in one binary with shared support code, and add nextest and coverage when they need them.

**In one sentence for a child:** we test our code in a flight simulator, where we choose the weather, freeze the clock, roll the same dice twice, photograph the dashboard and let a toddler press every button, so nothing surprises us when it flies for real.

**Next:** [06 Observability](../06_observability/README.md)
