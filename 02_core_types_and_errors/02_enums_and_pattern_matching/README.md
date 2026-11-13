<!-- hand-written lesson: gen_course must not overwrite -->
# Enums and Pattern Matching

**The Rust Book, chapter 6:** [Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
· Rust by Example: [Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html), [match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html)

A struct says "this AND that AND that". An **enum** says "this OR that OR that": a value that is exactly **one** of a fixed list of possibilities, called **variants**. Add `match`, which handles each variant and checks that you handled *all* of them, and you have the tool Rust programs use most to make wrong states impossible.

> 💡 **The big picture: a multiple-choice question.** An enum is a question with a fixed list of answers, and every value ticks **exactly one** box. A struct is a form where you fill in *every* box, while an enum is a quiz where you tick *one*. `match` is the teacher marking it: there must be a reaction written down for every possible answer, or the marking scheme is rejected.

## 1. Defining an enum

```rust
enum IpAddrKind { V4, V6 }

let four = IpAddrKind::V4;      // variants are namespaced with ::
fn route(kind: IpAddrKind) {}   // one type covers both
```

Variants can **carry data**, and each variant can carry a different kind:

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}
let home = IpAddr::V4(127, 0, 0, 1);  // the variant name works like a constructor

enum Message {
    Quit,                       // no data
    Move { x: i32, y: i32 },    // named fields, like a struct
    Write(String),              // one value
    ChangeColor(i32, i32, i32), // several values, like a tuple
}
```

With four separate structs, you could not write one function that accepts any message. With an enum, `Message` is **one type**, so a `Vec<Message>` can hold all four shapes. Enums get methods with `impl`, just like structs.

> 💡 **Analogy: "Other: please specify".** Some answers on a form are just a tick ("Quit"). Some come with a line to write on ("Write: ______"), or several lines ("Move to x: __ y: __"). Every answer can have its own lines, but you still tick only one. Whichever answer was ticked, the sheet is the same sheet, which is why one `Vec<Message>` can hold them all.

▶ `cargo run -p enums_and_pattern_matching --example defining_enums`

## 2. `Option<T>`: no more null

Rust has no `null`. The idea of "a value that might be missing" is an ordinary enum from the standard library:

```rust
enum Option<T> {
    None,
    Some(T),
}
```

It is so common that `Some` and `None` can be used without the `Option::` prefix. `<T>` means it works for any type (generics, lesson 03/01).

```rust
let some_number = Some(5);            // Option<i32>
let absent_number: Option<i32> = None; // a type annotation is needed for None
```

The key point: **an `Option<i8>` is not an `i8`**, so you cannot accidentally use a value that might be missing:

```text
error[E0277]: cannot add `Option<u32>` to `u32`
   |
16 |         let total = salary + bonus;
   |                            ^ no implementation for `u32 + Option<u32>`
```

(This is the real error from exercise 2.) To get at the value, you must decide what happens when it is `None`, with a `match`, `if let`, or a method like `unwrap_or`. `.unwrap()` crashes the program on `None`, so keep it for tests and quick experiments.

> 💡 **Analogy: a gift box.** `Option<Toy>` is a box that **might** have a toy inside (`Some(toy)`) or might be empty (`None`). In languages with `null`, you are handed what *looks* like a toy and only find out it is air when you try to play with it: the program crashes. In Rust, you are handed the **box**, and you can't play with a box. You have to open it and say what you will do if it is empty.

▶ `cargo run -p enums_and_pattern_matching --example option`

## 3. `match`

`match` compares a value against **patterns**, top to bottom, and runs the first arm that fits. The whole `match` is an expression, so it produces the value of the arm that ran:

```rust
fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => {          // an arm can be a block
            println!("Lucky penny!");
            1
        }
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

`match` works on any type, not just enums: `1 | 2` means "1 or 2", and `3..=9` is a range.

> 💡 **Analogy: a coin-sorting machine.** Drop a coin in the top and it rolls down until it falls through the first slot it fits. Each slot leads to its own tray (the arm's code). The coin only ever goes into **one** slot.

▶ `cargo run -p enums_and_pattern_matching --example match_basics`

### Patterns that bind values

A pattern can take the data *out* of a variant and give it a name:

```rust
match coin {
    Coin::Quarter(state) => println!("State quarter from {state:?}!"),
    Coin::Penny => /* … */,
    // …
}
match msg {
    Message::Move { x, y } => /* use x and y */,
    Message::Write(text) => /* use text */,
    // …
}
```

> 💡 **Analogy:** the quarter slot in the coin sorter has a little window that **reads the picture** on the back of the coin as it passes ("Alaska!"). The slot doesn't just sort the coin; it also hands you what's printed on it.

**Watch ownership.** Matching on a value **moves** any non-`Copy` data you bind out of it. Match on a **reference** (`match &msg`) to borrow instead. The real error from exercise 3:

```text
error[E0382]: borrow of partially moved value: `msg`
   |
18 |         Message::Write(text) => println!("text has {} bytes", text.len()),
   |                        ---- value partially moved here
19 |     }
20 |     println!("the message was {msg:?}");
   |                                ^^^ value borrowed here after partial move
help: borrow this binding in the pattern to avoid moving the value
   |
18 |         Message::Write(ref text) => println!("text has {} bytes", text.len()),
   |                        +++
```

Both fixes work. `match &msg` is the usual style today, and `ref` is the older way of saying the same thing.

▶ `cargo run -p enums_and_pattern_matching --example patterns_that_bind`

### Matching `Option<T>`

```rust
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}
```

▶ `cargo run -p enums_and_pattern_matching --example matching_option`

### Matches are exhaustive

A `match` must cover **every** possible value. Forget one and the program does not compile. Here is the real error from exercise 1, where a variant was added and the `match` was not updated:

```text
error[E0004]: non-exhaustive patterns: `Direction::West` not covered
   |
18 |     match d {
   |           ^ pattern `Direction::West` not covered
...
14 |     West,
   |     ---- not covered
```

This is why you cannot forget the `None` case of an `Option`. It is also why adding a variant to an enum is safe: the compiler points you to every `match` that needs updating.

> 💡 **Analogy:** the factory refuses to build a coin sorter that is missing a slot. A coin with no slot would jam the whole machine at runtime, so the fault is caught on the drawing board instead.

### Catch-all patterns

```rust
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    other => move_player(other), // matches anything, and binds it
}
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    _ => reroll(),               // matches anything, does not bind
}
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    _ => (),                     // do nothing
}
```

The catch-all must come **last**, because arms are tried in order. Use catch-alls on numbers and strings, which have too many values to list. On your **own** enums, prefer listing every variant (exercise 1 asks why).

> 💡 **Analogy:** a catch-all is the "everything else" bucket at the bottom of the sorter. `other` is a bucket you look into. `_` is a bucket you never look into. Put a bucket at the very top, and no coin ever reaches the real slots.

▶ `cargo run -p enums_and_pattern_matching --example catch_all`

## 4. `if let` and `let ... else`

When you only care about **one** pattern, a full `match` is noisy. `if let` handles one pattern and ignores the rest:

```rust
if let Some(max) = config_max {
    println!("The maximum is configured to be {max}");
}

if let Coin::Quarter(state) = coin {
    println!("State quarter from {state:?}!");
} else {
    count += 1;
}
```

You trade exhaustiveness checking for brevity: the compiler won't remind you about the other cases.

> 💡 **Analogy: a magnet.** `match` sorts every coin into a tray. `if let` is a magnet you wave over the pile: it picks out the one kind you're looking for, and leaves everything else where it is (or, with `else`, sweeps the rest into one pile).

**`let ... else`** binds a value if the pattern matches. If it doesn't, the `else` block must **leave**: `return`, `break`, `continue` or panic. The rest of the function stays flat, without nesting:

```rust
fn describe_state_quarter(coin: Coin) -> Option<String> {
    let Coin::Quarter(state) = coin else {
        return None;
    };
    // from here on, `state` is available and un-indented
    if state.existed_in(1900) { /* … */ }
}
```

> 💡 **Analogy: a bouncer at the door.** "No ticket? Go home." Everyone who gets past the bouncer definitely has a ticket, so nobody inside needs to check again. The party (the rest of the function) carries on without any "if you have a ticket" around it.

▶ `cargo run -p enums_and_pattern_matching --example if_let_and_let_else`

## Beyond the Book: `Option` combinators and `?`

Writing a `match` for every `Option` gets long. `Option` has methods for the common shapes:

| Method | Does | Example |
|---|---|---|
| `map(f)` | Change the value inside `Some`, leave `None` alone | `age.map(\|a\| a + 1)` |
| `and_then(f)` | Chain a step that can itself return `None` | `user.and_then(\|u\| u.nickname)` |
| `filter(p)` | Keep `Some` only if the test passes | `age.filter(\|a\| *a < 13)` |
| `unwrap_or(v)` | Fallback value | `bonus.unwrap_or(0)` |
| `unwrap_or_else(f)` | Fallback computed only if needed | `.unwrap_or_else(\|\| name.clone())` |
| `ok_or(e)` | Turn it into a `Result` (lesson 02/05) | `age.ok_or("unknown")` |
| `is_some()` / `is_none()` | Just ask | `nick.is_some()` |
| `as_ref()` / `as_deref()` | Look inside without moving: `Option<&T>` / `Option<&str>` | `name.as_deref()` |

Inside a function that returns `Option`, **`?`** means "if this is `None`, return `None` now, otherwise give me the value":

```rust
fn nickname_length(users: &[User], name: &str) -> Option<usize> {
    let user = users.iter().find(|u| u.name == name)?;
    let nick = user.nickname.as_ref()?;
    Some(nick.len())
}
```

`matches!(value, pattern)` is a quick true/false pattern test: `matches!(light, TrafficLight::Green)`.

> 💡 **Analogy: working on the gift without opening the box.** `map` is a machine that says "if there's a toy inside, paint it red. If the box is empty, pass the empty box along." `and_then` says "if there's a toy, swap it for whatever's in *its* box". `?` is "if the box is empty, stop and go home with an empty box".

▶ `cargo run -p enums_and_pattern_matching --example option_combinators`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| A `match` missing a variant | E0004 | Add the arm. On your own enums, list it instead of reaching for `_`. |
| Used an `Option<T>` as if it were a `T` | E0277 / E0308 | `match`, `if let`, `unwrap_or`, or `?` |
| Used a value after matching it and binding a `String` | E0382 (partial move) | `match &value` (or `ref` in the pattern) |
| `IpAddr::V4` written as `V4` | E0425 | Prefix with the enum name, or `use IpAddr::*;` |
| Catch-all `_` placed before other arms | warning: unreachable pattern | Put the catch-all last. |
| `let Some(x) = opt;` with no `else` | E0005 refutable pattern | `if let`, `let ... else`, or `match` |
| `.unwrap()` everywhere | panics at runtime | Handle `None`, or use `?` |
| Arms returning different types | E0308 | Every arm must produce the same type. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p enums_and_pattern_matching --example defining_enums
cargo run -p enums_and_pattern_matching --example option
cargo run -p enums_and_pattern_matching --example match_basics
cargo run -p enums_and_pattern_matching --example patterns_that_bind
cargo run -p enums_and_pattern_matching --example matching_option
cargo run -p enums_and_pattern_matching --example catch_all
cargo run -p enums_and_pattern_matching --example if_let_and_let_else
cargo run -p enums_and_pattern_matching --example option_combinators
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_non_exhaustive` | Make it compile (E0004), no `_` | `cargo run -p enums_and_pattern_matching --features exercises --bin ex01_fix_non_exhaustive` |
| 2 | `ex02_fix_option_arithmetic` | Make it compile (E0277), no `.unwrap()` | `cargo run -p enums_and_pattern_matching --features exercises --bin ex02_fix_option_arithmetic` |
| 3 | `ex03_fix_match_move` | Make it compile (E0382), no `.clone()` | `cargo run -p enums_and_pattern_matching --features exercises --bin ex03_fix_match_move` |
| 4 | `ex04_traffic_light` | Implement, pass 3 tests | `cargo test -p enums_and_pattern_matching --features exercises --bin ex04_traffic_light` |
| 5 | `ex05_shapes` | Implement, pass 2 tests | `cargo test -p enums_and_pattern_matching --features exercises --bin ex05_shapes` |
| 6 | `ex06_parse_command` | Implement a parser returning `Option`, pass 3 tests | `cargo test -p enums_and_pattern_matching --features exercises --bin ex06_parse_command` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p enums_and_pattern_matching --features solutions`).

**4. Read the mini project** in `src/main.rs` (`cargo run -p enums_and_pattern_matching`, `cargo test -p enums_and_pattern_matching`): a parcel tracker.
- Each `Status` variant carries only the data that makes sense for it. A parcel out for delivery has an ETA, and a delivered one has an optional signature.
- One `match` on the pair `(status, event)` handles every transition, and the compiler checks that none is forgotten.

> 💡 **Analogy:** the parcel is a counter on a board game. It is on exactly one square at a time, each square has its own card (the data), and every event card tells you which square to move to. Moves that make no sense, like "deliver" while still at "ordered", leave the counter where it is.

## Checklist

- [ ] [6.1 Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
- [ ] [6.2 The match Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html)
- [ ] [6.3 Concise Control Flow with if let and let...else](https://doc.rust-lang.org/book/ch06-03-if-let.html)
- [ ] Beyond the Book: `Option` combinators: `map`, `and_then`, `filter`, `ok_or`, `unwrap_or_else`
- [ ] Beyond the Book: `?` on `Option`, and `matches!`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- An enum value is exactly one of its variants, and each variant can carry its own data.
- `Option<T>` replaces null: `Some(value)` or `None`, and the compiler makes you handle both.
- `match` is exhaustive, so every case must be handled. Patterns can bind the data inside a variant. Match on `&value` to borrow instead of move.
- `other` and `_` are catch-alls and go last. `if let` handles one pattern, and `let ... else` binds a value or leaves early.
- `map`, `and_then`, `unwrap_or` and `?` handle `Option` without writing a `match` every time.

**In one sentence for a child:** an enum is a question where you tick exactly one answer, and `match` is a coin sorter that must have a slot for every kind of coin.

**Next:** [03 Packages, Crates and Modules](../03_packages_crates_modules/README.md) (Book chapter 7)
