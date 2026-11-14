<!-- hand-written lesson: gen_course must not overwrite -->
# Guessing Game

**The Rust Book, chapter 2:** [Programming a Guessing Game](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html)
· Crates: [`rand`](https://docs.rs/rand/0.9), [`colored`](https://docs.rs/colored/3)

Chapter 2 jumps straight in and builds a whole program: the computer picks a secret number from 1 to 100, you guess, and it says "Too small!", "Too big!" or "You win!". It's a whirlwind tour (variables, input, crates, `match`, loops, error handling), and every idea comes back later in more depth. Here, the aim is to see them work together.

> 💡 **The big picture: a game show.** The **host** (the program) writes a secret number on a card and hides it. Each round, the **contestant** shouts a guess into a **microphone** (`read_line`), the host checks what they said is really a number (`parse`), compares it with the card (`cmp`), and holds up one of three signs: **"Too small"**, **"Too big"** or **"You win"** (`match`). The show repeats rounds (`loop`) until someone wins (`break`).

```bash
cargo run -p guessing_game                       # play it
seq 1 100 | cargo run -p guessing_game           # bash: let a "robot" guess 1, 2, 3, ...
cargo test -p guessing_game
```

## 1. Setting up the project

The Book starts with `cargo new guessing_game`, exactly like lesson 01. In this course the project already exists as the workspace member `01_foundations/02_guessing_game`, so you run it with `-p guessing_game`.

> 💡 **Analogy:** before the show can start, someone builds the **empty studio**: a stage, a camera and a script that just says "Hello, world!". `cargo new` builds the studio; the rest of this chapter writes the show.

## 2. Processing a guess

```rust
use std::io;

let mut guess = String::new();
io::stdin()
    .read_line(&mut guess)
    .expect("Failed to read line");
println!("You guessed: {guess}");
```

- **`use std::io;`** brings the input/output library into scope. A small set of very common items (the **prelude**) is always in scope; everything else you `use`.
- **`let`** creates a variable. Variables are **immutable by default**; **`let mut`** makes one you can change (lesson 03 goes deeper).
- **`String::new()`** makes a new, empty, growable `String`. The **`::`** says `new` is an *associated function* of the `String` type, a function that belongs to the type itself, not to one particular string.
- **`read_line(&mut guess)`** reads what the user types and **appends** it to `guess`. The `&` makes a **reference**, which lends `guess` to `read_line` without giving it away. References are immutable by default too, hence `&mut` (lesson 06 is all about this).
- **`.expect(...)`**: `read_line` returns a **`Result`**, an enum with two variants: `Ok` (it worked, here holding the number of bytes read) or `Err` (it failed, holding the reason). `expect` gives you what's inside `Ok`, or **crashes** the program with your message if it was an `Err`.
- **`{}` placeholders**: `println!("x = {x}")` prints a variable by name; `println!("{} and {}", a, b + 1)` fills empty braces in order, and the values can be expressions.

If you leave off `.expect`, it still compiles, but Rust warns you that you've ignored a possible failure:

```text
warning: unused `Result` that must be used
  |
4 |     io::stdin().read_line(&mut guess);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this `Result` may be an `Err` variant, which should be handled
```

> 💡 **Analogy: the microphone and the notepad.** `String::new()` is a **blank notepad**. `read_line(&mut guess)` hands the notepad to the microphone operator and says "write down what they say" (`&mut`: you may write on it, but give it back). The operator returns a **ticket**, `Result`, saying either "done, I wrote 3 letters" or "the microphone broke". `expect` means "if the microphone broke, stop the show and announce this message".

▶ `cargo run -p guessing_game --example read_input` (shows the invisible newline that Enter adds)

## 3. Generating a secret number

The standard library has no random numbers, so we use a **crate**: a package of Rust code someone else published on [crates.io](https://crates.io). Add it under `[dependencies]` in `Cargo.toml`:

```toml
[dependencies]
rand = "0.9"
```

`"0.9"` is **semantic versioning** shorthand for `^0.9`: any version at least 0.9.0 and below 0.10.0, so you get bug fixes but nothing that would break your code. On the next build, Cargo downloads `rand` **and the crates `rand` depends on**, then compiles them all.

- **`Cargo.lock`** records the exact version Cargo picked (here `rand 0.9.5`). Every later build uses exactly that version, even if a newer 0.9.x appears, until you choose to move. That makes builds **reproducible**. In this workspace there is one `Cargo.lock` at the repo root, shared by every lesson.
- **`cargo update`** ignores the lock file, picks the newest versions that `Cargo.toml` still allows (0.9.x, never 0.10), and writes them to `Cargo.lock`. Moving to 0.10 means editing `Cargo.toml` yourself. (The root `Cargo.lock` already holds `rand 0.10` too, pulled in by `proptest` in lesson 03/02: one workspace can use two incompatible versions of a crate side by side.)
- **`cargo doc --open -p guessing_game`** builds documentation for your crate **and all of its dependencies**, and opens it in your browser. It's the quickest way to find out what a crate can do.

```rust
use rand::Rng;

let secret_number = rand::rng().random_range(1..=100);
```

- **`rand::rng()`** gives a random number generator, seeded by the operating system, that belongs to the current thread.
- **`random_range(1..=100)`**: `start..=end` is a range that **includes** the end; `1..100` would stop at 99.
- **`use rand::Rng;`**: `random_range` is a method of the **`Rng` trait** (a trait is a named set of methods; chapter 10). A trait's methods only work when the trait is in scope. Forget the `use` and the method seems not to exist. The real error from exercise 2:

```text
error[E0599]: no method named `random_range` found for struct `ThreadRng` in the current scope
   |
10 |     let roll = rand::rng().random_range(1..=6);
   |                            ^^^^^^^^^^^^
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `Rng` which provides `random_range` is implemented but not in scope; perhaps you want to import it
   |
 9 + use rand::Rng;
   |
```

**The Book uses rand 0.8**, where the same line is `rand::thread_rng().gen_range(1..=100)`. In 0.9 the names changed to `rand::rng()` and `random_range`, partly because `gen` became a reserved word in the 2024 edition.

> 💡 **Analogy: the corner shop and the receipt.** A **crate** is a ready-made ingredient from the shop, so you don't have to grow your own wheat. `Cargo.toml` is your shopping list ("rand, any 0.9"). `Cargo.lock` is the **receipt** with the exact brand and batch you bought, so next week you buy the same one. `cargo update` is going back to the shop for the newest batch of the *same* product. And `use rand::Rng` is **opening the instruction leaflet**: the dice could always roll, but you have to read the leaflet before you know the "roll" button is there.

▶ `cargo run -p guessing_game --example random_numbers` (also shows a *seeded* generator that repeats the same numbers every run)

## 4. Comparing the guess to the secret number

```rust
use std::cmp::Ordering;

match guess.cmp(&secret_number) {
    Ordering::Less => println!("Too small!"),
    Ordering::Greater => println!("Too big!"),
    Ordering::Equal => println!("You win!"),
}
```

- **`cmp`** compares two values and returns an **`Ordering`**, an enum with three variants: `Less`, `Greater` and `Equal`.
- **`match`** is made of **arms**: a *pattern*, `=>`, and the code to run. Rust tries the arms in order and runs the first one that fits. The compiler checks that **every** possible value is covered; leave one out (exercise 3) and it won't compile:

```text
error[E0004]: non-exhaustive patterns: `std::cmp::Ordering::Equal` not covered
   |
15 |     match guess.cmp(&secret_number) {
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^ pattern `std::cmp::Ordering::Equal` not covered
```

At this point the Book's program doesn't compile either, because `guess` is a `String` and `secret_number` is a number. Rust has a **strong, static type system** and never compares a string with a number. The real error from exercise 1:

```text
error[E0308]: mismatched types
    |
 17 |     match guess.cmp(&secret_number) {
    |                 --- ^^^^^^^^^^^^^^ expected `&String`, found `&u32`
    |                 |
    |                 arguments to this method are incorrect
    |
    = note: expected reference `&String`
               found reference `&u32`
```

(The full message goes on to mention `Iterator::cmp`, a different method with the same name. Ignore it: the first lines tell you what's wrong.)

The fix is to turn the text into a number:

```rust
let guess: u32 = guess.trim().parse().expect("Please type a number!");
```

- **Shadowing:** this is a **new** variable called `guess` that hides the old one. It can even have a different type. It's ideal for converting a value from one type to another while keeping the name.
- **`trim()`** removes whitespace at both ends, including the newline Enter added: `"5\n"` (or `"5\r\n"` on Windows) becomes `"5"`.
- **`parse()`** converts a string into another type. Which type? The **annotation** `: u32` (an unsigned, 32-bit integer) tells it. `parse` can fail ("A👍%" isn't a number), so it returns a `Result`, and `expect` crashes if it's an `Err`.
- Because `guess` is now a `u32`, Rust **infers** that `secret_number` is a `u32` too, so the comparison is between two values of the same type.

> 💡 **Analogy: the three signs.** `cmp` looks at the guess and the secret card and picks up one of exactly **three signs**: "less", "greater", "equal". `match` is the rule sheet saying what to announce for each sign, and the compiler won't let the show start unless the rule sheet covers all three. Comparing a `String` with a number is like comparing the **word** "fifty" written on paper with **50 sweets** in a jar: first you have to read the word and count out that many (`parse`).

▶ `cargo run -p guessing_game --example ordering_and_match`
▶ `cargo run -p guessing_game --example parse_and_shadow` (what `parse` says about `""`, `"seven"`, `"-5"` and `"99999999999"`)

## 5. Allowing multiple guesses with looping

```rust
loop {
    // ... read the guess ...
    let guess: u32 = match guess.trim().parse() {
        Ok(num) => num,
        Err(_) => continue,
    };

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => {
            println!("You win!");
            break;
        }
    }
}
```

- **`loop`** repeats its body forever.
- **`break`** leaves the loop. After a correct guess, there's nothing left in `main`, so the program ends.
- **Handling invalid input:** instead of crashing with `expect`, `match` on the `Result` that `parse` returns. `Ok(num) => num` hands the number out, and `Err(_) => continue` skips the rest of this round and asks again. The **`_`** is a catch-all: "any error, I don't care which".

> 💡 **Analogy: rounds of the show.** `loop` is "play another round". `continue` is the host saying "that wasn't a number, **this round doesn't count**, try again". `break` is the **confetti cannon**: someone won and the show is over.

▶ `cargo run -p guessing_game --example loop_break_continue` (the whole game loop with scripted guesses, so you can watch every branch)

## Beyond the Book: what our version adds

`src/main.rs` is the Book's final program with a few upgrades. Each one uses a feature that a later lesson explains fully.

### Constants and a match guard

```rust
const MIN: u32 = 1;
const MAX: u32 = 100;

fn parse_guess(input: &str) -> Option<u32> {
    match input.trim().parse() {
        Ok(num) if (MIN..=MAX).contains(&num) => Some(num),
        _ => None,
    }
}
```

- **`const`** names a value that never changes, so the range is written once (lesson 03).
- **`if ...` after a pattern is a match guard**: the arm only matches if the pattern fits **and** the condition is true. The Book's version happily accepts a guess of 500; ours says "Enter a whole number from 1 to 100".
- Pulling the check into its own **function** lets the tests in `src/main.rs` try it without typing anything. `Option` (`Some(number)` or `None`) is lesson 02/02.

> 💡 **Analogy:** a match guard is a **bouncer with a second question**. "Are you a number?" isn't enough to get in; "...and are you between 1 and 100?" has to be yes too.

### Colours with the `colored` crate

```rust
use colored::Colorize;
println!("{}", "Too small!".red());
println!("{}", format!("You win in {attempts} attempt(s)!").green());
```

`Colorize` is an **extension trait**: importing it adds methods like `.red()`, `.bold()` and `.on_blue()` to every string. Colours only appear in a real terminal: when the output is piped, or the `NO_COLOR` environment variable is set, `colored` prints plain text.

> 💡 **Analogy:** importing `Colorize` hands every string a **box of crayons**. The words don't change; they just get coloured in when somebody is watching on a screen.

▶ `cargo run -p guessing_game --example colored_output`

### When the input runs out

`read_line` returns `Ok(0)` when there's **no more input**: Ctrl+D on Linux, Ctrl+Z then Enter on Windows, or the end of piped input. The Book's program doesn't check, so it reads an empty line, fails to parse it, `continue`s, and loops forever. Ours checks the byte count and ends the game:

```text
$ printf '50\n' | cargo run -p guessing_game
Guess the number between 1 and 100!
Please input your guess.
You guessed: 50
Too small!
Please input your guess.
No more input. The number was 100.
```

> 💡 **Analogy:** if the contestant **walks off stage**, the host doesn't keep asking an empty chair for a guess forever. They reveal the card and end the show.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Compared the `String` from `read_line` with a number | E0308 mismatched types | `let guess: u32 = guess.trim().parse()...` |
| `.random_range(...)` without `use rand::Rng;` | E0599 no method named `random_range` | `use rand::Rng;` |
| `thread_rng()` / `gen_range` from the Book | not found in rand 0.9 | `rand::rng()` / `random_range` |
| `match` missing an `Ordering` variant | E0004 non-exhaustive patterns | Add the arm (better than `_`). |
| `let guess = String::new();` then `read_line(&mut guess)` | E0596 cannot borrow as mutable | `let mut guess` |
| `let guess = guess.trim().parse().expect(...)` with nothing else to go on | E0284 type annotations needed | `let guess: u32 = ...` |
| Parsed without `trim()` | runtime: `invalid digit found in string` | `.trim()` removes `\n` / `\r\n`. |
| Ignored the `Result` from `read_line` | warning: unused `Result` | `.expect("...")`, or handle it. |
| `continue` on bad input with no end-of-input check | endless loop when input is piped | Stop when `read_line` returns `Ok(0)`. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p guessing_game --example read_input
cargo run -p guessing_game --example random_numbers
cargo run -p guessing_game --example parse_and_shadow
cargo run -p guessing_game --example ordering_and_match
cargo run -p guessing_game --example loop_break_continue
cargo run -p guessing_game --example colored_output
```

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_compare_string` | Make it compile (E0308), the Book's way | `cargo run -p guessing_game --features exercises --bin ex01_fix_compare_string` |
| 2 | `ex02_fix_missing_trait` | Make it compile (E0599), one line | `cargo run -p guessing_game --features exercises --bin ex02_fix_missing_trait` |
| 3 | `ex03_fix_missing_arm` | Make it compile (E0004), no `_` | `cargo run -p guessing_game --features exercises --bin ex03_fix_missing_arm` |
| 4 | `ex04_parse_in_range` | Implement, pass 4 tests | `cargo test -p guessing_game --features exercises --bin ex04_parse_in_range` |
| 5 | `ex05_hint` | Implement with `cmp` and `match`, pass 3 tests | `cargo test -p guessing_game --features exercises --bin ex05_hint` |
| 6 | `ex06_warmer_colder` | Implement, pass 3 tests | `cargo test -p guessing_game --features exercises --bin ex06_warmer_colder` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p guessing_game --features solutions --bins`, or run one with `--features solutions --bin sol01_fix_compare_string`).

**4. Play the mini project**, the finished game in `src/main.rs`:

```bash
cargo run -p guessing_game                  # play: try "abc", 0, 500, then real guesses
seq 1 100 | cargo run -p guessing_game      # bash: a robot guesses 1, 2, 3, ... and wins
printf '50\n' | cargo run -p guessing_game  # one guess, then the input ends politely
cargo test -p guessing_game                 # 5 tests of parse_guess and the secret's range
```

The game reads the keyboard, so the tests check the **logic** (`parse_guess`) instead of the typing. That's a habit worth keeping: put decisions in small functions and test those.

> 💡 **Analogy:** you don't test a game show by sitting in the audience every time. You test the **rule card** ("is 500 a valid guess?") on its own, and trust the show to read from it.

## Checklist

- [ ] [Processing a guess](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html#processing-a-guess)
- [ ] [Generating a secret number](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html#generating-a-secret-number)
- [ ] [Comparing the guess to the secret number](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html#comparing-the-guess-to-the-secret-number)
- [ ] [Allowing multiple guesses with looping](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html#allowing-multiple-guesses-with-looping)
- [ ] Beyond the Book: constants and a match guard to validate the range
- [ ] Beyond the Book: colours with the `colored` crate (an extension trait)
- [ ] Beyond the Book: ending cleanly when `read_line` returns `Ok(0)`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `let` makes a variable, `let mut` a changeable one. `String::new()` is an associated function. `&mut guess` lends a variable so it can be changed.
- Things that can fail return a `Result` (`Ok` or `Err`). `expect` takes the `Ok` value or crashes with your message; `match` handles both.
- Crates go in `[dependencies]`. `Cargo.lock` pins exact versions, and `cargo update` moves within the allowed range. A trait must be `use`d before its methods work.
- `cmp` returns an `Ordering`, and `match` must cover every case. Shadowing turns the typed text into a number with the same name.
- `loop` repeats, `continue` skips to the next round, and `break` ends the loop.

**In one sentence for a child:** the computer hides a number, you shout guesses, and it holds up "too small", "too big" or "you win" until you get it right.

**Next:** [03 Variables and Data Types](../03_variables_and_data_types/README.md) (Book 3.1–3.2)
