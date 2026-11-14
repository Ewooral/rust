<!-- hand-written lesson: gen_course must not overwrite -->
# Hello World

**The Rust Book, chapter 1:** [Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
· [The rustup book](https://rust-lang.github.io/rustup/) · [The Cargo Book](https://doc.rust-lang.org/cargo/)

Chapter 1 is about tools, not the language: installing Rust, compiling one file by hand with `rustc`, and then letting Cargo do it for you. Every later lesson runs through Cargo, so it is worth knowing what it does on your behalf.

> 💡 **The big picture: a bakery.** Your source code is a **recipe**. `rustc` is the **oven**: it turns the recipe into bread (a program you can run). **Cargo** is the **head baker**: it keeps the recipes organised, fetches ingredients from the shop (other people's code, called *crates*), and switches the oven on for you. **rustup** is the **supplier** who delivers the oven and the baker, and swaps them for newer models when they come out.

## 1. Installation

Rust is installed and updated with **rustup**, which manages *toolchains*: the compiler `rustc`, the build tool `cargo`, the standard library, and extras like `rustfmt` and `clippy`.

| System | Install |
|---|---|
| Linux | `curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf \| sh`, plus a C toolchain for the linker (`sudo apt install build-essential` on Debian/Ubuntu) |
| Windows | Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). It offers to install the Visual Studio Build Tools (the MSVC linker and Windows SDK); say yes. |

Then, in a **new** terminal (so the updated `PATH` is picked up), check it worked:

```text
$ rustc --version
rustc 1.100.0 (0ef1bc386 2026-11-09)
$ cargo --version
cargo 1.100.0 (3acc79c0c 2026-11-05)
```

The format is `rustc x.y.z (commit-hash date)`; your numbers will be newer or older. If the command is not found, the folder `~/.cargo/bin` (on Windows `%USERPROFILE%\.cargo\bin`) is missing from your `PATH`: close and reopen the terminal, or log out and back in.

| Command | What it does |
|---|---|
| `rustup update` | Update to the newest stable Rust. A new version ships every six weeks. |
| `rustup self uninstall` | Remove Rust and rustup completely. |
| `rustup doc` | Open the **offline** documentation in your browser: the Book, the standard library, and more. `rustup doc --book` and `rustup doc --std` jump straight there. |

This course pins its toolchain in `rust-toolchain.toml` at the repo root (`channel = "stable"`, plus the `rustfmt` and `clippy` components). rustup reads that file whenever you run `cargo` or `rustc` inside the repo, so everyone uses the same tools. `rustup show` tells you which toolchain is active, and why.

> 💡 **Analogy: a garden shed with a caretaker.** rustup is the caretaker of your tool shed. You ask once ("install Rust") and the caretaker puts the oven, the head baker and the cookbooks on the shelves. `rustup update` is the caretaker swapping in this season's models. `rustup doc` is the bookshelf in the shed: the whole manual is there even with no internet.

## 2. Hello, World! (with `rustc`)

The smallest Rust program is one file:

```rust
fn main() {
    println!("Hello, world!");
}
```

Compile it, then run the program it produced:

```bash
rustc main.rs
./main          # Linux
.\main.exe      # Windows (rustc also writes main.pdb, the debugging information)
```

The anatomy, piece by piece:

- **`fn main() { ... }`** defines a function called `main`. Every executable starts in `main`. It has no parameters (`()`) and returns nothing.
- **`println!`** ends in `!`, so it is a **macro**, not a function. Macros generate code at compile time. For now: if you see `!`, it's a macro, and macros don't always follow the same rules as functions.
- **`"Hello, world!"`** is a string literal, in **double** quotes. Single quotes are for one character (`'a'`).
- **`;`** ends the statement. Most lines of Rust end with one.
- Rust style is 4 spaces of indent. `rustfmt` formats code for you (see *Beyond the Book* below).

**Compiling and running are separate steps.** Rust is an **ahead-of-time compiled** language: `rustc` produces a stand-alone executable that you can give to someone who has never installed Rust, and they can run it. In Python or JavaScript you would hand over the source and they would need the interpreter.

> 💡 **Analogy: baking bread.** The recipe (`main.rs`) goes into the oven (`rustc`) and out comes bread (`main.exe`). Anyone can eat the bread; they don't need your oven or your recipe. A *scripting* language is more like giving someone the recipe and a portable oven, and they bake it every time they want to eat.

If you forget the `!`, Rust looks for a *function* called `println` and finds none. This is the real error from exercise 1, and notice that the `help:` line tells you the fix:

```text
error[E0423]: cannot find function `println` in this scope
   |
10 |     println("Hello, world!");
   |     ^^^^^^^ not found in this scope
   |
   = note: a macro named `println` exists in another namespace
help: use `!` to invoke the macro
   |
10 |     println!("Hello, world!");
   |            +
```

And if there is no function named exactly `main` (exercise 2 has `fn Main`), there is nowhere to start:

```text
error[E0601]: `main` function not found in crate `ex02_fix_main_name`
   |
10 | }
   |  ^ consider adding a `main` function to `01_foundations\01_hello_world\exercises\ex02_fix_main_name.rs`
```

> 💡 **Analogy: the compiler is a patient teacher with a red pen.** It doesn't just say "wrong". It circles the exact spot (`^^^^^^^`), says what it expected, and often writes the correction in the margin (`help:`). Reading the whole message, slowly, is the most useful Rust skill there is.

### Try it: the `without_cargo/` folder

`without_cargo/main.rs` is a Hello World compiled the chapter 1 way, with no Cargo at all:

```bash
cd 01_foundations/01_hello_world/without_cargo
rustc main.rs
./main          # Linux:   Let's get Rusty!
.\main.exe      # Windows: Let's get Rusty!
```

The compiled `main`, `main.exe` and `main.pdb` are build output, so the root `.gitignore` ignores them (`**/without_cargo/main`, `*.exe`, `*.pdb`). This folder is not part of the Cargo workspace, which is why `cargo` commands never touch it.

That file is also squashed together (`fn main(){` with a one-space indent). Ask rustfmt what it would change, without changing anything:

```text
$ rustfmt --check main.rs
Diff in ...\without_cargo\main.rs:1:
-fn main(){
- println!("Let's get Rusty!");
+fn main() {
+    println!("Let's get Rusty!");
 }
```

▶ `cargo run -p hello_world --example anatomy` (the same program, with every part labelled)
▶ `cargo run -p hello_world --example print_macros` (`println!`, `print!` and `eprintln!`)

## 3. Hello, Cargo!

`rustc` is fine for one file. Real projects have many files and use other people's code, and **Cargo** handles both: it builds your code, downloads and builds your *dependencies*, and runs tests. It comes with rustup.

```text
$ cargo new hello_cargo
    Creating binary (application) `hello_cargo` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

That creates a folder with a `src/main.rs` (Hello World), a `.gitignore` and a new Git repository (unless it is already inside one), and a **`Cargo.toml`**:

```toml
[package]
name = "hello_cargo"
version = "0.1.0"
edition = "2024"

[dependencies]
```

- **TOML** (*Tom's Obvious, Minimal Language*) is Cargo's configuration format. `[package]` names the program, its version, and the **edition** of Rust it is written in.
- **`[dependencies]`** lists other crates the project uses. It's empty for now; lesson 02 adds `rand`.
- Cargo expects source code in **`src/`**. The top folder is for READMEs, licences and config.

The everyday commands:

| Command | What it does | Output |
|---|---|---|
| `cargo build` | Compile (dev profile: fast to build, unoptimised) | `target/debug/hello_cargo` (`.exe` on Windows) |
| `cargo run` | Build if anything changed, then run | prints `Running ...` then your output |
| `cargo check` | Check that it compiles, without producing a program | much faster than `build`; use it while you write |
| `cargo build --release` | Compile with optimisations | `target/release/hello_cargo` |

```text
$ cargo run
   Compiling hello_cargo v0.1.0 (...\hello_cargo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.03s
     Running `target\debug\hello_cargo.exe`
Hello, world!
```

The first build also writes **`Cargo.lock`**, which records the exact version of every dependency so the next build is identical. Cargo manages it; you never edit it by hand.

**Debug or release?** The dev profile compiles quickly and keeps extra checks (like integer-overflow panics, lesson 03), so it's what you use all day. `--release` takes longer to compile but runs faster, and it's what you ship or benchmark. Cargo also gives every Rust project the same layout and the same commands, so `git clone ...`, `cd`, `cargo build` works on almost any Rust project you'll ever download.

> 💡 **Analogy: the head baker.** Instead of carrying each recipe to the oven yourself, you tell the head baker "bake" (`cargo build`) or "bake and serve" (`cargo run`). `cargo check` is the baker **reading the recipe for mistakes** without lighting the oven: much quicker. `--release` is baking the **wedding cake**: slow and careful, because it's for the customer, while the dev profile is a quick test bake for yourself. `Cargo.lock` is the **exact shopping receipt**, so next week's cake uses exactly the same ingredients.

▶ `cargo run -p hello_world --example where_is_my_program` (then add `--release` and compare the paths)
▶ `cargo run -p hello_world --example debug_vs_release` (then add `--release`: the same loop runs several times faster)

## Beyond the Book: this course is a Cargo workspace

The Book's projects each have their own folder, `Cargo.toml`, `Cargo.lock` and `target/`. This repository is a **workspace** instead: one root `Cargo.toml` lists every lesson as a *member*, and they share **one** `Cargo.lock` and **one** `target/` folder at the root, so a crate like `rand` is downloaded and compiled once for every lesson.

- Pick the package with **`-p`**: `cargo run -p hello_world`, `cargo test -p hello_world`. Without it, Cargo doesn't know which of the many lessons you mean:

```text
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
```

- Pick what to run inside a package: `--example anatomy`, `--bin ex01_fix_missing_bang`.
- Turn on optional parts with **`--features`**: the exercises are behind `--features exercises`, so unfinished ones never break the build.
- A lesson's `Cargo.toml` says `version.workspace = true` and `edition.workspace = true`: it **inherits** those values from the root.
- Everything after **`--`** goes to *your* program, not to Cargo: `cargo run -p hello_world -- Ama`.

> 💡 **Analogy: a block of flats.** Every lesson is a flat with its own front door (its own `Cargo.toml`). They share one **basement** (`target/`) and one **shopping list** (`Cargo.lock`), so the building buys each ingredient once. `-p hello_world` is the flat number you give the delivery driver.

## Beyond the Book: `rustfmt` and `clippy`

Two tools come with the toolchain this repo pins ([Book appendix D](https://doc.rust-lang.org/book/appendix-04-useful-development-tools.html)):

| Command | What it does |
|---|---|
| `cargo fmt -p hello_world` | Reformats every file in the package to the standard Rust style. `rustfmt main.rs` does one file. |
| `cargo fmt -p hello_world --check` | Only reports what would change (useful in CI). |
| `cargo clippy -p hello_world` | Lints: hundreds of checks for code that works but could be simpler, faster or more correct. |

Every lesson in this course passes both with zero warnings. Make it a habit: `cargo fmt`, `cargo clippy`, then commit.

> 💡 **Analogy:** `rustfmt` is a **tidy-up robot** that straightens every book on the shelf the same way, so nobody argues about where the books go. `clippy` is a **kind teacher reading over your shoulder**: "that works, but here's a neater way to write it". (It's named after the paperclip assistant.)

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `println("Hi")` | E0423 cannot find function | Macros need `!`: `println!("Hi")`. |
| `fn Main()` | E0601 `main` function not found | Exactly `fn main()`, lower case. |
| `printn!("Hi")` | cannot find macro `printn` | Fix the spelling. Check the suggestion is what you meant. |
| `println!('Hello')` | E0762 unterminated character literal | Strings use **double** quotes. Single quotes hold one `char`. |
| Forgot the `;` between two statements | expected `;`, found `println` | Add the `;` where the `help:` points. |
| `cargo run` at the repo root | could not determine which binary to run | `cargo run -p hello_world` |
| `rustc: command not found` after installing | (shell error) | Open a new terminal so `~/.cargo/bin` is on `PATH`. |
| Ran `target/debug/...` directly after editing | old behaviour | `cargo run` rebuilds first. |
| Timed or benchmarked a dev build | looks slow | Measure with `--release`. |

Here's the missing-semicolon error, so you'll recognise it. It has no code number because it's found while *parsing*, before Rust even knows what the words mean:

```text
error: expected `;`, found `println`
  |
2 |     println!("Hello, world!")
  |                              ^ help: add `;` here
3 |     println!("Hello again!");
  |     ------- unexpected token
```

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p hello_world --example anatomy
cargo run -p hello_world --example print_macros
cargo run -p hello_world --example where_is_my_program
cargo run -p hello_world --example where_is_my_program --release
cargo run -p hello_world --example debug_vs_release
cargo run -p hello_world --example debug_vs_release --release
```

Then compile `without_cargo/main.rs` with `rustc` (see section 2).

**2. Do the exercises** in `exercises/`. They are switched off by default so unsolved ones never break `cargo test --workspace`; the `--features exercises` flag turns them on.

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_missing_bang` | Make it compile (E0423) | `cargo run -p hello_world --features exercises --bin ex01_fix_missing_bang` |
| 2 | `ex02_fix_main_name` | Make it compile (E0601) | `cargo run -p hello_world --features exercises --bin ex02_fix_main_name` |
| 3 | `ex03_fix_macro_name` | Make it compile (cannot find macro), output on two lines | `cargo run -p hello_world --features exercises --bin ex03_fix_macro_name` |
| 4 | `ex04_greeting` | Implement, pass 3 tests | `cargo test -p hello_world --features exercises --bin ex04_greeting` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p hello_world --features solutions --bins`, or run one with `--features solutions --bin sol01_fix_missing_bang`).

**4. Read the mini project** in `src/main.rs`: Hello World in a box, signed by Cargo.

```bash
cargo run -p hello_world                 # Hello, world! ... (debug build)
cargo run -p hello_world -- Ama          # Hello, Ama!
cargo run -p hello_world --release       # ... (release build)
cargo test -p hello_world                # 5 tests
```

```text
+---------------+
| Hello, world! |
+---------------+
built by Cargo: hello_world v0.1.0 (debug build)
```

It reads the package name and version from `Cargo.toml` with `env!`, and the profile with `cfg!(debug_assertions)`, so it is a small tour of what Cargo tells the compiler. Some of the code (`String`, `&str`, `format!`) is explained properly in later lessons; read it for the shape for now.

> 💡 **Analogy:** a birthday card with a sticker on the back that says which bakery made it and whether it was the quick test batch or the proper one.

## Checklist

- [ ] [1.1 Installation](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [ ] [1.2 Hello, World!](https://doc.rust-lang.org/book/ch01-02-hello-world.html)
- [ ] [1.3 Hello, Cargo!](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html)
- [ ] Beyond the Book: the course workspace, `-p`, `--example`, `--bin`, `--features` and `--`
- [ ] Beyond the Book: `cargo fmt` and `cargo clippy`
- [ ] Compiled `without_cargo/main.rs` with `rustc` and ran it
- [ ] Exercises 1–4 done without looking at the solutions

## Recap

- **rustup** installs and updates Rust (`rustup update`, `rustup doc`). **rustc** compiles. **Cargo** builds, runs, tests and fetches dependencies.
- Every program starts in `fn main()`. `println!` is a macro (the `!`). Strings use double quotes, and statements end with `;`.
- `rustc main.rs` makes a stand-alone executable: compiling and running are separate steps.
- `cargo new`, `cargo build`, `cargo run`, `cargo check`, `cargo build --release`. Output goes to `target/debug` or `target/release`. `Cargo.lock` pins exact versions.
- In this course: `-p <package>` picks the lesson, and `cargo fmt` plus `cargo clippy` keep code tidy.

**In one sentence for a child:** you write a recipe, the oven (`rustc`) bakes it into bread anyone can eat, and the head baker (Cargo) runs the oven and fetches ingredients so you don't have to.

**Next:** [02 Guessing Game](../02_guessing_game/README.md) (Book chapter 2)
