<!-- hand-written lesson: gen_course must not overwrite -->
# Packages, Crates and Modules

**The Rust Book, chapter 7:** [Managing Growing Projects with Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
· Rust by Example: [Modules](https://doc.rust-lang.org/rust-by-example/mod.html), [Crates](https://doc.rust-lang.org/rust-by-example/crates.html)

So far every lesson has fit in one file. Real programs don't. This chapter is about **organising** code: splitting it into named pieces, deciding which parts other code may use (and which stay private), and pulling in code other people have published.

> 💡 **The big picture: a school.** A **package** is the whole school, and `Cargo.toml` is its registration papers. A **crate** is a building in the school grounds. Some buildings are where things *happen* (a binary crate is a program you run), and one can be the library that others come to *borrow from* (a library crate). **Modules** are the rooms and corridors inside a building. **`pub`** is a door marked "open to visitors"; everything else is "staff only". A **path** is directions to a room. **`use`** is a shortcut you pin up so you don't have to give the full directions every time.

This lesson's own crate is the main example: look at its files while you read.

```text
03_packages_crates_modules/          the PACKAGE (Cargo.toml)
├── Cargo.toml
├── src/
│   ├── lib.rs                       LIBRARY crate root  (named `restaurant`)
│   ├── front_of_house.rs            mod front_of_house
│   ├── front_of_house/
│   │   ├── hosting.rs               mod front_of_house::hosting
│   │   └── serving.rs               mod front_of_house::serving
│   ├── back_of_house/
│   │   └── mod.rs                   mod back_of_house  (older file style)
│   ├── main.rs                      BINARY crate root  (the mini project)
│   └── bin/
│       └── kitchen.rs               a second BINARY crate
├── examples/                        each file (or folder with main.rs) is a crate too
├── exercises/
└── solutions/
```

## 1. Packages and crates

- A **crate** is the smallest amount of code the compiler works with at one time. Every `rustc` or `cargo build` compiles crates.
  - A **binary crate** has a `main` function and becomes a program you can run.
  - A **library crate** has no `main`. It provides code for other crates to use. When Rustaceans say "a crate", they usually mean a library.
- The **crate root** is the file the compiler starts from. Everything else is reached from it through `mod` declarations.
- A **package** is one or more crates, described by one `Cargo.toml`. A package may contain:
  - **at most one** library crate. Its root is `src/lib.rs`.
  - **any number** of binary crates. `src/main.rs` is the one named after the package, and each file in `src/bin/` is another.

This package has both a library and binaries. Cargo names the library after the package by default; here `[lib] name = "restaurant"` gives it its own name:

```toml
[package]
name = "packages_crates_modules"

[lib]
name = "restaurant"
path = "src/lib.rs"
```

```bash
cargo run -p packages_crates_modules               # src/main.rs
cargo run -p packages_crates_modules --bin kitchen # src/bin/kitchen.rs
```

> 💡 **Analogy:** the package is the school, and the crate root is each building's **front door**, where every visit starts. A school can have many classroom buildings (binaries) but only one library. In this school, the library has its own name on the sign ("restaurant"), different from the school's name.

**A key fact:** `src/main.rs` and `src/lib.rs` are **two different crates**. The binary uses the library just as a stranger would, with `use restaurant::...`, and it can only see what the library made `pub`. That is why a common pattern is a thin `main.rs` that calls into a library that does the real work.

## 2. Defining modules, and privacy

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        fn seat_at_table() {}        // private
    }
    mod serving {                    // private module
        pub fn take_order() {}
    }
}
```

- `mod name { … }` defines a module inline. `mod name;` with a semicolon loads it from a file (section 5).
- Modules form a tree, called the **module tree**, rooted at the implicit module `crate`.
- **Everything is private by default**: functions, structs, enums, constants and modules.
- **Privacy rule:** items in a child module can use **anything** in their ancestor modules, private items included. A parent **cannot** see a child's private items.
- `pub` on a module only lets you name the module. Its contents still need their own `pub`.

The real error from exercise 1:

```text
error[E0603]: function `make_tea` is private
   |
21 |     println!("{}", cafe::kitchen::make_tea());
   |                                   ^^^^^^^^ private function
note: the function `make_tea` is defined here
   |
10 |         fn make_tea() -> String {
```

> 💡 **Analogy: rooms inside rooms.** From inside a small room you can walk out into every room that **contains** you (your ancestors), even the staff-only ones, because you are already inside the building. But to go **into** a room, its door must say "open to visitors" (`pub`). Opening the corridor door (`pub mod`) does not open the doors of the rooms along that corridor.

Why private by default? So you can change the inside of your code without breaking the people who use it. Only what you deliberately make `pub` is a promise.

▶ `cargo run -p packages_crates_modules --example inline_modules`

## 3. Paths

To call an item, you name its path. A path is **absolute** or **relative**:

```rust
crate::front_of_house::hosting::add_to_waitlist();  // absolute: starts at the crate root
front_of_house::hosting::add_to_waitlist();         // relative: starts in this module
super::deliver_order();                             // relative: the parent module
```

- An **absolute path** starts with `crate` (in your own crate) or a crate's name (`std::`, `rand::`, `restaurant::`).
- A **relative path** starts in the current module, from `self`, `super` or the name of an item.
- **`super`** is the parent module, like `..` in a file path. `super::super` goes up two levels.
- The Book's advice is to prefer absolute paths, because you usually move code separately from the things it calls.

> 💡 **Analogy: giving directions.** An absolute path is a full postal address: "Ghana, Accra, High Street, number 12". It works from anywhere. A relative path is "two doors down from here": shorter, but it only makes sense from where you are standing. `super` means "step back out into the hallway".

▶ `cargo run -p packages_crates_modules --example paths`

### Making structs and enums public

```rust
pub struct Breakfast {
    pub toast: String,          // customers choose
    seasonal_fruit: String,     // the kitchen chooses
}
pub enum Appetizer { Soup, Salad }  // every variant is public
```

- A `pub struct` has **private fields** until each field is marked `pub`. If any field is private, code outside the module cannot build the struct with `Breakfast { … }`. It must use a public constructor such as `Breakfast::summer("Rye")`.
- A `pub enum` makes **every variant** public, because an enum whose variants you couldn't name would be useless.

The real error from exercise 2:

```text
error[E0451]: field `price` of struct `Ticket` is private
   |
24 |     let front = cinema::Ticket { seat: 3, price: 0 };
   |                                           ^^^^^ private field
```

Reading or writing a private field from outside gives **E0616**.

> 💡 **Analogy: a vending machine.** A `pub struct` is a machine everyone can see. Some parts are behind glass that you can reach (`pub` fields, like choosing your toast); others are sealed inside (private fields, like the price). You can't build your own machine with a price you made up; you press the button the owner provides (the constructor). An enum is the **menu** pinned to the wall: once the menu is public, every item on it can be read.

▶ `cargo run -p packages_crates_modules --example pub_structs_and_enums`

## 4. `use`: bringing paths into scope

`use` creates a shortcut, so you don't have to write the whole path every time:

```rust
use crate::front_of_house::hosting;
hosting::add_to_waitlist();
```

**A `use` only applies in the module where you write it.** A child module does not inherit its parent's `use` lines. The real error from exercise 3:

```text
error[E0433]: cannot find module or crate `hosting` in this scope
   |
20 |         hosting::add_to_waitlist()
   |         ^^^^^^^ use of unresolved module or unlinked crate `hosting`
```

Fix it with a `use` inside the child module, or with `super::hosting::…`.

> 💡 **Analogy: speed dial.** `use` saves a contact to speed dial **on the phone in this room**. Walk into another room and that phone doesn't have your speed dial; you set it up again, or dial the full number.

**Idioms:**

| Bring in | Idiomatic | Why |
|---|---|---|
| A function | the **parent module**: `use …::hosting;` then `hosting::add_to_waitlist()` | the call shows it isn't local |
| A struct, enum or other type | the **full path**: `use std::collections::HashMap;` then `HashMap::new()` | convention |
| Two items with the same name | the parent modules (`fmt::Result`, `io::Result`), or `as` | to tell them apart |

**More `use` tools:**

```rust
use std::io::Result as IoResult;        // `as` renames
use std::cmp::{Ordering, max};          // nested paths: one line
use std::io::{self, Write};             // `self` = std::io itself
use std::collections::*;                // glob: everything public (mainly in tests: `use super::*;`)
pub use crate::front_of_house::hosting; // re-export (below)
```

> 💡 **Analogy:** `as` is giving two friends both called "Kofi" nicknames in your contacts. A glob is importing your whole address book at once: quick, but you can no longer tell where a name came from.

▶ `cargo run -p packages_crates_modules --example use_keyword`

### Re-exporting with `pub use`

`pub use` brings a name into scope **and** makes it public from this module. In `src/lib.rs`:

```rust
mod front_of_house;                        // private
pub use crate::front_of_house::hosting;    // but `hosting` is published here
```

Outside crates write `restaurant::hosting`, not `restaurant::front_of_house::hosting`, which they couldn't reach anyway. The **public API** (how users see it) can be different from the **internal structure** (how you organised it). You can later move `hosting` anywhere inside the crate, and nobody's code breaks.

> 💡 **Analogy: a reception desk.** Visitors walk in and ask reception for "hosting", and reception sends them the right way. They never need to know which corridor, floor or wing it's in, and if you rebuild the corridors, reception's answer is all that changes.

▶ `cargo run -p packages_crates_modules --example re_exports` · exercise 5

### Using external packages

1. Add the package to `Cargo.toml`, like `rand = "0.9"`, or run `cargo add rand`.
2. `use` what you need: `use rand::Rng;`. The first part of the path is the crate name.

The standard library `std` is a crate too, but it ships with Rust, so it needs no line in `Cargo.toml`. In this lesson `rand` sits under `[dev-dependencies]`, because only an example uses it, so it isn't built into the library or the binaries.

> 💡 **Analogy: ordering from a catalogue.** crates.io is a catalogue of parts other people have built. Write the part on your shopping list (`Cargo.toml`), Cargo fetches it, and then you `use` it like anything else in the school. `std` is the furniture the school came with.

▶ `cargo run -p packages_crates_modules --example external_packages`

## 5. Separating modules into files

When a module grows, move its body to a file. The `mod` **declaration** stays where it was, ending in `;`:

```rust
// src/lib.rs
mod front_of_house;        // compiler loads src/front_of_house.rs
```

```rust
// src/front_of_house.rs
pub mod hosting;           // compiler loads src/front_of_house/hosting.rs
```

Where the compiler looks for `mod name;`:

| Declared in | Modern style (preferred) | Older style (still works) |
|---|---|---|
| A crate root (`lib.rs`, `main.rs`) | `name.rs` next to it | `name/mod.rs` |
| `src/parent.rs` | `src/parent/name.rs` | `src/parent/name/mod.rs` |

This crate uses both styles on purpose: `front_of_house.rs` is the modern one and `back_of_house/mod.rs` the older one. In your own code, pick one style per project. The older style gives you many tabs all called `mod.rs`.

- `mod` is **not** like `#include` or `import`. You declare each module **once**, in its parent. Everywhere else, you refer to it with a path or `use`.
- Moving a module to a file **changes nothing else**: the paths stay the same.

The real error from exercise 6, where the compiler tells you the exact files it tried:

```text
error[E0583]: file not found for module `library`
   |
43 | mod library;
   | ^^^^^^^^^^^^
   = help: to create the module `library`, create file "…\ex06_split_into_files\library.rs"
           or "…\ex06_split_into_files\library\mod.rs"
```

> 💡 **Analogy: a binder with chapters.** The crate root is the table of contents. `mod garden;` is a line in it that says "Garden: see separate booklet". The booklet (`garden.rs`) holds the pages, and its own table of contents can point to smaller booklets in a folder named after it (`garden/vegetables.rs`). Each booklet is listed in exactly one table of contents.

▶ `cargo run -p packages_crates_modules --example garden`. This example is a **folder**: `examples/garden/main.rs`, `garden.rs` and `garden/vegetables.rs`, the Book's layout exactly.

## Beyond the Book: visibility levels

| Write | Visible to |
|---|---|
| *(nothing)* | this module and its children |
| `pub(super)` | the parent module (and its children) |
| `pub(crate)` | the whole crate, but not other crates |
| `pub(in crate::some::path)` | that module and its children |
| `pub` | everyone who can reach the module |

The library uses two of these:
- `kitchen_secret` in `lib.rs` is `pub(crate)`, so `back_of_house::chef_note` may call it, but `main.rs`, which is another crate, may not.
- `tip` in `serving.rs` is `pub(super)`.

> 💡 **Analogy:** `pub(crate)` is a sign saying **"staff of this school only"**: any teacher in any building can go in, but visitors from other schools cannot. `pub(super)` is "this department only".

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Called a private function or used a private module | E0603 | `pub` on the item, **and** on every module in the path. |
| Built a struct that has a private field | E0451 | Use or write a public constructor. |
| Read or wrote a private field | E0616 | A getter method, or make the field `pub` if that's really intended. |
| `use` at the top, path fails inside a `mod { }` | E0433 | `use` inside that module, or `super::…` |
| `mod name;` with no file | E0583 | Create `name.rs` (or `name/mod.rs`) where the error says. |
| `mod utils;` in two files for the same module | duplicate module, confusing errors | Declare once in the parent. Elsewhere, `use crate::utils`. |
| `use` of a crate missing from `Cargo.toml` | E0432 / E0433 "unlinked crate" | `cargo add name` |
| Method exists but "not found" | E0599 | Bring its trait into scope: `use rand::Rng;` |
| `main.rs` can't see a `pub(crate)` item from `lib.rs` | E0603 | They are different crates. Make it `pub`, or move the code into the library. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p packages_crates_modules --example inline_modules
cargo run -p packages_crates_modules --example paths
cargo run -p packages_crates_modules --example pub_structs_and_enums
cargo run -p packages_crates_modules --example use_keyword
cargo run -p packages_crates_modules --example re_exports
cargo run -p packages_crates_modules --example external_packages
cargo run -p packages_crates_modules --example garden
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_private_function` | Make it compile (E0603) | `cargo run -p packages_crates_modules --features exercises --bin ex01_fix_private_function` |
| 2 | `ex02_fix_private_field` | Make it compile (E0451) with a constructor | `cargo run -p packages_crates_modules --features exercises --bin ex02_fix_private_field` |
| 3 | `ex03_fix_use_scope` | Make it compile (E0433), two ways | `cargo run -p packages_crates_modules --features exercises --bin ex03_fix_use_scope` |
| 4 | `ex04_organise_into_modules` | Wrap functions in modules, pass 2 tests | `cargo test -p packages_crates_modules --features exercises --bin ex04_organise_into_modules` |
| 5 | `ex05_re_export` | Add `pub use`, pass 2 tests | `cargo test -p packages_crates_modules --features exercises --bin ex05_re_export` |
| 6 | `ex06_split_into_files` | Create the module files (E0583) | `cargo run -p packages_crates_modules --features exercises --bin ex06_split_into_files` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p packages_crates_modules --features solutions`). Solution 6 is a folder with three files.

**4. Read the mini project**: the `restaurant` library in `src/` and the two binaries that use it.

- Start at `src/lib.rs` and follow each `mod` to its file. Compare the module tree drawn at the top of `lib.rs` with the files on disk.
- In `src/main.rs`, notice that it imports `restaurant::hosting`, the re-exported path, and look at the commented lines that **cannot** compile from a separate crate.
- Run the tests: `cargo test -p packages_crates_modules`. Tests live next to the code they test, in each module.

> 💡 **Analogy:** `src/main.rs` is a customer. It comes in through the front door (`restaurant::`), asks at reception (`pub use`), and sees the dining room and the menu. The kitchen's secrets stay in the kitchen.

## Checklist

- [ ] [7.1 Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [ ] [7.2 Control Scope and Privacy with Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [ ] [7.3 Paths for Referring to an Item in the Module Tree](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)
- [ ] [7.4 Bringing Paths Into Scope with the use Keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)
- [ ] [7.5 Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)
- [ ] Beyond the Book: `pub(crate)`, `pub(super)`, `pub(in path)`
- [ ] Beyond the Book: designing a public API with `pub use`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A **package** (one `Cargo.toml`) holds crates: at most one library and any number of binaries. A **crate** is one unit of compilation, with a root file.
- **Modules** organise a crate into a tree. Everything is **private by default**. Children can see their ancestors, while parents see only what children mark `pub`.
- **Paths** are absolute (`crate::`, `std::`) or relative (`self`, `super`). A `pub struct` keeps its fields private, and a `pub enum` opens all its variants.
- **`use`** makes shortcuts in the current module only. `pub use` re-exports, so a crate's public API can differ from its file layout.
- `mod name;` loads `name.rs` (or `name/mod.rs`). Declare each module once, and moving it to a file doesn't change its paths.

**In one sentence for a child:** your code is a school with buildings and rooms, every door is locked unless you put an "open" sign on it, and `use` is speed dial so you don't have to read out the whole address each time.

**Next:** [04 Collections](../04_collections/README.md) (Book chapter 8)
