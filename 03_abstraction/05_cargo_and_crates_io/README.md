<!-- hand-written lesson: gen_course must not overwrite -->
# More about Cargo and Crates.io

**The Rust Book, chapter 14:** [More about Cargo and Crates.io](https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html)
· [The Cargo Book](https://doc.rust-lang.org/cargo/) · [The rustdoc book](https://doc.rust-lang.org/rustdoc/)
· Rust by Example: [Cargo](https://doc.rust-lang.org/rust-by-example/cargo.html), [Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
· [Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html)

So far Cargo has built, run and tested your code. This chapter shows the rest of the tool: how to tune builds, how to write documentation people can read on the web, how to share a crate with the world on [crates.io](https://crates.io), how to grow one project into several crates, and how to install and invent Cargo commands. "Beyond the Book" adds the parts every real crate uses: features, `cfg`, build scripts, rustdoc links and lint settings.

> 💡 **The big picture: a bakery and the town market.** Cargo is your **bakery**. You can bake a quick practice batch or a perfect batch for the shop window (**profiles**). Each cake gets a **recipe card** (doc comments) and a **label** (metadata), and the best sellers go at the front of the counter (`pub use`). The **market** (crates.io) is where you sell to everyone. A **workspace** is several bakers sharing one kitchen, one oven and one shopping list. And you can buy ready-made **gadgets** at the market and hang them on your wall (`cargo install`), or teach the bakery a new trick (custom commands).

## 1. Release profiles

A **profile** is a set of compiler settings. Cargo has two main ones:

| | `dev` profile | `release` profile |
|---|---|---|
| Used by | `cargo build`, `cargo run`, `cargo test` | `cargo build --release`, `cargo run --release` |
| `opt-level` | `0`: no optimisation, quick to compile | `3`: full optimisation, slow to compile |
| `debug` (debug info) | on | off |
| `debug-assertions`, `overflow-checks` | on: `debug_assert!` runs, `255u8 + 1` panics | off: `debug_assert!` is skipped, `255u8 + 1` wraps to 0 |
| Output folder | `target/debug/` | `target/release/` |

Cargo prints the profile it used on every build:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.98s
    Finished `release` profile [optimized] target(s) in 5.53s
```

The example builds the same loop both ways. On this machine the release build ran it in 138 ms instead of 777 ms, and the overflow check that panics in `dev` simply wraps in `release`:

```text
built in:          target/debug/                   built in:          target/release/
opt-level:         0                               opt-level:         3
debug_assertions:  true                            debug_assertions:  false
loop result:       59999997 in 777ms               loop result:       59999997 in 138ms
255u8 + 1:         panicked: attempt to add …      255u8 + 1:         0 (wrapped: overflow checks are off)
```

**Change a profile** by adding a `[profile.*]` table. Each one starts from the defaults and overrides only what you write:

```toml
[profile.dev]
opt-level = 1        # a little optimisation: faster programs, still quick builds

[profile.release]
lto = true           # optimise across crates: smaller and faster, slower to build
panic = "abort"      # no unwinding on panic: a smaller binary

[profile.release-with-debug]   # a CUSTOM profile: cargo build --profile release-with-debug
inherits = "release"
debug = true
```

In a workspace, profiles belong in the **root** `Cargo.toml`. In a member's `Cargo.toml` they are ignored, and Cargo says so (the real warning, from a scratch workspace):

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   …\ws\member\Cargo.toml
workspace: …\ws\Cargo.toml
```

That's why this lesson only *shows* profile settings: the course's profiles live in the root `Cargo.toml`, which every lesson shares. Exercise 6 works out which settings a profile really ends up with.

> 💡 **Analogy: practice batch vs shop-window batch.** When you're trying a recipe, you bake fast, taste as you go, and check everything (`dev`: quick to make, extra checks). For the shop window you take your time to make it perfect (`release`: slow to build, fast to run). A **profile** is the oven setting. A **custom profile** is a note saying "like the shop-window setting, but also keep my tasting notes".

▶ `cargo run -p cargo_and_crates_io --example release_profiles` and again with `--release`

## 2. Publishing a crate to crates.io

### Documentation comments

```rust
/// Adds one to the number given.        <- `///` documents the item BELOW it
///
/// # Examples
///
/// ```
/// let answer = my_crate::add_one(5);
/// assert_eq!(6, answer);
/// ```
pub fn add_one(x: i32) -> i32 { x + 1 }
```

- `///` is a **doc comment**: Markdown that documents the next item. `//!` documents the item it's **inside**, usually the whole crate at the top of `lib.rs`.
- `cargo doc --open` builds the HTML and opens it. Add `-p cargo_and_crates_io --no-deps` to document just this lesson.
- When you publish, [docs.rs](https://docs.rs) builds and hosts these pages for you.

> 💡 **Analogy: the recipe card.** Every cake on sale comes with a card: what it is, how to serve it, what to watch out for. `///` writes the card for one cake. `//!` writes the card on the front of the whole recipe book.

**The usual sections.** Readers look for these headings:

| Heading | Write it when | In this lesson |
|---|---|---|
| `# Examples` | always, for public items | `Manifest::parse`, `check`, `report::render` |
| `# Panics` | the function can panic, and how to avoid it | `Finding::new` (empty message) |
| `# Errors` | it returns `Result`: which errors, and when | `Manifest::parse`, `PiggyBank::withdraw` |
| `# Safety` | it's an `unsafe fn`: what the caller must promise | `PiggyBank::withdraw_unchecked` (unsafe is chapter 20) |

> 💡 **Analogy:** the sections are the boxes on every card: **Serving suggestion** (Examples), **Do not microwave!** (Panics), **May not rise if…** (Errors), and **Adults only: handle with oven gloves** (Safety).

▶ `cargo run -p cargo_and_crates_io --example doc_comments`, then `cargo doc -p cargo_and_crates_io --example doc_comments --no-deps --open`

### Doc tests

The code blocks in doc comments **are tests**. `cargo test` compiles and runs every one in a library, so an example can never quietly go out of date:

```text
$ cargo test -p cargo_and_crates_io --doc
test 03_abstraction\05_cargo_and_crates_io\src\licenses.rs - licenses::is_known_license (line 16) ... ok
test 03_abstraction\05_cargo_and_crates_io\src\lib.rs - (line 18) ... ok
test 03_abstraction\05_cargo_and_crates_io\src\checks.rs - checks::check (line 61) ... ok
test 03_abstraction\05_cargo_and_crates_io\src\manifest.rs - manifest::Manifest::parse (line 86) ... ok
…
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

- A line starting with `# ` runs but is **hidden** from the page. The lesson uses `# Ok::<(), cargo_and_crates_io::ParseError>(())` so an example can use `?` without showing the boilerplate.
- Doc tests run for **library** crates only, which is why this lesson's tested examples live in `src/`, not in `examples/`.

> 💡 **Analogy:** before the recipe card is printed, someone actually bakes the example on it. If the cake doesn't come out, the card isn't printed.

### `pub use`: a public API that's easy to use

How *you* organise your modules isn't always how *users* want to find things. The Book's `art` crate keeps colours in `art::kinds` and functions in `art::utils`. Re-exporting them makes `art::PrimaryColor` and `art::mix` work too:

```rust
pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;
```

Nothing moves: the long paths still work, and both name the same item. `cargo doc` lists the re-exports on the crate's front page. The mini project does the same: `cargo_and_crates_io::check` is really `cargo_and_crates_io::checks::check`.

If the short path doesn't exist yet, the user's `use` fails (the real error from exercise 1):

```text
error[E0432]: unresolved imports `shapes::Circle`, `shapes::Square`, `shapes::area`
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex01_fix_reexports.rs:57:14
   |
57 | use shapes::{Circle, Square, area};
   |              ^^^^^^  ^^^^^^  ^^^^ no `area` in `shapes`
   |              |       |
   |              |       no `Square` in `shapes`
   |              no `Circle` in `shapes`
   |
   = help: consider importing this struct instead:
           crate::shapes::kinds::Circle
```

The compiler's help fixes the *user's* code. Exercise 1 asks you to fix the *library* instead, so every user gets the short path. You can only re-export what is public: `pub use` of a `pub(crate)` item is E0364, and of a private item E0603.

> 💡 **Analogy: the shop counter.** The cakes are baked in the back room, on shelves sorted the way the bakers like (modules). The customers don't go in the back. `pub use` puts the best sellers **on the counter at the front**. They're the same cakes, just easy to reach.

▶ `cargo run -p cargo_and_crates_io --example pub_use_reexports`

### Accounts, metadata and publishing (read, don't run)

**These steps upload code for the whole world, permanently.** Read them now and run them only for a crate of your own. This lesson's `Cargo.toml` has `publish = false` so it can't happen by accident.

1. **Account:** log in to crates.io with GitHub, create an API token at <https://crates.io/settings/tokens>, then run `cargo login` and paste it. Cargo keeps it in `~/.cargo/credentials.toml`. Keep the token secret.
2. **Metadata** in `[package]`. The name must be unique on crates.io, and `cargo publish` refuses a crate without a `description` and a `license` (or `license-file`):

```toml
[package]
name = "cargo_and_crates_io"
version.workspace = true
edition.workspace = true
rust-version = "1.85"                          # the oldest Rust that can build it (MSRV)
description = "Lesson for Book chapter 14: …"  # required
license = "MIT OR Apache-2.0"                  # required: an SPDX expression
readme = "README.md"
keywords = ["cargo", "crates-io", "publishing", "lesson"]   # at most 5
categories = ["development-tools::cargo-plugins"]           # from crates.io/category_slugs
publish = false                                # never upload this one
```

   Leave them out, and Cargo warns you (the real output, from a bare crate):

```text
warning: manifest has no description, license, license-file, documentation, homepage or repository
```

3. **Check before you upload.** `cargo package --list` shows every file that would be uploaded (`exclude = [...]` in `[package]` leaves files out), and `cargo publish --dry-run` does everything except the upload. For this lesson the guard rail works:

```text
$ cargo publish -p cargo_and_crates_io --dry-run
error: `cargo_and_crates_io` cannot be published.
`package.publish` must be set to `true` or a non-empty list in Cargo.toml to publish.
```

4. **Publish:** `cargo publish`. A published version is **permanent**: it can never be deleted or overwritten, so that every project depending on it keeps building.
5. **New versions:** change `version` following [semantic versioning](https://semver.org) and publish again. Fixes bump the PATCH (1.0.**1**), new features the MINOR (1.**1**.0), breaking changes the MAJOR (**2**.0.0). Cargo relies on this: `rand = "0.8.5"` means "0.8.5 or any compatible later version", up to but not including 0.9.0. Exercise 4 implements those rules.
6. **Yank** a broken version with `cargo yank --vers 1.0.1` (undo with `--undo`). New projects can no longer pick it, but projects whose `Cargo.lock` already uses it keep working. Yanking deletes nothing: if you published a password, change the password.

> 💡 **Analogy: a stall at the market.** The **label** (metadata) says what the cake is, who may eat it (licence) and where the kitchen is (repository). Without a label, the market won't take it. Once sold, a cake can't be un-sold, so you bring a *new* batch with a new number. **Yanking** is taking a batch off the shelf: nobody new can buy it, but people who already have one at home can still eat it.

▶ `cargo run -p cargo_and_crates_io --example package_metadata` shows the metadata from inside a program (`env!("CARGO_PKG_VERSION")` and friends).

## 3. Cargo workspaces

A **workspace** is a set of packages that share **one `Cargo.lock`** and **one `target/` folder**. This course is one. Its root `Cargo.toml` has no `[package]`, only a `[workspace]` (a *virtual* manifest):

```toml
[workspace]
resolver = "3"
members = [
    "01_foundations/01_hello_world",
    # … 22 more …
    "03_abstraction/05_cargo_and_crates_io",
    "06_framework_mastery/axum_basics",
]

[workspace.package]   # values members can share
edition = "2024"
version = "0.1.0"
```

- Every lesson's `Cargo.toml` says `version.workspace = true` and `edition.workspace = true`, so changing them once at the root changes all 24.
- **One lock file:** every member that uses, say, `serde` gets the *same* version, so the crates always work together. Lesson folders don't have their own `Cargo.lock`.
- **One `target/`:** a dependency used by ten lessons is compiled once, not ten times.
- **`-p` picks a member:** `cargo run -p ownership`, `cargo test -p cargo_and_crates_io`. `--workspace` means all of them.
- **Members can depend on each other** with a path: `add_one = { path = "../add_one" }`. External crates still need listing in each member that uses them.
- **Each crate is published on its own**, with `cargo publish -p name`.

The example reads the real root `Cargo.toml` with the mini project's library:

```text
workspace root:  C:\Users\…\rust\Cargo.toml
resolver:        3
shared version   0.1.0   (members say `version.workspace = true`)
shared edition   2024   (members say `edition.workspace = true`)

24 members:
  01_foundations               6
  02_core_types_and_errors     5
  03_abstraction               8
  04_systems_level             4
  06_framework_mastery         1

shared Cargo.lock:          true
Cargo.lock in this lesson:  false
```

> 💡 **Analogy: one big kitchen.** Several bakers (crates) each make their own cakes, but they share **one oven** (`target/`), so nothing is baked twice, and **one shopping list** (`Cargo.lock`), so everyone uses the same brand of flour. The head baker's notice board (`[workspace.package]`) says "everyone uses the 2024 edition", and each baker just writes "same as the notice board".

▶ `cargo run -p cargo_and_crates_io --example workspace_tour`

## 4. Installing binaries with `cargo install`

`cargo install` downloads a crate's source, builds it in release mode, and puts the program in `~/.cargo/bin` (on Windows `%USERPROFILE%\.cargo\bin`), which rustup put on your PATH. It's for **tools**, not for libraries you want to use in code (that's `[dependencies]`).

```bash
cargo install ripgrep              # from crates.io (gives you the `rg` command)
cargo install --path .             # from a folder on your machine
cargo install --locked …           # use the crate's Cargo.lock: the versions its author tested
cargo install --list               # what you have installed
cargo uninstall ripgrep
```

This lesson's mini project is an installable tool. Installing it into a scratch folder (with `--root`, so nothing touched the real `~/.cargo/bin`) printed:

```text
   Compiling cargo_and_crates_io v0.1.0 (…\03_abstraction\05_cargo_and_crates_io)
    Finished `release` profile [optimized] target(s) in 4.41s
  Installing …\install-root\bin\cargo-preflight.exe
   Installed package `cargo_and_crates_io v0.1.0 (…)` (executable `cargo-preflight.exe`)
```

> 💡 **Analogy: buying a kitchen gadget.** You don't want the gadget's *blueprints* in your recipe, you want the **gadget on your wall**, ready to grab. `cargo install` buys it at the market, assembles it (release build), and hangs it on the wall where you always look (PATH).

## 5. Extending Cargo with custom commands

If a program called **`cargo-something`** is on your PATH, then `cargo something` runs it. That's how tools such as `cargo-edit` or `cargo-watch` add commands, and `cargo --list` shows them all.

Cargo starts the program with the command's name as the **first argument**, and sets `CARGO` to its own path, so the tool can run Cargo itself:

```text
$ cargo greet Ama Kofi
started as: ["…\debug\examples\cargo-greet.exe", "greet", "Ama", "Kofi"]
Hello, Ama and Kofi!
CARGO = C:\Users\…\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe
```

The mini project ships a second binary, `src/bin/cargo-preflight.rs`, so after `cargo install --path 03_abstraction/05_cargo_and_crates_io --bin cargo-preflight` you can type `cargo preflight` in any Rust project.

> 💡 **Analogy: teaching the bakery a new word.** The head baker (Cargo) only knows a few orders: build, run, test. Hang a gadget labelled **"cargo-greet"** on the wall, and when someone shouts "cargo, greet!", the head baker knows which gadget to hand the job to.

▶ `cargo run -p cargo_and_crates_io --example cargo-greet -- greet Ama` (the file explains how to put it on your PATH)

## Beyond the Book: feature flags and optional dependencies

**Features** are named switches in `Cargo.toml` that turn code on or off at compile time:

```toml
[dependencies]
serde = { version = "1", features = ["derive"], optional = true }   # only built when asked for
serde_json = { version = "1", optional = true }

[features]
default = ["unicode"]                    # on unless --no-default-features
unicode = []                             # a plain switch
json = ["dep:serde", "dep:serde_json"]   # switches on two optional dependencies
```

```bash
cargo run -p cargo_and_crates_io --features json -- 03_abstraction/05_cargo_and_crates_io --json
cargo run -p cargo_and_crates_io --no-default-features      # ASCII marks: [x] [!] [ok]
cargo test -p cargo_and_crates_io --features json                # 23 unit tests: one more for JSON
```

- A feature can switch on other features and **optional dependencies** (`dep:name`). A dependency marked `optional = true` isn't even downloaded or compiled unless a feature asks for it.
- **Features are additive.** Cargo merges ("unifies") every feature anyone asks for, so two crates asking for different `serde` features get one `serde` with both. Never write a feature that *removes* something, because another crate could switch it on behind your back.
- A crate that depends on yours chooses: `cargo_and_crates_io = { version = "0.1", default-features = false, features = ["json"] }`.
- Asking for a feature that doesn't exist is an error: `error: the package 'cargo_and_crates_io' does not contain this feature: jsno` / `help: there is a similarly named feature: json`.

Exercise 5 implements how Cargo works out the final set of features.

> 💡 **Analogy: toppings.** The cake comes with **sprinkles** unless you say "no default toppings" (`default`). **Chocolate sauce** is extra and has to be asked for (`--features json`), and asking for it also brings out the sauce jug (an optional dependency) that otherwise stays in the cupboard. Toppings only ever **add**: if two people at the table ask for different toppings, the cake gets both.

▶ `cargo run -p cargo_and_crates_io --example features_and_cfg`, then with `--features json` and with `--no-default-features`

## Beyond the Book: conditional compilation with `cfg`

```rust
#[cfg(feature = "json")]            // the item EXISTS only with the json feature
fn to_json(score: &Score) -> String { /* … */ }

#[cfg(windows)]  fn platform() -> &'static str { "Windows" }
#[cfg(not(windows))] fn platform() -> &'static str { "not Windows" }

#[cfg_attr(feature = "json", derive(serde::Serialize))]   // an attribute only sometimes
struct Point { x: i32, y: i32 }

if cfg!(debug_assertions) { /* … */ }   // a true/false VALUE: both branches compile
```

| Condition | True when |
|---|---|
| `feature = "json"` | the package's `json` feature is on |
| `test` | building with `cargo test` |
| `debug_assertions` | the profile has debug assertions (normally `dev`) |
| `windows`, `unix`, `target_os = "linux"` | building for that platform |
| `target_pointer_width = "64"` | 64-bit targets |
| `all(a, b)`, `any(a, b)`, `not(a)` | combinations |

`#[cfg(...)]` **removes** code before it is type-checked. So every call to a `cfg`-gated item must be gated the same way, and `if cfg!(...)` is no help, because both branches of an `if` are compiled. Exercise 3 hits exactly this:

```text
error[E0425]: cannot find function `to_json` in this scope
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex03_fix_feature_gate.rs:37:20
   |
37 |     println!("{}", to_json(&score));
   |                    ^^^^^^^ not found in this scope
   |
note: found an item that was configured out
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex03_fix_feature_gate.rs:27:4
   |
26 | #[cfg(feature = "json")]
   |       ---------------- the item is gated behind the `json` feature
27 | fn to_json(score: &Score) -> String {
   |    ^^^^^^^
```

Cargo tells the compiler which features exist, so a typo is caught too (real output from a crate with a `json` feature):

```text
warning: unexpected `cfg` condition value: `jsno`
 --> member\src\main.rs:2:13
  |
2 |     if cfg!(feature = "jsno") {
  |             ^^^^^^^^^^------
  |                       |
  |                       help: there is a expected value with a similar name: `"json"`
  |
  = note: expected values for `feature` are: `json`
```

> 💡 **Analogy: "if it's a birthday, add candles".** `#[cfg]` is a step in the recipe that is **torn out of the book** when it doesn't apply, so on an ordinary day the baker never even reads it. That's why you can't say "add the candles from step 7" if step 7 was torn out. `cfg!` is different: the step stays in the book, and the baker reads "if it's a birthday…" every time.

## Beyond the Book: build scripts (`build.rs`)

A file called `build.rs` next to `Cargo.toml` is a small program Cargo compiles and runs **before** compiling your crate. It talks to Cargo by printing `cargo::` lines:

| Line | Does |
|---|---|
| `cargo::rerun-if-changed=licenses.txt` | re-run only when this file changes (by default: when anything in the package changes) |
| `cargo::rustc-env=NAME=value` | sets an environment variable for compiling the crate, read with `env!("NAME")` |
| `cargo::rustc-cfg=has_feature_x` | sets a custom `cfg` (declare it with `cargo::rustc-check-cfg=cfg(has_feature_x)`) |
| `cargo::rustc-link-lib=z` | links a C library |
| `cargo::warning=message` | shows a warning to whoever is building |

This lesson's `build.rs` does two common jobs:
1. **Generates code.** It turns `licenses.txt` into Rust in `$OUT_DIR/licenses.rs`, and `src/licenses.rs` pastes it in with `include!(concat!(env!("OUT_DIR"), "/licenses.rs"))`. Add a licence to the text file, rebuild, and the checker knows it.
2. **Passes on build facts.** Cargo gives build scripts variables such as `PROFILE`, `OPT_LEVEL` and `TARGET`. The script forwards them as `BUILD_*` variables, which `release_profiles` prints.

```text
  | /// SPDX licence identifiers this checker knows, sorted.
  | ///
  | /// Generated at build time by `build.rs` from `licenses.txt`.
  | pub const KNOWN_LICENSES: &[&str] = &[
  |     "0BSD",
  |     "AGPL-3.0-only",
  | ...
KNOWN_LICENSES has 29 entries:
```

Real projects use build scripts to compile C code (the `cc` crate), generate bindings, or embed version information. Keep them fast and deterministic, since they run before every fresh build. Crates that only the build script needs go in `[build-dependencies]`.

> 💡 **Analogy: getting everything ready before you bake.** Before the oven goes on, someone weighs the flour, chops the nuts and reads the oven's thermometer. `build.rs` is that **prep work**: it gets ingredients ready (generated code) and writes notes for the baker (environment variables). `rerun-if-changed` says "only redo the chopping if the nuts changed".

▶ `cargo run -p cargo_and_crates_io --example build_script_output`

## Beyond the Book: rustdoc and intra-doc links

- **Intra-doc links** link to other items by their Rust path, not by URL: ``[`Manifest::parse`]``, ``[`u64::MAX`]``, or ``[the parser](Manifest::parse)``. rustdoc checks them, so a renamed item gives a warning instead of a dead link.
- Linking to an item that only exists with a feature is a real trap. This lesson's first draft did it, and `cargo doc` caught it:

```text
warning: unresolved link to `report::render_json`
  --> 03_abstraction\05_cargo_and_crates_io\src\lib.rs:40:22
   |
40 | //! - `json`: adds [`report::render_json`] and the `--json` option. It
   |                      ^^^^^^^^^^^^^^^^^^^ no item named `render_json` in module `report`
```

- **`missing_docs`** warns about every public item without a doc comment. Many libraries switch it on before publishing. Exercise 2 makes it an error:

```text
error: missing documentation for a module
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex02_fix_missing_docs.rs:18:1
   |
18 | pub mod weather {
   | ^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex02_fix_missing_docs.rs:16:9
   |
16 | #![deny(missing_docs)]
   |         ^^^^^^^^^^^^

error: missing documentation for a struct
  --> 03_abstraction\05_cargo_and_crates_io\exercises\ex02_fix_missing_docs.rs:19:5
```

- Useful `cargo doc` options: `--open`, `--no-deps` (only your crates), `--document-private-items` (for your own team), `--example name` and `--bin name` (programs, private items included).

> 💡 **Analogy:** an intra-doc link is "**see the chocolate sponge recipe**" written with the recipe's real name, so the librarian can check that the recipe really exists. If someone renames it, the librarian tells you, instead of a reader finding a dead end.

## Beyond the Book: lint configuration (`[lints]` and `clippy.toml`)

A **lint** is a check that warns about code that compiles but is probably wrong, or messy. Each lint has a **level**: `allow`, `warn`, `deny` (an error) or `forbid` (an error nobody can override). This lesson sets them for every target in the package, in `Cargo.toml`:

```toml
[lints.rust]
missing_docs = "warn"     # every public item needs a doc comment

[lints.clippy]
unwrap_used = "warn"      # prefer `?` or expect("why")
dbg_macro = "warn"        # no forgotten dbg!() calls
```

- A whole workspace can share one table: `[workspace.lints]` in the root, and `lints.workspace = true` in each member.
- An attribute in the code overrides the table for that file or item: `#![deny(missing_docs)]`, `#[allow(clippy::unwrap_used)]`. (`forbid` can't be overridden.)
- **`clippy.toml`**, next to `Cargo.toml`, tunes *how* clippy lints behave. This lesson's says `allow-unwrap-in-tests = true`. To prove it works, running clippy with the file hidden (an empty `CLIPPY_CONF_DIR`) gave 20 warnings like this one, all in tests:

```text
warning: used `unwrap()` on a `Result` value
   --> 03_abstraction\05_cargo_and_crates_io\src\checks.rs:270:16
    |
270 |         check(&Manifest::parse(toml).unwrap())
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: if this value is an `Err`, it will panic
    = help: consider using `expect()` to provide a better panic message
    = note: requested on the command line with `-W clippy::unwrap-used`
```

  The last line shows how the `[lints]` table works: Cargo passes each entry to the compiler as a command-line flag.
- In CI, `cargo clippy --all-targets -- -D warnings` turns every warning into a failure.

> 💡 **Analogy: kitchen rules.** `[lints]` is the **list of rules on the kitchen wall**: "wash your hands" (warn), "no peanuts, ever" (deny). `clippy.toml` is the **small print** under a rule: "no tasting from the spoon, *except* when testing the recipe". A note on one recipe card (`#[allow]`) can relax a rule for that recipe, unless the rule says "no exceptions" (`forbid`).

## The mini project: `cargo preflight`

`src/` is a real tool: it reads a crate's `Cargo.toml` and tells you whether crates.io would accept it, **before** you find out from `cargo publish`.

```bash
cargo run -p cargo_and_crates_io                               # every member of the workspace
cargo run -p cargo_and_crates_io -- 03_abstraction/05_cargo_and_crates_io   # one crate, in detail
cargo run -p cargo_and_crates_io -- 01_foundations/06_ownership
cargo run -p cargo_and_crates_io --no-default-features -- 01_foundations/06_ownership
cargo run -p cargo_and_crates_io --features json -- 03_abstraction/05_cargo_and_crates_io --json
cargo test -p cargo_and_crates_io                               # 22 unit tests + 6 doc tests
```

```text
Preflight for the workspace at C:\Users\…\rust (24 members)

  ✗    hello_world                  2 err  1 warn  missing or wrong: description, license
  …
  ✓    cargo_and_crates_io          0 err  1 warn
  ✗    smart_pointers               2 err  1 warn  missing or wrong: description, license
  …
1 of 24 crates are ready for crates.io.

cargo_and_crates_io 0.1.0 (03_abstraction/05_cargo_and_crates_io\Cargo.toml)
  ⚠ repository   no `repository` link: people like to read the source before they depend on it
  ℹ publish      `publish = false`: `cargo publish` will refuse, on purpose
  ℹ workspace    from [workspace.package]: edition, version
Ready for crates.io: yes (0 errors, 1 warning)
```

It uses every idea in the chapter:

| Idea | Where |
|---|---|
| Doc comments, `# Examples`/`# Errors`/`# Panics`, doc tests | every public item in `src/` |
| `pub use` re-exports | `src/lib.rs`: `check`, `Manifest`, `Finding`, … at the top |
| Metadata, semver, `publish = false` | what `src/checks.rs` checks, and this lesson's own `Cargo.toml` |
| Workspaces and `x.workspace = true` | `src/workspace.rs`, `Manifest::inherit_from` |
| `cargo install`, custom command | `src/bin/cargo-preflight.rs` |
| Features, optional deps, `cfg`, `cfg_attr` | `unicode` and `json`; `src/report.rs` |
| Build script | `build.rs` generates the licence list |
| `[lints]`, `clippy.toml` | `Cargo.toml`, `clippy.toml` |

A **workspace** gives one summary line per member and always exits with 0. A **single crate** gets every finding, and exit code 1 if crates.io would refuse it, so you can use it in CI.

> 💡 **Analogy: the market inspector.** Before the market opens, an inspector walks past every stall and checks each label: is there a name, a description, a licence, a price tag (version)? Stalls with a missing label get a red cross and a note saying what to fix. The inspector doesn't bake anything; they just make sure nothing gets turned away at the gate.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `[profile.*]` in a member's `Cargo.toml` | warning: profiles for the non root package will be ignored | Put it in the workspace root `Cargo.toml`. |
| Benchmarked a `dev` build | everything looks 5-50× slower than it is | `cargo run --release` |
| Relied on `255u8 + 1` wrapping | panics in `dev`, wraps in `release` | Say what you mean: `wrapping_add`, `checked_add`, `saturating_add`. |
| `use mylib::Thing` when it lives in `mylib::kinds` | E0432 unresolved import | In the library: `pub use self::kinds::Thing;` |
| `pub use` of a `pub(crate)` or private item | E0364 / E0603 | Make the item `pub` first. |
| A doc example that no longer compiles | `cargo test` fails the doc test | Keep it working; hide set-up lines with `# `. |
| `?` in a doc example | the doc test doesn't compile | End with a hidden `# Ok::<(), MyError>(())`. |
| `cargo publish` without `description`/`license` | the upload is refused | Fill in the metadata; check with `cargo publish --dry-run`. |
| Published a secret, then yanked it | it's still downloadable | Yanking hides, never deletes: change the secret. |
| `"MIT/Apache-2.0"` | old syntax | `"MIT OR Apache-2.0"` |
| Called a `#[cfg(feature = …)]` item unconditionally | E0425 without the feature | Gate the call the same way. `if cfg!(…)` is not enough. |
| `cfg!(feature = "jsno")` | warning: unexpected `cfg` condition value | Fix the spelling, or add the feature to `[features]`. |
| A feature that *removes* code | breaks when another crate enables it | Features must only add. |
| `cargo run --features json` at the workspace root | `cargo run` could not determine which binary to run | Name the package: `cargo run -p cargo_and_crates_io --features json` |
| `--all-features` on this lesson | it also switches on `exercises`, which don't compile yet | List the features you want: `--features json` |
| `build.rs` without `rerun-if-changed` | it re-runs after every edit | Print `cargo::rerun-if-changed=…` for its inputs. |
| An intra-doc link to a feature-gated item | warning: unresolved link | Plain code formatting, or gate the link's text too. |
| `cargo install` for a library crate | error: no packages found with binaries or examples | Add it to `[dependencies]` instead. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p cargo_and_crates_io --example release_profiles
cargo run -p cargo_and_crates_io --example release_profiles --release
cargo run -p cargo_and_crates_io --example doc_comments
cargo run -p cargo_and_crates_io --example pub_use_reexports
cargo run -p cargo_and_crates_io --example package_metadata
cargo run -p cargo_and_crates_io --example workspace_tour
cargo run -p cargo_and_crates_io --example cargo-greet -- greet Ama
cargo run -p cargo_and_crates_io --example features_and_cfg
cargo run -p cargo_and_crates_io --example features_and_cfg --features json
cargo run -p cargo_and_crates_io --example build_script_output
cargo doc -p cargo_and_crates_io --no-deps --open
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_reexports` | Make it compile (E0432) with `pub use`, library side only | `cargo run -p cargo_and_crates_io --features exercises --bin ex01_fix_reexports` |
| 2 | `ex02_fix_missing_docs` | Make it compile (`missing_docs`): write the docs | `cargo run -p cargo_and_crates_io --features exercises --bin ex02_fix_missing_docs` |
| 3 | `ex03_fix_feature_gate` | Make it compile (E0425) with and without `--features json` | `cargo run -p cargo_and_crates_io --features exercises --bin ex03_fix_feature_gate` |
| 4 | `ex04_semver_requirements` | Cargo's caret rules, pass 4 tests | `cargo test -p cargo_and_crates_io --features exercises --bin ex04_semver_requirements` |
| 5 | `ex05_resolve_features` | Work out which features end up on, pass 3 tests | `cargo test -p cargo_and_crates_io --features exercises --bin ex05_resolve_features` |
| 6 | `ex06_profile_inheritance` | Profile defaults, overrides and `inherits`, pass 4 tests | `cargo test -p cargo_and_crates_io --features exercises --bin ex06_profile_inheritance` |

For exercise 3, check both builds: also run it with `--features exercises,json`.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p cargo_and_crates_io --features solutions --bins`).

**4. The mini project:** run `cargo preflight` on the workspace and on single lessons (commands above), read `src/lib.rs` from the top, then try:
- Add a licence to `licenses.txt`, rebuild, and watch `build.rs` regenerate the list.
- Install it for real with `cargo install --path 03_abstraction/05_cargo_and_crates_io --bin cargo-preflight --locked`, run `cargo preflight` in any Rust project, then `cargo uninstall cargo_and_crates_io`.

## Checklist

- [ ] [14.1 Customizing Builds with Release Profiles](https://doc.rust-lang.org/book/ch14-01-release-profiles.html)
- [ ] [14.2 Publishing a Crate to Crates.io](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html)
- [ ] [14.3 Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [ ] [14.4 Installing Binaries with cargo install](https://doc.rust-lang.org/book/ch14-04-installing-binaries.html)
- [ ] [14.5 Extending Cargo with Custom Commands](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html)
- [ ] Beyond the Book: feature flags and optional dependencies
- [ ] Beyond the Book: conditional compilation with `cfg`
- [ ] Beyond the Book: build scripts (`build.rs`)
- [ ] Beyond the Book: documentation with rustdoc, intra-doc links
- [ ] Beyond the Book: lint configuration: `[lints]` table, `clippy.toml`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- **Profiles** choose compiler settings: `dev` builds fast with checks, `release` runs fast. Change them in the workspace root, and build custom ones with `inherits`.
- `///` and `//!` write Markdown docs with `# Examples`, `# Panics`, `# Errors` and `# Safety`. Their code blocks are tested, and `pub use` gives users short paths.
- Publishing needs a token, a unique name, a description and a licence. Versions are permanent and follow semver, and yanking hides a version without deleting it.
- A **workspace** shares one `Cargo.lock` and one `target/`. This course is one, and members inherit values with `x.workspace = true`.
- `cargo install` puts tools in `~/.cargo/bin`, and any `cargo-xyz` on your PATH becomes `cargo xyz`.
- Features switch code and optional dependencies on, `cfg` removes code at compile time, `build.rs` runs before the build, and `[lints]` with `clippy.toml` sets the house rules.

**In one sentence for a child:** Cargo is a bakery that can bake quickly or perfectly, writes a tested recipe card for every cake, shares one kitchen between many bakers, and sells the best cakes at the market with a proper label on each.

**Next:** [06 Smart Pointers](../06_smart_pointers/README.md) (Book chapter 15)
