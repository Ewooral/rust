<!-- hand-written lesson: gen_course must not overwrite -->
# Iterators and Closures

**The Rust Book, chapter 13:** [Functional Language Features: Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
· Rust by Example: [Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html), [Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html)
· [`Iterator` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html) (the full list of adapters)

Two features borrowed from functional languages make up most idiomatic Rust:
- **Closures** are small anonymous functions you can store in variables, pass to other functions and return. Unlike ordinary functions, they can **capture** values from where they're written.
- **Iterators** process a sequence of items one at a time, through chains of small steps like `filter`, `map` and `sum`.

> 💡 **The big picture: a factory conveyor belt.** An **iterator** is a conveyor belt that delivers items one at a time. **Adapters** (`map`, `filter`, …) are machines placed along the belt. **Closures** are the little instruction cards you hand each machine ("keep only the red ones", "double it"), and a card can carry notes from your desk (captured values). The belt is **lazy**: it doesn't move until someone at the end starts taking items off (`collect`, `sum`, a `for` loop).

## 1. Closures

```rust
fn  add_one_v1   (x: u32) -> u32 { x + 1 }   // a function
let add_one_v2 = |x: u32| -> u32 { x + 1 };  // a closure, fully annotated
let add_one_v3 = |x|             { x + 1 };  // types inferred
let add_one_v4 = |x|               x + 1  ;  // one expression: no braces needed
```

- **Types are inferred** from how a closure is used, then **fixed**. Calling `|x| x` with a `String` and then with a number gives E0308.
- **Closures can use variables from their surroundings.** A `fn` can't: it gives E0434, "can't capture dynamic environment in a fn item".
- The Book's shirt giveaway: `user_preference.unwrap_or_else(|| self.most_stocked())`. The closure captures `self` and only runs if it's needed.

> 💡 **Analogy: an instruction card with notes on it.** A function is a recipe in a cookbook: it only knows what you hand it. A closure is a **sticky note** you write on the spot. It can mention things on your desk right now ("use **this** greeting"), because it was written there.

▶ `cargo run -p iterators_and_closures --example closures`

### Capturing: borrow, borrow mutably, or move

A closure captures each value in the **least powerful** way its body needs:

| Body does | Captures | Meanwhile, you can still |
|---|---|---|
| reads `list` | `&list` | read `list` too |
| changes `list` | `&mut list` (and the closure must be `mut`) | nothing, until the closure's last use (E0502) |
| `move \|\| …` | the value itself | nothing: it moved (E0382), except `Copy` values, which are copied |

`move` is needed when the closure may **outlive** the current function, as with a thread. Without it, `thread::spawn` gives E0373: "closure may outlive the current function, but it borrows `list`".

> 💡 **Analogy:** looking at a **photo** of your toy (`&`), lending the toy to a friend to **paint** (`&mut`): you can't touch it until they're done, or **packing the toy in their backpack** to take home (`move`).

▶ `cargo run -p iterators_and_closures --example capturing`

### `FnOnce`, `FnMut` and `Fn`

What the body does with captured values decides which traits a closure implements, and functions use those traits to say how they'll call it:

| Trait | Can be called | The body | Example API |
|---|---|---|---|
| `FnOnce` | once (all closures) | may **move** captured values out | `Option::unwrap_or_else` |
| `FnMut` | many times | may **change** captured values | `sort_by_key`, `iter().for_each` |
| `Fn` | many times | changes nothing | concurrent callbacks, `make_adder` |

The real error from exercise 1, where a closure gives away its captured `String` and is then called twice:

```text
error[E0382]: use of moved value: `make_label`
   |
13 |     let first = make_label();
   |                 ------------ `make_label` moved due to this call
14 |     let second = make_label();
   |                  ^^^^^^^^^^ value used here after move
note: closure cannot be invoked more than once because it moves the variable `name` out of its environment
```

**Returning closures:** each closure has its own unnamed type. Return `impl Fn(i32) -> i32` for a single one, or `Box<dyn Fn(i32) -> i32>` when different branches return different closures.

> 💡 **Analogy:** `FnOnce` is a **single-use ticket**: once it's used, it's gone. `FnMut` is a **stamp card** that changes every time you use it. `Fn` is a **library card**: use it as often as you like and it never changes.

▶ `cargo run -p iterators_and_closures --example fn_traits`

## 2. Iterators

```rust
let v1 = vec![1, 2, 3];
let v1_iter = v1.iter();      // creating it does nothing yet: iterators are lazy
for val in v1_iter { println!("Got: {val}"); }
```

The whole `Iterator` trait rests on **one** required method:

```rust
pub trait Iterator {
    type Item;                                   // what it produces (an associated type)
    fn next(&mut self) -> Option<Self::Item>;    // Some(item), or None when finished
    // ...plus about 75 provided methods built on next()
}
```

| Method | Yields | The collection afterwards |
|---|---|---|
| `iter()` | `&T` | still yours |
| `iter_mut()` | `&mut T` | changed in place |
| `into_iter()` | `T` | consumed (moved) |

- **Consuming adapters** call `next` until the end: `sum`, `count`, `collect`, `max`, `fold`.
- **Iterator adapters** return a *new* iterator and do nothing on their own: `map`, `filter`, `zip`, `take`. If you forget to consume one, you get this warning (an error in exercise 2):

```text
error: unused `Map` that must be used
   |
14 |     prices.iter().map(|p| p * 2);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: iterators are lazy and do nothing unless consumed
```

- **Adapters take closures, and closures capture.** `shoes.into_iter().filter(|s| s.size == shoe_size)` uses the function's `shoe_size` parameter.

> 💡 **Analogy:** `next()` is pressing the button on a **sweet dispenser**: one sweet comes out each press, until it's empty (`None`). `iter()` lets you **look** at each sweet, `iter_mut()` lets you **decorate** each one, and `into_iter()` lets you **eat** them, after which the jar is empty. A `map` with nobody collecting at the end is a machine standing beside a belt that never moves.

▶ `cargo run -p iterators_and_closures --example iterators`

## 3. Improving minigrep (the mini project)

`src/` is chapter 12's minigrep, rewritten the way Book section 13.3 does it:

```rust
// Before (ch12): index into a collected Vec and clone each String.
pub fn build(args: &[String]) -> Result<Config, &'static str> { /* args[1].clone() … */ }

// After (ch13): take the iterator itself and OWN each String it yields.
pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
    args.next();                                             // skip the program name
    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query string"),
    };
    let file_path = args.next().ok_or("Didn't get a file path")?;
    // …
}
```

and `main` passes `env::args()` straight in. `search` loses its mutable `Vec` and its loop:

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}
```

That's less code, with no `clone` and no mutable state to get wrong. Beyond the Book, the mini project also adds a `--summary` mode: one iterator pipeline that counts lines and words and finds the most frequent long word.

```bash
cd 03_abstraction/04_iterators_and_closures
cargo run -- to poem.txt
cargo run -- --summary poem.txt
cargo test -p iterators_and_closures
```

> 💡 **Analogy:** chapter 12's `build` photocopied the order slip and read lines off the copy (`clone`, indexing). Chapter 13's `build` just takes the **original slips off the spike one at a time** (`next()`), and keeps them.

## 4. Performance: loops vs iterators

Iterators are a **zero-cost abstraction**: the compiler turns a chain like `zip().map().sum()` into the same tight machine code as a hand-written loop, and sometimes better, for example by unrolling the 12-step loop in the Book's audio-decoder example. Choose whichever reads better; there's no speed penalty for iterators.

To compare, measure in **release** mode. A debug build skips the optimisations entirely, and timings vary with machine load, so run it more than once:

▶ `cargo run -p iterators_and_closures --example performance --release`

> 💡 **Analogy:** the conveyor belt is only on the **blueprint**. When the factory is actually built (compiled with optimisations), the belt and machines are melted down into one smooth set of moves, exactly what a worker doing it by hand would do.

## Beyond the Book: your own iterators

Implement `next`, and you get every adapter for free (`zip`, `skip`, `filter`, `sum`, …):

```rust
struct Counter { count: u32 }
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.count < 5 { self.count += 1; Some(self.count) } else { None }
    }
}
```

- **`IntoIterator`** lets your type work in a `for` loop. Implement it for `Playlist` (`for song in playlist`, consuming) and for `&Playlist` (`for song in &playlist`, borrowing).
- **Endless iterators** like `Fibonacci` never return `None`, so the caller stops them with `take` or `take_while`.
- **`std::iter`** helpers build iterators without a struct: `successors`, `from_fn`, `repeat`, `once`.

> 💡 **Analogy:** writing your own iterator is **building your own sweet dispenser**. You only design the button (`next`), and every machine in the factory already fits it.

▶ `cargo run -p iterators_and_closures --example custom_iterators`

## Beyond the Book: the adapter toolbox

| Adapter | Does | Example |
|---|---|---|
| `zip` | walks two sequences side by side | names with scores |
| `enumerate` | adds an index | `(0, "Ama")` |
| `filter_map` | filters and maps in one, keeping the `Some`s | parse what parses |
| `flat_map` / `flatten` | each item becomes several, joined | the letters of every name |
| `fold` | combines everything into one value | building a report string |
| `scan` | like `fold`, but yields every step | running totals |
| `windows` / `chunks` (slices) | overlapping pairs / fixed-size groups | day-to-day changes |
| `peekable` | look at the next item without taking it | a tokenizer |
| `take_while`, `skip_while`, `chain`, `rev`, `step_by` | trim, join, reverse, stride | |
| `any`, `all`, `find`, `position`, `max_by_key`, `partition` | ask questions, split in two | |
| `collect` | into `Vec`, `String`, `HashMap`, even `Result<Vec<_>, _>` | |

> 💡 **Analogy:** `fold` is a **snowball** rolling down the belt, picking up every item. `scan` is the same snowball, with a photo taken at every step. `peekable` is **peeking through the window** at the next parcel before deciding whether to take it.

▶ `cargo run -p iterators_and_closures --example adapter_toolbox`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Called an `FnOnce` closure twice | E0382 use of moved value | Don't move the capture out: clone it, or return a reference. |
| `iter().map(…)` with nothing after it | warning: unused `Map` | Finish with `collect`, `sum` or `for_each`. |
| Used a variable while a closure mutably borrows it | E0502 | Finish using the closure first. |
| `thread::spawn(\|\| …)` using a local | E0373 | `move \|\| …` |
| `fn` inside a function using a local variable | E0434 | Use a closure. |
| Called one closure with two different types | E0308 | One closure has one type. Use a generic `fn` instead. |
| `collect()` with no type | E0283 type annotations needed | `let v: Vec<_> = …` or `.collect::<Vec<_>>()` |
| Pushed a captured `String` inside `sort_by_key` | E0507 | `sort_by_key` needs `FnMut`: don't move captures out. |
| `for x in vec` then used `vec` | E0382 | `for x in &vec` |
| An endless iterator with no `take` | runs forever | `take(n)` or `take_while(…)` |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p iterators_and_closures --example closures
cargo run -p iterators_and_closures --example capturing
cargo run -p iterators_and_closures --example fn_traits
cargo run -p iterators_and_closures --example iterators
cargo run -p iterators_and_closures --example custom_iterators
cargo run -p iterators_and_closures --example adapter_toolbox
cargo run -p iterators_and_closures --example performance --release
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_fnonce` | Make it compile (E0382): turn `FnOnce` into `Fn` | `cargo run -p iterators_and_closures --features exercises --bin ex01_fix_fnonce` |
| 2 | `ex02_fix_lazy_iterator` | Make it compile and **work** (lazy `map`) | `cargo run -p iterators_and_closures --features exercises --bin ex02_fix_lazy_iterator` |
| 3 | `ex03_fix_mutable_capture` | Make it compile (E0502) by reordering | `cargo run -p iterators_and_closures --features exercises --bin ex03_fix_mutable_capture` |
| 4 | `ex04_iterator_pipelines` | Four functions, no loops allowed, pass 4 tests | `cargo test -p iterators_and_closures --features exercises --bin ex04_iterator_pipelines` |
| 5 | `ex05_custom_iterator` | `impl Iterator` for `Countdown`, pass 3 tests | `cargo test -p iterators_and_closures --features exercises --bin ex05_custom_iterator` |
| 6 | `ex06_closures_as_values` | Return and compose closures, pass 4 tests | `cargo test -p iterators_and_closures --features exercises --bin ex06_closures_as_values` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p iterators_and_closures --features solutions --bins`).

**4. The mini project:** minigrep v2 (section 3). Put `src/lib.rs` side by side with `../03_minigrep_cli_project/src/lib.rs` and spot every loop and `clone` that disappeared.

## Checklist

- [ ] [13.1 Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [ ] [13.2 Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [ ] [13.3 Improving Our I/O Project](https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html)
- [ ] [13.4 Performance in Loops vs. Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html)
- [ ] Beyond the Book: implementing `Iterator` and `IntoIterator` for your own types
- [ ] Beyond the Book: adapter toolbox: `zip`, `enumerate`, `flat_map`, `fold`, `scan`, `windows`, `chunks`, `peekable`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Closures (`|x| x + 1`) are anonymous functions that capture their surroundings by `&`, `&mut`, or by value with `move`.
- `FnOnce`, `FnMut` and `Fn` describe how a closure can be called. APIs choose the weakest one they need.
- An iterator only needs `next()`. Adapters are lazy, and consuming methods drive them. `iter`, `iter_mut` and `into_iter` decide whether you borrow, change or take the items.
- Iterator chains replace loops, indexes and `clone`s (minigrep v2) at zero runtime cost.
- Implement `Iterator` and `IntoIterator` for your own types, and you get the whole adapter toolbox for free.

**In one sentence for a child:** an iterator is a conveyor belt that hands you things one at a time, and closures are the little instruction cards you give the machines along it.

**Next:** [05 Cargo and Crates.io](../05_cargo_and_crates_io/README.md) (Book chapter 14)
