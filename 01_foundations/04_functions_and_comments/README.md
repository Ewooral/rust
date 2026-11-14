<!-- hand-written lesson: gen_course must not overwrite -->
# Functions and Comments

**The Rust Book, chapter 3:** [Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html), sections [3.3 Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html) and [3.4 Comments](https://doc.rust-lang.org/book/ch03-04-comments.html)
· Rust by Example: [Functions](https://doc.rust-lang.org/rust-by-example/fn.html), [Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)

Functions let you give a piece of work a name, write it once, and use it everywhere. Rust's functions look like other languages' functions, with one idea that's worth slowing down for: the difference between **statements** and **expressions**. Once that clicks, Rust's "no `return` needed" style makes sense. Comments are the notes you leave for humans, and Rust has a special kind that becomes real documentation.

> 💡 **The big picture: machines in a toy factory.** Each **function** is a machine with labelled **input slots** (parameters), one job to do, and an **output chute** (the return value). `main` is the **factory manager** who switches machines on in order. **Comments** are sticky notes on the machines for the workers; the machines ignore them. **Doc comments** are the official instruction manual printed on the side of each machine.

## 1. Defining and calling functions

```rust
fn main() {
    println!("Hello, world!");
    another_function();
}

fn another_function() {
    println!("Another function.");
}
```

- **`fn`**, then the name, then `()` and a body in `{ }`.
- Names are **snake_case**: lower case with underscores.
- The order doesn't matter: `another_function` is defined *after* `main` and that's fine. Rust only needs it to be defined somewhere the caller can see.
- Call it by name with parentheses: `another_function();`.

> 💡 **Analogy:** defining a function is **building a machine** and putting a name plate on it. Calling it is **pressing its button**. It doesn't matter which corner of the factory the machine stands in, as long as the manager knows its name.

## 2. Parameters

```rust
fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

print_labeled_measurement(5, 'h');   // prints "The measurement is: 5h"
```

- The variables in the definition are **parameters**; the values you pass in a call are **arguments**.
- **Every parameter must have a type.** This is a deliberate design choice: the compiler never has to guess what a function accepts, so it rarely needs annotations anywhere else, and its error messages can point at exactly the right place. Leave the type out and the code doesn't even parse: *expected one of `:`, `@`, or `|`, found `)`*.
- Several parameters are separated by commas. Arguments are matched by **position**, and you must pass exactly one for each. The real error from exercise 2:

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
   |
13 |     let room = area(4);
   |                ^^^^--- argument #2 of type `u32` is missing
   |
note: function defined here
   |
 8 | fn area(width: u32, length: u32) -> u32 {
   |    ^^^^             -----------
help: provide the argument
   |
13 |     let room = area(4, /* u32 */);
   |                      +++++++++++
```

A function only sees **its own** variables and its parameters. A variable from `main` is invisible inside another function, so you pass it in. The real error from exercise 3:

```text
error[E0425]: cannot find value `name` in this scope
   |
10 |     println!("Hello, {name}!");
   |                       ^^^^ not found in this scope
```

> 💡 **Analogy: shaped slots.** Each input slot on the machine is **shaped for one kind of thing**: a coin slot won't take a banana, and the slot has a label saying so (the type). The machine has **no windows**: it can't see what's lying around on the factory floor (`main`'s variables). If it needs something, you post it through a slot.

▶ `cargo run -p functions_and_comments --example defining_functions`

## 3. Statements and expressions

This is the idea that makes Rust's functions tick.

- A **statement** performs an action and **does not produce a value**. `let y = 6;` is a statement. A function definition is a statement too.
- An **expression** **evaluates to a value**. `5 + 6` is an expression (it's 11). So are a function call, a macro call, and a block in curly brackets.

Because `let y = 6` has no value, you can't assign it to something else (in C you can write `x = y = 6`; in Rust you can't):

```text
error: expected expression, found `let` statement
  |
2 |     let x = (let y = 6);
  |              ^^^
```

A **block is an expression**: its value is its **last line**, as long as that line has **no semicolon**.

```rust
let y = {
    let x = 3;
    x + 1          // no semicolon: the block's value is 4
};
```

Add a semicolon to that last line, and it becomes a statement: its value is thrown away, and the block's value is `()`, the empty **unit** value.

> 💡 **Analogy: a question vs an instruction.** An expression is a **question with an answer**: "what's 5 + 6?" gives back 11. A statement is an **instruction**: "put 6 in the box called y". It gets something done, but there's no answer to hand anyone. The semicolon is saying **"do it, and forget the answer"**.

▶ `cargo run -p functions_and_comments --example statements_and_expressions`

## 4. Functions with return values

```rust
fn five() -> i32 {
    5
}

fn plus_one(x: i32) -> i32 {
    x + 1
}
```

- **`-> i32`** after the parameters declares the return type.
- The value of the **final expression** in the body is returned. No `return` keyword needed.
- **`return value;`** leaves the function **early**. Use it for special cases at the top, and let the normal answer be the final expression.
- A function with no `->` returns `()`.
- To return several values, return a **tuple**: `fn split(total: u32) -> (u32, u32)`.

The classic mistake is a semicolon on the last line. The real error from exercise 1, where the `help:` line gives the fix:

```text
error[E0308]: mismatched types
   |
 9 | fn plus_one(x: i32) -> i32 {
   |    --------            ^^^ expected `i32`, found `()`
   |    |
   |    implicitly returns `()` as its body has no tail or `return` expression
10 |     x + 1;
   |          - help: remove this semicolon to return this value
```

The opposite mistake, returning a value but forgetting `->`, gives the same error code, the other way round:

```text
error[E0308]: mismatched types
  |
1 | fn five() {
  |          - help: try adding a return type: `-> i32`
2 |     5
  |     ^ expected `()`, found integer
```

> 💡 **Analogy: the output chute.** Whatever is **last on the conveyor belt** drops out of the chute: that's the return value. A semicolon is a worker **throwing it in the bin** just before the chute, so nothing comes out. `return` is the **emergency exit chute** halfway along: "this one's done, send it out now".

▶ `cargo run -p functions_and_comments --example return_values`

## 5. Comments

```rust
// A line comment runs to the end of the line.
// For several lines, start each one with //.

let lucky_number = 7; // a comment can end a line of code, too
```

- The compiler **ignores** comments completely. They're for people, including future you.
- Comments usually go on their own line, **above** the code they explain.
- Block comments `/* ... */` also exist and can span lines or sit inside one, but `//` is the normal style.
- Explain **why**, not what: `// prices are in pence so we never round` is more useful than `// multiply`.

> 💡 **Analogy: sticky notes.** Comments are **sticky notes on the machines**: "careful, this one jams on Mondays". The machine never reads them; the next worker does.

▶ `cargo run -p functions_and_comments --example comments`

## Beyond the Book: doc comments and `cargo doc`

The Book covers these in [chapter 14.2](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments), but they're worth using from day one.

| Comment | Documents | Where it goes |
|---|---|---|
| `/// ...` | the item **after** it (function, constant, struct, ...) | above the item |
| `//! ...` | the item it's **inside**: the whole crate or module | at the top of the file |

```rust
/// `percent` percent of `amount`, rounded to the nearest penny (halves go up).
///
/// For example, 10% of 1999p is 199.9p, which rounds to 200p.
fn percent_of(amount: u32, percent: u32) -> u32 {
    (amount * percent + 50) / 100
}
```

- Doc comments are **Markdown**: `code`, *emphasis*, lists and headings all work. By convention, readers look for sections like `# Examples`, `# Panics` and `# Errors`.
- **`cargo doc -p functions_and_comments --open`** turns them into a website and opens it. Your editor also shows them when you hover over a name.
- For a binary like this one, `cargo doc` includes private items. For a library, only `pub` items are documented by default, and code in `# Examples` is run as a **doc test** by `cargo test` (lessons 03/02 and 03/05).
- **`#![deny(missing_docs)]`** at the top of a crate makes an undocumented public item a compile error. Many libraries do this. The real error from exercise 6:

```text
error: missing documentation for a constant
   |
14 | pub const SPEED_OF_SOUND: f64 = 343.0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
   |
12 | #![deny(missing_docs)]
   |         ^^^^^^^^^^^^

error: missing documentation for a function
   |
16 | pub fn thunder_distance_km(seconds: f64) -> f64 {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

> 💡 **Analogy: the instruction manual.** Sticky notes (`//`) are for workers *inside* the factory. Doc comments (`///`) are the **manual printed on the side of the machine** for anyone who *uses* it, and `cargo doc` **binds all the manuals into a book** with an index and a search box. `//!` is the **cover page** of the book.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `x + 1;` as the last line of a function returning `i32` | E0308 expected `i32`, found `()` | Remove the semicolon. |
| Returned a value but forgot `-> Type` | E0308 expected `()`, found integer | Add the return type. |
| Called a function with too few or too many arguments | E0061 | One argument per parameter, in order. |
| Used another function's variable | E0425 cannot find value | Pass it in as a parameter. |
| Misspelled a function name | E0425 cannot find function | Check the spelling; the compiler often suggests the right name. |
| `fn f(x)` with no type | expected one of `:`, `@`, or `\|` | `fn f(x: i32)`: every parameter needs a type. |
| `let x = (let y = 6);` | expected expression, found `let` statement | `let` is a statement; write two lines. |
| `fn printLabel()` | warning: should have a snake case name | `fn print_label()` |
| A comment that repeats the code | none, but noise | Say **why**, or use a better name and delete the comment. |
| `#![deny(missing_docs)]` with an undocumented `pub` item | missing documentation for ... | Add a `///` comment. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p functions_and_comments --example defining_functions
cargo run -p functions_and_comments --example statements_and_expressions
cargo run -p functions_and_comments --example return_values
cargo run -p functions_and_comments --example comments
cargo doc -p functions_and_comments --example comments --document-private-items --open
```

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_semicolon_return` | Make it compile (E0308), delete one character | `cargo run -p functions_and_comments --features exercises --bin ex01_fix_semicolon_return` |
| 2 | `ex02_fix_missing_argument` | Make it compile (E0061) | `cargo run -p functions_and_comments --features exercises --bin ex02_fix_missing_argument` |
| 3 | `ex03_fix_out_of_scope` | Make it compile (E0425), add a parameter | `cargo run -p functions_and_comments --features exercises --bin ex03_fix_out_of_scope` |
| 4 | `ex04_pizza_party` | Small functions working together, pass 4 tests | `cargo test -p functions_and_comments --features exercises --bin ex04_pizza_party` |
| 5 | `ex05_split_time` | Return a tuple, pass 3 tests | `cargo test -p functions_and_comments --features exercises --bin ex05_split_time` |
| 6 | `ex06_document_it` | Make it compile (`missing_docs`) with doc comments (Beyond the Book) | `cargo run -p functions_and_comments --features exercises --bin ex06_document_it` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p functions_and_comments --features solutions --bins`, or run one with `--features solutions --bin sol01_fix_semicolon_return`).

**4. Read the mini project** in `src/main.rs`: a **shop receipt** built from five small functions.

```bash
cargo run -p functions_and_comments
cargo test -p functions_and_comments                 # 5 tests
cargo doc -p functions_and_comments --open           # the doc comments, as a website
```

```text
RECEIPT
3 x Notebook           £7.50
5 x Pen                £6.00
1 x Rust book         £39.99
----------------------------
Subtotal              £53.49
Discount 10%          -£5.35
VAT 20%               +£9.63
TOTAL                 £57.77
```

- Each function does **one** job (`line_total`, `percent_of`, `checkout`, `format_pounds`, `row`) and can be tested on its own.
- `checkout` returns **three values in a tuple**, and uses a **block expression** to work out the VAT.
- Money is stored as whole **pence** in a `u32`, so there are no floating-point surprises. `percent_of` rounds to the nearest penny by adding 50 before dividing by 100.
- Every item has a `///` doc comment, and the file starts with `//!` crate docs.

> 💡 **Analogy:** a **till** made of small machines in a row: one multiplies price by quantity, one works out percentages, one prints pounds and pence. If the total is wrong, you can test each machine on its own to find the broken one.

## Checklist

- [ ] [3.3 Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)
- [ ] [3.4 Comments](https://doc.rust-lang.org/book/ch03-04-comments.html)
- [ ] Beyond the Book: doc comments (`///`, `//!`) and `cargo doc --open`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `fn name(param: Type) -> ReturnType { ... }`. Every parameter has a type; names are snake_case; definition order doesn't matter.
- Statements do something and have no value. Expressions produce a value, and a block's value is its last expression.
- A function returns its final expression (no semicolon), or leaves early with `return`. A tuple returns several values.
- `//` comments are for readers of the code. `///` and `//!` are documentation that `cargo doc` turns into a website.

**In one sentence for a child:** a function is a machine with shaped slots for what goes in and a chute for what comes out, and comments are sticky notes that only people read.

**Next:** [05 Control Flow](../05_control_flow/README.md) (Book 3.5)
