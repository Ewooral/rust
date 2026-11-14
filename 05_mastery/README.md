# 05 Mastery

Beyond the Book: the topics that separate writing Rust from writing professional Rust. Start these once 01-04 are done. Every lesson follows the course format: an analogy for every concept, runnable examples, exercises behind `--features exercises`, solutions behind `--features solutions`, and a mini project.

| Lesson | Package | What you learn |
|---|---|---|
| [01 Async traits from scratch](01_async_traits_from_scratch/README.md) | `async_traits_from_scratch` | Rebuild an async `UserService` trait by hand, fixing each compiler error: dyn compatibility, boxed and pinned futures, `Send`/`Sync`, lifetime bounds, then `#[async_trait]`. (Based on Let's Get Rusty, "Rust's most complicated features explained".) |
| [02 Macros in depth](02_macros_in_depth/README.md) | `macros_in_depth` | `macro_rules!` patterns, hygiene and `$crate`; derive, attribute and function-like procedural macros with `syn` and `quote`; testing macros with trybuild |
| [03 Performance](03_performance/README.md) | `performance` | Measuring first, release profiles, `criterion` benchmarks, profilers, allocations, data layout, rayon |
| [04 API design](04_api_design/README.md) | `api_design` | The Rust API Guidelines: naming, conversions, builders, newtypes, typestate, sealed traits, `#[non_exhaustive]`, error design |
| [05 Advanced testing](05_advanced_testing/README.md) | `advanced_testing` | Fakes and `mockall`, injected clocks, `insta` snapshots, property tests, fuzz harnesses, coverage and nextest |
| [06 Observability](06_observability/README.md) | `observability` | `tracing`: events, spans, `#[instrument]`, layers, `EnvFilter`/`RUST_LOG`, JSON logs, metrics |
| [07 Serialization](07_serialization/README.md) | `serialization` | Serde in depth (also the course's Serde framework lesson): attributes, enum representations, custom (de)serializers, zero-copy, format migration |
| [08 `no_std` and embedded](08_no_std_and_embedded/README.md) | `no_std_and_embedded` | `core` vs `alloc` vs `std`, a `no_std` library tested on the PC, embedded-hal drivers, embassy (explained) |
| [09 WebAssembly](09_webassembly/README.md) | `webassembly` | wasm32 targets, `wasm-bindgen`, passing data to and from JavaScript, wasm-pack, WASI (target builds explained) |
| [10 Capstone: a production CLI tool](10_capstone_cli_tool/README.md) | `capstone_cli_tool` | `logsum`, a log summariser: clap, layered config, errors, JSON output, terminal detection, exit codes, end-to-end tests |
| [11 Capstone: an interpreter](11_capstone_interpreter/README.md) | `capstone_interpreter` | Kite, a small language: lexer, Pratt parser, AST, environments and closures, errors, a REPL |

Lessons that need tools or targets not installed by default (WebAssembly, embedded boards, cargo-fuzz, profilers) teach and test everything that runs on a normal PC, and mark the rest "not run here" with exact setup steps.

**Next:** [06 Framework Mastery](../06_framework_mastery/README.md)
