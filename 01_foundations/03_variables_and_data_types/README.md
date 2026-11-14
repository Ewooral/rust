<!-- hand-written lesson: gen_course must not overwrite -->
# Variables and Data Types

**The Rust Book, chapter 3:** [Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html), sections [3.1 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html) and [3.2 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
· Rust by Example: [Primitives](https://doc.rust-lang.org/rust-by-example/primitives.html), [Casting](https://doc.rust-lang.org/rust-by-example/types/cast.html)

Every language has variables and numbers. Rust's twist is that it makes you **say what you mean**: is this value allowed to change? How big can this number get? What should happen if it gets bigger? Those answers live in the code, where the compiler can check them.

> 💡 **The big picture: labelled boxes.** A variable is a **box with a label** on it. `let` makes a box and puts a value inside; by default the box is **sealed** (immutable), and `mut` gives it a lid you can open. The **type** is the box's **shape and size**: a box shaped for whole numbers can't hold 2.5, and a small box (`u8`) can't hold 300. The compiler is a strict warehouse manager who checks every box before anything ships.

## 1. Variables and mutability

Variables are **immutable by default**. Once a value is bound to a name, you can't change it. The real error from exercise 1:

```text
error[E0384]: cannot assign twice to immutable variable `score`
   |
10 |     let score = 0;
   |         ----- first assignment to `score`
11 |     println!("kick-off: {score}");
12 |     score += 10;
   |     ^^^^^^^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
10 |     let mut score = 0;
   |         +++
```

Why default to immutable? If one part of the code assumes a value never changes and another part changes it, you get bugs that are hard to track down. With immutability as the default, the compiler **guarantees** it. When a value *should* change, write **`let mut`**: it tells the compiler, and every future reader, "this one changes".

```rust
let mut score = 0;
score += 10;          // fine
```

If you write `mut` and then never change the value, Rust tells you that too:

```text
warning: variable does not need to be mutable
  |
2 |     let mut never_changes = 1;
  |         ----^^^^^^^^^^^^^
  |         |
  |         help: remove this `mut`
```

> 💡 **Analogy: sealed boxes and boxes with lids.** `let x = 5;` puts 5 in a **sealed box** labelled `x`: anyone can look through the window, but nobody can swap what's inside, so everyone can trust it. `let mut score = 0;` is a box **with a lid**. You can swap the contents, and the lid is a warning to everyone else: "this might be different next time you look".

▶ `cargo run -p variables_and_data_types --example mutability`

## 2. Constants

```rust
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

| | `let` | `const` |
|---|---|---|
| Can be `mut` | yes | never |
| Type annotation | optional | **required** |
| Allowed where | inside functions | **any scope**, including the top of a file |
| Value comes from | anything, even at run time | a **constant expression** the compiler can work out |
| Naming | `snake_case` | `SCREAMING_SNAKE_CASE` |

Constants are perfect for values many parts of the program need, like a maximum score or the speed of light. Name them once, and a change later happens in one place. Because the compiler evaluates them, it also checks them. Exercise 4's stadium holds 300 people, but its type tops out at 255:

```text
error[E0080]: attempt to compute `200_u8 + 100_u8`, which would overflow
   |
10 | const STADIUM_CAPACITY: u8 = 200 + 100;
   |                              ^^^^^^^^^ evaluation of `STADIUM_CAPACITY` failed here
```

> 💡 **Analogy: the rules painted on the classroom wall.** "Lunch is at 12:00" was decided before the school opened, it's painted on, and nobody can change it during the day. Everyone reads the same rule. And because it's painted *before* opening day, a mistake in it ("lunch at 25:00") is caught before any child arrives.

## 3. Shadowing

You can declare a **new** variable with the same name as an old one. The new one **shadows** the old one: from then on, the name means the new value.

```rust
let x = 5;
let x = x + 1;          // a new x: 6
{
    let x = x * 2;      // only inside these braces: 12
}
// x is 6 again
```

Shadowing isn't the same as `mut`:
- Without `let`, reassigning is an error. With `let`, it's a **new variable**, and the result is immutable again.
- Because it's a new variable, it can have a **different type**. This is very handy for conversions:

```rust
let spaces = "   ";        // &str
let spaces = spaces.len(); // usize
```

Try that with `mut` (exercise 2) and the type can't change:

```text
error[E0308]: mismatched types
   |
10 |     let mut spaces = "   ";
   |                      ----- expected due to this value
11 |     println!("spaces: {spaces:?}");
12 |     spaces = spaces.len();
   |              ^^^^^^^^^^^^ expected `&str`, found `usize`
```

> 💡 **Analogy: a new box with the same name tag.** Shadowing doesn't open the old box. It puts a **brand-new box** in front of it with the same label, so when you ask for "spaces" you get the front one. The new box can be a different shape (a number instead of text). The old box is still behind it, hidden, and if the new box was only placed for one room (an inner `{ }`), the old one is back when you leave.

▶ `cargo run -p variables_and_data_types --example shadowing`

## Beyond the Book: `const` vs `static`

A `static` also lives for the whole program, but it is **one value at one fixed memory address**. A `const` is copied into every place it's used.

| | `const` | `static` |
|---|---|---|
| Where the value lives | inlined at each use | one fixed address |
| Can it change? | never | only with `static mut` (needs `unsafe`), or with a thread-safe type like an atomic or `Mutex` |
| Can be used for array lengths, `match` patterns | yes | no |
| Use it for | almost every fixed value | a single shared thing: a global counter, a big lookup table you want stored once |

```rust
static APP_NAME: &str = "Data Types Demo";
static VISITS: AtomicU32 = AtomicU32::new(0);   // a safe changeable global
```

Changing a `static mut` without `unsafe` is refused, because two threads could change it at the same time:

```text
error[E0133]: use of mutable static is unsafe and requires unsafe block
  |
3 |     COUNTER += 1;
  |     ^^^^^^^ use of mutable static
  |
  = note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
```

Prefer `const`. Reach for `static` only when you need exactly one shared thing, and make it changeable only through atomics or a `Mutex` (lesson 04/01).

> 💡 **Analogy: a rubber stamp vs the noticeboard.** A `const` is a **rubber stamp**: wherever it's needed, you stamp a fresh copy. A `static` is the **one noticeboard in the hallway**: there's only one, and everyone walks to the same board. Letting anyone scribble on the noticeboard while others are reading it is chaos, so Rust only allows it with a special pen (atomics, a `Mutex`) or a signed permission slip (`unsafe`).

▶ `cargo run -p variables_and_data_types --example constants_and_statics`

## 4. Data types: every value has one

Rust is **statically typed**: the type of every value must be known at compile time. Usually the compiler **infers** it from the value and how you use it. When several types are possible, you must say which, with an **annotation**. Exercise 3's `parse` could produce any number type:

```text
error[E0284]: type annotations needed
    |
 10 |     let guess = "42".parse().expect("Not a number!");
    |         ^^^^^        ----- type must be known at this point
    |
help: consider giving `guess` an explicit type
    |
 10 |     let guess: /* Type */ = "42".parse().expect("Not a number!");
    |              ++++++++++++
```

The fix is `let guess: u32 = ...`. Types fall into two groups: **scalar** (one value) and **compound** (several values grouped together).

> 💡 **Analogy:** the warehouse manager must know the **shape of every box** before the lorry leaves. Usually they can tell by looking at what's inside (inference). If you say "a number" without saying which size of box, they stop and ask: "which box?" (an annotation).

## 5. Scalar types

### Integers

| Length | Signed | Unsigned |
|---|---|---|
| 8-bit | `i8` (-128 to 127) | `u8` (0 to 255) |
| 16-bit | `i16` | `u16` (0 to 65,535) |
| 32-bit | **`i32`** (the default) | `u32` |
| 64-bit | `i64` | `u64` |
| 128-bit | `i128` | `u128` |
| arch | `isize` | `usize` (64-bit on 64-bit machines) |

A signed `iN` holds -2<sup>N-1</sup> to 2<sup>N-1</sup>-1, and an unsigned `uN` holds 0 to 2<sup>N</sup>-1. Use `i32` unless you have a reason not to, and `usize` for indexes and lengths.

| Literal | Example |
|---|---|
| Decimal | `98_222` (the `_` is only for readability) |
| Hex | `0xff` |
| Octal | `0o77` |
| Binary | `0b1111_0000` |
| Byte (`u8` only) | `b'A'` |
| With a type suffix | `57u8` |

**Integer overflow.** A `u8` holds up to 255. What is `255 + 1`?
- In a **debug** build, Rust checks and **panics**: `attempt to add with overflow`.
- In a **release** build (`--release`), it doesn't check, and the value **wraps around** to 0.
- If the compiler can see the overflow while compiling (`1u32 - 2`, or a `const`), it refuses to build at all.

```text
$ cargo run -p variables_and_data_types --example integers_and_overflow -- 255
thread 'main' (11392) panicked at 01_foundations\03_variables_and_data_types\examples\integers_and_overflow.rs:50:16:
attempt to add with overflow

$ cargo run -p variables_and_data_types --example integers_and_overflow --release -- 255
255 + 1 = 0
```

Relying on the wrap-around is considered a bug. If you want it, say so (see the next section).

> 💡 **Analogy: boxes of different sizes.** A `u8` is an **egg box for 255 eggs**; a `u64` is a lorry. Put the 256th egg in the egg box and in a debug build the **alarm goes off** (panic). In a release build nobody is watching, and like a car's **odometer** rolling from 999999 to 000000, the count quietly starts again at 0.

### Beyond the Book: choosing what overflow does

Each integer type has methods that make overflow behaviour explicit, and they work the same in debug and release:

| Method | `250u8 + 10` gives | Use it when |
|---|---|---|
| `checked_add(10)` | `None` (and `Some(x)` when it fits) | overflow means "this can't be done": prices, sizes |
| `wrapping_add(10)` | `4` | wrapping is the point: hashes, frame counters |
| `saturating_add(10)` | `255` (stops at the edge) | a value has a ceiling or floor: health, volume |
| `overflowing_add(10)` | `(4, true)` | you want the wrapped value **and** to know it wrapped |

Each comes in `_add`, `_sub`, `_mul`, `_div`, `_pow` and more. `3u8.saturating_sub(5)` is `0`, not a panic.

> 💡 **Analogy: three ways to pour into a full glass.** **Checked** is a careful waiter who says "that won't fit" before pouring (`None`). **Saturating** fills the glass to the brim and stops: the rest is ignored. **Wrapping** is the odometer again, starting over from zero, on purpose.

▶ `cargo run -p variables_and_data_types --example integers_and_overflow`

### Floating-point numbers and numeric operations

`f64` is the default (`let x = 2.0;`), and `f32` is half the size. Both are IEEE-754 binary floats, so some decimals can't be stored exactly: `0.1 + 0.2` prints `0.30000000000000004`. Compare floats with a small tolerance, not `==`.

`+ - * / %` work on all number types, but **both sides must be the same type**: `5 + 2.5` is an error (E0277, *cannot add a float to an integer*). Integer division **truncates toward zero**: `7 / 2 == 3` and `-7 / 2 == -3`. `%` is the remainder.

> 💡 **Analogy: counting sweets vs pouring water.** Integers **count** whole things: you can't share 7 sweets between 2 children and give each 3.5, so each gets 3 and 1 is left over (`7 % 2`). Floats **measure**, like water in a jug, but the jug's markings are slightly blurry, so 0.1 + 0.2 lands a hair's width from 0.3.

### Booleans and characters

- **`bool`** is `true` or `false`, one byte. You'll use it in `if` conditions (lesson 05).
- **`char`** is one Unicode scalar value in **single quotes**: `'z'`, `'ℤ'`, `'😻'`. A `char` is always **4 bytes**. Strings use double quotes; `'ab'` is an error (*character literal may only contain one codepoint*).

> 💡 **Analogy:** a `bool` is a **light switch**: on or off, nothing in between. A `char` is **one tile in a Scrabble set that has every letter and emoji in the world**: always one tile, and every tile is the same size box (4 bytes), even though some letters take fewer bytes when written into a string.

▶ `cargo run -p variables_and_data_types --example floats_bools_chars`

### Beyond the Book: casting with `as`, and its pitfalls

`as` converts between number types, and between `char`/`bool` and integers. It **never fails**, which is exactly the danger: when the value doesn't fit, it quietly gives a different number.

| Cast | Result | Why |
|---|---|---|
| `200u8 as u32` | `200` | widening: always safe |
| `300i32 as u8` | `44` | keeps the low 8 bits (300 - 256) |
| `-1i32 as u32` | `4294967295` | same bits, read as unsigned |
| `3.99 as i32` | `3` | the fraction is cut off, not rounded |
| `1e10 as i32` | `2147483647` | floats clamp to the integer's range |
| `-1.5 as u8` | `0` | clamps to 0 |
| `'A' as u32` | `65` | the character's Unicode number |

The safer tools are the standard **conversion traits**:
- **`From`** exists only where a conversion can never lose data: `u32::from(200u8)`, `f64::from(300i32)`. If it compiles, it's safe.
- **`TryFrom`** returns a `Result` where it might not fit: `u8::try_from(300)` is `Err(...)`, and `u8::try_from(42)` is `Ok(42)`.

Use `as` when truncating or clamping really is what you want, or for `int as f64` in arithmetic. Otherwise reach for `from` and `try_from`.

> 💡 **Analogy: pouring a big jug into a small cup.** `as` just **pours**: if the cup is too small, the extra water spills on the floor and nobody says a word (`300` becomes `44`). `from` only lets you pour into a cup that's **definitely big enough**. `try_from` **checks first** and says "won't fit" instead of spilling.

▶ `cargo run -p variables_and_data_types --example casting`

## 6. Compound types

### Tuples

A tuple groups a fixed number of values of **different** types:

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;          // destructuring
let five_hundred = tup.0;     // access by position, from 0
```

The empty tuple `()` is called **unit**. It's the value of "nothing", and functions that don't return anything return `()`.

> 💡 **Analogy: a lunchbox.** One compartment holds a sandwich, one an apple, one a juice: different things, a fixed number of slots. You can tip everything out at once (destructuring) or reach into compartment 0. An empty lunchbox, `()`, means "nothing for lunch".

### Arrays

An array holds a fixed number of values of the **same** type, and the length is part of the type:

```rust
let a: [i32; 5] = [1, 2, 3, 4, 5];
let zeros = [0; 5];              // [0, 0, 0, 0, 0]
let first = a[0];
```

Use an array when the count never changes (the months of the year). For a list that grows, use `Vec` (lesson 02/04).

Every index is **checked**. A bad index that's only known at run time stops the program with a panic, instead of reading whatever memory lies next to the array (a classic security hole in C):

```text
$ cargo run -p variables_and_data_types --example tuples_and_arrays -- 12
thread 'main' (24820) panicked at 01_foundations\03_variables_and_data_types\examples\tuples_and_arrays.rs:52:38:
index out of bounds: the len is 12 but the index is 12
```

If the bad index is a constant like `a[10]`, the compiler spots it and refuses to build (*this operation will panic at runtime*).

> 💡 **Analogy: an egg carton.** Every slot holds the same kind of thing, there's a fixed number of slots, and they're numbered from **0**. Reach for egg number 12 in a 12-egg carton (slots 0 to 11) and Rust **stops your hand** rather than letting you grab whatever happens to be sitting next to the carton.

▶ `cargo run -p variables_and_data_types --example tuples_and_arrays`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Changed a variable declared with plain `let` | E0384 cannot assign twice | `let mut`, or shadow with a new `let`. |
| `let mut x = "text"; x = x.len();` | E0308 mismatched types | Shadow: `let x = x.len();` |
| `let n = "42".parse().expect(...)` | E0284 type annotations needed | `let n: u32 = ...` |
| A `const` whose value doesn't fit its type | E0080 | A bigger type (`u16`, `u32`, ...). |
| `let x: f32 = 3;` | E0308 expected `f32`, found integer | `3.0` |
| `5 + 2.5`, or `i32 + i64` | E0277 / E0308 | Convert one side: `5.0 + 2.5`, `i64::from(a) + b`. |
| `'ab'` | character literal may only contain one codepoint | Strings use `"double quotes"`. |
| `COUNTER += 1` on a `static mut` | E0133 | An atomic or a `Mutex`. Avoid `static mut`. |
| Overflowed a value at run time | panic: attempt to add with overflow | `checked_`, `saturating_` or `wrapping_` methods, or a bigger type. |
| `300 as u8` | none: silently 44 | `u8::try_from(300)` |
| `7 / 2` expecting 3.5 | none: it's 3 | `7.0 / 2.0`, or convert with `f64::from`. |
| `a[i]` with `i >= a.len()` | panic: index out of bounds | Check the index, or use `a.get(i)` (lesson 02/04). |
| `mut` on a value that never changes | warning: does not need to be mutable | Remove the `mut`. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p variables_and_data_types --example mutability
cargo run -p variables_and_data_types --example shadowing
cargo run -p variables_and_data_types --example constants_and_statics
cargo run -p variables_and_data_types --example integers_and_overflow
cargo run -p variables_and_data_types --example integers_and_overflow -- 255
cargo run -p variables_and_data_types --example integers_and_overflow --release -- 255
cargo run -p variables_and_data_types --example floats_bools_chars
cargo run -p variables_and_data_types --example casting
cargo run -p variables_and_data_types --example tuples_and_arrays
cargo run -p variables_and_data_types --example tuples_and_arrays -- 12
cargo run -p variables_and_data_types --example data_types_tour
```

`data_types_tour` is this lesson's original program, kept as a one-page quick reference to scalar types, tuples, arrays and operators (including the bitwise ones: `&`, `|`, `^`, `<<`, `>>`). Try its TODO: changing `1i32 - 2` to `1u32 - 2` makes the build fail with *this arithmetic operation will overflow*, because a `u32` can't go below 0.

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_immutable_assign` | Make it compile (E0384), one word | `cargo run -p variables_and_data_types --features exercises --bin ex01_fix_immutable_assign` |
| 2 | `ex02_fix_shadow_type_change` | Make it compile (E0308), with shadowing | `cargo run -p variables_and_data_types --features exercises --bin ex02_fix_shadow_type_change` |
| 3 | `ex03_fix_parse_annotation` | Make it compile (E0284) | `cargo run -p variables_and_data_types --features exercises --bin ex03_fix_parse_annotation` |
| 4 | `ex04_fix_const_overflow` | Make it compile (E0080), smallest type | `cargo run -p variables_and_data_types --features exercises --bin ex04_fix_const_overflow` |
| 5 | `ex05_safe_arithmetic` | Checked, saturating and wrapping maths, pass 4 tests (Beyond the Book) | `cargo test -p variables_and_data_types --features exercises --bin ex05_safe_arithmetic` |
| 6 | `ex06_tuples_arrays_casts` | Tuples, arrays, `f64::from`, `try_from`, pass 5 tests | `cargo test -p variables_and_data_types --features exercises --bin ex06_tuples_arrays_casts` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p variables_and_data_types --features solutions --bins`, or run one with `--features solutions --bin sol01_fix_immutable_assign`).

**4. Read the mini project** in `src/main.rs`: a **pocket unit converter**.

```bash
cargo run -p variables_and_data_types              # converts 1 of everything
cargo run -p variables_and_data_types -- 26.2      # a marathon: 26.2 mi = 42.16 km
cargo test -p variables_and_data_types             # 6 tests
```

```text
Converting 26.2:
  26.2 mi = 42.16 km    26.2 km = 16.28 mi
  26.2 lb = 11.88 kg    26.2 kg = 57.76 lb
  26.2 in = 66.55 cm    26.2 cm = 10.31 in
  26.2 US gal = 99.18 L    26.2 L = 6.92 US gal
```

It uses `const` for the conversion factors, a constant **array of tuples** as its table, **shadowing** to turn the typed text into an `f64`, and destructuring to take each row apart. The tests compare floats within a tiny margin, never with plain `==` on a calculated value.

> 💡 **Analogy:** a laminated **conversion card** you keep in your pocket: each row is printed once (the constants), the card has exactly four rows (the array), and each row has three columns (the tuple).

## Checklist

- [ ] [3.1 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- [ ] [3.2 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [ ] Beyond the Book: integer overflow: `checked_*`, `wrapping_*`, `saturating_*`
- [ ] Beyond the Book: casting with `as` and its pitfalls
- [ ] Beyond the Book: `const` vs `static`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Variables are immutable unless declared `mut`. Constants are always immutable, typed, and worked out at compile time.
- Shadowing makes a **new** variable with the same name, so it can change the type; `mut` can't.
- Scalars: integers (`i32` by default, `usize` for indexes), floats (`f64` by default), `bool`, and `char` (4 bytes, single quotes).
- Overflow panics in debug and wraps in release. Say what you want with `checked_`, `wrapping_` or `saturating_`. `as` never fails, so prefer `from` and `try_from`.
- Tuples group different types. Arrays hold a fixed number of one type, and every index is checked.

**In one sentence for a child:** every value lives in a labelled box of the right shape and size, most boxes are sealed so nobody changes them by surprise, and Rust checks that nothing is ever too big for its box.

**Next:** [04 Functions and Comments](../04_functions_and_comments/README.md) (Book 3.3–3.4)
