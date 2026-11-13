<!-- hand-written lesson: gen_course must not overwrite -->
# Structs

**The Rust Book, chapter 5:** [Using Structs to Structure Related Data](https://doc.rust-lang.org/book/ch05-00-structs.html)
· Rust by Example: [Structures](https://doc.rust-lang.org/rust-by-example/custom_types/structs.html), [Methods](https://doc.rust-lang.org/rust-by-example/fn/methods.html)

A **struct** lets you group related values under one name and give each value a name of its own. Then you attach behaviour to it with **methods**. Structs (and enums, next lesson) are how you create your own types in Rust, and almost every real program is built out of them.

> 💡 **The big picture: a school registration form.** The struct **definition** is the blank form printed by the school: it lists the boxes (`name`, `age`, `class`) and what kind of answer goes in each. An **instance** is one filled-in copy of that form. **Methods** are the things you can do with a form: read it, correct it, or hand it in. Every section below is one part of this picture.

## 1. Defining and instantiating

```rust
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

let mut user1 = User {
    email: String::from("someone@example.com"),   // any order
    username: String::from("someusername123"),
    active: true,
    sign_in_count: 1,
};
user1.email = String::from("another@example.com"); // dot notation
```

- Every field must be given a value. The order you write them in does not matter.
- **Mutability belongs to the whole instance.** You cannot mark just one field as `mut`.
- **Field init shorthand**: if a variable has the same name as the field, write `email` instead of `email: email`.

> 💡 **Analogy: pen or pencil.** `let user1` fills in the form in **pen**: nothing on it can change. `let mut user1` fills it in **pencil**: any box can be rubbed out and rewritten. There is no such thing as a form that is half pen and half pencil, which is why there is no `mut` on single fields.

The real error from exercise 1:

```text
error[E0594]: cannot assign to `player.level`, as `player` is not declared as mutable
   |
20 |     player.level += 1;
   |     ^^^^^^^^^^^^^^^^^ cannot assign
help: consider changing this to be mutable
   |
16 |     let mut player = Player {
   |         +++
```

> 💡 **Analogy: shorthand.** If the box is called "email" and the note in your hand is also called "email", you don't say "put the email in the email box". You just say "email".

▶ `cargo run -p structs --example defining_and_instantiating`

## 2. Struct update syntax

Create a new instance that reuses most of an old one's values. `..user1` must come last and means "take every field I did not list from `user1`":

```rust
let user2 = User {
    email: String::from("another@example.com"),
    ..user1
};
```

This works like **assignment**, so ownership rules apply field by field. `String` fields are **moved** out of `user1`, and `Copy` fields (`bool`, `u64`) are copied. Afterwards you can still use the fields of `user1` that were not moved, but not the moved ones. The real error from exercise 2:

```text
error[E0382]: borrow of moved value: `p1.country`
   |
24 |       let p2 = Profile {
   |  ______________-
25 | |         username: String::from("corro"),
26 | |         ..p1
27 | |     };
   | |_____- value moved here
...
30 |       println!("{} is from {}", p1.username, p1.country);
   |                                              ^^^^^^^^^^ value borrowed here after move
```

If you give the new instance fresh values for every non-`Copy` field, only the `Copy` fields are taken, and the old instance stays whole.

> 💡 **Analogy: copying a friend's form.** You copy your friend's form and change only your name. The numbers (their age, their class) you just write down again: now you both have them. But if a box contains a **real object**, like their library card, you can't photocopy it: copying the form *hands you the card*, and your friend no longer has it. Fill in that box with your own card, and your friend keeps theirs.

▶ `cargo run -p structs --example struct_update_syntax`

## 3. Tuple structs and unit-like structs

```rust
struct Color(i32, i32, i32);   // tuple struct: named type, unnamed fields
struct Point(i32, i32, i32);
struct AlwaysEqual;            // unit-like struct: no fields at all

let black = Color(0, 0, 0);
let origin = Point(0, 0, 0);
let Point(x, y, z) = origin;   // destructure by naming the type
println!("{}", black.0);       // or index with .0 .1 .2
```

`Color` and `Point` hold the same three numbers, but they are **different types**: a function that takes a `Color` will not accept a `Point` (E0308). A unit-like struct takes up zero bytes. It is useful when you need a type to hang behaviour on (traits, lesson 03/01) but have no data to store.

> 💡 **Analogy: titled forms without box labels.** A tuple struct is a form with a **title** ("Colour" or "Point") but whose boxes are only numbered 1, 2, 3. Two forms can have the same three boxes and still be different forms, and the office will not accept a Point form at the Colour desk. A unit-like struct is a **badge with nothing written on it**: having the badge *is* the message.

▶ `cargo run -p structs --example tuple_and_unit_structs`

## 4. An example program: rectangles

The Book's section 5.2 refactors one calculation three times, and each version says more about what the data *means*:

| Version | Signature | Problem |
|---|---|---|
| Two variables | `fn area(width: u32, height: u32)` | Nothing says the two numbers belong together. |
| A tuple | `fn area(dimensions: (u32, u32))` | Grouped, but which one is the width? `.0` says nothing. |
| A struct | `fn area(rect: &Rectangle)` | Grouped **and** named. Borrowed, so the caller keeps it. |

To print a struct with `{:?}` you must opt in with `#[derive(Debug)]`. The real error from exercise 3, where the compiler even tells you the fix:

```text
error[E0277]: `Book` doesn't implement `Debug`
   |
18 |     println!("{book:?}");
   |               ^^^^^^^^ `Book` cannot be formatted using `{:?}` because it doesn't implement `Debug`
help: consider annotating `Book` with `#[derive(Debug)]`
   |
 8 + #[derive(Debug)]
 9 | struct Book {
```

- `{:?}` prints on one line, and `{:#?}` pretty-prints one field per line.
- `dbg!(expr)` prints the file, line, expression and value to **stderr**, then *returns* the value, so you can wrap any expression with it. It takes ownership, so pass `dbg!(&rect)` if you still need `rect`.

> 💡 **Analogy: shoes in a box.** Two loose numbers are two shoes lying on the floor. A tuple is a box with both shoes in it, but nothing tells you which is left and which is right. A struct is a box with **L** and **R** written inside. `#[derive(Debug)]` asks the factory to stick a "describe what's inside" label on the box. `dbg!` is a sticky note that says what was in the box *and where you were standing* when you looked.

▶ `cargo run -p structs --example rectangles`

## 5. Methods

Methods are functions defined inside an `impl` block, and their first parameter is always some form of `self`, the instance the method is called on:

```rust
impl Rectangle {
    fn area(&self) -> u32 { self.width * self.height }      // read
    fn double(&mut self) { self.width *= 2; self.height *= 2; } // change
    fn into_tuple(self) -> (u32, u32) { (self.width, self.height) } // consume
    fn can_hold(&self, other: &Rectangle) -> bool {           // more parameters
        self.width > other.width && self.height > other.height
    }
}
rect.area();
```

| Write | Short for | Use when the method… |
|---|---|---|
| `&self` | `self: &Self` | only reads. The most common. |
| `&mut self` | `self: &mut Self` | changes the instance. |
| `self` | `self: Self` | turns the instance into something else. The caller cannot use it afterwards. |

> 💡 **Analogy: what you do with a form.** `&self` is **reading** the form: anyone can look, and nothing changes. `&mut self` is **correcting** the form with your pencil, and you give it back changed. `self` is **handing the form in** at the office: it's filed away, and you can't read or correct it any more.

- A method can share a field's name: `rect.width()` calls the method and `rect.width` reads the field. Methods like this are often **getters**: the field stays private and the method gives read-only access (privacy is lesson 02/03).
- **Automatic referencing:** Rust has no `->` operator. When you write `rect.double()`, Rust adds the `&`, `&mut` or `*` the method's `self` needs, so `rect.double()` means `(&mut rect).double()`.

> 💡 **Analogy:** automatic referencing is a helpful shop assistant. You say "I'd like to look at this" or "I'd like to change this", and they hand it over the right way. You never have to say how it should be carried.

Exercise 6 makes you choose the `self` for each method. Until you do, the compiler cannot find the method:

```text
error[E0599]: no method named `increment` found for struct `Counter` in the current scope
```

▶ `cargo run -p structs --example methods`

## 6. Associated functions

Any function in an `impl` block is an **associated function**. Those without `self` are not methods, and they are usually constructors. Call them with `::`:

```rust
impl Rectangle {
    fn new(width: u32, height: u32) -> Self { Self { width, height } }
    fn square(size: u32) -> Self { Self { width: size, height: size } }
}
let sq = Rectangle::square(3);
```

- `Self` inside an `impl` is an alias for the type being implemented.
- `new` is a **convention**, not a keyword. Named constructors like `square` say what kind of value you get.
- You have used these since lesson 01: `String::from`, `Vec::new`.
- A type can have **several `impl` blocks**. This is useful with generics and traits.
- A method is an associated function too: `sq.area()` and `Rectangle::area(&sq)` are the same call.

> 💡 **Analogy: the registration office.** You don't need a form to ask the office for a new one; you go to the counter (`Rectangle::`) and ask for "a new form" or "a square form, size 3". Methods need a form in your hand (`sq.`); associated functions are asked of the office itself. Several `impl` blocks are several instruction pages stapled together: one manual.

▶ `cargo run -p structs --example associated_functions`

## Beyond the Book: derives

`#[derive(...)]` asks the compiler to write standard trait implementations for you:

| Derive | Gives you | Needs |
|---|---|---|
| `Debug` | `{:?}`, `{:#?}`, `dbg!` | every field `Debug` |
| `Clone` | `.clone()` | every field `Clone` |
| `Copy` | implicit copies instead of moves | `Clone`, every field `Copy`, no `Drop` |
| `PartialEq`, `Eq` | `==`, `!=` | every field the same |
| `PartialOrd`, `Ord` | `<`, `>`, sorting (field by field, top to bottom) | `PartialEq` / `Eq` |
| `Hash` | `HashMap` / `HashSet` keys (with `Eq`) | every field `Hash` |
| `Default` | `Type::default()`, and `..Default::default()` | every field `Default` |

> 💡 **Analogy: the tick boxes at the factory.** When you order a new type, the factory shows you a list of extras: "can be described", "can be photocopied", "can be compared", "can be put in order". Tick a box, and the factory builds that feature for you, as long as every part inside the box supports it too.

▶ `cargo run -p structs --example derives`

## Beyond the Book: newtypes and builders

A **newtype** wraps one value in a tuple struct, like `struct Metres(f64)`. This stops you from mixing up values of different kinds: `speed(Seconds(9.58), Metres(100.0))` is a compile error, not a wrong answer. It costs nothing at runtime.

> 💡 **Analogy: labelled jars.** Sugar and salt look identical. Put them in jars labelled SUGAR and SALT, and nobody ever puts salt in the tea again. The jar weighs nothing, and the label does all the work.

A **builder** creates a value with many optional settings step by step: `PizzaBuilder::new().size(35).cheese().topping("mushroom").build()`. Each step takes `self` and returns it, so the calls chain together.

> 💡 **Analogy: ordering at a sandwich counter.** You don't shout every choice at once. You say "large", then "add cheese", then "add mushroom", then "that's it" (`build`), and the finished sandwich is handed over. Anything you don't mention gets the default.

▶ `cargo run -p structs --example newtype_and_builder`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Assigned to a field of a non-`mut` instance | E0594 | `let mut x = …` (the whole instance) |
| Used the old instance after `..old` | E0382 | Give the new instance its own values for the `String`/`Vec` fields, or use the old one first. |
| `println!("{x:?}")` on your struct | E0277 | `#[derive(Debug)]` |
| Forgot `self` in a method | E0599 "no method named" | Add `&self`, `&mut self` or `self`. |
| Called a constructor as `rect.new(...)` | E0599 | `Rectangle::new(...)` |
| Changed a field in a `&self` method | E0594 | The method needs `&mut self`. |
| Passed a `Point` where a `Color` is expected | E0308 | They are different types on purpose. |
| `&str` field with no lifetime | E0106 | Use `String` for now. Lifetimes come in lesson 03/01. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p structs --example defining_and_instantiating
cargo run -p structs --example struct_update_syntax
cargo run -p structs --example tuple_and_unit_structs
cargo run -p structs --example rectangles
cargo run -p structs --example methods
cargo run -p structs --example associated_functions
cargo run -p structs --example derives
cargo run -p structs --example newtype_and_builder
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_mutability` | Make it compile (E0594) | `cargo run -p structs --features exercises --bin ex01_fix_mutability` |
| 2 | `ex02_fix_update_move` | Make it compile (E0382), no `.clone()` | `cargo run -p structs --features exercises --bin ex02_fix_update_move` |
| 3 | `ex03_fix_missing_debug` | Make it compile (E0277) | `cargo run -p structs --features exercises --bin ex03_fix_missing_debug` |
| 4 | `ex04_rectangle_methods` | Implement, pass 4 tests | `cargo test -p structs --features exercises --bin ex04_rectangle_methods` |
| 5 | `ex05_temperature_newtype` | Implement, pass 2 tests (Beyond the Book) | `cargo test -p structs --features exercises --bin ex05_temperature_newtype` |
| 6 | `ex06_counter` | Choose each `self`, pass 2 tests | `cargo test -p structs --features exercises --bin ex06_counter` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p structs --features solutions`).

**4. Read the mini project** in `src/main.rs` (`cargo run -p structs`, `cargo test -p structs`): a tiny bank where an `Account` is opened with an associated function, read with `&self`, changed with `&mut self`, and closed with `self`. Notice how `transfer_to` borrows **two different** accounts mutably at once, and why paying yourself is a compile error.

> 💡 **Analogy:** each `Account` is a piggy bank with its owner's name on it. You can peek inside (`&self`), drop coins in or take them out (`&mut self`), or smash it open and take everything (`self`): after that, there is no piggy bank.

**5. A preview:** `examples/hogwarts_preview.rs` models a Hogwarts student with a struct, an enum and a custom error type. Read it now for the struct parts, and again after lessons 02/02 and 02/05.

## Checklist

- [ ] [5.1 Defining and Instantiating Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html)
- [ ] [5.2 An Example Program Using Structs](https://doc.rust-lang.org/book/ch05-02-example-structs.html)
- [ ] [5.3 Methods](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
- [ ] Beyond the Book: deriving `Debug`, `Clone`, `PartialEq`, `Default`, `Hash`
- [ ] Beyond the Book: newtype pattern
- [ ] Beyond the Book: builder pattern
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A struct groups named fields into a new type. Instances are mutable as a whole, or not at all.
- Field init shorthand and `..other` save typing. `..other` **moves** the non-`Copy` fields.
- Tuple structs give a name to a tuple, and unit-like structs have no data. Different names mean different types.
- `#[derive(Debug)]` turns on `{:?}` and `dbg!`.
- Methods take `&self` (read), `&mut self` (change) or `self` (consume). Associated functions without `self` are called with `Type::name` and usually construct values.

**In one sentence for a child:** a struct is a blank form, an instance is a filled-in copy, and methods are the things you can do with it: read it, correct it, or hand it in.

**Next:** [02 Enums and Pattern Matching](../02_enums_and_pattern_matching/README.md) (Book chapter 6)
