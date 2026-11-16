# Rust Syntax Quick Reference

Every symbol, keyword and common form in one page, each with a plain one-line meaning and the lesson that teaches it in depth. Use your editor's search (Ctrl+F) for the symbol you're stuck on.

Official references for anything not here: [Book appendix B: operators and symbols](https://doc.rust-lang.org/book/appendix-02-operators.html) · [appendix A: keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html) · [The Rust Reference](https://doc.rust-lang.org/reference/) · [std docs](https://doc.rust-lang.org/std/)

**Contents:** [Symbol index](#symbol-index-what-does-this-mean) · [Comments and printing](#comments-and-printing) · [Variables](#variables-and-constants) · [Types and literals](#types-and-literals) · [Operators](#operators) · [Functions and closures](#functions-and-closures) · [Control flow](#control-flow) · [Ownership and references](#ownership-borrowing-and-lifetimes) · [Structs, enums, impl](#structs-enums-and-impl) · [Traits and generics](#traits-and-generics) · [Patterns](#patterns) · [Errors](#error-handling) · [Collections](#collections-and-strings) · [Modules](#modules-and-visibility) · [Macros](#macros) · [Attributes](#attributes) · [Smart pointers](#smart-pointers) · [Concurrency and async](#concurrency-and-async) · [Unsafe](#unsafe-and-ffi) · [All keywords](#all-keywords) · [Cargo commands](#cargo-commands)

---

## Symbol index: "what does this mean?"

| You see | It means | Example |
|---|---|---|
| `::` | path separator: an item inside a module or type | `std::fs::read`, `String::new()` |
| `.` | field access or method call | `user.name`, `s.len()` |
| `;` | ends a statement; a block's last line **without** `;` is its value | `let x = 5;` |
| `&` | borrow (a shared reference); also bitwise AND | `&s`, `fn f(s: &str)` |
| `&mut` | mutable borrow: may change the value | `&mut v`, `fn f(v: &mut Vec<i32>)` |
| `*` | dereference ("the value behind this reference"); also multiply; raw pointer types | `*count += 1`, `*const T` |
| `'a` | a lifetime name | `fn f<'a>(x: &'a str) -> &'a str` |
| `'static` | lives for the whole program / owns no short borrows | `&'static str`, `T: 'static` |
| `'label:` | a loop label | `'outer: loop { break 'outer; }` |
| `'c'` | a `char` (one Unicode character) | `let c = 'é';` |
| `b'A'` / `b"hi"` | a byte (`u8`) / a byte string (`&[u8; N]`) | |
| `!` | macro call; logical NOT; the never type | `println!()`, `!done`, `fn f() -> !` |
| `?` | if `Err`/`None`, return it early; else unwrap the value | `let n: i32 = s.parse()?;` |
| `->` | function return type | `fn add(a: i32) -> i32` |
| `=>` | match arm: pattern => result | `Some(x) => x,` |
| `\|args\|` | closure parameters | `\|x\| x + 1` |
| `\|` | "or" in patterns; bitwise OR | `1 \| 2 => ...` |
| `_` | ignore this value / let the compiler infer the type | `let _ = f();`, `Vec<_>` |
| `..` | range (end excluded); "the rest" in patterns and struct update | `0..10`, `Point { x, .. }`, `..Default::default()` |
| `..=` | inclusive range | `1..=100`, `'a'..='z' =>` |
| `@` | bind a name while matching a pattern | `n @ 1..=9 => ...` |
| `#[...]` / `#![...]` | attribute on the next item / on the whole file or module | `#[derive(Debug)]`, `#![allow(dead_code)]` |
| `$name` | a macro variable (inside `macro_rules!`) | `$x:expr` |
| `<T>` | a generic type parameter | `Vec<T>`, `fn f<T: Display>(t: T)` |
| `::<T>` | "turbofish": give a generic type explicitly | `"5".parse::<i32>()`, `collect::<Vec<_>>()` |
| `T: Trait` | trait bound: T must implement Trait | `fn show<T: Display>(t: T)` |
| `+` (in bounds) | must implement **both** traits | `T: Clone + Debug` |
| `dyn Trait` | a trait object (type decided at run time) | `Box<dyn Error>` |
| `impl Trait` | "some type that implements Trait" | `fn f() -> impl Iterator<Item = u8>` |
| `Self` / `self` | the current type / the current value (method receiver) | `fn new() -> Self`, `fn len(&self)` |
| `r#name` | raw identifier: use a keyword as a name | `r#type` |
| `r"..."` / `r#"..."#` | raw string: no escapes | `r"C:\path"`, `r#"say "hi""#` |
| `{}` inside `"..."` | a format placeholder (in `println!`, `format!`) | `println!("{x} is {}", y)` |

---

## Comments and printing

| Syntax | Meaning |
|---|---|
| `// text` | line comment |
| `/* text */` | block comment |
| `/// text` | doc comment for the next item (shows in `cargo doc`) |
| `//! text` | doc comment for the enclosing file/module/crate |
| `println!("...")` / `print!` | print with / without a newline (stdout) |
| `eprintln!` / `eprint!` | print to stderr (errors) |
| `format!("...")` | build a `String` instead of printing |
| `dbg!(expr)` | print file:line, the expression and its value to stderr; returns the value |

**Format placeholders** ([lesson 01/01](01_foundations/01_hello_world/README.md)):

| Placeholder | Prints |
|---|---|
| `{}` | with `Display` (for people) |
| `{:?}` / `{:#?}` | with `Debug` / pretty-printed `Debug` |
| `{x}` / `{0}` | variable `x` / the first argument |
| `{:5}` `{:<5}` `{:>5}` `{:^5}` | width 5 / left / right / centre aligned |
| `{:05}` | width 5, padded with zeros |
| `{:.2}` | 2 decimal places |
| `{:x}` `{:X}` `{:b}` `{:o}` `{:e}` | hex / HEX / binary / octal / scientific |
| `{:#x}` `{:+}` | with `0x` prefix / always show the sign |
| `{{` `}}` | a literal `{` or `}` |

---

## Variables and constants

[Lesson 01/03](01_foundations/03_variables_and_data_types/README.md)

| Syntax | Meaning |
|---|---|
| `let x = 5;` | immutable variable (can't be changed) |
| `let mut x = 5;` | mutable variable |
| `let x: i32 = 5;` | with a type annotation |
| `let x = x + 1;` | **shadowing**: a new variable with the same name (may change type) |
| `let (a, b) = (1, 2);` | destructure while declaring |
| `let x;` then `x = 5;` | declare now, assign once later |
| `const MAX: u32 = 100;` | compile-time constant, type required, `UPPER_CASE` |
| `static NAME: &str = "app";` | one fixed memory location for the whole program |
| `static mut` | mutable global: `unsafe` to use, avoid (use atomics or `Mutex`) |

---

## Types and literals

[Lesson 01/03](01_foundations/03_variables_and_data_types/README.md)

| Type | What it is |
|---|---|
| `i8 i16 i32 i64 i128 isize` | signed integers (`i32` is the default) |
| `u8 u16 u32 u64 u128 usize` | unsigned integers (`usize` for indexes and lengths) |
| `f32 f64` | floating point (`f64` is the default) |
| `bool` | `true` or `false` |
| `char` | one Unicode character, 4 bytes, `'a'` |
| `()` | the unit type: "nothing" (a function with no return value returns this) |
| `!` | the never type: this code never returns (`panic!`, `loop {}`) |
| `(i32, &str)` | tuple; access with `.0`, `.1` |
| `[i32; 3]` | array: fixed length 3 |
| `[0; 5]` | array of five zeros |
| `&[T]` | slice: a borrowed view of part of an array or `Vec` |
| `&str` / `String` | borrowed text / owned, growable text |
| `type Km = i32;` | type alias (a new name, not a new type) |

| Literal | Meaning |
|---|---|
| `98_222` | `_` is just a separator for readability |
| `0xff` `0o77` `0b1111_0000` | hex, octal, binary |
| `5u8` `2.5f32` `1_000i64` | type suffix |
| `1e6` | 1,000,000.0 (a float) |
| `"text"` | string literal, type `&'static str` |
| `"\n \t \\ \" \u{1F600}"` | escapes: newline, tab, backslash, quote, Unicode |
| `r"C:\no\escapes"` / `r#"has "quotes""#` | raw strings |
| `b'A'` / `b"bytes"` / `br"raw bytes"` | byte / byte string / raw byte string |
| `c"text"` | C string literal (`&CStr`), for FFI |

**Casts:** `x as u8` converts primitive numbers. It can silently truncate or wrap; prefer `u8::try_from(x)` when the value might not fit.

---

## Operators

| Operator | Meaning | Assign form |
|---|---|---|
| `+ - * / %` | add, subtract, multiply, divide, remainder | `+= -= *= /= %=` |
| `== != < > <= >=` | comparison (needs `PartialEq` / `PartialOrd`) | |
| `&& \|\| !` | logical AND, OR, NOT (short-circuiting) | |
| `& \| ^ ! << >>` | bitwise AND, OR, XOR, NOT, shift | `&= \|= ^= <<= >>=` |
| `a..b` `a..=b` `a..` `..b` `..` | ranges | |
| `-x` | negation | |
| `&x` `&mut x` `*x` | borrow, mutable borrow, dereference | |
| `expr?` | propagate `Err`/`None` | |
| `x as T` | primitive cast | |

Integer overflow panics in debug builds and wraps in release. Say what you mean with `checked_add`, `wrapping_add`, `saturating_add` or `overflowing_add`.

---

## Functions and closures

[Lessons 01/04](01_foundations/04_functions_and_comments/README.md) · [03/04](03_abstraction/04_iterators_and_closures/README.md)

| Syntax | Meaning |
|---|---|
| `fn name(a: i32, b: &str) -> bool { ... }` | function: parameter types are required |
| last expression without `;` | the return value |
| `return x;` | return early |
| `fn main() -> Result<(), Box<dyn Error>>` | `main` may return a `Result`, so `?` works in it |
| `\|x\| x + 1` | closure |
| `\|x: i32\| -> i32 { x + 1 }` | closure with types |
| `move \|\| ...` | closure takes ownership of what it uses |
| `Fn` / `FnMut` / `FnOnce` | closure that reads / mutates / consumes what it captured |
| `fn apply(f: impl Fn(i32) -> i32)` | take a closure |
| `fn make() -> impl Fn(i32) -> i32` | return a closure |
| `fn(i32) -> i32` | a function pointer type |
| `const fn` | can run at compile time |
| `async fn` | returns a future (see async) |

---

## Control flow

[Lessons 01/05](01_foundations/05_control_flow/README.md) · [02/02](02_core_types_and_errors/02_enums_and_pattern_matching/README.md)

| Syntax | Meaning |
|---|---|
| `if x > 5 { ... } else if ... { ... } else { ... }` | condition must be a `bool` (no truthy numbers) |
| `let y = if ok { 1 } else { 2 };` | `if` is an expression; both branches must have the same type |
| `loop { ... }` | forever, until `break` |
| `let v = loop { break 42; };` | `break` with a value |
| `while x < 10 { ... }` | loop while true |
| `for item in &list { ... }` | loop over anything iterable |
| `for i in 0..10` / `(0..10).rev()` / `.step_by(2)` | count up / down / in steps |
| `for (i, x) in v.iter().enumerate()` | with the index |
| `continue` / `break` | next iteration / leave the loop |
| `'outer: for ... { break 'outer; }` | break out of an outer loop |
| `match x { 1 => ..., 2 \| 3 => ..., _ => ... }` | match: every case must be covered |
| `if let Some(v) = opt { ... } else { ... }` | match one pattern |
| `while let Some(top) = stack.pop() { ... }` | loop while the pattern matches |
| `let Some(v) = opt else { return; };` | `let ... else`: bind, or leave |
| `if let Some(a) = x && a > 3 { ... }` | let chains (edition 2024) |
| `matches!(x, Some(1..=9))` | pattern test that returns a `bool` |

---

## Ownership, borrowing and lifetimes

[Lessons 01/06](01_foundations/06_ownership/README.md) · [03/01](03_abstraction/01_generics_traits_lifetimes/README.md)

| Syntax | Meaning |
|---|---|
| `let b = a;` | **move**: `a` can't be used any more (unless the type is `Copy`) |
| `let b = a.clone();` | deep copy: both stay usable |
| `#[derive(Clone, Copy)]` | small values copied automatically instead of moved |
| `&a` | shared borrow: read-only, any number at once |
| `&mut a` | mutable borrow: exactly one at a time, and no `&a` meanwhile |
| `*r` | the value a reference points to |
| `fn f(s: &str)` | borrow a parameter (the caller keeps ownership) |
| `fn f(s: String)` | take ownership of a parameter |
| `fn f<'a>(x: &'a str, y: &'a str) -> &'a str` | the result lives as long as both inputs |
| `struct S<'a> { r: &'a str }` | a struct holding a reference can't outlive what it points to |
| `'_` | "some lifetime, inferred" |
| `std::mem::take(&mut x)` / `replace` / `swap` | move out of a `&mut` by leaving something in its place |

---

## Structs, enums and impl

[Lessons 02/01](02_core_types_and_errors/01_structs/README.md) · [02/02](02_core_types_and_errors/02_enums_and_pattern_matching/README.md)

| Syntax | Meaning |
|---|---|
| `struct User { name: String, age: u32 }` | struct with named fields |
| `struct Point(i32, i32);` | tuple struct; fields `.0`, `.1` |
| `struct Marker;` | unit struct (no fields) |
| `User { name, age: 9 }` | create one (field init shorthand for `name`) |
| `User { age: 10, ..other }` | struct update: take the rest from `other` |
| `enum Dir { Up, Down }` | enum: a value is exactly one variant |
| `enum Msg { Quit, Move { x: i32 }, Write(String) }` | variants can carry data |
| `Option<T>` = `Some(v)` / `None` | maybe a value |
| `Result<T, E>` = `Ok(v)` / `Err(e)` | success or error |
| `impl User { fn new() -> Self { ... } }` | associated function (no `self`): call `User::new()` |
| `fn name(&self)` / `(&mut self)` / `(self)` | method that reads / changes / consumes the value |
| `#[derive(Debug, Clone, PartialEq, Default)]` | generate trait impls automatically |

---

## Traits and generics

[Lessons 03/01](03_abstraction/01_generics_traits_lifetimes/README.md) · [03/07](03_abstraction/07_trait_objects_and_oop/README.md) · [04/03](04_systems_level/03_advanced_features/README.md)

| Syntax | Meaning |
|---|---|
| `trait Speak { fn speak(&self) -> String; }` | define shared behaviour |
| `fn greet(&self) -> String { ... }` (inside a trait) | default method: implementors get it for free |
| `impl Speak for Dog { ... }` | implement a trait for a type |
| `fn f<T: Speak>(x: T)` | generic with a trait bound (static dispatch) |
| `fn f(x: &impl Speak)` | the same, shorter |
| `fn f<T>(x: T) where T: Speak + Clone` | bounds in a `where` clause |
| `Box<dyn Speak>` / `&dyn Speak` | trait object: mixed types, decided at run time |
| `trait Iterator { type Item; }` | associated type |
| `trait Child: Parent` | supertrait: implementing `Child` requires `Parent` |
| `impl<T: Display> MyTrait for T` | blanket impl: for every type that qualifies |
| `T: ?Sized` | `T` may be unsized (like `str`, `[T]`, `dyn Trait`) |
| `<T as Trait>::method(x)` | fully qualified call (when two traits share a method name) |
| `struct Matrix<const N: usize>` | const generic: a value as a generic parameter |
| `PhantomData<T>` | marks a type parameter that no field uses |

---

## Patterns

[Lesson 03/08](03_abstraction/08_patterns_and_matching/README.md)

| Pattern | Matches |
|---|---|
| `5` / `"text"` / `'c'` | that exact value |
| `x` | anything, and names it `x` |
| `_` | anything, ignored |
| `_x` | anything, named but unused (no warning) |
| `1 \| 2` | either |
| `1..=9` / `'a'..='z'` | a range |
| `(a, b)` / `[first, .., last]` | destructure a tuple / slice |
| `Point { x, y: 0 }` | struct with `y == 0`, binding `x` |
| `Some(v)` / `Err(e)` | enum variant, binding its data |
| `..` | "the rest" |
| `n @ 1..=9` | match a range and bind the value |
| `Some(x) if x > 5` | match guard: an extra condition |
| `&(a, b)` | match through a reference |
| `ref x` / `ref mut x` | bind by reference instead of moving (older style) |

---

## Error handling

[Lesson 02/05](02_core_types_and_errors/05_error_handling/README.md)

| Syntax | Meaning |
|---|---|
| `panic!("msg")` | stop the program (for bugs, not expected failures) |
| `x?` | return the `Err`/`None` early, otherwise unwrap |
| `.unwrap()` / `.expect("why")` | value, or panic (with your message) |
| `.unwrap_or(v)` / `.unwrap_or_else(\|\| ...)` / `.unwrap_or_default()` | value, or a fallback |
| `.map(f)` / `.and_then(f)` / `.ok_or(e)` / `.ok()` | transform, chain, convert `Option`↔`Result` |
| `if let Err(e) = f() { ... }` | handle only the error |
| `Box<dyn std::error::Error>` | "any error" return type |
| `impl From<IoError> for MyError` | lets `?` convert errors automatically |
| `#[derive(thiserror::Error)]` / `anyhow::Result` | error types for libraries / applications |

---

## Collections and strings

[Lesson 02/04](02_core_types_and_errors/04_collections/README.md)

| Syntax | Meaning |
|---|---|
| `vec![1, 2, 3]` / `Vec::new()` | create a vector |
| `v.push(x)` / `v.pop()` / `v.len()` | add / remove last / length |
| `v[0]` / `v.get(0)` | index (panics if out of range) / `Option` |
| `&v[1..3]` | slice of elements 1 and 2 |
| `String::from("hi")` / `"hi".to_string()` | owned string |
| `s.push_str("x")` / `s.push('c')` / `s1 + &s2` / `format!` | grow a string |
| `s.chars()` / `s.bytes()` / `s.len()` | characters / bytes / length in **bytes** |
| `&s[0..4]` | string slice (byte offsets, must fall on character boundaries) |
| `HashMap::new()` / `m.insert(k, v)` / `m.get(&k)` | hash map |
| `*m.entry(k).or_insert(0) += 1` | insert-or-update |
| `HashSet`, `BTreeMap`, `VecDeque`, `BinaryHeap` | set, sorted map, double-ended queue, priority queue |

**Common iterator chain:** `v.iter().filter(|x| ...).map(|x| ...).collect::<Vec<_>>()`. See [lesson 03/04](03_abstraction/04_iterators_and_closures/README.md).

---

## Modules and visibility

[Lesson 02/03](02_core_types_and_errors/03_packages_crates_modules/README.md)

| Syntax | Meaning |
|---|---|
| `mod garden;` | load module from `garden.rs` (or `garden/mod.rs`) |
| `mod tests { ... }` | inline module |
| `pub` | public (visible outside its module) |
| `pub(crate)` / `pub(super)` | visible in this crate / in the parent module |
| `use std::collections::HashMap;` | bring a name into scope |
| `use std::io::{self, Read};` | several items at once |
| `use crate::x` / `super::x` / `self::x` | from the crate root / the parent module / this module |
| `use x as y;` | rename on import |
| `pub use inner::Thing;` | re-export |
| `extern crate`, `std`, `core`, `alloc` | crates; `core` and `alloc` are for `no_std` ([05/08](05_mastery/08_no_std_and_embedded/README.md)) |

---

## Macros

[Lessons 04/03](04_systems_level/03_advanced_features/README.md) · [05/02](05_mastery/02_macros_in_depth/README.md)

| Macro | Does |
|---|---|
| `println!` `format!` `eprintln!` `write!` | formatted output |
| `vec!` | create a `Vec` |
| `assert!` `assert_eq!` `assert_ne!` `debug_assert!` | check a condition (panic if false) |
| `panic!` `todo!` `unimplemented!` `unreachable!` | stop: bug / not written yet / never here |
| `matches!` | pattern test returning `bool` |
| `dbg!` | debug-print a value |
| `include_str!` `env!` `concat!` `file!` `line!` | compile-time helpers |
| `macro_rules! name { ($x:expr) => { ... }; }` | define your own macro |
| `$x:expr` `$i:ident` `$t:ty` `$($x:expr),*` | macro fragment types and repetition |

---

## Attributes

| Attribute | Means |
|---|---|
| `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]` | generate these trait impls |
| `#[test]` / `#[should_panic]` / `#[ignore]` | test function / must panic / skip unless asked ([03/02](03_abstraction/02_automated_tests/README.md)) |
| `#[cfg(test)]` | only compiled for `cargo test` |
| `#[cfg(feature = "x")]` / `#[cfg(target_os = "windows")]` | conditional compilation |
| `#[allow(dead_code)]` / `#[deny(...)]` / `#[warn(...)]` | lint levels |
| `#![...]` at the top of a file | applies to the whole file/crate, e.g. `#![no_std]` |
| `#[must_use]` | warn if the return value is ignored |
| `#[inline]` | hint to inline the function |
| `#[non_exhaustive]` | outside crates must handle future variants with `_` |
| `#[repr(C)]` | C-compatible memory layout (FFI) |
| `#[tokio::main]` / `#[tokio::test]` | async `main` / async test |

---

## Smart pointers

[Lesson 03/06](03_abstraction/06_smart_pointers/README.md)

| Type | Use it for |
|---|---|
| `Box<T>` | put a value on the heap; recursive types; `Box<dyn Trait>` |
| `Rc<T>` | several owners, one thread (`Rc::clone(&a)`) |
| `Arc<T>` | several owners, across threads |
| `RefCell<T>` | change something behind a shared reference, checked at run time (`.borrow()`, `.borrow_mut()`) |
| `Cell<T>` | the same for `Copy` values (`.get()`, `.set()`) |
| `Weak<T>` | a non-owning link (breaks `Rc` cycles) |
| `Mutex<T>` / `RwLock<T>` | shared mutable data across threads (`.lock()`) |
| `Cow<'a, T>` | borrow when possible, clone only when changed |

---

## Concurrency and async

[Lessons 04/01](04_systems_level/01_fearless_concurrency/README.md) · [04/02](04_systems_level/02_async_await/README.md) · [06/02](06_framework_mastery/02_tokio/README.md)

| Syntax | Meaning |
|---|---|
| `thread::spawn(move \|\| { ... })` | start a thread; returns a `JoinHandle` (`.join()`) |
| `thread::scope(\|s\| { s.spawn(...); })` | threads that may borrow local data |
| `let (tx, rx) = mpsc::channel();` | channel: `tx.send(v)`, `rx.recv()` |
| `Arc<Mutex<T>>` | shared, mutable, thread-safe |
| `Send` / `Sync` | safe to move to / share between threads |
| `async fn f() -> T` | returns a future that produces `T` |
| `f().await` | run the future and wait for it (only inside `async`) |
| `async move { ... }` | async block that takes ownership |
| `tokio::spawn(fut)` | run a future as a task |
| `tokio::join!(a, b)` / `tokio::select! { ... }` | wait for all / for the first |
| `Pin<Box<dyn Future<Output = T> + Send>>` | a boxed future (dyn-compatible async) |

---

## Unsafe and FFI

[Lesson 04/03](04_systems_level/03_advanced_features/README.md)

| Syntax | Meaning |
|---|---|
| `unsafe { ... }` | "I promise the rules hold here": allows the 5 unsafe operations |
| `*const T` / `*mut T` | raw pointers (dereferencing them needs `unsafe`) |
| `unsafe fn` | a function with conditions the caller must uphold |
| `unsafe extern "C" { fn abs(x: i32) -> i32; }` | declare a C function (edition 2024 syntax) |
| `extern "C" fn` + `#[unsafe(no_mangle)]` | let C call a Rust function |
| `unsafe impl Send for T {}` | promise a trait's rules hold |

---

## All keywords

**Used:** `as` (cast / rename import) · `async` · `await` · `break` · `const` · `continue` · `crate` (this crate's root) · `dyn` (trait object) · `else` · `enum` · `extern` (foreign code) · `false` · `fn` · `for` · `if` · `impl` · `in` (in `for` loops) · `let` · `loop` · `match` · `mod` · `move` (closure takes ownership) · `mut` · `pub` · `ref` (bind by reference) · `return` · `self` · `Self` · `static` · `struct` · `super` (parent module) · `trait` · `true` · `type` (alias / associated type) · `unsafe` · `use` · `where` (bounds) · `while`

**Context-dependent:** `union` · `'static` · `macro_rules` · `raw` · `safe` (in `unsafe extern` blocks)

**Reserved for the future** (can't be used as names; use `r#name` if you must): `abstract` `become` `box` `do` `final` `gen` `macro` `override` `priv` `try` `typeof` `unsized` `virtual` `yield`

---

## Cargo commands

[Lessons 01/01](01_foundations/01_hello_world/README.md) · [03/05](03_abstraction/05_cargo_and_crates_io/README.md)

| Command | Does |
|---|---|
| `cargo new name` / `cargo new --lib name` | new binary / library project |
| `cargo run -p pkg` | build and run one lesson (`-p` picks the package in this workspace) |
| `cargo run -p pkg --example name` | run an example |
| `cargo run -p pkg -- args` | pass arguments to **your** program (after `--`) |
| `cargo run -p pkg --features exercises --bin ex01_name` | run an exercise in this course |
| `cargo test -p pkg` / `cargo test name` / `-- --nocapture` | run tests / only matching names / show `println!` output |
| `cargo check` | type-check without building (fast) |
| `cargo build --release` | optimised build |
| `cargo clippy` | lints: catches common mistakes |
| `cargo xtask fmt` | format the whole course (in this repo, instead of `cargo fmt --all`) |
| `cargo xtask ci` | format check + clippy + all tests |
| `cargo doc --open` | build and open the docs for your code and its dependencies |
| `cargo add serde` | add a dependency |
| `rustc --explain E0382` | explain an error code in detail |
