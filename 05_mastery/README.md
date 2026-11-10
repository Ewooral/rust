# 05 Mastery

Beyond the Book. Start these once 01-04 are done.

- [ ] **Async traits from scratch**: rebuild a `UserService` async trait by hand, fixing each compiler error: dyn compatibility, boxed futures, `Pin`/`Unpin`, `Send`/`Sync`, lifetime bounds, then replace it all with `#[async_trait]`. (Based on Let's Get Rusty, "Rust's most complicated features explained".)
- [ ] **Macros in depth**: `macro_rules!` patterns, procedural macros (derive, attribute, function-like) with `syn` and `quote`
- [ ] **Performance**: profiling, allocation, `cargo bench` with `criterion`, release profiles
- [ ] **API design**: the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), builder and newtype patterns, error design
- [ ] **Advanced testing**: integration tests, mocking with traits, fuzzing
- [ ] **Observability**: structured logging and spans with `tracing`
- [ ] **Serialization**: `serde` derive, custom serializers, zero-copy deserialization
- [ ] **`no_std` and embedded**: `core` vs `alloc` vs `std`, writing a `no_std` library, embedded with `embassy`
- [ ] **WebAssembly**: compiling to `wasm32`, `wasm-bindgen`, calling Rust from JavaScript
- [ ] **Capstones**: a production CLI tool, an interpreter, a REST API with auth and database
