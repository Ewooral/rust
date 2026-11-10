# Mastering Rust: basics to mastery

A hands-on course that follows [The Rust Programming Language](https://doc.rust-lang.org/book/) (the Book) chapter by chapter, then goes beyond it into mastery topics and real frameworks.

Every lesson is a crate in one Cargo workspace. Each lesson folder has a `README.md` with the Book sections it covers as a checklist, plus "Beyond the Book" extras.

```bash
cargo test --workspace      # run every lesson's tests
cargo run -p ownership      # run one lesson
cargo clippy --workspace --all-targets
```

## Course map

| Category | Book chapters | What you learn |
|---|---|---|
| [01 Foundations](01_foundations/README.md) | 1-4 | Cargo, syntax, data types, functions, control flow, ownership |
| [02 Core Types and Errors](02_core_types_and_errors/README.md) | 5-9 | Structs, enums, modules, collections, error handling |
| [03 Abstraction](03_abstraction/README.md) | 10-15, 18-19 | Generics, traits, lifetimes, tests, CLI project, iterators, Cargo, smart pointers, trait objects, patterns |
| [04 Systems Level](04_systems_level/README.md) | 16-17, 20-21 | Threads, async, unsafe and advanced features, multithreaded web server |
| [05 Mastery](05_mastery/README.md) | beyond the Book | Async internals, macros, performance, API design, capstones |
| [06 Framework Mastery](06_framework_mastery/README.md) | beyond the Book | Axum, Tokio, Serde, SQLx, Clap, Actix-web |

## Reference (Book appendix)

Keep these open while you work:
[Keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html) ·
[Operators and symbols](https://doc.rust-lang.org/book/appendix-02-operators.html) ·
[Derivable traits](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) ·
[Development tools](https://doc.rust-lang.org/book/appendix-04-useful-development-tools.html) ·
[Editions](https://doc.rust-lang.org/book/appendix-05-editions.html) ·
[Nightly Rust](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)

Going deeper than the Book:
[The Rust Reference](https://doc.rust-lang.org/reference/) (the full language) ·
[The Rustonomicon](https://doc.rust-lang.org/nomicon/) (unsafe Rust) ·
[The Async Book](https://rust-lang.github.io/async-book/) ·
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

More examples for any topic: [Rust by Example](https://doc.rust-lang.org/rust-by-example/) · Standard library: [std docs](https://doc.rust-lang.org/std/)
