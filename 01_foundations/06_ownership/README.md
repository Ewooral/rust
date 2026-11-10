<!-- hand-written lesson: gen_course must not overwrite -->
# Ownership

**The Rust Book, chapter 4:** [Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
· Rust by Example: [Scoping rules](https://doc.rust-lang.org/rust-by-example/scope.html)

Ownership is the idea that makes Rust different from every language you have used before. It is how Rust guarantees memory safety **without a garbage collector**, and every later chapter (structs with references, lifetimes, iterators, closures, smart pointers, threads, async) builds on it. Take your time here.

> 💡 **The big picture: toys in a playroom.** Every toy belongs to exactly one child. A child can **give** a toy away (then it is not theirs any more), or **lend** it (they get it back). When a child goes home, their toys are tidied away. A strict teacher (the compiler) checks every rule *before* playtime starts, so nobody ever fights over a toy or grabs one that was already put away. Keep this picture in mind: every section below is one part of it.

## Why ownership exists

Every program has to manage memory. Languages pick one of three approaches:

| Approach | Examples | Cost |
|---|---|---|
| Garbage collector finds unused memory at runtime | Java, C#, Python, Go | Runtime overhead, pauses |
| The programmer allocates and frees by hand | C, C++ (manually) | Leaks, double frees, use-after-free |
| **Ownership rules checked at compile time** | **Rust** | You learn the rules; **zero** runtime cost |

If your program breaks a rule, it does not compile. That is the whole deal.

> 💡 **Analogy: who tidies the playroom?** A garbage collector is a cleaner who walks around *during* playtime picking up toys nobody seems to be using; play pauses while they work. Manual memory is "everyone tidies their own toys", and somebody always forgets (a leak) or puts the same toy away twice (a double free). Rust is a rule the teacher checks *before* play begins: if the plan says every toy gets put away exactly once, you may play. No cleaner, no forgetting.

## Stack and heap, in one minute

- The **stack** holds values of a known, fixed size: `i32`, `bool`, `char`, `(i32, f64)`. Pushing and popping is fast, and values vanish automatically when a function returns.
- The **heap** holds data whose size is unknown at compile time or can change, like the text in a `String` or the items in a `Vec`. Allocating means asking for space and getting back a pointer.

> 💡 **Analogy: a stack of plates and a cloakroom.** The **stack** is a pile of plates: you can only add or take from the top, which is very fast, and every plate is the same known size. The **heap** is a cloakroom: you hand in a coat of any size, and you get back a small **ticket** with a number on it. The ticket (the pointer) is small and lives on the stack; the coat (the data) lives in the cloakroom. Ownership is all about who holds the ticket.

Ownership is mostly about **heap** data: making sure every allocation is freed exactly once, and never used after it is freed.

## The three rules

> 1. Each value in Rust has an **owner**.
> 2. There can only be **one owner at a time**.
> 3. When the owner goes **out of scope**, the value is **dropped** (freed).

Everything else in this chapter follows from these three lines.

> 💡 **Analogy:** 1. Every toy has a child who owns it. 2. A toy belongs to only one child at a time; no sharing ownership. 3. When that child goes home, their toy is tidied away.

## 1. Scope and drop

A variable is valid from where it is declared until the end of its scope (the closing `}`). At that point Rust calls `drop` and frees the value. You never call `free` yourself.

> 💡 **Analogy: a hotel room.** The `{` is check-in and the `}` is check-out. When you check out, housekeeping clears the room automatically; you never have to clean it yourself, and you cannot get back in afterwards. Rooms inside rooms (inner scopes) are cleared first, like a child's room before the parents' suite.

```rust
{
    let s = String::from("hello"); // s is valid from here
    // use s
}                                  // scope over: s is dropped, memory freed
```

▶ `cargo run -p ownership --example scope_and_drop` prints exactly when each value is dropped. Notice the inner value is dropped first.

## 2. Move

For simple `Copy` values, assignment copies:

```rust
let x = 5;
let y = x;      // both x and y are 5
```

A `String` is three values on the stack (pointer, length, capacity) pointing to text on the heap. Assignment copies those three values but **not** the heap text:

```text
   s1 (stack)              heap               s2 (stack)
 ┌──────────┬──────┐    ┌───┬───┬───┬───┬───┐    ┌──────────┬──────┐
 │ ptr      │  ●───┼───▶│ h │ e │ l │ l │ o │◀───┼──●       │ ptr  │
 │ len      │  5   │    └───┴───┴───┴───┴───┘    │  5       │ len  │
 │ capacity │  5   │                             │  5       │ cap  │
 └──────────┴──────┘                             └──────────┴──────┘
     (invalid after the move)
```

If both `s1` and `s2` stayed valid, both would try to free the same memory at the end of the scope: a **double free**. So Rust says `s1` has been **moved** into `s2`, and `s1` can no longer be used.

> 💡 **Analogy: handing over the cloakroom ticket.** `let s2 = s1;` gives your coat ticket to a friend. The coat never moves; only the little ticket changes hands, which is why a move is so cheap. Now your friend owns the coat and *you have no ticket*, so you cannot collect it. If Rust let you both keep a ticket, you would both try to collect the same coat at closing time: that is the double free. Here is the real error from exercise 1:

```text
error[E0382]: borrow of moved value: `name`
   |
13 |     let name = String::from("Ferris");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
14 |     print_greeting(name);
   |                    ---- value moved here
15 |     println!("{name} is the Rust mascot");
   |                ^^^^ value borrowed here after move
```

Rust **never** makes a deep copy automatically, so moves are always cheap.

▶ `cargo run -p ownership --example move_semantics`

## 3. Clone and Copy

- **`.clone()`** makes an explicit deep copy, including the heap data. It can be expensive, and seeing `.clone()` in code is your signal that work is happening.
- **`Copy`** types are copied implicitly on assignment because copying them is trivial: integers, floats, `bool`, `char`, shared references `&T`, and tuples or arrays made only of `Copy` types.

A type can be `Copy` only if all its fields are `Copy` and it does not implement `Drop`. That is why `#[derive(Clone, Copy)]` works on `struct Point { x: i32, y: i32 }` but not on a struct holding a `String`.

> 💡 **Analogy: telling vs photocopying.** `Copy` is telling a friend your age: now you both know "9", it costs nothing, and nobody lost anything. `.clone()` is photocopying a whole book: you each end up with a full book, but someone had to stand at the photocopier, so you only do it on purpose.

▶ `cargo run -p ownership --example clone_and_copy`

## 4. Ownership and functions

Passing a value to a function works exactly like assignment: it **moves** (or copies, for `Copy` types). Returning a value moves ownership back out to the caller.

> 💡 **Analogy: posting a parcel.** Calling `takes_ownership(s)` is posting your toy to a friend. Once it is in the post it is theirs, and when they are done with it *they* throw it away. If the function returns it, they posted it back. Posting toys back and forth all day is tiring, which is why the next section exists.

```rust
fn takes_ownership(s: String) { /* ... */ }  // s is dropped at the end
let s = String::from("hello");
takes_ownership(s);
// s is no longer valid here
```

Handing ownership in and out of every function (returning tuples like `(String, usize)`) quickly gets tedious. That is the problem references solve.

▶ `cargo run -p ownership --example ownership_and_functions`

## 5. References and borrowing

A **reference** (`&T`) lets you use a value without taking ownership. Creating one is called **borrowing**. When the reference goes away, nothing is dropped, because it never owned anything.

> 💡 **Analogy: lending a book.** `&s1` is lending your book to a friend. They can read it, but it is still *your* book: they give it back, and they are not allowed to throw it away. `&mut s` is lending your colouring book *and saying "you may colour in it"*. They change it, then give it back to you, changed.

```rust
fn calculate_length(s: &String) -> usize { s.len() }

let s1 = String::from("hello");
let len = calculate_length(&s1);  // lend s1
println!("{s1} has length {len}");   // s1 is still ours
```

To change a borrowed value you need a **mutable reference**, `&mut T`, and the variable itself must be `mut`:

```rust
fn change(s: &mut String) { s.push_str(", world"); }

let mut s = String::from("hello");
change(&mut s);
```

▶ `cargo run -p ownership --example references`

## 6. The borrowing rules

> At any given time you can have **either one mutable reference or any number of immutable references**.
> References must always be **valid**.

This prevents **data races** at compile time: two pointers to the same data, at least one writing, with no synchronisation. Here is the real error from exercise 2:

```text
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
   |
11 |     let first = &scores[0];
   |                  ------ immutable borrow occurs here
12 |     scores.push(40);
   |     ^^^^^^^^^^^^^^^ mutable borrow occurs here
13 |     println!("first score: {first}");
   |                             ----- immutable borrow later used here
```

Why is this dangerous? `push` may need more room, so the `Vec` can move its whole buffer to a new place on the heap. `first` would then point at freed memory. Two `&mut` borrows of the same value at once give error **E0499**.

> 💡 **Analogy: the classroom whiteboard.** Any number of pupils may *read* the whiteboard at the same time. Or exactly one pupil may hold the marker and *write* on it. Never both: if someone rubs it out and rewrites it while you are copying it down, your notes are now nonsense. Here, `first` is a pupil still copying from the board when `push` starts rewriting it (and maybe moves the whole board to another room).

A borrow lasts from where it is created to where it is **last used**, not to the end of the scope (this is called *non-lexical lifetimes*). So you can often fix these errors just by finishing with one borrow before starting another.

> 💡 **Analogy:** you only count as "reading the board" until you finish copying, not until the end of the lesson. Once everyone has finished reading, the next pupil may pick up the marker.

▶ `cargo run -p ownership --example borrowing_rules` (uncomment the marked lines to see E0499 and E0502)

## 7. Dangling references

A dangling reference points to memory that has already been freed. Rust makes them impossible.

> 💡 **Analogy: an address of a demolished house.** Returning `&greeting` is like giving a friend the address of a sandcastle just before the tide comes in. By the time they arrive, it is gone. Rust refuses to hand out that address. The fix, returning the `String` itself, is giving them the sandcastle in a bucket to take home. Here is the real error from exercise 3:

```text
error[E0515]: cannot return reference to local variable `greeting`
   |
11 |     &greeting
   |     ^^^^^^^^^ returns a reference to data owned by the current function
```

`greeting` is dropped when the function ends, so a reference to it would dangle. The fix is to return the owned `String` and move it to the caller. (If the function takes no references at all, as in the Book's `dangle()` example, you get **E0106**, "missing lifetime specifier", instead. Lifetimes are chapter 10.)

## 8. Slices

A **slice** borrows a contiguous part of a collection instead of the whole thing.

> 💡 **Analogy: "read pages 10 to 20".** A slice is not a new book; it is a bookmark that says *which pages* of an existing book to look at. Nothing is copied. And because it is still a loan, the owner cannot tear those pages out (`s.clear()`) while someone is reading them.

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];   // or &s[..5]
let world: &str = &s[6..];    // to the end
let a = [1, 2, 3, 4, 5];
let middle: &[i32] = &a[1..4]; // [2, 3, 4]
```

- A string slice has type **`&str`**. String literals are slices too: `"hi"` is a `&'static str` pointing into your compiled program.
- Prefer **`&str` parameters over `&String`**. A `&str` parameter accepts both `&String` (through deref coercion) and literals. Clippy will suggest this to you.
- Slices keep you safe: while `word = first_word(&s)` borrows from `s`, calling `s.clear()` is a compile error (E0502), so `word` can never point at stale text.
- Slice ranges are **byte** offsets. They must fall on UTF-8 character boundaries, or the program panics: `"héllo"` has 5 characters but 6 bytes.

▶ `cargo run -p ownership --example slices`

## Beyond the Book: moving out of a `&mut`

You cannot move a value out of something you only borrow (error **E0507**). But you can swap something else in:

> 💡 **Analogy: the museum frame.** You may not walk off with a painting from a museum you are only visiting; the frame would be left empty and broken. But you *can* swap it: take the painting and hang a blank canvas in its place (`take`), or hang a different painting (`replace`), or swap two paintings between frames (`swap`). The frame is never empty, so the rules are happy.

| Function | What it does |
|---|---|
| `std::mem::take(&mut x)` | Returns `x`, leaves `Default::default()` (an empty `Vec`, `String`, …) |
| `std::mem::replace(&mut x, new)` | Returns `x`, leaves `new` |
| `std::mem::swap(&mut a, &mut b)` | Exchanges the two values |

```rust
fn drain(&mut self) -> Vec<String> {
    std::mem::take(&mut self.messages)   // not `self.messages`, which is E0507
}
```

▶ `cargo run -p ownership --example mem_take_replace_swap`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Used a `String` after passing it to a function | E0382 | Make the function take `&str` or `&T`. Only `.clone()` if you truly need two copies. |
| Pushed to a `Vec` while holding `&v[i]` | E0502 | Finish using the reference first, or copy the element out (`let x = v[i];`). |
| Two `&mut` to the same value at once | E0499 | Split into separate scopes, or restructure so one borrow ends first. |
| Returned `&local` from a function | E0515 / E0106 | Return the owned value (`String`, `Vec<T>`). |
| `&mut` on a variable not declared `mut` | E0596 | `let mut x = …` |
| Moved a field out of `&mut self` | E0507 | `std::mem::take` / `replace`, or `.clone()`. |
| `fn f(s: &String)` | clippy `ptr_arg` | `fn f(s: &str)` |
| `.clone()` everywhere to silence errors | none, but slow | Ask "who should own this?" and borrow instead. |

## Practise

**1. Run the examples**, in order. Each is one idea, with the compile errors left as comments to uncomment:

```bash
cargo run -p ownership --example scope_and_drop
cargo run -p ownership --example move_semantics
cargo run -p ownership --example clone_and_copy
cargo run -p ownership --example ownership_and_functions
cargo run -p ownership --example references
cargo run -p ownership --example borrowing_rules
cargo run -p ownership --example slices
cargo run -p ownership --example mem_take_replace_swap
```

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_move` | Make it compile (E0382), no `.clone()` | `cargo run -p ownership --features exercises --bin ex01_fix_move` |
| 2 | `ex02_fix_borrow_conflict` | Make it compile (E0502) | `cargo run -p ownership --features exercises --bin ex02_fix_borrow_conflict` |
| 3 | `ex03_fix_dangling` | Make it compile (E0515) | `cargo run -p ownership --features exercises --bin ex03_fix_dangling` |
| 4 | `ex04_first_word` | Implement, pass 5 tests | `cargo test -p ownership --features exercises --bin ex04_first_word` |
| 5 | `ex05_borrow_or_own` | Implement, pass 3 tests | `cargo test -p ownership --features exercises --bin ex05_borrow_or_own` |
| 6 | `ex06_mem_take` | Implement, pass 2 tests (Beyond the Book) | `cargo test -p ownership --features exercises --bin ex06_mem_take` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p ownership --features solutions`).

**4. Read the mini project** in `src/main.rs` (`cargo run -p ownership`, `cargo test -p ownership`): a text report where every function borrows, and the only allocations are the ones we choose to make.

## Checklist

- [ ] [4.1 What is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [ ] [4.2 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [ ] [4.3 The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html)
- [ ] Beyond the Book: `Copy` vs `Clone` in depth
- [ ] Beyond the Book: moves out of collections with `std::mem::take`, `replace`, `swap`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Every value has exactly one owner; when the owner goes out of scope, the value is dropped.
- Assigning or passing a heap-owning value **moves** it. `Copy` types are copied instead. `.clone()` is an explicit deep copy.
- **Borrow** with `&T` to read and `&mut T` to change: many readers or one writer, never both.
- References can never dangle; the compiler proves it.
- Slices (`&str`, `&[T]`) borrow part of a collection. Prefer `&str` and `&[T]` parameters.

**In one sentence for a child:** every toy has one owner; you can give it away, lend it to lots of people to look at, or to one person to play with, and it is tidied away when its owner goes home.

**Next:** [02 Core Types and Errors / 01 Structs](../../02_core_types_and_errors/01_structs/README.md) (Book chapter 5)
