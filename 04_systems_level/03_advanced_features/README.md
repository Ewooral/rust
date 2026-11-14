<!-- hand-written lesson: gen_course must not overwrite -->
# Advanced Features

**The Rust Book, chapter 20:** [Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html)
· [The Rustonomicon](https://doc.rust-lang.org/nomicon/) (unsafe Rust in depth)
· Reference: [Unsafety](https://doc.rust-lang.org/reference/unsafety.html), [Type layout](https://doc.rust-lang.org/reference/type-layout.html), [Macros by example](https://doc.rust-lang.org/reference/macros-by-example.html), [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
· std: [`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html), [`std::ops`](https://doc.rust-lang.org/std/ops/index.html), [`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html), [`MaybeUninit`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html)
· [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)

Run it: `cargo run -p advanced_features`  ·  Test it: `cargo test -p advanced_features`

This chapter is a tour of features you won't use every day, but will meet in every serious codebase: `unsafe` code and talking to C, the finer points of traits and types, functions as values, and macros, which are code that writes code. Most of `std` is built with them: `Vec` is unsafe code behind a safe API, `+` is a trait, and `println!` is a macro.

> 💡 **The big picture: the workshop behind the shop.** Until now you've worked in the **shop**, where every tool has a guard and the manager (the compiler) checks everything you do. Chapter 20 opens the door to the **workshop out the back**. It has power tools without guards (`unsafe`), moulds for making new kinds of tools (advanced traits and types), recipe cards you can hand to someone else (function pointers and closures), and a machine that writes instructions for you (macros). You won't need the workshop most days, but everything in the shop was built there.

## 1. Unsafe Rust

The compiler's checks are **conservative**: if it can't prove code is safe, it rejects it, even when the code is actually fine. And the hardware, the operating system and C libraries can't be checked at all. `unsafe` is how you say "I have checked this myself". It unlocks exactly five extra abilities, the **unsafe superpowers**:

| Superpower | Example |
|---|---|
| 1. dereference a raw pointer | `*ptr` |
| 2. call an unsafe function or method | `slice::from_raw_parts_mut(ptr, n)` |
| 3. access or modify a mutable static | `COUNTER += 1` |
| 4. implement an unsafe trait | `unsafe impl Send for MyType {}` |
| 5. access the fields of a union | `value.i` |

`unsafe` does **not** turn off the borrow checker, type checking or anything else. Keep unsafe blocks small, and put them behind safe functions so the rest of the program never needs to know.

> 💡 **Analogy: the "staff only" cupboard.** `unsafe` doesn't switch off the alarms in the rest of the building. It unlocks five particular cupboards, and every time you open one you sign the logbook: "I checked this myself." If something goes wrong later, everyone knows which cupboards to look in first.

### Dereferencing a raw pointer

```rust
let mut num = 5;
let r1 = &raw const num;   // *const i32
let r2 = &raw mut num;     // *mut i32
unsafe {
    println!("r1 is: {}", *r1);
    *r2 += 1;
}
```

Raw pointers (`*const T`, `*mut T`) may break the borrowing rules (here a `*const` and a `*mut` point at the same value), aren't guaranteed to point at valid memory, may be null, and have no automatic clean-up. **Creating** one is safe; **dereferencing** one is the unsafe part. `&raw const` and `&raw mut` make a raw pointer directly, without making a reference first.

> 💡 **Analogy: an address on a scrap of paper.** Writing an address down is harmless, and you can even write down one that doesn't exist. Walking in through the front door (dereferencing) is the moment you need to be sure the house is really there.

### Calling unsafe functions

An `unsafe fn` has a **contract** the compiler can't check, written in a `# Safety` section of its documentation. Calling it needs an `unsafe` block, which is you saying you've kept the contract. Since edition 2024, the **body** of an `unsafe fn` is not one big unsafe block: each unsafe operation inside it still needs its own `unsafe { }` (the `unsafe_op_in_unsafe_fn` lint).

The real error from exercise 1, where the Book's `split_at_mut` lost its `unsafe` block. Even `ptr.add` is an unsafe method, because pointer arithmetic can run off the end:

```text
error[E0133]: call to unsafe function `std::slice::from_raw_parts_mut` is unsafe and requires unsafe block
  --> 04_systems_level\03_advanced_features\exercises\ex01_fix_unsafe_split.rs:25:9
   |
25 |         slice::from_raw_parts_mut(ptr, mid),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior

error[E0133]: call to unsafe function `std::ptr::mut_ptr::<impl *mut T>::add` is unsafe and requires unsafe block
  --> 04_systems_level\03_advanced_features\exercises\ex01_fix_unsafe_split.rs:26:35
   |
26 |         slice::from_raw_parts_mut(ptr.add(mid), len - mid),
   |                                   ^^^^^^^^^^^^ call to unsafe function
```

> 💡 **Analogy: a tool with a "read the manual first" sticker.** The `# Safety` section is the manual. Writing `unsafe { }` around the call is signing to say you read it and did what it says.

### A safe abstraction over unsafe code

Splitting one mutable slice into two halves is perfectly safe, but the borrow checker can't see that the halves don't overlap:

```rust
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    assert!(mid <= len);
    (&mut values[..mid], &mut values[mid..])
}
```

```text
error[E0499]: cannot borrow `*values` as mutable more than once at a time
  |
4 |     (&mut values[..mid], &mut values[mid..])
  |     --------------------------^^^^^^--------
  |     |     |                   |
  |     |     |                   second mutable borrow occurs here
  |     |     first mutable borrow occurs here
  |     returning this value requires that `*values` is borrowed for `'1`
```

The fix is a few lines of unsafe code inside a **safe** function. The `assert!` is what makes it safe for *any* caller, and the `// SAFETY:` comment records the reasoning:

```rust
let ptr = values.as_mut_ptr();
assert!(mid <= len);
// SAFETY: mid <= len, so both slices are inside `values` and don't overlap.
unsafe {
    (slice::from_raw_parts_mut(ptr, mid), slice::from_raw_parts_mut(ptr.add(mid), len - mid))
}
```

> 💡 **Analogy: a saw inside a cabinet with one big green button.** Anyone may press the button (call the safe function). Only the grown-up who built the cabinet ever touched the blade (the unsafe code), and the `assert!` is the guard that stops fingers going in, whatever button-presser comes along.

### `extern` functions: calling other languages

```rust
unsafe extern "C" {
    safe fn abs(input: i32) -> i32;           // safe to call: no unsafe block needed
    fn strlen(s: *const c_char) -> usize;     // unsafe to call: the pointer must be valid
}
```

`"C"` is the **ABI**, the agreement on how arguments are passed. Edition 2024 requires `unsafe extern`, because the declaration itself is a promise the compiler can't check: `extern "C" { … }` alone is `error: extern blocks must be unsafe`. Going the other way, `#[unsafe(no_mangle)] pub extern "C" fn call_from_c()` makes a Rust function callable from C by name. (Plain `#[no_mangle]` is `error: unsafe attribute used without unsafe` in edition 2024.) Details in [Beyond the Book: FFI](#beyond-the-book-ffi-calling-c-from-rust-and-rust-from-c).

> 💡 **Analogy: a phone call to a company abroad.** You agree on a language for the call (the ABI, `"C"`), but you can't see how they work inside. So you sign to say you've written down their number and their rules correctly.

### Mutable static variables

A `static` lives for the whole program at one fixed address. Reading an immutable `static` is safe, but **every** access to a `static mut` is unsafe, because any thread could be changing it at the same moment (a data race). Edition 2024 goes further and forbids even a shared *reference* to a `static mut`. `println!` borrows its arguments, so even printing one is an error:

```text
error: creating a shared reference to mutable static
   --> 04_systems_level\03_advanced_features\examples\unsafe_superpowers.rs:105:29
    |
105 |     unsafe { println!("{}", COUNTER) }; // error: creating a shared reference to mutable static
    |                             ^^^^^^^ shared reference to mutable static
    |
    = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
    = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default
```

Copy the value out (`let n = unsafe { COUNTER };`) or use a raw pointer from `&raw const COUNTER`. Better still, avoid `static mut` altogether: an `AtomicU32`, a `Mutex` or a `OnceLock` in an ordinary `static` is safe and thread-safe.

> 💡 **Analogy: the whiteboard in the school corridor.** Anyone from any classroom (any thread) can scribble on it at the same moment, so the scribbles can end up nonsense. An atomic is a whiteboard with **one pen** that people take turns with: no unsafe needed.

### Unsafe traits

A trait is `unsafe` when implementing it makes a promise other code relies on for safety. The famous ones are `Send` and `Sync`. The compiler implements them automatically; a type holding a raw pointer doesn't get them, and `unsafe impl Send for MyType {}` is you promising it really is safe to move between threads.

> 💡 **Analogy: a "safe to share" certificate.** The compiler hands them out automatically for toys it can inspect. For a toy it can't see inside (a raw pointer), you sign the certificate yourself, and everyone else trusts your signature.

### Unions

A `union` is like a struct whose fields all share the **same bytes**. Rust can't know which field was last written, so reading any field is unsafe. They exist mainly to match C unions.

> 💡 **Analogy: one lunchbox with two labels**, "sandwich" and "salad". Only one is inside, and Rust doesn't remember which you packed. Opening it as the wrong one is your problem.

### Using `unsafe` correctly: Miri

[Miri](https://github.com/rust-lang/miri) runs your program in an interpreter that checks every memory access and reports **undefined behaviour**, even when the program seems to work: `rustup +nightly component add miri`, then `cargo +nightly miri test`. It can't prove code correct, but it catches a lot.

> 💡 **Analogy: a driving examiner** sitting next to your unsafe code. They watch every move while it runs and stop you the moment you break a rule, even if nothing crashed.

▶ `cargo run -p advanced_features --example unsafe_superpowers`

## 2. Advanced traits

### Associated types

```rust
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> { … }
}
```

An **associated type** is a type placeholder the implementation fills in **once**. With a generic trait instead (`trait Converter<T>`), a type could implement it many times, and every call would have to say which one it means: `let x = 7u32.convert();` is E0283, type annotations needed.

> 💡 **Analogy: a gumball machine.** It always gives gumballs: the machine decides the snack once (the associated type). A generic trait is a machine with a dial, where you must choose the snack every time you use it.

### Default generic type parameters and operator overloading

Every operator is a trait in `std::ops`. `Add` is declared with a **default type parameter**, `Rhs = Self`:

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

impl Add for Point { type Output = Point; … }                 // Point + Point
impl Add<Meters> for Millimeters { type Output = Millimeters; … }  // Millimeters + Meters
```

Defaults let a trait grow without breaking existing code, and let most users ignore a parameter they rarely need. If there's no `Add` at all you get E0369 (``cannot add `Meters` to `Meters` ``). If there are `Add` impls, but not for that right-hand side, it's E0277, from the mini project:

```text
error[E0277]: cannot add `Seconds` to `Meters`
  --> 04_systems_level\03_advanced_features\src\main.rs:50:25
   |
50 |     let oops = distance + time; // E0277: cannot add `Seconds` to `Meters`
   |                         ^ no implementation for `Meters + Seconds`
   |
   = help: the trait `Add<Seconds>` is not implemented for `Meters`
```

> 💡 **Analogy:** operator overloading is **teaching `+` a new game**: for points it means "walk both distances". The default type parameter is a form with "same as above" already filled in. You only write something different when you need to, like metres plus millimetres.

### Fully qualified syntax for disambiguation

When two traits (and maybe the type itself) have a method with the same name, `person.fly()` calls the type's own method. Pick a trait's version with `Pilot::fly(&person)`; without the inherent method the plain call is E0034, multiple applicable items in scope. An associated function has no `self` to tell Rust which type you mean, so you need the full form, `<Type as Trait>::function(args)`:

```text
error[E0790]: cannot call associated function on trait without specifying the corresponding `impl` type
   --> 04_systems_level\03_advanced_features\examples\advanced_traits.rs:173:20
    |
102 |     fn baby_name() -> String; // no `self`: an associated FUNCTION
    |     ------------------------- `Animal::baby_name` defined here
...
173 |     println!("{}", Animal::baby_name()); // E0790
    |                    ^^^^^^^^^^^^^^^^^^^ cannot call associated function of trait
    |
help: use the fully-qualified path to the only available implementation
    |
173 |     println!("{}", <Dog as Animal>::baby_name()); // E0790
    |                    +++++++       +
```

> 💡 **Analogy: two children in the class called Sam.** Shout "Sam!" and the teacher asks "Sam *Mensah* or Sam *Owusu*?" Fully qualified syntax is using the full name.

### Supertraits

```rust
trait OutlinePrint: fmt::Display {
    fn outline_print(&self) { let output = self.to_string(); … }
}
```

`OutlinePrint: Display` means "only types that already implement `Display` may implement `OutlinePrint`", so its default methods can use `to_string()`. Forget the `Display` impl and you get E0277, `` `Point` doesn't implement `std::fmt::Display` ``.

> 💡 **Analogy:** to join the swimming **team** you must already have your swimming **badge**. The supertrait is the badge you need before you can earn this one.

### The newtype pattern for external traits

The orphan rule ([lesson 03/01, section 9](../../03_abstraction/01_generics_traits_lifetimes/README.md#9-phantomdata-variance-and-the-orphan-rule)) forbids implementing a foreign trait on a foreign type. The real error from exercise 2:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  --> 04_systems_level\03_advanced_features\exercises\ex02_fix_orphan_rule.rs:18:1
   |
18 | impl fmt::Display for Vec<String> {
   | ^^^^^^^^^^^^^^^^^^^^^^-----------
   |                       |
   |                       `Vec` is not defined in the current crate
   |
   = note: define and implement a trait or new type instead
```

Wrap it: `struct Wrapper(Vec<String>);` is local, so `impl Display for Wrapper` is allowed. The wrapper hides the inner type's methods; implement `Deref` if you want them back.

> 💡 **Analogy:** you can't put your stickers on someone else's bike. But you can put their bike in **your own box** and decorate the box.

▶ `cargo run -p advanced_features --example advanced_traits`

## 3. Advanced types

### Newtypes for type safety and abstraction

`struct UserId(u32)` and `struct OrderId(u32)` hold the same number, but passing a `UserId` where an `OrderId` is wanted is E0308. A newtype can also **hide** its inside: a `People` type exposing only `add` and `name_of` can swap its `HashMap` for something else later without breaking callers. Newtypes cost nothing at run time.

> 💡 **Analogy: two identical keys with different coloured covers.** Same metal inside, but you can't put the red key in the blue door by mistake.

### Type aliases

```rust
type Kilometers = i32;                              // just another name
type Thunk = Box<dyn Fn() + Send + 'static>;        // shortens a long type
type Result<T> = std::result::Result<T, io::Error>; // how std::io::Result is made
```

An alias is **not** a new type: a `Kilometers` and an `i32` mix freely, so it gives no type safety. Use a newtype when you want the compiler to keep things apart.

> 💡 **Analogy: a nickname.** Calling Kwame "Kay" doesn't create a second person: anything Kwame can do, Kay can do. A newtype is a genuinely different person who happens to look the same.

### The never type `!`

`!` is the type of expressions that **never produce a value**: `panic!`, `continue`, `return`, a `loop` with no `break`, and functions like `fn fail(msg: &str) -> !`. Because `!` never happens, it can be coerced into any type, which is why this `match` has one type even though one arm panics:

```rust
let guess: u32 = match text.trim().parse() {
    Ok(n) => n,
    Err(_) => fail("not a number"), // `!` fits where a u32 is expected
};
```

> 💡 **Analogy: a trip with no return ticket.** Nobody ever comes back from it, so it doesn't matter what they'd have brought back: the compiler lets it stand in for any type.

### Dynamically sized types and `Sized`

`str`, `[T]` and `dyn Trait` have no size known at compile time (DSTs), so they always live behind a pointer: `&str`, `Box<[T]>`, `&dyn Trait`. That pointer is **fat**: an address plus a length or a vtable, 16 bytes instead of 8. Every generic parameter is quietly `T: Sized`; write `T: ?Sized` ("may or may not be sized") to accept DSTs behind a reference. The real error from exercise 3:

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  --> 04_systems_level\03_advanced_features\exercises\ex03_fix_sized_bound.rs:22:29
   |
22 |     println!("{}", describe("hello")); // str
   |                    -------- ^^^^^^^ doesn't have a size known at compile-time
   |                    |
   |                    required by a bound introduced by this call
   |
   = help: the trait `Sized` is not implemented for `str`
note: required by an implicit `Sized` bound in `describe`
  --> 04_systems_level\03_advanced_features\exercises\ex03_fix_sized_bound.rs:15:13
   |
15 | fn describe<T: Debug>(value: &T) -> String {
   |             ^ required by the implicit `Sized` requirement on this type parameter in `describe`
help: consider relaxing the implicit `Sized` restriction
   |
15 | fn describe<T: Debug + ?Sized>(value: &T) -> String {
   |                      ++++++++
```

Similarly, a function can't return a bare `dyn Trait` (E0746); return `impl Trait` or `Box<dyn Trait>`.

> 💡 **Analogy: a parcel service.** It must know a box's size before it can be sent. "Some amount of text" (`str`) has no fixed size, so you never send the text itself. You send a **label** (`&str`) that says where the text is *and how long it is*.

▶ `cargo run -p advanced_features --example advanced_types`

## 4. Advanced functions and closures

### Function pointers

```rust
fn add_one(x: i32) -> i32 { x + 1 }
fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 { f(arg) + f(arg) }

do_twice(add_one, 5);                                       // 12
let strings: Vec<String> = numbers.iter().map(ToString::to_string).collect();
let statuses: Vec<Status> = (0u32..20).map(Status::Value).collect();  // a variant is a function too
```

`fn` (lowercase) is a **type**: a pointer to a function, which captures nothing. `fn` pointers implement all three closure traits, so an `impl Fn(i32) -> i32` parameter accepts functions *and* closures; prefer that in your own APIs, and take `fn` only when you must (for example when passing it to C). A closure that captures nothing coerces to a `fn` pointer; one that captures can't:

```text
error[E0308]: mismatched types
  --> 04_systems_level\03_advanced_features\examples\functions_and_closures.rs:64:14
   |
64 |     do_twice(|x| x + offset, 5); // E0308: expected fn pointer, found closure (it captures `offset`)
   |     -------- ^^^^^^^^^^^^^^ expected fn pointer, found closure
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected fn pointer `fn(i32) -> i32`
note: closures can only be coerced to `fn` types if they do not capture any variables
```

> 💡 **Analogy: a phone number on a card.** You can pass the card around and anyone can ring the number (call the function). A closure is a helper who carries a **backpack** of things it captured. A phone number can't carry a backpack, so only closures with an empty backpack fit on the card.

### Returning closures

Every closure has its own unnamed type, so a function returns "some closure" with `impl Fn(i32) -> i32`. But that means **one** hidden type: two different closures from one function (or from two functions in one `Vec`) is E0308, `` `if` and `else` have incompatible types `` and "no two closures, even if identical, have the same type". Box them as trait objects to give them one type:

```rust
fn make(add: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
    if add { Box::new(move |x| x + n) } else { Box::new(move |x| x * n) }
}
```

> 💡 **Analogy: hand-made toys with no brand name.** Two closures can look identical, but each is its own one-off toy. `impl Fn` promises "one particular toy that does this". To hand out different toys from the same shelf, put each in a box with the same label (`Box<dyn Fn>`).

▶ `cargo run -p advanced_features --example functions_and_closures`

## 5. Macros

A **macro** is code that writes code. It runs at compile time, before type checking, so it can do things a function can't: take any number of arguments (`println!("{} {}", a, b)`), or generate whole `impl` blocks. The cost: macros are harder to read and debug, and a `macro_rules!` macro must be defined (or imported) **before** it's used in the file: otherwise ``error: cannot find macro `early` in this scope``.

> 💡 **Analogy:** a function is a **chef** who cooks when you order. A macro is a **recipe writer** who writes new recipe cards before the kitchen even opens.

### Declarative macros with `macro_rules!`

```rust
#[macro_export]          // make it usable from other crates
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $( temp_vec.push($x); )*
            temp_vec
        }
    };
}
```

Like `match`, but on **code**: each rule is `(pattern) => { expansion }`. `$x:expr` matches any expression and names it `$x`; `$( … ),*` matches zero or more of them separated by commas; the `$( … )*` in the expansion repeats once per match. `vec![1, 2, 3]` becomes three `push` calls. Repetition, recursion and hygiene are in [Beyond the Book](#beyond-the-book-declarative-macros-in-depth-repetition-hygiene-recursion).

> 💡 **Analogy: a mail-merge letter.** The pattern says where the blanks are; the expansion is the letter, printed once per name on the list.

▶ `cargo run -p advanced_features --example declarative_macros`

### Procedural macros

A procedural macro is an ordinary Rust **function** that runs inside the compiler: a `TokenStream` comes in, a `TokenStream` goes out. It must live in its own crate with `proc-macro = true`. There are three kinds:

| Kind | Looks like | Does |
|---|---|---|
| custom derive | `#[derive(HelloMacro)]` | **adds** code next to a struct or enum |
| attribute-like | `#[trace]`, `#[route(GET, "/")]` | **replaces** the item it's attached to |
| function-like | `shout!("hi")`, `sql!(SELECT …)` | **replaces** the call, with any input syntax |

This lesson has its own proc-macro crate, [`macros/`](macros/src/lib.rs), with one of each and **no dependencies**: it walks the tokens by hand, so you can see everything. Real macros use [`syn`](https://docs.rs/syn) to parse tokens into a syntax tree and [`quote!`](https://docs.rs/quote) to write code, as the Book does.

```rust
use advanced_features::{HelloMacro, shout, trace};   // HelloMacro: the trait AND the derive

#[derive(HelloMacro)]
struct Pancakes;            // Pancakes::hello_macro() prints "Hello, Macro! My name is Pancakes!"

#[trace]
fn add(a: i32, b: i32) -> i32 { a + b }   // prints "-> entering add" and "<- leaving add"

const LOUD: &str = shout!("hello, macros!");   // "HELLO, MACROS!", worked out at compile time
```

- A proc macro reports a problem with `compile_error!`, so you get a real error at the call: `#[derive(HelloMacro)]` on `struct Stack<T>` says `error: this little HelloMacro derive does not support generic types`.
- Unlike `macro_rules!`, procedural macros are **not hygienic**: names they generate can clash with yours. That's why `#[trace]` names its helper `__TraceGuard`.

> 💡 **Analogy: robots on a factory line.** The **derive** robot reads your blueprint and bolts an extra part on the side. The **attribute** robot takes the whole machine apart and rebuilds it with additions. The **function-like** robot is handed a note and builds whatever the note describes.

▶ `cargo run -p advanced_features --example procedural_macros` · `cargo test -p advanced_features --test proc_macros`

## Beyond the Book: FFI: calling C from Rust and Rust from C

Every Rust program on Windows and Linux already links the C standard library, so the example calls C with no extra tools:

| Direction | How | In the example |
|---|---|---|
| Rust calls C | `unsafe extern "C" { fn strlen(s: *const c_char) -> usize; }` | `abs`, `strlen` |
| C calls Rust (callback) | pass an `extern "C" fn` to C | C's `qsort` calls our `compare_descending` |
| C calls Rust (by name) | `#[unsafe(no_mangle)] pub extern "C" fn …` in a `cdylib` or `staticlib` crate | `call_from_c` |

- **C types** live in `std::ffi`: `c_int`, `c_char`, `c_void`. Never assume `long` is `i64`: it's 32 bits on Windows (`c_long`).
- **Strings:** C strings end with a NUL byte. `CString::new("hi")` makes one to pass to C; `CStr` borrows one from C; `c"literal"` is a `&CStr` literal.
- **Structs** passed to C need `#[repr(C)]` (see memory layout below).
- **Panics must not escape into C.** A panic trying to unwind out of an `extern "C"` function aborts the process. Catch it with `std::panic::catch_unwind` at the boundary, or use `extern "C-unwind"` when both sides support it.
- **Linking your own C code:** a `build.rs` with the [`cc`](https://docs.rs/cc) crate compiles it; [`bindgen`](https://rust-lang.github.io/rust-bindgen/) writes the `extern` block from a C header, and [`cbindgen`](https://github.com/mozilla/cbindgen) writes a C header for your Rust functions.

> 💡 **Analogy: a border crossing.** Two countries agree on one form (the C ABI) for everything that crosses. Rust's own rules stop at the border, so the guards (`unsafe`) make you sign that the paperwork is right, and nothing on fire (a panic) is allowed to cross.

▶ `cargo run -p advanced_features --example ffi`

## Beyond the Book: the never type `!`, zero-sized types, `MaybeUninit`

- **`!` in practice.** Stable Rust only lets you write `!` as a function's return type. For "an error that can't happen", std uses `std::convert::Infallible`, an enum with no variants. A `Result<T, Infallible>` can be unpacked with `let Ok(x) = result;`, no `Err` arm needed, because no `Err` value can exist. (`advanced_types` example.)
- **Zero-sized types (ZSTs)** take **no memory**: `()`, unit structs like `struct Marker;`, `PhantomData<T>`, `[T; 0]`. `HashSet<T>` is literally a `HashMap<T, ()>`, and a `Vec` of ZSTs never allocates; it only counts. Marker types are ZSTs used purely as labels for the type checker. (`memory_layout` example.)
- **`MaybeUninit<T>`** is "space for a `T` that may not hold one yet". Reading memory that was never written is undefined behaviour, so you `write` first and only then `assume_init`. It replaces the deprecated `mem::uninitialized`. Get it wrong and rustc warns: ``the type `i32` does not permit being left uninitialized``, "this code causes undefined behavior when executed". (`safe_abstractions` example.)

> 💡 **Analogy:** `!` is the trip with no return ticket. A ZST is a **name badge**: it tells everyone who you are but has no pockets, so it weighs nothing. `MaybeUninit` is a **seat with a "reserved" sign**: the seat exists, but you mustn't talk to whoever's sitting there until someone actually sits down.

## Beyond the Book: memory layout: `repr(C)`, size and alignment

Every type has a **size** and an **alignment** (its address must be a multiple of it). Structs get **padding** so each field is aligned:

| Layout | `{ a: u8, b: u32, c: u8 }` | Notes |
|---|---|---|
| default, `repr(Rust)` | **8** bytes | the compiler may **reorder** fields to save space |
| `#[repr(C)]` | **12** bytes (offsets 0, 4, 8) | declared order and C's rules: needed for FFI, file formats, hardware |
| `#[repr(C, packed)]` `{u8, u32}` | 5 bytes, align 1 | no padding; `&p.b` is E0793 (unaligned reference), copy the field instead |
| `#[repr(transparent)]` | same as the one field | a newtype that crosses FFI as its inner type |
| `#[repr(u8)] enum` | 1 byte | choose the tag type and values, e.g. to match C |

`size_of`, `align_of` and `offset_of!` tell you the numbers. **Niches:** a `&T`, `Box<T>` or `NonZeroU32` can never be zero, so `Option` uses zero to mean `None` and costs nothing extra; `Option<u32>` has no spare value and needs a tag (8 bytes).

> 💡 **Analogy: packing a suitcase.** With `repr(Rust)` the packer may rearrange your things to fit more in. With `repr(C)` they pack in exactly the order on your list, because someone else (C code) will unpack by following the same list. Padding is the bubble wrap that makes each heavy item sit on the right shelf.

▶ `cargo run -p advanced_features --example memory_layout`

## Beyond the Book: soundness and safe abstractions over unsafe

**Undefined behaviour (UB)** is anything the language says must never happen: reading uninitialised memory, a dangling or misaligned pointer, two live `&mut` to one value, a `bool` that is 3, a data race. The optimiser assumes UB never happens, so a program with UB can do *anything*, including seeming to work. A safe function is **sound** if no safe caller, whatever arguments it passes, can cause UB through it.

```rust
// UNSOUND: safe to call, but nth_unsound(&[1, 2, 3], 10) reads past the end.
fn nth_unsound(values: &[i32], n: usize) -> i32 { unsafe { *values.as_ptr().add(n) } }

// Sound: check first...
fn nth_checked(values: &[i32], n: usize) -> Option<i32> { … if n < values.len() … }
// ...or make the function unsafe and document the caller's promise in `# Safety`.
unsafe fn nth_unchecked(values: &[i32], n: usize) -> i32 { … }
```

The rules professionals follow ([the Rustonomicon](https://doc.rust-lang.org/nomicon/) has the full story):
- **Keep unsafe tiny** and wrap it in a safe API. The mini project's one unsafe block is in [`src/raw.rs`](src/raw.rs).
- **Write `// SAFETY:` above every unsafe block** and `# Safety` on every `unsafe fn`. Clippy's `undocumented_unsafe_blocks` lint can enforce it.
- **Guard invariants with privacy.** The example's `ArrayStack<T, N>` (a fixed-capacity stack built on `[MaybeUninit<T>; N]`) is sound only while "slots below `len` are initialised" stays true, so its fields are private and every method keeps the invariant. It implements `Drop`, because `MaybeUninit` never drops what's inside.
- **Test under Miri.**

> 💡 **Analogy: a playground fence.** The playground (your safe API) must be safe for *any* child, whatever they do. A gap anywhere in the fence makes the whole playground unsafe, even if no child has found the gap yet. That's what "unsound" means.

▶ `cargo run -p advanced_features --example safe_abstractions`

## Beyond the Book: declarative macros in depth: repetition, hygiene, recursion

| Fragment | Matches | Fragment | Matches |
|---|---|---|---|
| `expr` | an expression | `ident` | a name |
| `ty` | a type | `literal` | `42`, `"hi"` |
| `pat` | a pattern | `block` | `{ … }` |
| `tt` | any one token tree | `item`, `path`, `meta`, `vis`, `lifetime` | as named |

- **Repetition:** `$( … ),*` (zero or more, comma-separated), `$( … );+` (one or more), `$( … )?` (optional). `$(,)?` allows a trailing comma. Repetitions **nest**: `$( $group:ident: [ $( $member:expr ),* ] );*`.
- **Recursion:** rules are tried top to bottom, so put the base case first. `max!(a, b, c)` = compare `a` with `max!(b, c)`. `count!` replaces each token with `1usize +`. Bind each argument with `let` so it's **evaluated once**.
- **Internal rules:** an `@name` prefix marks helper rules (`shout_all!(@one $word)`).
- **Hygiene:** a `let temp` inside a macro is a different variable from the caller's `temp`, so they can't clash. The flip side: the macro can't see the caller's variables unless they're passed in (using `x` inside is E0425, with "an identifier with the same name exists, but is not accessible due to macro hygiene").
- **`$crate::`** in an exported macro always names the defining crate. The mini project's `trail!` uses it.
- **Expression macros** wrap their body in `{{ … }}`, so several statements become one expression.
- **No matching rule** is a compile error: `max!()` gives `error: unexpected end of macro invocation`.
- **See the expansion:** `cargo install cargo-expand`, then `cargo expand -p advanced_features --example declarative_macros`.

> 💡 **Analogy: hygiene is every macro bringing its own labelled lunchbox.** Even if both you and the macro pack a sandwich called `temp`, they never get mixed up. Recursion is a set of **Russian dolls**: each rule opens one doll and hands the rest to itself, until the smallest doll (the base case) is reached.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `*ptr` or an unsafe call outside `unsafe` | E0133 | A small `unsafe { }` with a `// SAFETY:` comment. Better: is there a safe API? |
| Two `&mut` halves of one slice by indexing | E0499 | `slice.split_at_mut(mid)` |
| `extern "C" { … }` in edition 2024 | error: extern blocks must be unsafe | `unsafe extern "C" { … }`, with `safe fn` for truly safe functions |
| `#[no_mangle]` in edition 2024 | error: unsafe attribute used without unsafe | `#[unsafe(no_mangle)]` |
| `println!("{}", COUNTER)` on a `static mut` | error: creating a shared reference to mutable static | Copy it out, or use an atomic / `Mutex` instead of `static mut`. |
| `impl Display for Vec<T>` | E0117 | A newtype: `struct List(Vec<T>)` |
| `a + b` on your type with no `Add` | E0369 | `impl Add for MyType { type Output = …; … }` |
| `a + b` with the wrong right-hand type | E0277 cannot add | `impl Add<Rhs> for …`, or convert first |
| `Trait::assoc_fn()` with several impls | E0790 | `<Type as Trait>::assoc_fn()` |
| `x.method()` defined by two traits | E0034 | `TraitName::method(&x)` |
| `impl SubTrait for T` without the supertrait | E0277 | Implement the supertrait (e.g. `Display`) first. |
| `fn f<T: Debug>(x: &T)` called with `str`, `[T]` or `dyn` | E0277 size not known | `T: Debug + ?Sized` |
| `fn f() -> dyn Trait` | E0746 | `impl Trait` or `Box<dyn Trait>` |
| A capturing closure where `fn(…)` is expected | E0308 expected fn pointer | Take `impl Fn(…)` instead. |
| Two different closures behind one `impl Fn` | E0308 | `Box<dyn Fn(…)>` |
| Using a macro above its `macro_rules!` | error: cannot find macro | Define it first, or `#[macro_export]` / `use` it. |
| `max!()` with no matching rule | error: unexpected end of macro invocation | Add a rule for that input. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p advanced_features --example unsafe_superpowers
cargo run -p advanced_features --example ffi
cargo run -p advanced_features --example advanced_traits
cargo run -p advanced_features --example advanced_types
cargo run -p advanced_features --example functions_and_closures
cargo run -p advanced_features --example declarative_macros
cargo run -p advanced_features --example procedural_macros
cargo run -p advanced_features --example memory_layout
cargo run -p advanced_features --example safe_abstractions
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_unsafe_split` | Make it compile (E0133), with a `// SAFETY:` comment | `cargo run -p advanced_features --features exercises --bin ex01_fix_unsafe_split` |
| 2 | `ex02_fix_orphan_rule` | Make it compile (E0117) with a newtype | `cargo run -p advanced_features --features exercises --bin ex02_fix_orphan_rule` |
| 3 | `ex03_fix_sized_bound` | Make it compile (E0277), one bound | `cargo run -p advanced_features --features exercises --bin ex03_fix_sized_bound` |
| 4 | `ex04_money_operators` | `Add`, `Sub`, `Neg`, `Mul<u32>`, `AddAssign`, `Sum`, `Display`, pass 3 tests | `cargo test -p advanced_features --features exercises --bin ex04_money_operators` |
| 5 | `ex05_macro_rules` | `hashmap!`, recursive `max!`, `square_all!`, pass 3 tests | `cargo test -p advanced_features --features exercises --bin ex05_macro_rules` |
| 6 | `ex06_fn_pointers_and_closures` | A `fn` lookup table, returned closures, a pipeline, pass 3 tests | `cargo test -p advanced_features --features exercises --bin ex06_fn_pointers_and_closures` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p advanced_features --features solutions --bins`).

**4. The mini project** is a **typed units-and-vectors toolkit** (`src/lib.rs` and its modules), and `src/main.rs` takes it for a walk:

- **Newtypes made by a macro:** `units.rs` uses a `unit!` macro to write `Meters`, `Millimeters`, `Seconds` and `MetersPerSecond`, each with `+ - * Sum Display`.
- **Operator overloading across types:** `Meters / Seconds = MetersPerSecond`, `MetersPerSecond * Seconds = Meters`, and `Meters + Millimeters` (a non-default `Rhs`). Adding seconds to metres doesn't compile.
- **`Vec2`** with `+ - * += *=` and unary `-`, including `2.0 * v` (`impl Mul<Vec2> for f64`, allowed by the orphan rule because `Vec2` is ours).
- **An associated type:** `Trail::segments()` returns an iterator with `type Item = (Vec2, Vec2)`.
- **A supertrait:** `OutlinePrint: Display` frames any displayable value; `Trail` gets it for free.
- **Function pointers and closures:** `transform::NAMED` is a `const` table of `fn(Vec2) -> Vec2`, looked up by name; `translate` and `scale` return closures; `chain` combines boxed closures.
- **A `vec!`-style macro:** `trail![(0, 0), (3, 4), …]`, exported with `$crate::`.
- **Audited unsafe:** `raw::split_at_mut` (one `unsafe` block with a full `// SAFETY:` comment) lets `Trail::scale_in_parallel` hand each half of the points to a different thread.
- **Procedural macros** from `macros/`: `#[derive(HelloMacro)]`, `#[trace]` and `shout!`.

```bash
cargo run -p advanced_features
cargo test -p advanced_features    # 19 unit tests, 3 proc-macro integration tests, 2 doc tests
```

> 💡 **Analogy:** the toolkit is a **measuring tape that knows its units**. It won't let you add minutes to metres, it can follow a trail of chalk marks and tell you how far you walked and how fast, and the one sharp blade inside (the unsafe split) is fixed behind a guard.

## Checklist

- [ ] [20.1 Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
- [ ] [20.2 Advanced Traits](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [ ] [20.3 Advanced Types](https://doc.rust-lang.org/book/ch20-03-advanced-types.html)
- [ ] [20.4 Advanced Functions and Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html)
- [ ] [20.5 Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- [ ] Beyond the Book: FFI: calling C from Rust and Rust from C
- [ ] Beyond the Book: the never type `!`, zero-sized types, `MaybeUninit`
- [ ] Beyond the Book: memory layout: `repr(C)`, size and alignment
- [ ] Beyond the Book: soundness and writing safe abstractions over unsafe ([Rustonomicon](https://doc.rust-lang.org/nomicon/))
- [ ] Beyond the Book: declarative macros in depth: repetition, hygiene, recursion
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `unsafe` unlocks five superpowers (raw pointers, unsafe calls, `static mut`, unsafe traits, unions) and switches off nothing else. Keep it small, explain it with `// SAFETY:`, wrap it in a sound safe API, and test it with Miri.
- Edition 2024 tightened the rules: `unsafe extern`, `#[unsafe(no_mangle)]`, unsafe blocks inside `unsafe fn`, and no references to `static mut`.
- Advanced traits: associated types fix one type per impl, default type parameters power operator overloading, fully qualified syntax picks between same-named items, supertraits require other traits, and newtypes get around the orphan rule.
- Advanced types: newtypes keep values apart, aliases only rename, `!` never returns and fits anywhere, and DSTs live behind fat pointers (`?Sized`).
- Functions are values: `fn` pointers capture nothing, closures do, and different closures need `Box<dyn Fn>` to share a type.
- Macros write code at compile time: `macro_rules!` matches patterns (with repetition, recursion and hygiene); procedural macros are compiler plugins in their own crate.

**In one sentence for a child:** behind the safe shop is a workshop with sharp tools, and the rule is to use them carefully, lock them inside safe machines with one green button, and write down why every sharp edge can't hurt anyone.

**Next:** [04 Web Server Project](../04_web_server_project/README.md) (Book chapter 21)
