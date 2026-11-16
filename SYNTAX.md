# Rust Syntax Quick Reference (with Python equivalents)

Every symbol, keyword and common form in one page, each with a plain one-line meaning, **its closest Python equivalent**, and the lesson that teaches it in depth. Use your editor's search (Ctrl+F) for the symbol you're stuck on.

**The Python column** uses Python 3.12 (one entry needs 3.13 and says so). A "—" means Python has nothing like it; the short reason next to it is often the most useful part, because it shows what Rust checks for you before the program runs that Python leaves to run time (or doesn't check at all). Every Python claim on this page was checked by running it.

Official references for anything not here: [Book appendix B: operators and symbols](https://doc.rust-lang.org/book/appendix-02-operators.html) · [appendix A: keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html) · [The Rust Reference](https://doc.rust-lang.org/reference/) · [std docs](https://doc.rust-lang.org/std/)

**Contents:** [Coming from Python](#coming-from-python-the-big-differences) · [Symbol index](#symbol-index-what-does-this-mean) · [Comments and printing](#comments-and-printing) · [Variables](#variables-and-constants) · [Types and literals](#types-and-literals) · [Operators](#operators) · [Functions and closures](#functions-and-closures) · [Control flow](#control-flow) · [Ownership and references](#ownership-borrowing-and-lifetimes) · [Structs, enums, impl](#structs-enums-and-impl) · [Traits and generics](#traits-and-generics) · [Patterns](#patterns) · [Errors](#error-handling) · [Collections](#collections-and-strings) · [Modules](#modules-and-visibility) · [Macros](#macros) · [Attributes](#attributes) · [Smart pointers](#smart-pointers) · [Concurrency and async](#concurrency-and-async) · [Unsafe](#unsafe-and-ffi) · [Newer syntax](#newer-syntax) · [All keywords](#all-keywords) · [Python keywords in Rust](#python-keywords-and-what-rust-uses-instead) · [Cargo commands](#cargo-commands)

Checked against the Book's complete lists: every entry in [appendix A](https://doc.rust-lang.org/book/appendix-01-keywords.html) (keywords) and [appendix B](https://doc.rust-lang.org/book/appendix-02-operators.html) (operators and symbols) appears here.

---

## Coming from Python: the big differences

| | Python | Rust |
|---|---|---|
| **Types** | checked while the program runs; hints are optional and never enforced | checked before it runs; every value has one fixed type |
| **Memory** | a garbage collector frees an object when nothing refers to it any more | each value has one **owner**; it is freed the moment its owner goes out of scope |
| **Names** | every name is a reference: `b = a` makes two names for one object | `let b = a;` **moves** the value to `b`; `&a` lends it out (a borrow) |
| **Changing things** | anything can change | nothing changes unless it is marked `mut` |
| **Missing values** | `None` can turn up anywhere | `Option<T>`: the compiler makes you handle the "nothing" case |
| **Errors** | exceptions jump up the call stack invisibly | `Result<T, E>` is an ordinary return value; `?` passes it up, visibly |
| **Blocks** | indentation and `:`; statements | `{ }` and `;`; almost everything is an expression that has a value |
| **Interfaces** | duck typing, `Protocol`, abstract base classes | traits, checked at compile time |
| **Running** | `python main.py` (interpreted) | `cargo run` (compiled to a native program first) |

> 💡 **Analogy:** Python is a relaxed librarian who lends any book to anyone and sorts out problems when they happen. Rust is a strict librarian who checks every loan slip *before the library opens*, so once it is open nothing can go wrong. The extra symbols in Rust (`&`, `mut`, `'a`, `?`) are those loan slips.

---

## Symbol index: "what does this mean?"

| You see | It means | Example | Python |
|---|---|---|---|
| `::` | path separator: an item inside a module or type | `std::fs::read`, `String::new()` | `.`: `os.path.join`, `str.upper` |
| `::path` (leading) | absolute path from the crate root / an external crate | `::std::fmt::Display` | — (Python imports are absolute already) |
| `<Type>::item` | path on a type that has no plain name | `<[i32]>::len(&a)`, `<&str>::default()` | — |
| `,` | separates arguments, elements and fields | `f(a, b)`, `[1, 2]` | same: `f(a, b)`, `[1, 2]` |
| `:` | "has type", struct field value, loop label, or trait bound | `x: i32`, `User { age: 9 }`, `'a: loop`, `T: Clone` | type hint `x: int`; field value uses `=`: `User(age=9)`; bound `def f[T: Clone]`; no loop labels |
| `(x,)` | a ONE-element tuple (the comma matters; `(x)` is just `x` in brackets) | `let t = (5,);` | identical: `t = (5,)` |
| `...` | old inclusive-range pattern: an error in edition 2024 (E0783), write `..=` | ~~`1...9`~~ → `1..=9` | `...` is Python's `Ellipsis` placeholder: unrelated |
| `.` | field access or method call | `user.name`, `s.len()` | same: `user.name`, `s.upper()` |
| `;` | ends a statement; a block's last line **without** `;` is its value | `let x = 5;` | a new line ends a statement; there is no "last line is the value" rule, so write `return` |
| `&` | borrow (a shared reference); also bitwise AND | `&s`, `fn f(s: &str)` | no borrows (every name is already a reference); bitwise AND is the same `&` |
| `&mut` | mutable borrow: may change the value | `&mut v`, `fn f(v: &mut Vec<i32>)` | — (pass a list and any function can change it) |
| `*` | dereference ("the value behind this reference"); also multiply; raw pointer types | `*count += 1`, `*const T` | multiply is the same; `*` also unpacks (`*args`, `[*a, *b]`); no dereference |
| `'a` | a lifetime name | `fn f<'a>(x: &'a str) -> &'a str` | — (the garbage collector keeps objects alive, so Python never needs lifetimes) |
| `'static` | lives for the whole program / owns no short borrows | `&'static str`, `T: 'static` | — |
| `'label:` | a loop label | `'outer: loop { break 'outer; }` | — (put the loops in a function and `return`) |
| `'c'` | a `char` (one Unicode character) | `let c = 'é';` | `c = "é"` (a 1-character `str`; in Python `'` and `"` both make strings) |
| `b'A'` / `b"hi"` | a byte (`u8`) / a byte string (`&[u8; N]`) | | `ord("A")` or `b"A"[0]` / `b"hi"` |
| `!` | macro call; logical NOT; the never type | `println!()`, `!done`, `fn f() -> !` | no macros; `not done`; `-> NoReturn` |
| `?` | if `Err`/`None`, return it early; else unwrap the value | `let n: i32 = s.parse()?;` | — (`n = int(s)`: an exception leaves the function by itself) |
| `->` | function return type | `fn add(a: i32) -> i32` | same in hints: `def add(a: int) -> int:` |
| `=>` | match arm: pattern => result | `Some(x) => x,` | `case x:` (the colon) |
| `\|args\|` | closure parameters | `\|x\| x + 1` | `lambda x: x + 1` |
| `\|` | "or" in patterns; bitwise OR | `1 \| 2 => ...` | same: `case 1 \| 2:`, bitwise OR; also `int \| None` in hints |
| `_` | ignore this value / let the compiler infer the type | `let _ = f();`, `Vec<_>` | `_ = f()`, `case _:`; no "infer this type" marker |
| `..` | range (end excluded); "the rest" in patterns and struct update | `0..10`, `Point { x, .. }`, `..Default::default()` | `range(0, 10)`; `*rest` in patterns; `dataclasses.replace(...)` |
| `..=` | inclusive range | `1..=100`, `'a'..='z' =>` | `range(1, 101)` |
| `@` | bind a name while matching a pattern | `n @ 1..=9 => ...` | `as`: `case int() as n if 1 <= n <= 9:` (Python's `@` is a decorator or matrix multiply) |
| `#[...]` / `#![...]` | attribute on the next item / on the whole file or module | `#[derive(Debug)]`, `#![allow(dead_code)]` | closest is a decorator: `@dataclass` |
| `$name` | a macro variable (inside `macro_rules!`) | `$x:expr` | — (Python has no macros) |
| `<T>` | a generic type parameter | `Vec<T>`, `fn f<T: Display>(t: T)` | square brackets: `list[T]`, `def f[T](t: T)` |
| `::<T>` | "turbofish": give a generic type explicitly | `"5".parse::<i32>()`, `collect::<Vec<_>>()` | — (call the type instead: `int("5")`, `list(...)`) |
| `T: Trait` | trait bound: T must implement Trait | `fn show<T: Display>(t: T)` | `def show[T: Display](t: T)` (with `Display` a `Protocol`) |
| `+` (in bounds) | must implement **both** traits | `T: Clone + Debug` | — (no "both" types; write one `Protocol` with both methods) |
| `dyn Trait` | a trait object (type decided at run time) | `Box<dyn Error>` | every Python value already works this way; hint with the base class: `Exception` |
| `impl Trait` | "some type that implements Trait" | `fn f() -> impl Iterator<Item = u8>` | `def f() -> Iterator[int]:` |
| `Self` / `self` | the current type / the current value (method receiver) | `fn new() -> Self`, `fn len(&self)` | `typing.Self` / `self`: the same idea |
| `r#name` | raw identifier: use a keyword as a name | `r#type` | trailing underscore by convention: `type_`, `class_` |
| `r"..."` / `r#"..."#` | raw string: no escapes | `r"C:\path"`, `r#"say "hi""#` | same `r"C:\path"`; `r'say "hi"'` |
| `{}` inside `"..."` | a format placeholder (in `println!`, `format!`) | `println!("{x} is {}", y)` | an f-string: `print(f"{x} is {y}")` |

---

## Comments and printing

| Rust | Meaning | Python |
|---|---|---|
| `// text` | line comment | `# text` |
| `/* text */` | block comment | — (start every line with `#`) |
| `/// text` | doc comment for the next item (shows in `cargo doc`) | a docstring: `"""text"""` as the first line *inside* the function or class |
| `//! text` | doc comment for the enclosing file/module/crate | a docstring at the top of the file |
| `/** text */` / `/*! text */` | block versions of `///` and `//!` (rarely used) | — |
| `println!("...")` / `print!` | print with / without a newline (stdout) | `print("...")` / `print("...", end="")` |
| `eprintln!` / `eprint!` | print to stderr (errors) | `print("...", file=sys.stderr)` |
| `format!("...")` | build a `String` instead of printing | `f"..."` or `"...".format(...)` |
| `dbg!(expr)` | print file:line, the expression and its value to stderr; returns the value | `print(f"{expr=}")` prints `expr=value` (no file:line, returns nothing) |

**Format placeholders** ([lesson 01/01](01_foundations/01_hello_world/README.md)). Rust's format language was modelled on Python's, so most of it is identical:

| Placeholder | Prints | Python |
|---|---|---|
| `{}` | with `Display` (for people) | `{x}` (uses `str()`) |
| `{:?}` / `{:#?}` | with `Debug` / pretty-printed `Debug` | `{x!r}` (uses `repr()`) / `pprint.pformat(x)` |
| `{x}` / `{0}` | variable `x` / the first argument | `f"{x}"` / `"{0}".format(a)` |
| `{:5}` `{:<5}` `{:>5}` `{:^5}` | width 5 / left / right / centre aligned | same: `{x:5}` `{x:<5}` `{x:>5}` `{x:^5}` (in both, numbers go right and text left by default) |
| `{:05}` | width 5, padded with zeros | same: `{x:05}` |
| `{:.2}` | 2 decimal places | `{x:.2f}`. Careful: Python's `{x:.2}` means 2 *significant digits* (`3.14159` → `3.1`) |
| `{:x}` `{:X}` `{:b}` `{:o}` `{:e}` | hex / HEX / binary / octal / scientific | same letters; `e` looks different: Rust `1.2345e3`, Python `1.234500e+03` |
| `{:#x}` `{:+}` | with `0x` prefix / always show the sign | same: `{x:#x}` `{x:+}` |
| `{{` `}}` | a literal `{` or `}` | same |

---

## Variables and constants

[Lesson 01/03](01_foundations/03_variables_and_data_types/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `let x = 5;` | immutable variable (can't be changed) | `x = 5` (Python can't forbid changes; `x: Final = 5` asks a type checker to) |
| `let mut x = 5;` | mutable variable | `x = 5` (every Python name can be reassigned) |
| `let x: i32 = 5;` | with a type annotation | `x: int = 5` (a hint only; nothing checks it while running) |
| `let x = x + 1;` | **shadowing**: a new variable with the same name (may change type) | `x = x + 1` (Python just points the name somewhere new) |
| `let (a, b) = (1, 2);` | destructure while declaring | `a, b = 1, 2` |
| `let x;` then `x = 5;` | declare now, assign once later | `x: int` then `x = 5` |
| `const MAX: u32 = 100;` | compile-time constant, type required, `UPPER_CASE` | `MAX: Final = 100` (`UPPER_CASE` is only a convention) |
| `static NAME: &str = "app";` | one fixed memory location for the whole program | a module-level variable: `NAME = "app"` |
| `static mut` | mutable global: `unsafe` to use, avoid (use atomics or `Mutex`) | a module global changed with the `global` keyword (nothing checks it) |

---

## Types and literals

[Lesson 01/03](01_foundations/03_variables_and_data_types/README.md)

| Type | What it is | Python |
|---|---|---|
| `i8 i16 i32 i64 i128 isize` | signed integers (`i32` is the default) | `int`: one type, any size, never overflows |
| `u8 u16 u32 u64 u128 usize` | unsigned integers (`usize` for indexes and lengths) | `int` (Python has no unsigned type) |
| `f32 f64` | floating point (`f64` is the default) | `float` (always 64-bit, like `f64`) |
| `bool` | `true` or `false` | `bool`: `True` or `False` (capitals) |
| `char` | one Unicode character, 4 bytes, `'a'` | — (a 1-character `str`) |
| `()` | the unit type: "nothing" (a function with no return value returns this) | `None` (what a function without `return` gives back) |
| `!` | the never type: this code never returns (`panic!`, `loop {}`) | `typing.NoReturn` |
| `(i32, &str)` | tuple; access with `.0`, `.1` | `tuple[int, str]`; access with `t[0]`, `t[1]` |
| `[i32; 3]` | array: fixed length 3 | — (a `list` can always grow; a `tuple` is the fixed-size choice) |
| `[0; 5]` | array of five zeros | `[0] * 5` |
| `&[T]` | slice: a borrowed view of part of an array or `Vec` | — (`v[1:3]` makes a **copy**; `memoryview` is the view, for bytes) |
| `&str` / `String` | borrowed text / owned, growable text | `str` for both (always immutable: "changing" one builds a new one) |
| `type Km = i32;` | type alias (a new name, not a new type) | `type Km = int` |

| Literal | Meaning | Python |
|---|---|---|
| `98_222` | `_` is just a separator for readability | same: `98_222` |
| `0xff` `0o77` `0b1111_0000` | hex, octal, binary | same |
| `5u8` `2.5f32` `1_000i64` | type suffix | — (there is one `int` and one `float`) |
| `1e6` | 1,000,000.0 (a float) | same |
| `"text"` | string literal, type `&'static str` | `"text"` or `'text'` (in Rust `'x'` is a `char`, never a string) |
| `"\n \t \\ \" \u{1F600}"` | escapes: newline, tab, backslash, quote, Unicode | `"\n \t \\ \" \U0001F600"` (Python's long Unicode escape is `\U` + 8 hex digits) |
| `r"C:\no\escapes"` / `r#"has "quotes""#` | raw strings | `r"C:\no\escapes"` / `r'has "quotes"'` |
| `b'A'` / `b"bytes"` / `br"raw bytes"` | byte / byte string / raw byte string | `ord("A")` / `b"bytes"` / `rb"raw bytes"` |
| `c"text"` | C string literal (`&CStr`), for FFI | `ctypes.c_char_p(b"text")` |

**Casts:** `x as u8` converts primitive numbers. It can silently truncate or wrap; prefer `u8::try_from(x)` when the value might not fit. **Python:** `int(x)` and `float(x)`. Python never wraps: `300 as u8` is `44` in Rust, while in Python you'd write `300 & 0xFF` to get the same. For `try_from`, check the range yourself (`if not 0 <= x <= 255: raise ValueError`).

---

## Operators

| Operator | Meaning | Assign form | Python |
|---|---|---|---|
| `+ - * / %` | add, subtract, multiply, divide, remainder | `+= -= *= /= %=` | same symbols, **but** `/` always gives a float (`7 / 2` is `3.5`); whole-number division is `//`, which rounds down (`-7 // 2` is `-4`; Rust's `-7 / 2` is `-3`); and `%` takes the sign of the right side (`-7 % 3` is `2`; Rust gives `-1`) |
| `== != < > <= >=` | comparison (needs `PartialEq` / `PartialOrd`) | | same; Python can also chain: `1 < x < 9` |
| `&& \|\| !` | logical AND, OR, NOT (short-circuiting) | | `and or not` |
| `& \| ^ ! << >>` | bitwise AND, OR, XOR, NOT, shift | `&= \|= ^= <<= >>=` | `& \| ^ ~ << >>` (bitwise NOT is `~` in Python) |
| `a..b` `a..=b` `a..` `..b` `..` | ranges | | `range(a, b)`, `range(a, b + 1)`, `itertools.count(a)`, `range(b)`; in slices `v[a:]`, `v[:b]`, `v[:]` |
| `-x` | negation | | same |
| `&x` `&mut x` `*x` | borrow, mutable borrow, dereference | | — |
| `expr?` | propagate `Err`/`None` | | — (exceptions propagate by themselves) |
| `x as T` | primitive cast | | `int(x)`, `float(x)` |

**Overloading:** implement the matching `std::ops` trait for your own type; Python's equivalent is a "dunder" method in your class. `&&`, `||`, `=`, `..` and `?` can't be overloaded this way. See [lesson 04/03](04_systems_level/03_advanced_features/README.md).

| Operator | Rust trait | Python method |
|---|---|---|
| `+` `-` `*` `/` `%` | `Add` `Sub` `Mul` `Div` `Rem` | `__add__` `__sub__` `__mul__` `__truediv__` `__mod__` |
| unary `-` / `!` | `Neg` / `Not` | `__neg__` / `__invert__` (that's `~`; Python's `not` only asks `__bool__`) |
| `&` `\|` `^` `<<` `>>` | `BitAnd` `BitOr` `BitXor` `Shl` `Shr` | `__and__` `__or__` `__xor__` `__lshift__` `__rshift__` |
| `+=` (and so on) | `AddAssign` (and so on) | `__iadd__` (and so on) |
| `x[i]` | `Index` / `IndexMut` | `__getitem__` / `__setitem__` |
| `*x` | `Deref` / `DerefMut` | — (closest: forwarding with `__getattr__`) |
| `==` | `PartialEq` | `__eq__` |
| `<` `>` `<=` `>=` | `PartialOrd` | `__lt__` `__gt__` `__le__` `__ge__` (or `@functools.total_ordering`) |

Integer overflow panics in debug builds and wraps in release. Say what you mean with `checked_add`, `wrapping_add`, `saturating_add` or `overflowing_add`. **Python:** `int` never overflows (`2**64` is just a bigger number), so none of these exist.

---

## Functions and closures

[Lessons 01/04](01_foundations/04_functions_and_comments/README.md) · [03/04](03_abstraction/04_iterators_and_closures/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `fn name(a: i32, b: &str) -> bool { ... }` | function: parameter types are required | `def name(a: int, b: str) -> bool:` (hints are optional and not checked while running) |
| last expression without `;` | the return value | — (Python needs `return`; without it the function returns `None`) |
| `return x;` | return early | `return x` |
| `fn main() -> Result<(), Box<dyn Error>>` | `main` may return a `Result`, so `?` works in it | `if __name__ == "__main__": main()`; an uncaught exception ends the program with exit code 1 |
| `\|x\| x + 1` | closure | `lambda x: x + 1` |
| `\|x: i32\| -> i32 { x + 1 }` | closure with types | a nested `def` (a `lambda` can't have hints or more than one expression) |
| `move \|\| ...` | closure takes ownership of what it uses | — (a Python closure sees the variable itself, so later changes show up: in a loop, `lambda: i` gives the last `i`; `lambda i=i: i` freezes today's value) |
| `Fn` / `FnMut` / `FnOnce` | closure that reads / mutates / consumes what it captured | — (one kind of function: `Callable`) |
| `fn apply(f: impl Fn(i32) -> i32)` | take a closure | `def apply(f: Callable[[int], int]):` |
| `fn make() -> impl Fn(i32) -> i32` | return a closure | `def make() -> Callable[[int], int]: return lambda x: x + 1` |
| `fn(i32) -> i32` | a function pointer type | `Callable[[int], int]` (functions and lambdas are the same kind of value) |
| `const fn` | can run at compile time | — |
| `async fn` | returns a future (see async) | `async def` |

---

## Control flow

[Lessons 01/05](01_foundations/05_control_flow/README.md) · [02/02](02_core_types_and_errors/02_enums_and_pattern_matching/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `if x > 5 { ... } else if ... { ... } else { ... }` | condition must be a `bool` (no truthy numbers) | `if x > 5:` / `elif ...:` / `else:` (Python accepts "truthy" values like `if items:`; Rust wants `if !items.is_empty()`) |
| `let y = if ok { 1 } else { 2 };` | `if` is an expression; both branches must have the same type | `y = 1 if ok else 2` |
| `loop { ... }` | forever, until `break` | `while True:` |
| `let v = loop { break 42; };` | `break` with a value | — (set a variable, then `break`) |
| `while x < 10 { ... }` | loop while true | `while x < 10:` |
| `for item in &list { ... }` | loop over anything iterable | `for item in items:` |
| `for i in 0..10` / `(0..10).rev()` / `.step_by(2)` | count up / down / in steps | `range(10)` / `reversed(range(10))` / `range(0, 10, 2)` |
| `for (i, x) in v.iter().enumerate()` | with the index | `for i, x in enumerate(v):` |
| `continue` / `break` | next iteration / leave the loop | same |
| `'outer: for ... { break 'outer; }` | break out of an outer loop | — (put the loops in a function and `return`) |
| `match x { 1 => ..., 2 \| 3 => ..., _ => ... }` | match: every case must be covered | `match x:` with `case 1:`, `case 2 \| 3:`, `case _:` (Python doesn't check that every case is covered) |
| `if let Some(v) = opt { ... } else { ... }` | match one pattern | `if (v := opt) is not None:` / `else:` |
| `while let Some(top) = stack.pop() { ... }` | loop while the pattern matches | `while stack: top = stack.pop()` |
| `let Some(v) = opt else { return; };` | `let ... else`: bind, or leave | `if (v := opt) is None: return` |
| `if let Some(a) = x && a > 3 { ... }` | let chains (edition 2024) | `if (a := x) is not None and a > 3:` |
| `matches!(x, Some(1..=9))` | pattern test that returns a `bool` | `x is not None and 1 <= x <= 9` |

---

## Ownership, borrowing and lifetimes

[Lessons 01/06](01_foundations/06_ownership/README.md) · [03/01](03_abstraction/01_generics_traits_lifetimes/README.md)

This is the part of Rust with the least Python behind it: Python shares objects freely and lets the garbage collector clean up, so most rows are "—".

| Rust | Meaning | Python |
|---|---|---|
| `let b = a;` | **move**: `a` can't be used any more (unless the type is `Copy`) | `b = a` makes two names for **one** object; nothing moves, nothing is copied |
| `let b = a.clone();` | deep copy: both stay usable | `b = copy.deepcopy(a)` (`a.copy()` or `list(a)` for a shallow copy) |
| `#[derive(Clone, Copy)]` | small values copied automatically instead of moved | — (numbers, strings and tuples can't change in Python, so sharing them is safe) |
| `&a` | shared borrow: read-only, any number at once | — (every name is a reference, and nothing stops two parts of a program changing the same list) |
| `&mut a` | mutable borrow: exactly one at a time, and no `&a` meanwhile | — |
| `*r` | the value a reference points to | — |
| `fn f(s: &str)` | borrow a parameter (the caller keeps ownership) | `def f(s: str):` (Python always passes a reference to the object) |
| `fn f(s: String)` | take ownership of a parameter | — (the caller can always keep using it) |
| `fn f<'a>(x: &'a str, y: &'a str) -> &'a str` | the result lives as long as both inputs | — (the garbage collector keeps an object alive while anything refers to it) |
| `struct S<'a> { r: &'a str }` | a struct holding a reference can't outlive what it points to | — |
| `'_` | "some lifetime, inferred" | — |
| `&'a T` / `&'a mut T` | a reference that is valid for lifetime `'a` | — |
| `T: 'a` | `T` holds no references shorter than `'a` | — |
| `'b: 'a` | lifetime `'b` lasts at least as long as `'a` ("outlives") | — |
| `T: 'static` | `T` holds no short-lived borrows (an owned `String` qualifies) | — |
| `std::mem::take(&mut x)` / `replace` / `swap` | move out of a `&mut` by leaving something in its place | `old, x = x, []` / `a, b = b, a` |

---

## Structs, enums and impl

[Lessons 02/01](02_core_types_and_errors/01_structs/README.md) · [02/02](02_core_types_and_errors/02_enums_and_pattern_matching/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `struct User { name: String, age: u32 }` | struct with named fields | `@dataclass` on `class User:` with fields `name: str` and `age: int` |
| `struct Point(i32, i32);` | tuple struct; fields `.0`, `.1` | `class Point(NamedTuple):` with `x: int`, `y: int` (fields `p[0]` or `p.x`) |
| `struct Marker;` | unit struct (no fields) | `class Marker: pass` |
| `User { name, age: 9 }` | create one (field init shorthand for `name`) | `User(name=name, age=9)` |
| `User { age: 10, ..other }` | struct update: take the rest from `other` | `dataclasses.replace(other, age=10)` |
| `enum Dir { Up, Down }` | enum: a value is exactly one variant | `class Dir(Enum):` with `UP = auto()`, `DOWN = auto()` |
| `enum Msg { Quit, Move { x: i32 }, Write(String) }` | variants can carry data | one dataclass per variant and a union: `type Msg = Quit \| Move \| Write` |
| `Option<T>` = `Some(v)` / `None` | maybe a value | `T \| None`: the value itself, or `None` |
| `Result<T, E>` = `Ok(v)` / `Err(e)` | success or error | — (return the value, or `raise` the error) |
| `impl User { fn new() -> Self { ... } }` | associated function (no `self`): call `User::new()` | `def __init__(self, ...)` for the constructor; others are `@classmethod` or `@staticmethod` |
| `fn name(&self)` / `(&mut self)` / `(self)` | method that reads / changes / consumes the value | `def name(self):` (one form; Python doesn't say whether a method changes or uses up the object) |
| `#[derive(Debug, Clone, PartialEq, Default)]` | generate trait impls automatically | `@dataclass` writes `__repr__` (≈ `Debug`) and `__eq__` (≈ `PartialEq`); `copy.copy` (≈ `Clone`); field defaults (≈ `Default`) |

---

## Traits and generics

[Lessons 03/01](03_abstraction/01_generics_traits_lifetimes/README.md) · [03/07](03_abstraction/07_trait_objects_and_oop/README.md) · [04/03](04_systems_level/03_advanced_features/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `trait Speak { fn speak(&self) -> String; }` | define shared behaviour | `class Speak(Protocol):` with `def speak(self) -> str: ...` (or an `ABC` with `@abstractmethod`) |
| `fn greet(&self) -> String { ... }` (inside a trait) | default method: implementors get it for free | an ordinary method in an `ABC` base class |
| `impl Speak for Dog { ... }` | implement a trait for a type | `class Dog(Speak):` with an `ABC`; with a `Protocol`, just give `Dog` a `speak` method (duck typing) |
| `fn f<T: Speak>(x: T)` | generic with a trait bound (static dispatch) | `def f[T: Speak](x: T):` |
| `fn f(x: &impl Speak)` | the same, shorter | `def f(x: Speak):` |
| `fn f<T>(x: T) where T: Speak + Clone` | bounds in a `where` clause | — (no "both" bounds) |
| `Box<dyn Speak>` / `&dyn Speak` | trait object: mixed types, decided at run time | `list[Speak]`: every Python method call is decided at run time |
| `trait Iterator { type Item; }` | associated type | — (make the Protocol generic: `class Iterator[T](Protocol):`) |
| `trait Child: Parent` | supertrait: implementing `Child` requires `Parent` | `class Child(Parent, Protocol):` |
| `impl<T: Display> MyTrait for T` | blanket impl: for every type that qualifies | — |
| `T: ?Sized` | `T` may be unsized (like `str`, `[T]`, `dyn Trait`) | — |
| `Trait::method(&x)` / `Type::method(&x)` | call a method through its trait or type (to pick between same-named methods) | `Speak.speak(x)` / `Dog.speak(x)`: calling through the class works the same way |
| `<T as Trait>::method(x)` | fully qualified call (when two traits share a method name) | `Parent.method(x)` |
| `Iterator<Item = u8>` | a trait with its associated type fixed | `Iterator[int]` |
| `Self::Item` | an associated type, used inside the trait or impl | — |
| `type Output = P;` (in an `impl`) | choose the associated type for this implementation | — |
| `const SIDES: u32;` (in a trait) | associated constant; used as `Square::SIDES` | a class attribute: `SIDES: ClassVar[int] = 4`, used as `Square.SIDES` |
| `trait Add<Rhs = Self>` | generic parameter with a default | `class Add[Rhs = int]:` (Python 3.13) |
| `T: 'a + Trait` / `dyn Trait + Send` | bounds combined with `+` (lifetimes and auto traits too) | — |
| `F: for<'a> Fn(&'a str) -> usize` | higher-ranked bound: works for EVERY lifetime `'a` | — |
| `-> impl Iterator<Item = &'a i32> + use<'a>` | precise capturing: say which generics/lifetimes the hidden type may use | — |
| `type Item<'a> where Self: 'a;` | generic associated type (GAT) | — |
| `struct Matrix<const N: usize>` | const generic: a value as a generic parameter | — (closest: `Literal[3]` in a hint) |
| `PhantomData<T>` | marks a type parameter that no field uses | — (Python generics never need a field to use `T`) |

---

## Patterns

[Lesson 03/08](03_abstraction/08_patterns_and_matching/README.md)

Python's `match` statement (3.10+) borrowed many of these ideas, so this table maps closely.

| Pattern | Matches | Python `case` |
|---|---|---|
| `5` / `"text"` / `'c'` | that exact value | `case 5:` / `case "text":` / `case "c":` |
| `x` | anything, and names it `x` | `case x:` (the same trap in both: a bare name matches everything) |
| `_` | anything, ignored | `case _:` |
| `_x` | anything, named but unused (no warning) | — |
| `1 \| 2` | either | `case 1 \| 2:` |
| `1..=9` / `'a'..='z'` | a range | no range patterns; use a guard: `case int() as n if 1 <= n <= 9:` |
| `(a, b)` / `[first, .., last]` | destructure a tuple / slice | `case (a, b):` / `case [first, *_, last]:` |
| `Point { x, y: 0 }` | struct with `y == 0`, binding `x` | `case Point(x=x, y=0):` |
| `Some(v)` / `Err(e)` | enum variant, binding its data | a class pattern: `case Write(text):` (dataclasses allow positional fields) |
| `..` | "the rest" | `*rest` in sequences, `**rest` in dicts |
| `n @ 1..=9` | match a range and bind the value | `case int() as n if 1 <= n <= 9:` |
| `Some(x) if x > 5` | match guard: an extra condition | `case x if x > 5:` |
| `&(a, b)` | match through a reference | — |
| `ref x` / `ref mut x` | bind by reference instead of moving (older style) | — |

---

## Error handling

[Lesson 02/05](02_core_types_and_errors/05_error_handling/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `panic!("msg")` | stop the program (for bugs, not expected failures) | `raise RuntimeError("msg")` that nothing catches |
| `x?` | return the `Err`/`None` early, otherwise unwrap | — (an exception leaves the function by itself; Rust makes you write `?` so every place that can fail is visible) |
| `.unwrap()` / `.expect("why")` | value, or panic (with your message) | just use the value / `assert x is not None, "why"` |
| `.unwrap_or(v)` / `.unwrap_or_else(\|\| ...)` / `.unwrap_or_default()` | value, or a fallback | `x if x is not None else v` (or `d.get(k, v)`) / `... else make()` / `x or 0` (careful: `or` also replaces `0` and `""`) |
| `.map(f)` / `.and_then(f)` / `.ok_or(e)` / `.ok()` | transform, chain, convert `Option`↔`Result` | `f(x) if x is not None else None`; for `ok_or`: `if x is None: raise e`; for `ok()`: `try`/`except` that returns `None` |
| `if let Err(e) = f() { ... }` | handle only the error | `try: f()` / `except Exception as e:` |
| `Box<dyn std::error::Error>` | "any error" return type | `Exception` |
| `impl From<IoError> for MyError` | lets `?` convert errors automatically | `except OSError as e: raise MyError(...) from e` |
| `#[derive(thiserror::Error)]` / `anyhow::Result` | error types for libraries / applications | `class MyError(Exception): ...` / let exceptions travel up, adding context with `raise ... from e` |

---

## Collections and strings

[Lesson 02/04](02_core_types_and_errors/04_collections/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `vec![1, 2, 3]` / `Vec::new()` | create a vector | `[1, 2, 3]` / `[]` |
| `v.push(x)` / `v.pop()` / `v.len()` | add / remove last / length | `v.append(x)` / `v.pop()` / `len(v)` (on an empty list Python's `pop()` raises `IndexError`; Rust's returns `None`) |
| `v[0]` / `v.get(0)` | index (panics if out of range) / `Option` | `v[0]` (raises `IndexError`) / `v[0] if v else None` |
| `&v[1..3]` | slice of elements 1 and 2 | `v[1:3]` (but a Python slice is a **copy**; a Rust slice borrows the original) |
| `&v[..]` / `&v[2..]` / `&v[..2]` / `&v[1..=3]` | all / from 2 / before 2 / 1 to 3 inclusive | `v[:]` / `v[2:]` / `v[:2]` / `v[1:4]` |
| `String::from("hi")` / `"hi".to_string()` | owned string | `"hi"` / `str(x)` |
| `s.push_str("x")` / `s.push('c')` / `s1 + &s2` / `format!` | grow a string | `s += "x"` (builds a new string) / `s1 + s2` / an f-string; for many pieces `"".join(parts)` |
| `s.chars()` / `s.bytes()` / `s.len()` | characters / bytes / length in **bytes** | `for c in s` / `s.encode()` / `len(s.encode())`. Python's `len(s)` counts characters: `len("café")` is 4, its UTF-8 length is 5 |
| `&s[0..4]` | string slice (byte offsets, must fall on character boundaries) | `s[0:4]` (Python counts characters, so it never splits one) |
| `HashMap::new()` / `m.insert(k, v)` / `m.get(&k)` | hash map | `{}` / `m[k] = v` / `m.get(k)` |
| `*m.entry(k).or_insert(0) += 1` | insert-or-update | `m[k] = m.get(k, 0) + 1` (or `collections.Counter`, `defaultdict(int)`) |
| `HashSet`, `BTreeMap`, `VecDeque`, `BinaryHeap` | set, sorted map, double-ended queue, priority queue | `set`; no sorted dict in the standard library (`sorted(m.items())`); `collections.deque`; `heapq` (a **min**-heap, while `BinaryHeap` is a max-heap) |

**Common iterator chain:** `v.iter().filter(|x| ...).map(|x| ...).collect::<Vec<_>>()`. See [lesson 03/04](03_abstraction/04_iterators_and_closures/README.md). **Python:** a list comprehension, `[f(x) for x in v if cond(x)]`. Rust iterators are lazy like a generator expression, `(f(x) for x in v if cond(x))`, until `collect` runs them.

---

## Modules and visibility

[Lesson 02/03](02_core_types_and_errors/03_packages_crates_modules/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `mod garden;` | load module from `garden.rs` (or `garden/mod.rs`) | `import garden` (from `garden.py`, or a `garden/` folder with `__init__.py`) |
| `mod tests { ... }` | inline module | — (one file is one module) |
| `pub` | public (visible outside its module) | — (everything is public; a leading `_` means "private" by convention, and `__all__` lists what `import *` exports) |
| `pub(crate)` / `pub(super)` | visible in this crate / in the parent module | `_name` (convention only) |
| `use std::collections::HashMap;` | bring a name into scope | `from collections import Counter` |
| `use std::io::{self, Read};` | several items at once | `import io` and `from io import BytesIO, StringIO` |
| `use crate::x` / `super::x` / `self::x` | from the crate root / the parent module / this module | `from mypkg import x` / `from .. import x` / `from . import x` (Python's `super()` is different: the parent *class*) |
| `use x as y;` | rename on import | `import x as y` / `from m import x as y` |
| `pub use inner::Thing;` | re-export | `from .inner import Thing` in the package's `__init__.py` |
| `extern crate`, `std`, `core`, `alloc` | crates; `core` and `alloc` are for `no_std` ([05/08](05_mastery/08_no_std_and_embedded/README.md)) | — (`pip install` a package, then `import` it) |

---

## Macros

[Lessons 04/03](04_systems_level/03_advanced_features/README.md) · [05/02](05_mastery/02_macros_in_depth/README.md)

Python has no macros (code that writes code before the program runs). The left half of this table maps to ordinary Python functions; the right half doesn't exist.

| Macro | Does | Python |
|---|---|---|
| `println!` `format!` `eprintln!` `write!` | formatted output | `print`, f-strings, `print(..., file=sys.stderr)`, `f.write(...)` |
| `vec!` | create a `Vec` | a list literal `[...]` |
| `assert!` `assert_eq!` `assert_ne!` `debug_assert!` | check a condition (panic if false) | `assert cond`, `assert a == b`, `assert a != b`; `debug_assert!` is gone in release builds, just as `python -O` drops every `assert` |
| `panic!` `todo!` `unimplemented!` `unreachable!` | stop: bug / not written yet / never here | `raise RuntimeError(...)` / `raise NotImplementedError` / `raise NotImplementedError` / `typing.assert_never(x)` |
| `matches!` | pattern test returning `bool` | a `match` statement, or `isinstance(x, ...)` |
| `dbg!` | debug-print a value | `print(f"{x=}")` |
| `include_str!` `env!` `concat!` `file!` `line!` | compile-time helpers | at run time instead: `Path("f").read_text()`, `os.environ["X"]`, `+`, `__file__`, `inspect.currentframe().f_lineno` |
| `macro_rules! name { ($x:expr) => { ... }; }` | define your own macro | — (functions and decorators cover most of what you'd use one for) |
| `$x:expr` `$i:ident` `$t:ty` `$p:pat` `$b:block` `$l:literal` `$tt:tt` | macro fragment types ("metavariables") | — |
| `$( ... ),*` / `$( ... );+` / `$( ... )?` | repeat zero or more / one or more / zero or one times (here separated by `,` / `;`) | — |
| `$crate::path` | refer to the macro's own crate, wherever it's used | — |
| `name!(...)` `name![...]` `name!{...}` | three equivalent bracket styles (`vec![]` and `macro_rules! {}` by convention) | — |
| `#[macro_export]` | make a `macro_rules!` macro usable from other crates | — |

---

## Attributes

| Attribute | Means | Python |
|---|---|---|
| `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]` | generate these trait impls | `@dataclass(order=True, frozen=True)`: `order` ≈ `PartialOrd`/`Ord`, `frozen` makes it immutable and hashable (≈ `Hash`) |
| `#[test]` / `#[should_panic]` / `#[ignore]` | test function / must panic / skip unless asked ([03/02](03_abstraction/02_automated_tests/README.md)) | with pytest: a function named `test_...` / `with pytest.raises(SomeError):` / `@pytest.mark.skip` |
| `#[cfg(test)]` | only compiled for `cargo test` | — (tests live in separate `test_*.py` files) |
| `#[cfg(feature = "x")]` / `#[cfg(target_os = "windows")]` | conditional compilation | decided at run time instead: `if sys.platform == "win32":` |
| `#[allow(dead_code)]` / `#[deny(...)]` / `#[warn(...)]` | lint levels | `# noqa` comments, or linter settings in `pyproject.toml` |
| `#![...]` at the top of a file | applies to the whole file/crate, e.g. `#![no_std]` | — |
| `#[must_use]` | warn if the return value is ignored | — |
| `#[inline]` | hint to inline the function | — |
| `#[non_exhaustive]` | outside crates must handle future variants with `_` | — |
| `#[repr(C)]` | C-compatible memory layout (FFI) | `class P(ctypes.Structure):` with `_fields_ = [...]` |
| `#[tokio::main]` / `#[tokio::test]` | async `main` / async test | `asyncio.run(main())` / `@pytest.mark.asyncio` (the pytest-asyncio plugin) |

---

## Smart pointers

[Lesson 03/06](03_abstraction/06_smart_pointers/README.md)

| Type | Use it for | Python |
|---|---|---|
| `Box<T>` | put a value on the heap; recursive types; `Box<dyn Trait>` | — (every Python object already lives on the heap) |
| `Rc<T>` | several owners, one thread (`Rc::clone(&a)`) | what every Python name already is: CPython counts the references to each object (`sys.getrefcount`) |
| `Arc<T>` | several owners, across threads | the same, across threads |
| `RefCell<T>` | change something behind a shared reference, checked at run time (`.borrow()`, `.borrow_mut()`) | — (Python objects can always be changed) |
| `Cell<T>` | the same for `Copy` values (`.get()`, `.set()`) | — |
| `Weak<T>` | a non-owning link (breaks `Rc` cycles) | `weakref.ref(obj)` |
| `Mutex<T>` / `RwLock<T>` | shared mutable data across threads (`.lock()`) | `threading.Lock()` (no read-write lock in the standard library). Python's lock sits *beside* the data; Rust's `Mutex` *holds* it, so you can't forget to lock |
| `Cow<'a, T>` | borrow when possible, clone only when changed | — |

---

## Concurrency and async

[Lessons 04/01](04_systems_level/01_fearless_concurrency/README.md) · [04/02](04_systems_level/02_async_await/README.md) · [06/02](06_framework_mastery/02_tokio/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `thread::spawn(move \|\| { ... })` | start a thread; returns a `JoinHandle` (`.join()`) | `t = threading.Thread(target=f)`, `t.start()`, `t.join()` |
| `thread::scope(\|s\| { s.spawn(...); })` | threads that may borrow local data | `with ThreadPoolExecutor() as ex: ex.submit(f)` (the `with` waits for them all) |
| `let (tx, rx) = mpsc::channel();` | channel: `tx.send(v)`, `rx.recv()` | `q = queue.Queue()`: `q.put(v)`, `q.get()` |
| `Arc<Mutex<T>>` | shared, mutable, thread-safe | a shared object plus a `threading.Lock` |
| `Send` / `Sync` | safe to move to / share between threads | — (Python doesn't check thread safety; the GIL hides some races, not all) |
| `async fn f() -> T` | returns a future that produces `T` | `async def f() -> T:` |
| `f().await` | run the future and wait for it (only inside `async`) | `await f()` (Python puts `await` in front) |
| `async move { ... }` | async block that takes ownership | — |
| `tokio::spawn(fut)` | run a future as a task | `asyncio.create_task(coro)` |
| `tokio::join!(a, b)` / `tokio::select! { ... }` | wait for all / for the first | `await asyncio.gather(a, b)` / `asyncio.wait(..., return_when=asyncio.FIRST_COMPLETED)` |
| `Pin<Box<dyn Future<Output = T> + Send>>` | a boxed future (dyn-compatible async) | `Awaitable[T]` |

---

## Unsafe and FFI

[Lesson 04/03](04_systems_level/03_advanced_features/README.md)

| Rust | Meaning | Python |
|---|---|---|
| `unsafe { ... }` | "I promise the rules hold here": allows the 5 unsafe operations | — (Python has no compile-time checks to switch off) |
| `*const T` / `*mut T` | raw pointers (dereferencing them needs `unsafe`) | `ctypes.POINTER(ctypes.c_int)` |
| `unsafe fn` | a function with conditions the caller must uphold | — |
| `unsafe extern "C" { fn abs(x: i32) -> i32; }` | declare a C function (edition 2024 syntax) | `ctypes`: `ctypes.cdll.msvcrt.abs(-3)` on Windows, `ctypes.CDLL("libc.so.6").abs(-3)` on Linux |
| `extern "C" fn` + `#[unsafe(no_mangle)]` | let C call a Rust function | `ctypes.CFUNCTYPE(...)` wraps a Python function for C to call |
| `unsafe impl Send for T {}` | promise a trait's rules hold | — |

---

## Newer syntax

Recent additions, all checked to compile on this course's toolchain (edition 2024).

| Rust | Meaning | Python |
|---|---|---|
| `if let Some(a) = x && a > 3 { ... }` | let chains: several `let` patterns and conditions joined with `&&` | `if (a := x) is not None and a > 3:` |
| `'block: { if done { break 'block value; } other }` | labelled block: leave a block early with a value | — (use a function and `return`) |
| `const { 1 + 2 }` | inline const block: evaluated at compile time | — |
| `async \|a\| a + 1` | async closure: returns a future | — (a `lambda` can't be async; write an `async def`) |
| `&raw const x` / `&raw mut x` | create a raw pointer directly, without making a reference first | — |
| `#[unsafe(no_mangle)]` / `#[unsafe(export_name = "...")]` | attributes that can break safety must be marked `unsafe` (edition 2024) | — |
| `unsafe extern "C" { ... }` / `safe fn` inside it | extern blocks are `unsafe`; items you have checked can be marked `safe` | — |
| `impl Trait + use<'a, T>` | precise capturing (see Traits and generics) | — |

---

## All keywords

| Rust keyword | Meaning | Python |
|---|---|---|
| `as` | cast / rename an import | `int(x)` for casts; `import x as y` (the same word for renaming) |
| `async` / `await` | async functions / wait for a future | `async` / `await` (Python writes `await f()`, Rust `f().await`) |
| `break` / `continue` | leave the loop / next iteration | same |
| `const` | compile-time constant | `Final` (a hint) |
| `crate` | this crate's root | your top-level package name |
| `dyn` | trait object | — (every Python call is dynamic) |
| `else` | | `else` (and `elif` for Rust's `else if`) |
| `enum` | | `class X(Enum):` |
| `extern` | foreign code | `ctypes` |
| `false` / `true` | | `False` / `True` |
| `fn` | | `def` |
| `for` / `in` / `while` / `if` / `return` | | the same words |
| `impl` | add methods or a trait to a type | the methods written inside the `class` |
| `let` | new variable | — (just `x = ...`) |
| `loop` | loop forever | `while True:` |
| `match` | | `match` (3.10+) |
| `mod` | module | a `.py` file |
| `move` | closure takes ownership | — |
| `mut` | may change | — (everything may change) |
| `pub` | public | — (no leading `_`) |
| `ref` | bind by reference | — |
| `self` / `Self` | the current value / type | `self` / `typing.Self` |
| `static` | global | a module-level variable |
| `struct` | | `@dataclass` + `class` |
| `super` | parent module | `from .. import x` (Python's `super()` means the parent class) |
| `trait` | | `Protocol` or `ABC` |
| `type` | alias / associated type | `type` (the alias statement) |
| `unsafe` | | — |
| `use` | | `import` / `from ... import` |
| `where` | bounds | — |

**Context-dependent:** `union` · `'static` · `macro_rules` · `raw` · `safe` (in `unsafe extern` blocks)

**Reserved for the future** (can't be used as names; use `r#name` if you must): `abstract` `become` `box` `do` `final` `gen` `macro` `override` `priv` `try` `typeof` `unsized` `virtual` `yield`

---

## Python keywords and what Rust uses instead

The reverse lookup: you know what you'd write in Python and want the Rust way.

| Python | Rust |
|---|---|
| `def` | `fn` |
| `class` | `struct` (the data) + `impl` (the methods) + traits (shared behaviour); there is no inheritance |
| `elif` | `else if` |
| `pass` | an empty block `{}` |
| `lambda x: x + 1` | `\|x\| x + 1` |
| `None` | `None` (a variant of `Option`; the compiler makes you handle it) |
| `True` / `False` | `true` / `false` |
| `and` / `or` / `not` | `&&` / `\|\|` / `!` |
| `is` | `std::ptr::eq(a, b)` for "same object"; for `None` use `x.is_none()`; `==` compares values |
| `try` / `except` / `raise` | return a `Result`: `return Err(e)` instead of `raise`; the caller handles it with `match` or passes it up with `?` (`panic!` is only for bugs) |
| `finally` / `with open(...) as f:` | nothing to write: a value is cleaned up (file closed, lock released) when its owner goes out of scope (the `Drop` trait) |
| `del x` | `drop(x)` |
| `yield` | implement the `Iterator` trait, or `std::iter::from_fn` (`gen` is reserved for the future) |
| `global` | `static` (with `Mutex` or atomics to change it) |
| `nonlocal` | nothing to write: closures capture variables they change automatically (`FnMut`) |
| `import` / `from ... import` | `use` (and `mod` to include a file) |
| `assert` | `assert!` |
| `async` / `await` | `async` / `.await` (after the expression) |
| `match` / `case` | `match` with `pattern => value` arms |

---

## Cargo commands

[Lessons 01/01](01_foundations/01_hello_world/README.md) · [03/05](03_abstraction/05_cargo_and_crates_io/README.md)

The pieces line up like this: `Cargo.toml` ≈ `pyproject.toml`, `Cargo.lock` ≈ `uv.lock` or a pinned `requirements.txt`, crates.io ≈ PyPI, a crate ≈ a package.

| Command | Does | Python |
|---|---|---|
| `cargo new name` / `cargo new --lib name` | new binary / library project | `uv init name` / `uv init --lib name` (or a folder plus `python -m venv .venv`) |
| `cargo run -p pkg` | build and run one lesson (`-p` picks the package in this workspace) | `python main.py` |
| `cargo run -p pkg --example name` | run an example | `python examples/name.py` |
| `cargo run -p pkg -- args` | pass arguments to **your** program (after `--`) | `python main.py args` (read them from `sys.argv`) |
| `cargo run -p pkg --features exercises --bin ex01_name` | run an exercise in this course | — |
| `cargo test -p pkg` / `cargo test name` / `-- --nocapture` | run tests / only matching names / show `println!` output | `pytest` / `pytest -k name` / `pytest -s` |
| `cargo check` | type-check without building (fast) | `mypy .` or `pyright` |
| `cargo build --release` | optimised build | — (Python isn't compiled ahead of time; PyInstaller can bundle an app) |
| `cargo clippy` | lints: catches common mistakes | `ruff check` |
| `cargo xtask fmt` | format the whole course (in this repo, instead of `cargo fmt --all`) | `ruff format` (or `black`) |
| `cargo xtask ci` | format check + clippy + all tests | `ruff format --check`, `ruff check`, `pytest` |
| `cargo doc --open` | build and open the docs for your code and its dependencies | `python -m pydoc -b` |
| `cargo add serde` | add a dependency | `uv add name` (or `pip install name`) |
| `rustc --explain E0382` | explain an error code in detail | — (Python errors have no codes; read the traceback) |
