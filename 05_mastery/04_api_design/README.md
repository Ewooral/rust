<!-- hand-written lesson: gen_course must not overwrite -->
# API Design

**Official docs:** [The Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and its [checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
· The Cargo Book: [SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
· Reference: [`#[non_exhaustive]`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
· std: [`std::convert`](https://doc.rust-lang.org/std/convert/index.html), [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
· The rustdoc book: [Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
· crates: [`thiserror`](https://docs.rs/thiserror)

Run it: `cargo run -p api_design`  ·  Test it: `cargo test -p api_design`

Code that only you call can be shaped any way you like. A library is different: other people build on it, they can't see inside, and every public name, type and trait impl is a promise you'll have to keep. The [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) collect what the standard library and the best crates do, as short rules with ids like C-CONV or C-GETTER. After this lesson you will be able to:

- name things so Rust programmers know what they do and what they cost before reading the docs;
- give your types the conversions and traits users expect, and accept flexible arguments;
- use builders, newtypes, typestate and sealed traits to make wrong use hard or impossible;
- design error types for a library, document it with tested examples, and change it later without breaking anyone.

You've met many of the tools: traits and the orphan rule ([03/01](../../03_abstraction/01_generics_traits_lifetimes/README.md#6-conversion-traits-and-common-std-traits)), `thiserror` ([02/05](../../02_core_types_and_errors/05_error_handling/README.md#beyond-the-book-thiserror-and-anyhow)), docs and `pub use` ([03/05](../../03_abstraction/05_cargo_and_crates_io/README.md)), newtypes ([04/03](../../04_systems_level/03_advanced_features/README.md#newtypes-for-type-safety-and-abstraction)). This lesson is about using them **for other people**.

> 💡 **The big picture: designing the controls of a machine for strangers.** Your library is a machine other people use without opening it. Good controls are **labelled like every other machine's** (naming), **take any sensible plug** (conversions and generic arguments), **can't be pressed in the wrong order** (typestate), **say clearly what went wrong** (errors), **come with a manual whose examples really work** (doc tests), and the next model **keeps the buttons where they were** (semver).

## 1. Naming

| Rule | Example | Meaning |
|---|---|---|
| `as_…` | `str::as_bytes`, `Url::as_str` | a **free, borrowed view** of the same data |
| `to_…` | `str::to_uppercase`, `Path::to_path_buf` | **makes new data**, may be expensive; `self` is borrowed |
| `into_…` | `String::into_bytes`, `Url::into_string` | **consumes** `self` and hands over what's inside; usually free |
| getters | `Url::host()`, `Vec::len()` | named after the field, **no `get_`** (`get` alone is for lookups like `HashMap::get`) |
| `…_mut` | `first_mut`, `iter_mut`, `get_mut` | the mutable twin |
| iterators | `iter()`, `iter_mut()`, `into_iter()` | yields `&T`, `&mut T`, `T` |
| constructors | `new`, `with_capacity`, `from_…`, `builder()` | `new` for the main one; `try_…` if it can fail |
| case | `UpperCamelCase` types and traits, `snake_case` functions, `SCREAMING_SNAKE_CASE` constants | acronyms are words: `Url`, `HttpClient`, not `URL` |

The example shows each rule on the standard library and on a small `Word` type: `as_str` (a view), `to_shouting` (a new `Word`), `into_string` (the `Word` is gone afterwards).

> 💡 **Analogy: the labels on a kitchen.** "**as**" is looking through a window: nothing moves. "**to**" is photocopying the recipe: you get a new page, and it takes a moment. "**into**" is handing the recipe over: it's theirs now, and you don't have it any more. Every kitchen in town uses the same three labels, so visitors know what each drawer does without asking.

▶ `cargo run -p api_design --example naming_conventions`

## 2. Conversions

| Trait | For | You get |
|---|---|---|
| `From<T>` | conversions that **can't fail** | `Into` for free: implement `From`, never `Into` directly |
| `TryFrom<T>` | conversions that **can fail**, with an error type | `TryInto` for free |
| `FromStr` | parsing text | `"…".parse::<T>()` |
| `AsRef<T>` | a **cheap borrowed view** as `&T` | `fn f(x: impl AsRef<str>)` takes `&str`, `String`, `&String`, `Url` … |
| `Display` | text for users | `.to_string()` |

- `?` converts errors with `From`: once `impl From<ParseIntError> for MyError` exists, `text.parse()?` works inside a function returning `MyError`. The `conversions` example's `FromStr for Percent` uses two such conversions.
- Implementing `Into` instead of `From` compiles, but clippy warns: "an implementation of `From` is preferred since it gives you `Into<_>` for free where the reverse isn't true" (`clippy::from_over_into`).
- The library's `Url` has the full set: `Url::parse`, `FromStr`, `TryFrom<&str>`, `TryFrom<String>`, `AsRef<str>`, `Display` and `From<Url> for String`.

> 💡 **Analogy: plugs and adapters.** `From` is an adapter that always fits. `TryFrom` is an adapter with a fuse: it may refuse a plug that's the wrong voltage, and tells you why. `AsRef` is a **see-through cover**: you can read the label underneath without unplugging anything. And you only ever build the adapter one way round (`From`); the other direction comes in the box.

▶ `cargo run -p api_design --example conversions`

## 3. Common traits

A public type should implement every common trait that makes sense (C-COMMON-TRAITS): `Debug` (always), `Clone`, `Copy` for small plain data, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Hash`, `Default`, and `Display` when there's one obvious text form. Each one is a promise users build on: `Debug` for `{:?}` and `assert_eq!`, `Ord` for `sort()` and `BTreeMap` keys, `Hash + Eq` for `HashMap` keys, `Default` for `..Default::default()`.

**Users can't add them for you.** The orphan rule forbids implementing a foreign trait for a foreign type, so a missing derive in your library is a wall in theirs. Exercise 1's `Version` has none:

```text
error[E0277]: the trait bound `Version: Ord` is not satisfied
   --> 05_mastery\04_api_design\exercises\ex01_fix_missing_traits.rs:47:14
    |
 47 |     versions.sort();
    |              ^^^^ the trait `Ord` is not implemented for `Version`
    |
help: consider annotating `Version` with `#[derive(Ord)]`

error[E0277]: `Version` doesn't implement `Debug`
    --> 05_mastery\04_api_design\exercises\ex01_fix_missing_traits.rs:48:23
     |
  48 |     println!("sorted: {versions:?}");
     |                       ^^^^^^^^^^^^ `Version` cannot be formatted using `{:?}` because it doesn't implement `Debug`

error[E0277]: the trait bound `Version: Default` is not satisfied
  --> 05_mastery\04_api_design\exercises\ex01_fix_missing_traits.rs:53:26
   |
53 |     let start: Version = Default::default();
   |                          ^^^^^^^^^^^^^^^^^^ the trait `Default` is not implemented for `Version`
```

(`Version::default()` written as a method call is E0599 instead: "no associated function or constant named `default` found".)

**`Send` and `Sync`** (C-SEND-SYNC) come automatically when every field has them, and disappear silently when one doesn't (an `Rc` field, say). The library's `public_types_are_send_and_sync` test pins them down at compile time:

```rust
fn assert_send_sync<T: Send + Sync + 'static>() {}
assert_send_sync::<Request>();
assert_send_sync::<RequestError>();
```

Adding a trait impl later is (almost always) fine. **Removing** one is a breaking change.

> 💡 **Analogy: standard sockets on a new appliance.** A kettle with no standard plug is useless however good it is, and the customer isn't allowed to rewire it (the orphan rule). So the factory fits the standard sockets before it ships: print it, copy it, compare it, sort it, use it as a key, start it empty.

▶ `cargo run -p api_design --example common_traits`

## 4. Builders

When a type has many optional settings, a constructor with eight arguments is unreadable (`Server::new("api", None, Some(8), true, …)`). A **builder** names each setting and puts defaults and validation in one place, `build()`:

| Style | Methods | Good for |
|---|---|---|
| **consuming** | `fn port(mut self, p: u16) -> Self` | one chained expression; can move values in (`RequestBuilder`) |
| **non-consuming** | `fn filter(&mut self, c: &str) -> &mut Self` | configuring in steps or in `if`s, building several results (`std::process::Command`) |

- `#[must_use = "a builder does nothing until you call .build()"]` on the builder turns a forgotten `build()` into a warning.
- `build()` returns `Result` when settings can be wrong. Errors that only show up while chaining (a bad URL, a bad header name) can be **remembered** and returned by `build()`, so every step stays chainable. The library's `RequestBuilder` does this, like `reqwest`.

```rust
let request = Request::builder()
    .method(Method::Post)
    .url("http://localhost:8080/users")
    .header("Content-Type", "application/json")
    .body(r#"{"name": "Ada"}"#)
    .build()?;
```

> 💡 **Analogy: ordering a sandwich at a counter.** Instead of shouting ten words at once in a fixed order, you say one thing at a time: "brown bread", "no onions", "extra cheese". Anything you don't mention gets the house default, and the sandwich is only made (and checked) when you say "that's it" (`build`).

▶ `cargo run -p api_design --example builder_and_newtype`

## 5. Newtypes

A **newtype** is a struct with one field, `struct UserId(u64)`. It costs nothing at run time and does three jobs (C-NEWTYPE):

- **Type safety.** `cancel(user: UserId, order: OrderId)` can't be called with the ids swapped: `cancel(OrderId(7), UserId(1))` is `error[E0308]: arguments to this function are incorrect`. As two `u64`s it would compile and cancel the wrong order.
- **Validation** ("parse, don't validate"). `Url`'s fields are private and `Url::parse` is the only way in, so every `Url` in a program is valid and nothing downstream checks again. `Port::new(80)` in the example refuses system ports the same way.
- **Hiding the implementation** (C-NEWTYPE-HIDE). `Headers::iter` returns `HeaderIter<'_>`, a newtype around `slice::Iter`. Users can't depend on what's inside, so it can change without a new major version.

> 💡 **Analogy: coloured tickets.** A cinema ticket and a train ticket are both just paper with a number, but they're printed in different colours so nobody can use one as the other. A newtype is the colour. Some tickets are only printed at the ticket office after a check (validation), and the ticket never says how the office files them (hiding).

▶ `cargo run -p api_design --example builder_and_newtype`

## 6. Typestate

**Typestate** puts a value's state in its **type**, so calling a method in the wrong state is a compile error. States are empty marker types, and a `PhantomData<State>` field (0 bytes) carries them:

```rust
struct Post<State> { text: String, approvals: u8, state: PhantomData<State> }

impl Post<Draft>     { fn edit(&mut self, …); fn submit(self) -> Post<InReview> }
impl Post<InReview>  { fn approve(self) -> Self; fn decide(self) -> Result<Post<Published>, Post<Draft>> }
impl Post<Published> { fn url(&self) -> String }
```

`post.url()` on a draft is ``E0599: no method named `url` found for struct `Post<Draft>` ``, and editing after `submit(self)` is `E0382: borrow of moved value`. The library's builder uses the same idea: `RequestBuilder<NoUrl>` has `url` but no `build`; `url` returns a `RequestBuilder<HasUrl>`, which has `build` but no `url`. Exercise 3 calls them in the wrong order:

```text
error[E0599]: no method named `build` found for struct `RequestBuilder<NoUrl>` in the current scope
  --> 05_mastery\04_api_design\exercises\ex03_fix_typestate_order.rs:22:10
   |
18 |     let login = Request::builder()
19 |         .method(Method::Post)
20 |         .header("Content-Type", "application/x-www-form-urlencoded")
21 |         .body("user=ada&password=hunter2")
22 |         .build()
   |          ^^^^^ method not found in `RequestBuilder<NoUrl>`
   |
   = note: the method was found for `RequestBuilder<HasUrl>`

error[E0599]: no method named `url` found for struct `RequestBuilder<HasUrl>` in the current scope
  --> 05_mastery\04_api_design\exercises\ex03_fix_typestate_order.rs:29:10
   |
29 |         .url("https://example.org/profile")
   |          ^^^ private field, not a method
```

(The second message also notices the builder's private *field* called `url`; the point is the same: no `url` method in the `HasUrl` state.) Use typestate for orders that really matter (connect before send, configure before start), not for every setting: each state is a type users have to read about.

> 💡 **Analogy: a vending machine.** The "dispense" button doesn't exist until you've paid: it's not greyed out, it simply isn't on the panel. You can't press a button that isn't there.

▶ `cargo run -p api_design --example typestate`

## 7. Sealed traits

A public trait that other crates **may use but not implement** (C-SEALED). The trick is a supertrait they can't name:

```rust
mod private {                       // a PRIVATE module…
    pub trait Sealed {}             // …with a public trait in it
    impl Sealed for String {}
    impl Sealed for &str {}         // … one per allowed type
}

pub trait IntoBody: private::Sealed {
    fn into_body(self) -> Body;
}
```

Why? Adding a method to a trait breaks every impl outside your crate. If there can't be any, you're free to add methods in a minor version. Trying it from another crate (real output):

```text
error[E0277]: the trait bound `Json: api_design::body::private::Sealed` is not satisfied
   = note: `IntoBody` is a "sealed trait", because to implement it you also need to implement
           `api_design::body::private::Sealed`, which is not accessible; this is usually done to
           force you to use one of the provided types that already implement it
```

Users still call `.body("text")`, `.body(vec![1, 2])` or `body.into_body()` as normal.

> 💡 **Analogy: a club with a members' list.** Anyone may **use** the club's services, but only people on the list held at the front desk can **join**, and the list is kept in the back office where visitors can't write their own name in.

▶ `cargo run -p api_design --example sealed_and_non_exhaustive`

## 8. `#[non_exhaustive]`

`#[non_exhaustive]` reserves the right to grow a type later. It only affects **other crates** (inside your own crate, nothing changes):

| On | Other crates must | Or they get |
|---|---|---|
| an enum (`Method`, `RequestError`) | add a `_ =>` arm to every `match` | E0004 non-exhaustive patterns: `_` not covered |
| a struct (`Parts`) | write `..` when destructuring | E0638 `..` required with struct marked as non-exhaustive |
| a struct | not build it with a struct literal | E0639 cannot create non-exhaustive struct using struct expression |
| an enum variant | not build that variant with a literal | E0639 |

Exercise 2 is another crate matching both library enums with every variant listed:

```text
error[E0004]: non-exhaustive patterns: `_` not covered
  --> 05_mastery\04_api_design\exercises\ex02_fix_non_exhaustive_match.rs:20:11
   |
20 |     match method {
   |           ^^^^^^ pattern `_` not covered
   |
note: `Method` defined here
  --> 05_mastery\04_api_design\src\method.rs:22:1
   |
22 | pub enum Method {
   | ^^^^^^^^^^^^^^^
   = note: the matched value is of type `Method`
   = note: `Method` is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively
```

Use it on error enums (new failures happen) and on structs of public fields or options that will grow. Don't use it where users genuinely need to handle every case, like `Ordering`.

> 💡 **Analogy: "and more coming soon".** A menu that says "more dishes coming soon" warns you to have an answer ready for a dish you've never heard of. Without that line, adding a new dish would confuse every waiter who learnt the menu by heart.

## 9. Error design

For a **library** (C-GOOD-ERR):

- **One error enum** per library, or per area of a big one, with a **variant per kind of failure**, so callers can `match` on what they can handle. `thiserror` writes `Display`, `Error` and `From` for you; it's the usual choice for libraries. Applications use something like `anyhow` to collect any error ([02/05](../../02_core_types_and_errors/05_error_handling/README.md#beyond-the-book-thiserror-and-anyhow)).
- **Details as data** in each variant (`InvalidUrl { url, source }`), not only in the message.
- **Keep the cause** as `source()`: `#[source]`, or `#[from]` which also writes the `From` impl for `?`.
- **Messages:** lower case, no full stop, and **don't repeat the source's text**, because whoever prints the chain prints that too.
- **`Send + Sync + 'static`**, so it fits in `Box<dyn Error + Send + Sync>` and crosses threads; and `#[non_exhaustive]`.

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RequestError {
    #[error("invalid URL {url:?}")]
    InvalidUrl { url: String, #[source] source: UrlError },
    #[error("could not read the body from {}", path.display())]
    BodyFile { path: PathBuf, #[source] source: std::io::Error },
    // …
}
```

An application walks the chain with `source()`. The mini project's output:

```text
error: invalid URL "http://example.org:99999/"
  caused by: invalid port "99999"
  caused by: number too large to fit in target type
```

Three layers, three messages, none repeated: `RequestError` knows the URL, `UrlError` knows the port, and `ParseIntError` knows why.

> 💡 **Analogy: a doctor's note with a referral.** The note says what's wrong in plain words ("can't come to school"), the form underneath has the details (which day, which illness), and it's **stapled** to the specialist's report (the source). Whoever reads it can stop at the top page or follow the staples all the way down.

▶ `cargo run -p api_design --example error_design`

## 10. Accepting generic arguments

| Parameter | Accepts | Use when |
|---|---|---|
| `impl AsRef<Path>` | `&str`, `String`, `&Path`, `PathBuf` | you only need to look at a path (like `std::fs::read`) |
| `impl AsRef<str>` | `&str`, `String`, `&String`, `Url` | you only need to read text |
| `impl Into<String>` | `&str` (copied), `String` (moved) | you need to **own** the string anyway |
| `impl IntoIterator<Item = T>` | `Vec`, arrays, slices' iterators, `map(…)` | you only loop over it |

- **Caller control** (C-CALLER-CONTROL): if you're going to store a `String`, take `impl Into<String>` (or `String`). Taking `&str` and calling `.to_string()` forces a copy even on callers who had a `String` to give away.
- Each generic function is compiled once per argument type. Keep it tiny: convert, then call a **non-generic inner function** with the real work. `RequestBuilder::body_file` does this.
- Don't be generic without a reason: `fn port(self, p: u16)` is clearer than `impl Into<u16>`.

> 💡 **Analogy: a phone charger with several heads.** The charger works the same however you plug it in; the head (`AsRef`, `Into`) just adapts the shape. And when the shop needs to **keep** your phone for repair (`Into<String>`), it's quicker to hand it over than to have it copied.

▶ `cargo run -p api_design --example generics_and_iterators`

## 11. Returning iterators

- **`-> impl Iterator<Item = &str> + '_`**: the simplest. Callers can loop, filter and collect, and you can change how it's computed. The `'_` says it borrows `self`.
- A **named iterator type** for a public API (C-ITER-TY): `HeaderIter<'a>` can be stored in a struct field, and it promises more: `ExactSizeIterator` (`.len()`) and `DoubleEndedIterator` (`.rev()`).
- **`IntoIterator for &T`** (and for `T`) makes `for x in &collection` work, as for every std collection.
- Different iterators in different branches are different types: `if only_low { a.filter(…) } else { a }` is `` error[E0308]: `if` and `else` have incompatible types ``. A `Box<dyn Iterator<Item = T> + '_>` is one type for both, at the cost of one allocation.

> 💡 **Analogy: a conveyor belt instead of a box.** Instead of packing every item into a box and handing it over (a `Vec`), you switch on a belt: the customer takes items as they come, stops whenever they like, and never needs to know what's behind the curtain.

▶ `cargo run -p api_design --example generics_and_iterators`

## 12. Documentation and doc tests

The library is documented the way the guidelines ask (C-CRATE-DOC, C-EXAMPLE, C-FAILURE):

- `#![warn(missing_docs)]` at the top of `lib.rs`: a public item without a doc comment is a warning (and an error in CI with `-D warnings`).
- A **crate-level example** in `//!` that shows the main use in a few lines.
- **`# Errors`** on every function returning `Result` (which errors, when), `# Panics` if it can panic, `# Safety` on `unsafe fn`.
- Examples use `?`, not `unwrap()`, with a hidden last line `# Ok::<(), api_design::RequestError>(())`.
- A ` ```compile_fail ` doc test proves the typestate works: `Request::builder().build()` must NOT compile.
- Intra-doc links (``[`Url::parse`]``) are checked by `cargo doc`.

```text
$ cargo test -p api_design --doc
test 05_mastery\04_api_design\src\lib.rs - (line 20) ... ok
test 05_mastery\04_api_design\src\request.rs - request::Request::get (line 44) ... ok
test 05_mastery\04_api_design\src\headers.rs - headers::Headers (line 10) ... ok
test 05_mastery\04_api_design\src\url.rs - url::Url (line 42) ... ok
test 05_mastery\04_api_design\src\request.rs - request::RequestBuilder (line 138) ... ok
test 05_mastery\04_api_design\src\method.rs - method::Method (line 12) ... ok
test result: ok. 6 passed; 0 failed; …
test 05_mastery\04_api_design\src\request.rs - request::RequestBuilder (line 151) - compile fail ... ok
```

`cargo doc -p api_design --no-deps --open` shows the result. [03/05](../../03_abstraction/05_cargo_and_crates_io/README.md#documentation-comments) has the basics of doc comments.

> 💡 **Analogy: a manual where every example has been tried.** Before the manual is printed, someone follows every example in it on a real machine. If one doesn't work, the manual isn't printed. Users trust a manual like that.

## 13. Semver-compatible evolution

Version numbers are a promise ([semver](https://semver.org)): a **minor** release (1.**2**.0) mustn't break code that compiled against 1.1. The [Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html) lists every case; the common ones, all shown in the `semver_evolution` example:

| Change | Breaking? | Users' code fails with | Plan ahead with |
|---|---|---|---|
| add a public field to a struct | yes | E0063 missing field `host` in initializer | private fields + constructor, or `#[non_exhaustive]` |
| add an enum variant | yes | E0004 non-exhaustive patterns: `v2::Level::Medium` not covered | `#[non_exhaustive]` |
| add a trait method with no default | yes | E0046 not all trait items implemented | a default body, or a sealed trait |
| add a function parameter | yes | E0061 this function takes 2 arguments but 1 argument was supplied | a new function; `#[deprecated]` on the old |
| remove or rename anything public, remove a trait impl | yes | E0425, E0599, E0277 … | `#[deprecated]` first, remove in the next major |
| add a new function, type, module or trait impl | no (in practice) | — | — |
| add a method with a default body to a sealed trait | no | — | — |

`#[deprecated(since = "2.0.0", note = "use connect_on to choose the port")]` keeps the old function working and warns: ``warning: use of deprecated function `v2::connect`: use `connect_on` to choose the port``. The [`cargo-semver-checks`](https://crates.io/crates/cargo-semver-checks) tool compares your crate with its last published version and lists breaking changes before you publish (it's installed with `cargo install`; this course doesn't).

> 💡 **Analogy: a new model of a car.** Drivers expect the pedals in the same place. You can add a new button (a new function) or a heated seat (a new trait impl), but moving the brake pedal (renaming, adding a required argument) needs a big new model number and a warning first, so nobody crashes.

▶ `cargo run -p api_design --example semver_evolution`

## The mini project: a request library

`src/` is a small library that builds HTTP/1.1 requests (it doesn't send them), where every public item follows a guideline:

| File | What's in it | Guidelines |
|---|---|---|
| [`src/url.rs`](src/url.rs) | `Url` (newtype, always valid), `UrlError` | C-NEWTYPE, C-CONV, C-GETTER, `as_`/`into_` |
| [`src/method.rs`](src/method.rs) | `Method`, `ParseMethodError` | C-COMMON-TRAITS, `FromStr`, `#[non_exhaustive]` |
| [`src/headers.rs`](src/headers.rs) | `Headers`, `HeaderIter`, `IntoHeaders` | C-ITER, C-ITER-TY, `impl Into<String>` |
| [`src/body.rs`](src/body.rs) | `Body`, sealed `IntoBody` | C-SEALED |
| [`src/request.rs`](src/request.rs) | `Request`, typestate `RequestBuilder<NoUrl \| HasUrl>`, `Parts` | C-BUILDER, typestate, `#[non_exhaustive]` struct |
| [`src/error.rs`](src/error.rs) | `RequestError` | C-GOOD-ERR, `source()` chains, `Send + Sync` |
| [`src/lib.rs`](src/lib.rs) | crate docs, `#![warn(missing_docs)]`, flat re-exports, tests | C-CRATE-DOC, C-SEND-SYNC |
| [`src/main.rs`](src/main.rs) | builds requests and prints them, or prints the error chain | how an application uses a library |

```bash
cargo run -p api_design                                       # a tour: 3 requests, 2 errors
cargo run -p api_design -- GET "http://example.org/search?q=rust"
cargo run -p api_design -- POST http://localhost:8080/notes "buy milk"
cargo run -p api_design -- GET http://example.org:99999/      # exit code 1, with the causes
cargo test -p api_design                                      # 11 unit tests + 7 doc tests
cargo doc -p api_design --no-deps --open                      # the documentation
```

```text
--- request 2 ---
POST /v1/notes HTTP/1.1
host: api.example.org:8443
content-type: application/json
content-length: 20

{"text": "buy milk"}

--- request 5 ---
error: could not read the body from no/such/file.txt
  caused by: The system cannot find the path specified. (os error 3)
```

(The `os error` text comes from the operating system, so it reads differently on Linux.)

> 💡 **Analogy: a post office form.** The form has clear boxes with standard labels, accepts addresses written in any reasonable way, won't let you post the letter before there's an address on it, and when something's wrong it hands back the form with a note that says *which* box and *why*, with the clerk's notes stapled behind.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `fn get_name(&self)` | works, but un-Rusty | `fn name(&self)` |
| `fn as_upper(&self) -> String` | the name promises a free view | `to_upper` (allocates) |
| `impl Into<B> for A` | clippy `from_over_into` | `impl From<A> for B` |
| A public type without `Debug`/`Clone`/`PartialEq` … | users hit E0277 and can't fix it (orphan rule) | Derive the common traits |
| `pub` fields on a type with rules | users build invalid values; adding a field breaks them (E0063) | Private fields, a checked constructor, getters |
| Two `u64` ids in one signature | swapped arguments compile | Newtypes: `UserId(u64)` |
| A builder whose `build()` is easy to forget | nothing happens, silently | `#[must_use]` on the builder |
| A required setting checked at run time | errors only in production | Typestate: no `build()` until it's set |
| A public trait you'll want to extend | a new method breaks users' impls (E0046) | Seal it, or give new methods a default |
| `match` on another crate's `#[non_exhaustive]` enum without `_` | E0004 | A `_ =>` arm |
| `String` errors in a library | callers can't tell failures apart | An error enum (`thiserror`) |
| The source's message repeated in your `Display` | the chain prints it twice | Keep it in `source()` only |
| An error type with an `Rc` inside | not `Send`: can't cross threads or box as `dyn Error + Send + Sync` | Owned, thread-safe data |
| `fn open(path: &str)` | `PathBuf` callers must convert | `impl AsRef<Path>` |
| `fn new(name: &str)` that stores `name.to_string()` | a copy even when the caller had a `String` | `impl Into<String>` |
| A big generic function | compiled once per type: slow builds, big binary | A non-generic inner function |
| `if … { iter.filter(…) } else { iter }` with `impl Iterator` | E0308 incompatible types | `Box<dyn Iterator>` (or two functions) |
| `unwrap()` in doc examples | readers copy it | `?` and a hidden `# Ok::<(), E>(())` |
| A breaking change in a minor version | users' builds break on `cargo update` | Plan ahead; `#[deprecated]`; `cargo-semver-checks` |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p api_design --example naming_conventions
cargo run -p api_design --example conversions
cargo run -p api_design --example common_traits
cargo run -p api_design --example builder_and_newtype
cargo run -p api_design --example typestate
cargo run -p api_design --example sealed_and_non_exhaustive
cargo run -p api_design --example error_design
cargo run -p api_design --example generics_and_iterators
cargo run -p api_design --example semver_evolution
```

Each example is a separate crate, so `sealed_and_non_exhaustive` uses the library exactly as a stranger would. Uncomment the failing lines in the comments to see the errors.

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_missing_traits` | Make it compile (E0277): the common traits and `Display` | `cargo run -p api_design --features exercises --bin ex01_fix_missing_traits` |
| 2 | `ex02_fix_non_exhaustive_match` | Make it compile (E0004): matches from another crate | `cargo run -p api_design --features exercises --bin ex02_fix_non_exhaustive_match` |
| 3 | `ex03_fix_typestate_order` | Make it compile (E0599): the builder's states | `cargo run -p api_design --features exercises --bin ex03_fix_typestate_order` |
| 4 | `ex04_newtype_conversions` | A validated `Email` newtype with `TryFrom`, `FromStr`, `AsRef`, `Display`, `From`, pass 4 tests | `cargo test -p api_design --features exercises --bin ex04_newtype_conversions` |
| 5 | `ex05_builder_with_errors` | A consuming builder with a `thiserror` enum, pass 4 tests | `cargo test -p api_design --features exercises --bin ex05_builder_with_errors` |
| 6 | `ex06_iterators_and_generics` | `impl Into`/`AsRef` arguments, iterator returns, `IntoIterator`, `FromIterator`, pass 4 tests | `cargo test -p api_design --features exercises --bin ex06_iterators_and_generics` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p api_design --features solutions --bins`).

**4. The mini project:** read [`src/lib.rs`](src/lib.rs) from the top, run the commands above, then extend it, without breaking any existing test (that's the point):
- Add `Method::Patch`. Which code in *this* crate needs changing, and why doesn't any user's code?
- Add a `Request::header(&self, name)` convenience getter, with a doc test.
- Add a `query` part to `Url` (`url.query() -> Option<&str>`) without changing what `path()` returns for existing URLs.

## Checklist

- [ ] Naming: `as_`/`to_`/`into_`, getters without `get_`, `iter`/`iter_mut`/`into_iter`, constructors, case
- [ ] Conversions: `From` (not `Into`), `TryFrom`, `FromStr`, `AsRef`, `?` and `From`
- [ ] Common traits: `Debug`, `Clone`, `PartialEq`, `Ord`, `Hash`, `Default`, `Display`, and `Send`/`Sync` checks
- [ ] Builders: consuming and non-consuming, `#[must_use]`, validation in `build`
- [ ] Newtypes: type safety, validation, hiding the implementation
- [ ] Typestate with marker types and `PhantomData`
- [ ] Sealed traits
- [ ] `#[non_exhaustive]` on enums, structs and variants, from another crate
- [ ] Error design: `thiserror` enums, data in variants, `source()`, message style, `Send + Sync`
- [ ] Accepting `impl AsRef<Path>`, `impl Into<String>`, `impl IntoIterator`; the inner-function trick
- [ ] Returning iterators: `impl Iterator + '_`, named iterator types, `IntoIterator for &T`, `Box<dyn Iterator>`
- [ ] Documentation: `missing_docs`, `# Errors`/`# Panics`, doc tests with `?`, `compile_fail`
- [ ] Semver: what breaks, what doesn't, `#[deprecated]`, `cargo-semver-checks`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- Names carry meaning: `as_` is free, `to_` makes new data, `into_` consumes; getters have no `get_`; collections have `iter`, `iter_mut` and `into_iter`.
- Implement the standard conversions and common traits on every public type, because users can't add them later.
- Builders name the options, newtypes make invalid values unrepresentable, typestate makes wrong orders uncompilable, and sealed traits keep a trait yours to extend.
- A library returns one error enum per area, with data in the variants and the cause in `source()`, and it's `Send + Sync`.
- Accept flexible arguments, return iterators, document everything with examples that are tested, and plan every public item so the next version can grow without breaking anyone.

**In one sentence for a child:** a good library is like a well-made machine for strangers: the buttons have the usual labels, any sensible plug fits, it won't let you press things in the wrong order, it tells you clearly what's wrong, and the next model keeps the buttons in the same place.

**Next:** [05 Advanced testing](../05_advanced_testing/README.md): integration tests, mocking with traits and fuzzing.
