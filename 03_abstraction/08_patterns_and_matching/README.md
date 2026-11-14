<!-- hand-written lesson: gen_course must not overwrite -->
# Patterns and Matching

**The Rust Book, chapter 19:** [Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html)
· Rust by Example: [match](https://doc.rust-lang.org/rust-by-example/flow_control/match.html), [Destructuring](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring.html)
· Reference: [Patterns](https://doc.rust-lang.org/reference/patterns.html)

A **pattern** is a shape that a value is compared against. If the value has that shape, the pattern can also take it apart and give names to the pieces. You've used patterns since lesson 01: every `let x = 5;` has one (`x`). Lesson 02/02 used them with `match`, `if let` and `let ... else` on enums. This chapter covers the whole picture: **where** patterns can go, **whether** they can fail, and **every piece of syntax** they can use.

| Pattern | Matches | Example |
|---|---|---|
| literal | exactly that value | `1`, `"move"`, `'a'`, `true` |
| name | anything, and binds it | `x`, `mut count` |
| `_` / `..` | anything, binds nothing / the rest | `(first, _, third)`, `Point { x, .. }` |
| `\|` | either alternative | `1 \| 2` |
| range | a value in the range | `1..=5`, `'a'..='z'`, `26..` |
| struct / enum / tuple | that shape, then its parts | `Point { x, y: 0 }`, `Some(n)`, `(a, b)` |
| slice | a length and its elements | `[first, .., last]`, `[head, tail @ ..]` |
| `&` | through a reference | `&(x, y)` |
| guard | the pattern, if a condition holds | `Some(n) if n > 0` |
| `@` | a sub-pattern, and keep the value | `n @ 1..=9` |

> 💡 **The big picture: a shape sorter with pockets.** A pattern is a cut-out shape in a board. Hold a value up to it. If the value **fits** the shape, it drops through, and its pieces fall into **labelled pockets** (variables) you can use. Some holes are so big that everything fits (`x`, `_`): those are *irrefutable*. Some only fit certain values (`Some(x)`, `0`, `1..=5`): those are *refutable*, so the code needs a plan for values that don't fit.

## 1. All the places patterns can be used

```rust
match maybe_age {                                   // match arms
    None => String::from("unknown age"),
    Some(age) => format!("{age} years old"),
}

let (x, y, z) = (1, 2, 3);                          // let statements

if let Some(color) = favorite_color {               // if let, mixed with else if…
} else if is_tuesday {
} else if let Ok(age) = age {                       // …and else if let
}

while let Some(top) = stack.pop() { }               // while let

for (index, value) in v.iter().enumerate() { }      // for loops

fn print_coordinates(&(x, y): &(i32, i32)) { }      // function parameters
pairs.iter().map(|&(n, _)| n * 10);                 // closure parameters

let Ok(age) = text.parse::<u8>() else { return Err(...) };   // let ... else
```

- **`match` arms** must be **exhaustive**: together they cover every possible value. A final `_` catches whatever is left.
- **`let PATTERN = EXPRESSION;`** A tuple pattern needs a tuple with the same number of parts: `let (a, b) = (1, 2, 3);` is E0308, "expected a tuple with 3 elements, found one with 2 elements".
- **`if let`** can be chained freely with `else if` and `else if let`, and each condition may test something different. A variable bound by `if let Ok(age) = age` **shadows** the outer `age` only inside its `{ }`. The compiler does **not** check that an `if let` chain is exhaustive.
- **`while let`** keeps looping as long as the pattern matches, like popping a stack until it's empty.
- **`for`**: the bit straight after `for` is a pattern, which is why `for (index, value) in …enumerate()` works.
- **Function and closure parameters** are patterns, so `&(x, y)` takes a borrowed tuple apart right in the signature.
- **`let ... else`** binds if the pattern matches. If not, the `else` block runs and must **leave**: `return`, `break`, `continue` or a panic. If it doesn't, you get E0308: "`else` clause of `let...else` does not diverge".

> 💡 **Analogy: one shape-sorter board, built into different machines.**
> - **`match`** is the big sorting machine with a hole for every shape, and it must catch every block.
> - **`let`** is the front door: it only has one hole, so everybody has to fit.
> - **`if let`** is a side door: "if you fit this shape, come in; if not, carry on".
> - **`while let`** is a conveyor belt that keeps running while each block still fits, and stops at the first one that doesn't.
> - **`for`** opens every parcel on the belt the same way.
> - **Function parameters** are the delivery hatch: whatever arrives is unpacked right at the hatch.
> - **`let ... else`** is the bouncer: "fit the shape, or go home now".

▶ `cargo run -p patterns_and_matching --example places_for_patterns`

## 2. Refutability: whether a pattern might fail to match

- **Irrefutable** patterns match *every* possible value: `x`, `(a, b)` for a pair, `Point { x, y }`, `[a, b, c]` for a 3-element array.
- **Refutable** patterns can fail for some value: `Some(x)` (fails on `None`), `Ok(n)`, `0`, `1..=5`, `[first, ..]` on a slice (fails when it's empty).

| Where | Allows |
|---|---|
| `let`, function parameters, closure parameters, `for` | **irrefutable** only, because there's nowhere to go if it fails |
| `if let`, `while let`, `let ... else` | **refutable**, because they have a plan for "no match" |
| `match` arms | refutable, but the arms **together** must cover everything |

Put a refutable pattern where only irrefutable ones are allowed, and it's a hard error. The real error from exercise 1:

```text
error[E0005]: refutable pattern in local binding
   |
21 |     let Some(price) = price_in_pence(item); // <- fix this line
   |         ^^^^^^^^^^^ pattern `None` not covered
   |
   = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-02-refutability.html
   = note: the matched value is of type `Option<u32>`
help: you might want to use `let...else` to handle the variant that isn't matched
   |
21 |     let Some(price) = price_in_pence(item) else { todo!() }; // <- fix this line
   |                                            ++++++++++++++++
```

The note links straight to this section of the Book. The same mistake in a parameter or a `for` loop is also E0005 ("refutable pattern in function argument", "refutable pattern in `for` loop binding").

The other way round, an **irrefutable** pattern in `if let` is legal but pointless, so it's only a **warning**:

```text
warning: irrefutable `if let` pattern
  |
2 |     if let x = 5 {
  |        ^^^^^^^^^
  |
  = note: this pattern will always match, so the `if let` is useless
  = help: consider replacing the `if let` with a `let`
  = note: `#[warn(irrefutable_let_patterns)]` on by default
```

> 💡 **Analogy: the front door and the side door.** `let` is the **front door** of the house: it must let *everyone* in, so it can only use a hole every block fits through. If you want a **star-shaped** door, you must also say what happens to the square blocks, which is what the `else` of `if let` and `let ... else` is for. Putting a hole every block fits on a *side* door ("if it fits, come in") isn't wrong, just silly, so the compiler only raises an eyebrow (a warning).

▶ `cargo run -p patterns_and_matching --example refutability`

## 3. Pattern syntax

### Matching literals

```rust
match x {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("anything"),
}
```

> 💡 **Analogy:** a literal is a **lock that opens for one exact key**. Key number 1 opens door 1, and nothing else does.

### Named variables, and the shadowing trap

A plain name in a pattern **never compares** with an outer variable of the same name. It creates a **new** variable that matches anything:

```rust
let x = Some(5);
let y = 10;
match x {
    Some(50) => println!("Got 50"),
    Some(y) => println!("Matched, y = {y}"),   // a NEW y = 5, shadowing the outer one
    _ => println!("Default case, x = {x:?}"),
}
println!("at the end: x = {x:?}, y = {y}");     // y is 10 again
```

To compare with the outer `y`, use a match guard: `Some(n) if n == y`.

> 💡 **Analogy:** writing `y` in a pattern sews a **new empty pocket** onto the board with the label "y". It catches *anything*. It doesn't check what's in the other pocket outside labelled "y", which is still there, untouched, when you leave the room.

### Multiple patterns with `|`

```rust
match x {
    1 | 2 => println!("one or two"),
    3 => println!("three"),
    _ => println!("anything"),
}
```

Every alternative must bind the **same names** with the same types, or the arm's code couldn't know which variables exist. The real error from exercise 3:

```text
error[E0408]: variable `k` is not bound in all patterns
   |
22 |         Event::KeyPress(key) | Event::KeyRelease(k) => format!("key {key:?}"),
   |         ^^^^^^^^^^^^^^^^^^^^                     - variable not in all patterns
   |         |
   |         pattern doesn't bind `k`
```

That exercise also shows E0381 ("used binding `key` is possibly-uninitialized") as a follow-on. Fix the names and both go away. Binding the same name twice in **one** alternative, as in `let (x, x) = pair;`, is a different error, E0416: "identifier `x` is bound more than once in the same pattern".

> 💡 **Analogy:** two doors into the same room. Whichever door you come through, you must be carrying the same bag with the same name tag, because the room only knows how to unpack "the bag called `key`".

### Matching ranges with `..=`

```rust
match x {
    1..=5 => println!("one through five"),
    _ => println!("something else"),
}
match c {
    'a'..='j' => println!("early ASCII letter"),
    'k'..='z' => println!("late ASCII letter"),
    _ => println!("something else"),
}
```

Ranges are only allowed for numbers and `char`, the only types where the compiler can check at compile time that a range isn't empty. Rust also allows **exclusive** (`0..15`) and **half-open** (`26..`, `..0`) range patterns. Ranges count towards exhaustiveness, so this `match` needs no `_`:

```rust
let feel = match temp {   // temp: i8
    ..0 => "freezing",
    0..15 => "cold",
    15..=25 => "nice",
    26.. => "hot",
};
```

> 💡 **Analogy:** the **height bar** at a fairground ride. You don't need a separate sign for every height; "1 m to 1.5 m may ride" covers them all.

▶ `cargo run -p patterns_and_matching --example literals_variables_ranges`

### Destructuring structs

```rust
let Point { x: a, y: b } = p;   // field x into a new variable a
let Point { x, y } = p;         // shorthand: variables named after the fields

match p {
    Point { x: 0, y: 0 } => println!("the origin"),
    Point { x, y: 0 } => println!("on the x axis at {x}"),   // test y, bind x
    Point { x: 0, y } => println!("on the y axis at {y}"),
    Point { x, y } => println!("on neither axis: ({x}, {y})"),
}
```

A struct pattern must mention **every** field, or say `..` to skip the rest. `let Point { x } = p;` is E0027: "pattern does not mention field `y`".

> 💡 **Analogy: unpacking a lunchbox.** The lunchbox has a sandwich compartment and an apple compartment. `Point { x, y }` tips each compartment into a bowl with the same name. `Point { x, y: 0 }` means "only open this lunchbox if the apple compartment is empty".

### Destructuring enums, nested values, structs and tuples

The pattern has the same shape as the variant: nothing for `Quit`, `{ x, y }` for a struct-like variant, and `(text)` for a tuple-like one. Patterns **nest**, so you can open an enum inside an enum in one go:

```rust
match msg {
    Message::Quit => {}
    Message::Move { x, y } => println!("move by ({x}, {y})"),
    Message::Write(text) => println!("{text}"),
    Message::ChangeColor(Color::Rgb(r, g, b)) => println!("red {r}, green {g}, blue {b}"),
    Message::ChangeColor(Color::Hsv(h, s, v)) => println!("hue {h}, saturation {s}, value {v}"),
}

let ((feet, inches), Point { x, y }) = ((3, 10), Point { x: 3, y: -10 });   // mixed
```

> 💡 **Analogy: Russian dolls.** A nested pattern opens the big doll, the middle doll and the tiny doll in one movement, and hands you whatever was inside the smallest one.

▶ `cargo run -p patterns_and_matching --example destructuring`

### Ignoring values: `_`, `_name` and `..`

| Write | Means |
|---|---|
| `_` | match anything, **don't bind** it: `fn foo(_: i32, y: i32)`, `(Some(_), Some(_))`, `(first, _, third, _, fifth)` |
| `_name` | a real variable that doesn't warn when unused |
| `..` | "and the rest": `Point { x, .. }`, `(first, .., last)` |

The difference between `_` and `_name` matters for ownership. `_s` **binds**, so it moves the value; `_` binds nothing, so nothing moves:

```text
error[E0382]: borrow of partially moved value: `s`
  |
3 |     if let Some(_s) = s {
  |                 -- value partially moved here
...
6 |     println!("{s:?}");
  |                ^ value borrowed here after partial move
```

`..` can appear only **once** per tuple or slice. In `(.., second, ..)`, which number is `second`? The compiler refuses to guess: "error: `..` can only be used once per tuple pattern" (this one has no error code).

> 💡 **Analogy:** `_` is a hole with **no pocket**: the piece drops straight through and nobody keeps it. `_name` is a pocket you don't plan to look in, but the piece is still **in your pocket**, so it's no longer on the table. `..` is "and the rest of the train", however many carriages there are.

▶ `cargo run -p patterns_and_matching --example ignoring_values`

### Extra conditionals with match guards

A **match guard** is an `if` after the pattern. The arm runs only if the pattern matches **and** the guard is true:

```rust
match num {
    Some(x) if x % 2 == 0 => println!("{x} is even"),
    Some(x) => println!("{x} is odd"),
    None => (),
}
```

- A guard can use **outer variables**, which fixes the shadowing trap: `Some(n) if n == y`.
- A guard applies to the **whole** `|` pattern: `4 | 5 | 6 if y` means `(4 | 5 | 6) if y`.
- The compiler **doesn't look inside guards** when it checks exhaustiveness. A `u8` match with `n if n % 2 == 0` and `n if n % 2 == 1` still needs a final `_`, or you get E0004 "`0_u8..=u8::MAX` not covered".

> 💡 **Analogy: a second check at the gate.** The block fits the shape, but before it drops through, a guard asks one more question ("are you even?"). If the answer is no, the block moves on to the next hole.

### `@` bindings

`name @ pattern` tests the value against the pattern **and** keeps it in `name`:

```rust
match msg {
    Message::Hello { id: id @ 3..=7 } => println!("Found an id in range: {id}"),
    Message::Hello { id: 10..=12 } => println!("Found an id in another range"),   // tests, binds nothing
    Message::Hello { id } => println!("Found some other id: {id}"),
}

match c {
    vowel @ ('a' | 'e' | 'i' | 'o' | 'u') => println!("{vowel} is a vowel"),   // @ with |
    letter @ 'a'..='z' => println!("{letter} is a consonant"),
    other => println!("{other} is something else"),
}
```

> 💡 **Analogy: a turnstile that takes a photo.** The height bar checks you're between 3 and 7, and the camera keeps your picture (`id`) so you can be named afterwards. A range without `@` lets you through but takes no photo.

▶ `cargo run -p patterns_and_matching --example guards_and_bindings`

## Beyond the Book: slice patterns

Patterns can match **arrays** and **slices** by length and by element:

| Pattern | Matches | Binds |
|---|---|---|
| `[]` | an empty slice | nothing |
| `[a, b]` | exactly two elements | both |
| `[first, ..]` / `[.., last]` | at least one | one end |
| `[first, .., last]` | at least two | both ends |
| `[head, tail @ ..]` | at least one | the first, and the **rest as a slice** |

```rust
fn sum(numbers: &[i32]) -> i32 {
    match numbers {
        [] => 0,
        [head, tail @ ..] => head + sum(tail),   // "the first, then the rest"
    }
}

match words.as_slice() {                         // a Vec: match on .as_slice()
    ["add", a, b] => …,
    ["greet", names @ ..] if !names.is_empty() => …,
    [unknown, ..] => …,
    [] => …,
}
```

- **Arrays have a fixed length**, so `let [r, g, b] = [255, 128, 0];` is irrefutable. On a **slice**, the length is unknown, so `let [a, b] = slice;` is E0005: "patterns `&[]`, `&[_]` and `&[_, _, _, ..]` not covered".
- **Exhaustiveness by length:** `[]`, `[x]`, `[x, y]` and `[x, .., y]` together cover every length with no `_`. Leave a length out and the compiler names it. The real error from exercise 2:

```text
error[E0004]: non-exhaustive patterns: `&[_, _, _, ..]` not covered
   |
11 |     match names {
   |           ^^^^^ pattern `&[_, _, _, ..]` not covered
   |
   = note: the matched value is of type `&[&str]`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
14 ~         [first, second] => format!("{first} and {second}"),
15 ~         &[_, _, _, ..] => todo!(),
```

- **Strings** aren't slices of `char`, so match `s.as_bytes()` or a collected `Vec<char>` (`chars.as_slice()`).

> 💡 **Analogy: a toy train.** `[engine, .., caboose]` grabs the first and last carriages, however long the train is. `[head, tail @ ..]` uncouples the engine and keeps the **rest of the train** as its own, shorter train, which you can uncouple again, and again, until nothing is left (`[]`).

▶ `cargo run -p patterns_and_matching --example slice_patterns`

## Beyond the Book: binding modes (move, copy or borrow?)

| You match | With | The binding is |
|---|---|---|
| a value: `match msg` | `Write(text)` | **moved** (or copied) out: `msg` is partly used up (E0382 later) |
| a reference: `match &msg` | `Write(text)` | **borrowed** automatically: `text: &String` |
| a mutable reference: `match &mut pos` | `Move { x, y }` | `x: &mut i32`, so `*x += 10` changes it in place |
| a value | `Write(ref text)` / `Some(ref mut n)` | borrowed, the older spelling |
| a reference | `&(number, letter)` | dereferenced and **copied** out |

The second row is called **match ergonomics**: when a non-reference pattern meets a reference, the default binding mode switches to "borrow". That's why `for (n, c) in &pairs` gives you references, and `for &(n, c) in &pairs` gives copies.

**The 2024 edition rule.** Inside a pattern that is already borrowing implicitly, you may not also write `ref`, `mut` or `&`. These errors have no error code:

```text
error: cannot explicitly borrow within an implicitly-borrowing pattern
error: cannot mutably bind by value within an implicitly-borrowing pattern
error: cannot explicitly dereference within an implicitly-borrowing pattern
```

The first comes from `if let Some(ref s) = &opt`, the second from `let [mut c] = &[5]`, the third from `let (x, &y) = &(1, &2)`. The fix the compiler suggests: drop the modifier (`Some(s)`), or write the outer `&` yourself so nothing is implicit (`let &[mut c] = &[5];`).

> 💡 **Analogy: a museum.** Matching a value is taking the painting **home**: it's no longer in the museum (moved). Matching a reference is looking through the **glass**: you can see everything, and the painting stays (borrowed). A `&` pattern is buying a **postcard** of a small picture: you get your own copy (`Copy` types only). The 2024 rule says: once you're behind the glass, you can't also ask to take the painting home.

▶ `cargo run -p patterns_and_matching --example binding_modes`

## Beyond the Book: `matches!`, let chains and `if let` guards

```rust
let operators = tokens.iter().filter(|t| matches!(t, Token::Symbol('+' | '-' | '*' | '/'))).count();

if let Some(Token::Number(n)) = first        // let chains (2024 edition, Rust 1.88+)
    && let Some(Token::Word(w)) = second
    && *n > 1
{
    println!("{n} {w}s");
}

match token {
    Token::Word(w) if let Some(c) = w.chars().next() => …,   // an `if let` guard
    …
}
```

- **`matches!(value, pattern)`** is a pattern as a `true`/`false` test, guard included: `matches!(t, Token::Number(n) if *n < 0)`.
- **Let chains** join `let` patterns and ordinary conditions with `&&`. Later parts can use names bound by earlier ones. Without them, the code above needs three nested `if`s.
- **`if let` guards** put a pattern inside a match guard. They are newer than let chains and are stable on this course's toolchain.

> 💡 **Analogy:** `matches!` is a **yes/no question**: "does this block fit the star hole?", without keeping anything. A let chain is a **checklist at the door**: "has a ticket, AND the ticket is for today, AND they're tall enough". You stop at the first "no".

▶ `cargo run -p patterns_and_matching --example let_chains_and_matches`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `let Some(x) = opt;` | E0005 refutable pattern in local binding | `let Some(x) = opt else { … };`, `if let` or `match` |
| `fn f(Some(x): Option<i32>)`, `for Some(x) in v` | E0005 | Match inside the body, or `.into_iter().flatten()`. |
| A `match` missing a case (often a slice length) | E0004 non-exhaustive patterns | Add the arm the compiler names; prefer it to a blind `_`. |
| Guards that "obviously" cover everything | E0004 | The compiler ignores guards: add a final arm. |
| `A(x) \| B(y) =>` | E0408 variable not bound in all patterns | Same names on both sides: `A(x) \| B(x)`. |
| `let (x, x) = pair;` | E0416 bound more than once | Use two names. |
| `let Point { x } = p;` | E0027 pattern does not mention field `y` | `Point { x, .. }` |
| `let (a, b) = (1, 2, 3);` | E0308 expected a tuple with 3 elements | Match the shape: `(a, b, _)` or `(a, ..)`. |
| `Some(y) =>` to compare with an outer `y` | compiles, matches everything | `Some(n) if n == y` |
| `(.., second, ..)` | error: `..` can only be used once | Name the positions: `(_, second, ..)`. |
| `if let Some(_s) = s` then using `s` | E0382 partially moved | `Some(_)` (binds nothing) or match `&s`. |
| `let ... else` whose `else` doesn't leave | E0308 does not diverge | End the block with `return`, `break`, `continue` or a panic. |
| `ref` / `mut` / `&` inside a pattern matching a reference | 2024 edition error (no code) | Drop the modifier, or write the outer `&` yourself. |
| `_` arm first | warning: unreachable pattern | Catch-alls go last. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p patterns_and_matching --example places_for_patterns
cargo run -p patterns_and_matching --example refutability
cargo run -p patterns_and_matching --example literals_variables_ranges
cargo run -p patterns_and_matching --example destructuring
cargo run -p patterns_and_matching --example ignoring_values
cargo run -p patterns_and_matching --example guards_and_bindings
cargo run -p patterns_and_matching --example slice_patterns
cargo run -p patterns_and_matching --example binding_modes
cargo run -p patterns_and_matching --example let_chains_and_matches
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_refutable_let` | Make it compile (E0005), with `let ... else` | `cargo run -p patterns_and_matching --features exercises --bin ex01_fix_refutable_let` |
| 2 | `ex02_fix_non_exhaustive_slice` | Make it compile (E0004), one slice arm, no `_` | `cargo run -p patterns_and_matching --features exercises --bin ex02_fix_non_exhaustive_slice` |
| 3 | `ex03_fix_or_pattern_bindings` | Make it compile (E0408), same names on both sides of `\|` | `cargo run -p patterns_and_matching --features exercises --bin ex03_fix_or_pattern_bindings` |
| 4 | `ex04_classify_points` | Destructuring, ranges, guards and `@`, pass 3 tests | `cargo test -p patterns_and_matching --features exercises --bin ex04_classify_points` |
| 5 | `ex05_slice_patterns` | `ends`, a recursive `is_sorted`, a web router, pass 3 tests | `cargo test -p patterns_and_matching --features exercises --bin ex05_slice_patterns` |
| 6 | `ex06_rpn_calculator` | A stack calculator with a pattern in every place, pass 3 tests | `cargo test -p patterns_and_matching --features exercises --bin ex06_rpn_calculator` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p patterns_and_matching --features solutions --bins`).

**4. The mini project** is a **robot interpreter**: a toy robot on a grid follows a script of text commands, and every kind of pattern in this chapter is used somewhere in `src/lib.rs`, with a comment naming it.

```bash
cargo run -p patterns_and_matching                    # the built-in demo script (with 4 mistakes on purpose)
cargo run -p patterns_and_matching -- my_script.txt   # your own script
cargo test -p patterns_and_matching                   # 8 unit tests
```

```text
Ran 13 commands:
  line  2: walked 3 steps from (0, 0) to (0, 3)
  line  3: now facing East
  ...
4 lines skipped:
  line  6: unknown command "fly"
  line 10: 99 steps is too far (max 10)
  line 13: usage: teleport <x> <y>
  line 18: no such colour "purple"

The robot is at (-3, 1), in the north-west, facing South. Map (H = home):
+..brrrr
o..b...r
v..b...r
...H....
```

| Where | Patterns used |
|---|---|
| `parse_line` | slice patterns on the words, literals with `\|`, `words @ ..` with a guard, `known @ ("move" \| …)` |
| `parse_steps`, `parse_coord`, `parse_number` | `let ... else`, `n @ 1..=MAX_STEPS` (a range with a constant), a named catch-all |
| `Colour::mark` | literals inside a variant (`Rgb(0, 0, 0)`), a guard for greys, `Rgb(..)` |
| `Robot::apply`, `walk`, `undo` | nested enums (`Turn(Turn::Left)`), a struct variant, ranges inside a struct pattern, `let (dx, dy)`, `if let Some(Snapshot { .. })` |
| `Event`'s `Display` | literal fields (`steps: 1`), `home @ Point { x: 0, y: 0 }` |
| `describe_position` | a function parameter pattern `&Point { x, y }`, tuple patterns, half-open ranges |
| `run_script`, `map`, `main.rs` | `for (index, line)`, `Ok(Some(command))`, `while let`, closure parameter `&(point, _)`, `else if let`, `matches!`, a parameter pattern taking a whole `Report` apart |

> 💡 **Analogy: a toy robot reading instruction cards.** Each line of the script is a card. The robot holds every card up to its shape-sorter board: a "move" card with a number fits the move hole, a "paint" card fits the paint hole, and a card that fits no hole goes on the "I don't understand" pile with a note saying why. Then the robot does each card in order, and keeps every old card in a stack so `undo` can put things back.

## Checklist

- [ ] [19.1 All the Places Patterns Can Be Used](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html)
- [ ] [19.2 Refutability: Whether a Pattern Might Fail to Match](https://doc.rust-lang.org/book/ch19-02-refutability.html)
- [ ] [19.3 Pattern Syntax](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html)
- [ ] Beyond the Book: slice patterns: `[first, .., last]`, `[head, tail @ ..]`
- [ ] Beyond the Book: binding modes: `ref`, `ref mut`, `&` patterns, match ergonomics and the 2024 rules
- [ ] Beyond the Book: `matches!`, let chains and `if let` guards
- [ ] Exercises 1–6 and the robot interpreter done

## Recap

- Patterns appear in `match`, `let`, `if let`, `while let`, `for`, function and closure parameters, and `let ... else`.
- **Irrefutable** patterns always match and are the only kind `let`, parameters and `for` accept. **Refutable** ones need `if let`, `while let`, `let ... else` or `match`.
- The syntax: literals, names (which shadow, never compare), `|`, ranges, destructuring of structs, enums, tuples and nested values, `_`, `_name` and `..` to ignore, guards for extra conditions, and `@` to test and keep.
- Slice patterns match by length and elements, and are exhaustive without `_` when every length is covered.
- Matching a reference borrows; matching a value moves. `matches!` and let chains put patterns into conditions.

**In one sentence for a child:** a pattern is a shape cut into a board: when a value fits the shape, it drops through and its pieces land in labelled pockets, and Rust checks that you've cut a hole for every shape that could ever arrive.

**Next:** [04 Systems Level / 01 Fearless Concurrency](../../04_systems_level/01_fearless_concurrency/README.md) (Book chapter 16)
