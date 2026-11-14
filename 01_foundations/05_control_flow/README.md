<!-- hand-written lesson: gen_course must not overwrite -->
# Control Flow

**The Rust Book, chapter 3:** [Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html), section [3.5 Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
· Rust by Example: [Flow of Control](https://doc.rust-lang.org/rust-by-example/flow_control.html)

So far, programs have run from top to bottom, once. **Control flow** decides *whether* a piece of code runs (`if`) and *how many times* (`loop`, `while`, `for`). Rust's versions look familiar, with two twists: conditions must be real `bool`s, and both `if` and `loop` are **expressions** that can produce a value.

> 💡 **The big picture: a train on a railway.** The program is a train running along the track. An **`if`** is a set of **points** (a junction): the signal (a `bool`) decides which track the train takes. A **loop** is a **circular track**: the train goes round and round until something switches it off the circle (`break`). A **label** is a name painted on a circle, so that when there are circles inside circles, you can say exactly which one to leave.

## 1. `if` expressions

```rust
let number = 3;

if number < 5 {
    println!("condition was true");
} else {
    println!("condition was false");
}
```

- The condition doesn't need brackets, but the body always needs `{ }`.
- `else` is optional. Without it, nothing happens when the condition is false.
- **The condition must be a `bool`.** Rust never treats a number as true or false. The real error from exercise 1:

```text
error[E0308]: mismatched types
   |
12 |     if unread_messages {
   |        ^^^^^^^^^^^^^^^ expected `bool`, found integer
```

Write what you mean, such as `if unread_messages > 0` or `if number != 0`. It's clearer for readers, and it can never accidentally be true.

> 💡 **Analogy: railway points.** The signal at a junction only shows **green or red** (a `bool`). You can't set it to "3": what would that mean? So Rust makes you say "green if there are more than 0 messages".

### Several conditions with `else if`

```rust
if number.is_multiple_of(4) {
    println!("divisible by 4");
} else if number.is_multiple_of(3) {
    println!("divisible by 3");
} else if number.is_multiple_of(2) {
    println!("divisible by 2");
} else {
    println!("not divisible by 4, 3, or 2");
}
```

Rust checks each condition **in order** and runs only the **first** block whose condition is true, then skips the rest. With 6, only "divisible by 3" prints, even though 6 is divisible by 2 as well. So **order matters**: put the most specific check first (exercise 4, FizzBuzz, is all about this). A long chain of `else if` is often clearer as a `match` (lesson 02/02).

> 💡 **Analogy: a queue of doors.** You walk along a corridor of doors, trying each handle in turn. You go through the **first** one that opens, and never even try the others.

▶ `cargo run -p control_flow --example if_else`

## 2. `if` in a `let` statement

Because `if` is an expression, it can choose a value:

```rust
let number = if condition { 5 } else { 6 };
```

Each branch is a block, and its last expression is its value. A variable has exactly one type, so **both branches must give the same type**. The real error from exercise 2:

```text
error[E0308]: `if` and `else` have incompatible types
   |
11 |     let lives = if hard_mode { 1 } else { "three" };
   |                                -          ^^^^^^^ expected integer, found `&str`
   |                                |
   |                                expected because of this
```

This also fixes a classic bug: declaring a variable first and setting it inside an `if`. If one path forgets to set it, Rust refuses to use it. The real error from exercise 3:

```text
error[E0381]: used binding `advice` is possibly-uninitialized
   |
12 |     let advice;
   |         ------ binding declared here but left uninitialized
13 |     if temperature > 20 {
   |        ---------------- if this `if` condition is `false`, `advice` is not initialized
14 |         advice = "wear a T-shirt";
15 |     }
   |      - an `else` arm might be missing here, initializing `advice`
16 |     println!("Today: {advice}");
   |                       ^^^^^^ `advice` used here but it is possibly-uninitialized
```

The neatest fix is one `let advice = if ... { ... } else { ... };`, so every path gives a value in one place.

> 💡 **Analogy: a vending machine with two buttons.** Whichever button you press, something drops into the **same tray**, so it has to fit the tray: you can't have one button give a can and the other a pizza. And the tray must **never be empty**, so a machine with only one working button isn't allowed.

▶ `cargo run -p control_flow --example if_in_let`

## 3. Repetition with loops

Rust has three loops: `loop`, `while` and `for`.

### `loop`: repeat until told to stop

`loop` runs its body forever, until a **`break`**. (If a program gets stuck in a loop, **Ctrl+C** stops it.) **`continue`** skips the rest of this round and starts the next one.

**`break` can carry a value out of the loop**, which makes the whole `loop` an expression:

```rust
let mut counter = 0;
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;
    }
};
// result is 20
```

This is perfect for "keep trying until it works, then tell me the answer". (`return` would leave the whole *function*; `break` leaves only the loop.)

> 💡 **Analogy: a circular track.** The train goes round and round. **`continue`** is "skip the rest of this lap, start the next one". **`break`** is switching the points to **leave the circle**, and `break value` is leaving with a **parcel** for whoever is waiting at the station.

▶ `cargo run -p control_flow --example loop_break_value`

### Loop labels

With loops inside loops, `break` and `continue` apply to the **innermost** loop. A **label**, written `'name:` before a loop, lets you pick which one:

```rust
'counting_up: loop {
    loop {
        if remaining == 9 {
            break;               // leaves the inner loop
        }
        if count == 2 {
            break 'counting_up;  // leaves the OUTER loop
        }
        // ...
    }
}
```

Labels start with a single quote. They're ideal for searching a grid: when the inner loop finds the answer, `break 'outer;` stops **both** loops at once (exercise 6). `continue 'outer;` skips to the next round of the outer loop.

> 💡 **Analogy: circles inside circles.** A toy railway has a small circle inside a big one. "Leave the circle!" is confusing: which one? So you **paint a name** on the big circle, `'counting_up`, and say "leave `'counting_up`". The train comes off both circles at once.

▶ `cargo run -p control_flow --example loop_labels`

### `while`: repeat while a condition is true

```rust
let mut number = 3;
while number != 0 {
    println!("{number}!");
    number -= 1;
}
println!("LIFTOFF!!!");
```

`while` checks the condition **before** every round, and stops as soon as it's false. It replaces a `loop` with an `if`, `else` and `break` inside it. Use it when you don't know in advance how many rounds there will be (exercise 5).

You *can* walk through an array with `while index < a.len()`, but it's fragile: get the condition wrong and it panics, and every `a[index]` is bounds-checked. That's what `for` is for.

> 💡 **Analogy: "while the light is green, keep driving".** You look at the light **before** each block of road. As soon as it turns red, you stop.

▶ `cargo run -p control_flow --example while_loop`

### `for`: once for each item

```rust
let a = [10, 20, 30, 40, 50];
for element in a {
    println!("the value is: {element}");
}

for number in (1..4).rev() {
    println!("{number}!");     // 3!, 2!, 1!
}
println!("LIFTOFF!!!");
```

`for` visits every item of a collection, with no index to get wrong and nothing to go out of bounds. It's the loop Rustaceans use most, even to repeat something a number of times: `for _ in 0..3`. `1..4` is a **range** from 1 up to, but not including, 4, and `.rev()` reverses it.

> 💡 **Analogy: handing out sweets.** `for` is walking along a **row of children** and giving each one a sweet. You don't count positions or worry about walking past the end of the row; you just visit each child once.

▶ `cargo run -p control_flow --example for_loops`

## Beyond the Book: ranges and `for` helpers

| Write | Gives | Use it for |
|---|---|---|
| `1..4` | 1, 2, 3 | counting up to, but not including, the end (like indexes `0..len`) |
| `1..=4` | 1, 2, 3, 4 | counting up to and including the end |
| `(1..4).rev()` | 3, 2, 1 | counting down |
| `(0..60).step_by(15)` | 0, 15, 30, 45 | every nth number |
| `a.iter().enumerate()` | (0, &a[0]), (1, &a[1]), ... | when you need the position **and** the item |
| `for _ in 0..3` | three rounds | repeating, when you don't need the number |

```rust
let podium = ["Ama", "Kofi", "Esi"];
for (place, name) in podium.iter().enumerate() {
    println!("{}. {name}", place + 1);   // 1. Ama, 2. Kofi, 3. Esi
}
```

These are **iterator adapters**; chapter 13 (lesson 03/04) explains how they work and adds many more (`map`, `filter`, `sum`, ...).

> 💡 **Analogy:** a range is a **ruler**. `..` stops just before the last mark, and `..=` includes it. `.rev()` reads the ruler backwards, `.step_by(15)` only looks at every 15th mark, and `.enumerate()` sticks a numbered label on each item as it goes past.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `if count { ... }` with a number | E0308 expected `bool`, found integer | `if count > 0` / `if count != 0` |
| `if` and `else` giving different types | E0308 `if` and `else` have incompatible types | Make both branches give the same type. |
| Set a variable in an `if` with no `else`, then used it | E0381 possibly-uninitialized | `let x = if ... { ... } else { ... };` |
| `break 'outer` with no `'outer:` label | E0426 use of undeclared label | Put `'outer:` before the loop you mean. |
| `break value` inside a `for` or `while` | E0571 `break` with value from a `for` loop | Only `loop` can break with a value. Use a `mut` variable, or `loop`. |
| `else if` checks in the wrong order | none, but a wrong answer | Most specific condition first (FizzBuzz: 15 before 3 and 5). |
| `while i <= a.len()` then `a[i]` | panic: index out of bounds | `for item in a` |
| `for i in 1..10` expecting 10 | none, but stops at 9 | `1..=10` |
| A `loop` with no `break` | runs forever | Add a `break`; stop a stuck program with Ctrl+C. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p control_flow --example if_else
cargo run -p control_flow --example if_in_let
cargo run -p control_flow --example loop_break_value
cargo run -p control_flow --example loop_labels
cargo run -p control_flow --example while_loop
cargo run -p control_flow --example for_loops
```

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_non_bool_condition` | Make it compile (E0308) | `cargo run -p control_flow --features exercises --bin ex01_fix_non_bool_condition` |
| 2 | `ex02_fix_if_else_types` | Make it compile (E0308) | `cargo run -p control_flow --features exercises --bin ex02_fix_if_else_types` |
| 3 | `ex03_fix_uninitialized` | Make it compile (E0381), one `if` expression | `cargo run -p control_flow --features exercises --bin ex03_fix_uninitialized` |
| 4 | `ex04_fizzbuzz` | `if` / `else if` order, pass 3 tests, then a `for` loop | `cargo test -p control_flow --features exercises --bin ex04_fizzbuzz` |
| 5 | `ex05_collatz_steps` | A `while` loop, pass 3 tests | `cargo test -p control_flow --features exercises --bin ex05_collatz_steps` |
| 6 | `ex06_find_in_grid` | Nested loops and a label, pass 3 tests | `cargo test -p control_flow --features exercises --bin ex06_find_in_grid` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p control_flow --features solutions --bins`, or run one with `--features solutions --bin sol01_fix_non_bool_condition`).

**4. Read the mini project** in `src/main.rs`. The Book ends chapter 3 by suggesting three programs to write, and here they all are, plus a puzzle for loop labels:

```bash
cargo run -p control_flow
cargo test -p control_flow       # 8 tests
```

| Part | Book suggestion | Control flow it shows |
|---|---|---|
| 1. Temperatures | convert between Fahrenheit and Celsius | `for` over `(-10..=40).step_by(10)`, an `if` / `else if` chain as a return value |
| 2. Fibonacci | generate the nth Fibonacci number | an early `return`, a `for _ in 1..n` loop |
| 3. The Twelve Days of Rust | print a cumulative song, using its repetition | nested `for` loops, `.rev()`, an `if` for the "and" |
| 4. Pythagorean triples | (ours) | `break 'search` out of two loops |

The Book suggests the traditional carol "The Twelve Days of Christmas". Ours keeps the same cumulative shape, where each verse adds a gift and then counts back down to the first, with Rust-themed gifts:

```text
On the third day of Rust, the compiler gave to me:
three Cargo crates
two borrowed slices
and a crab named Ferris
```

> 💡 **Analogy:** the song is a **stack of presents**. Each day you put a new present on top, then read the whole pile from top to bottom. That's `(1..=day).rev()`. The very bottom present gets an "and", but only once there's more than one.

## Checklist

- [ ] [3.5 Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [ ] Beyond the Book: ranges and `for` helpers: `..=`, `.rev()`, `.step_by()`, `.enumerate()`
- [ ] The Book's three suggested programs, in `src/main.rs`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `if` needs a real `bool`. `else if` runs only the first true branch, so order matters.
- `if` is an expression: `let x = if c { a } else { b };`, with the same type in both branches.
- `loop` repeats until `break`, and `break value` makes the loop produce a value. `continue` skips to the next round.
- Labels (`'outer:`) let `break` and `continue` target an outer loop.
- `while` repeats while a condition holds. `for` visits each item of a collection or range, and is the safest and most common loop.

**In one sentence for a child:** `if` is a junction where a green-or-red signal picks the track, and loops are circular tracks the train goes round until someone switches it off the circle.

**Next:** [06 Ownership](../06_ownership/README.md) (Book chapter 4)
