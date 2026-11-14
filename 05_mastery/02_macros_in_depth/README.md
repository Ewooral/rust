<!-- hand-written lesson: gen_course must not overwrite -->
# Macros in Depth

**Official docs:** The Rust Reference: [Macros by example](https://doc.rust-lang.org/reference/macros-by-example.html), [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html), [Hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene)
· std: [`proc_macro`](https://doc.rust-lang.org/proc_macro/)
· [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
· crates: [`syn`](https://docs.rs/syn), [`quote`](https://docs.rs/quote), [`proc-macro2`](https://docs.rs/proc-macro2), [`trybuild`](https://docs.rs/trybuild)
· David Tolnay's [proc-macro workshop](https://github.com/dtolnay/proc-macro-workshop) (the classic next step)

Run it: `cargo run -p macros_in_depth -- --help`  ·  Test it: `cargo test -p macros_in_depth -p macros_in_depth_derive`

[Lesson 04/03](../../04_systems_level/03_advanced_features/README.md#5-macros) introduced both kinds of macro: `macro_rules!` with repetition, and three procedural macros written by hand with no dependencies. This lesson goes all the way. After it you will be able to:

- write `macro_rules!` macros that use every fragment specifier, nested repetitions, recursion and internal rules, and know exactly what hygiene and `$crate` do for you;
- debug a macro: see what it captured and what it expanded to, and replace confusing errors with clear ones;
- build all three kinds of procedural macro with `syn` and `quote`, with errors that point at the exact token that's wrong;
- test macros three ways: unit tests on the expansion function, tests that use the macro, and compile-fail tests with `trybuild`.

> 💡 **The big picture: a print shop.** A function is a worker who handles *values* while the program runs. A macro works earlier, on the *text of the program* (its tokens), before it's compiled. `macro_rules!` is a **rubber stamp with blanks**: it checks the shape of what you hand it and stamps out code with the blanks filled in. A procedural macro is a **robot typesetter**: a real program that reads your code as a tree (`syn`), sets new code (`quote!`), and can refuse a job with a sticky note on the exact word that's wrong (a spanned error).

## Part 1: `macro_rules!`

### 1. How a macro matches: rules and fragment specifiers

A `macro_rules!` macro is a list of **rules**, `(pattern) => { expansion };`. The compiler tries them **top to bottom** and uses the first whose pattern matches. A pattern is literal tokens plus **metavariables** like `$name:kind`, where the kind (the *fragment specifier*) says what may go there:

| Fragment | Matches | Fragment | Matches |
|---|---|---|---|
| `expr` | an expression: `2 + 3 * 4` | `ident` | a name: `user_name` |
| `ty` | a type: `Vec<Option<u8>>` | `literal` | `"hello"`, `42`, `-1`, `true` |
| `pat` | a pattern, `\|` included: `Some(1..=9) \| None` | `block` | `{ 6 * 7 }` |
| `path` | `std::collections::HashMap` | `item` | a whole `fn`, `struct`, `impl` … |
| `lifetime` | `'a` | `meta` | an attribute's inside: `derive(Debug)` |
| `vis` | `pub`, `pub(crate)`, or nothing | `tt` | any **one token tree**: a token, or a whole `(…)`, `[…]`, `{…}` group |

- `$($t:tt)*` matches *anything*, so it belongs in the **last** rule.
- In edition 2024, `expr` also matches `const { … }` blocks and `_`; `expr_2021` keeps the older meaning.
- **What may follow a fragment is limited**, so that future Rust syntax can't change what an old macro matches. After `expr` only `=>`, `,` or `;` may come. The real error (no error code):

```text
error: `$e:expr` is followed by `foo`, which is not allowed for `expr` fragments
  = note: allowed there are: `=>`, `,` or `;`
```

> 💡 **Analogy: a stamp with labelled blanks.** Each blank says what fits: "a name goes here", "a sum goes here", "any one block goes here". The shop tries its stamps in order and uses the first one whose blanks fit your page. `tt` is the blank that takes **anything**, so it's the last stamp in the drawer.

▶ `cargo run -p macros_in_depth --example fragment_specifiers`

### 2. Repetitions

`$( … ) sep rep` repeats a part of the pattern: `rep` is `*` (zero or more), `+` (one or more) or `?` (zero or one), and the optional `sep` is one token, usually `,` or `;`. In the expansion, `$( … )*` repeats once per match.

```rust
macro_rules! my_vec {
    () => { Vec::new() };
    ($($item:expr),+ $(,)?) => {{          // `$(,)?` allows a trailing comma
        let mut v = Vec::new();
        $( v.push($item); )*               // one push per item
        v
    }};
}
```

- Metavariables in **one** repetition repeat **in step**: `$($field:ident : $ty:ty),*` gives you pairs, and the example generates a struct and one getter per pair with them.
- Repetitions **nest**: `$($team:ident: [$($member:literal),*]);*`. Each `$(…)*` in the expansion must sit at the same depth as in the pattern.
- The empty case gets its **own rule**. Without it, `my_vec![]` expands to `let mut v = Vec::new(); v`, and the compiler warns "variable does not need to be mutable", pointing at the *caller's* `my_vec![]`. Warnings in expanded code land on your users.

Mistakes with repetitions (all real, none has an error code):

| You wrote | Error |
|---|---|
| `$a` outside its `$(…)*` | ``variable `a` is still repeating at this depth`` |
| `$( let $a = $b; )*` with 2 `a`s and 3 `b`s | ``meta-variable `a` repeats 2 times, but `b` repeats 3 times`` |
| `$($a:ident)* $b:ident` | ``local ambiguity when calling macro `amb`: multiple parsing options: built-in NTs ident ('a') or ident ('b').`` |

> 💡 **Analogy: a mail merge.** The pattern is a form with a list of rows; the expansion is the letter, printed once per row. Two columns in the same row (a name and an address) always go together. The macro won't guess what to do if the name list is longer than the address list.

▶ `cargo run -p macros_in_depth --example repetitions`

### 3. Recursion, internal rules and tt munchers

A macro may call itself. Put the **base case first**, since rules are tried in order:

```rust
macro_rules! max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let first = $x;                    // evaluate each argument ONCE
        let rest = max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}
```

Without the `let`s, `$x` is pasted twice and **runs** twice. The example counts: `max!(next(1), next(5), next(2))` calls `next` 3 times, the careless version 7 times.

A **tt muncher** reads its input one token tree at a time: each rule matches the first token, does something, and calls the macro again with the rest. The example's `rpn!(3 4 + 2 *)` is a reverse-Polish calculator that keeps a stack in `@stack [...]` **internal rules** (the `@` is just a convention meaning "helper, don't call this yourself").

Two things the calculator taught us the hard way:

- **A fragment that starts parsing doesn't back off.** With the number rule (`$n:literal`) above the `-` rule, `rpn!(10 2 3 * -)` failed: `-` *can* start a negative literal, so the literal parser took it and then hit the end of the input: ``error: unexpected token: `<eof>` … while parsing argument for this `literal` macro fragment``. Put rules for specific tokens before rules with fragments that could swallow them.
- **Every step is one level of recursion**, and there's a limit: ``error: recursion limit reached while expanding `forever!` `` (no code), with the help `consider increasing the recursion limit by adding a #![recursion_limit = "256"] attribute to your crate`.

> 💡 **Analogy: Russian dolls, and a hungry caterpillar.** Recursion opens one doll and hands the rest to itself until the smallest doll (the base case) is left. A tt muncher is a caterpillar eating a leaf one bite at a time, deciding after each bite what to do next. If it bites into something that only *looks* like food, it can't spit it out again.

▶ `cargo run -p macros_in_depth --example recursion_and_internal_rules`

### 4. Hygiene: which names a macro can see

`macro_rules!` macros are **hygienic** for local variables, labels and `$crate`: a name written *in the macro* is looked up where the macro was **defined**; a name passed *in by the caller* keeps the caller's meaning.

- A `let value` inside the macro is a different variable from the caller's `value`, so they can't clash.
- The other way round, a macro defined at the top level can't see a caller's locals. Exercise 1's `check_range!` uses `min` and `max` from `main`:

```text
error[E0425]: cannot find value `min` in this scope
  --> 05_mastery\02_macros_in_depth\exercises\ex01_fix_hygiene.rs:22:21
   |
20 | macro_rules! check_range {
21 |     ($value:expr) => {
22 |         if $value < min || $value > max {
   |                     ^^^ not found in this scope
...
37 |     check_range!(speed);
   |     ------------------- in this macro invocation
   |
help: an identifier with the same name exists, but is not accessible due to macro hygiene
  --> 05_mastery\02_macros_in_depth\exercises\ex01_fix_hygiene.rs:35:10
   |
35 |     let (min, max) = (1, 10);
   |          ^^^
```

  The fix is to pass them in (`$min:expr`). Ignore the compiler's other suggestion, `use std::cmp::min;`: that's a function, not your variable.
- **Items are not hygienic.** A `fn` or `struct` a macro defines is visible to the caller (`make_greeter!(hello, "Hello")` defines a `fn hello` you can call). Paths such as `helpers::double` are resolved where the macro is **used** (next section).

> 💡 **Analogy: every macro brings its own labelled lunchbox.** If you and the macro both pack a sandwich called `value`, nobody eats the wrong one. But the macro can't reach into *your* lunchbox either: if it needs your sandwich, you have to hand it over (pass it as an argument).

▶ `cargo run -p macros_in_depth --example hygiene_and_crate_paths`

### 5. `$crate`, scope and exporting

| Where the macro should work | How |
|---|---|
| later in the same module | nothing: a `macro_rules!` macro exists from its definition to the end of the module (**textual scope**). Above it: ``error: cannot find macro `early` in this scope`` (no code) |
| in modules declared after this one | `#[macro_use]` on the `mod` that defines it |
| anywhere in the crate, imported by path | `pub(crate) use my_macro;` after the definition, then `use crate::module::my_macro;` |
| in other crates | `#[macro_export]`: the macro moves to the **crate root**, so users write `my_crate::my_macro!` or `use my_crate::my_macro;` |

Paths inside a macro are resolved where it's used. **`$crate`** always means "the crate that defined this macro", from anywhere. Exercise 2's macro calls `units::celsius_to_fahrenheit`, which works at the crate root but not inside `report`:

```text
error[E0433]: cannot find module or crate `units` in this scope
  --> 05_mastery\02_macros_in_depth\exercises\ex02_fix_crate_path.rs:23:13
   |
21 |     macro_rules! fahrenheit {
22 |         ($celsius:expr) => {
23 |             units::celsius_to_fahrenheit($celsius)
   |             ^^^^^ use of unresolved module or unlinked crate `units`
...
34 |         format!("{celsius}°C = {}°F", fahrenheit!(celsius))
   |                                       -------------------- in this macro invocation
```

`$crate::units::celsius_to_fahrenheit($celsius)` fixes it for every caller. For an exported macro it's the only way to reach your own helpers from a user's crate. This lesson's library does exactly that: `settings_map!` calls `$crate::__private::to_value`, a function in a `#[doc(hidden)] pub mod __private` that exists only for macro-generated code (serde has the same kind of module). Std items are written in full, `::std::collections::BTreeMap`, so the expansion doesn't depend on what the user imported.

> 💡 **Analogy: a return address.** A stamp that says "fetch the ink from the shelf" works only in the room where the shelf is. `$crate` writes the shop's **full address** on the stamp: "fetch the ink from Rust Street 5, top shelf", which works from any room in town.

### 6. Debugging macros

| Tool | Gives you |
|---|---|
| **`cargo expand`** (`cargo install cargo-expand`) | the whole crate, or one target, after every macro has expanded: `cargo expand -p macros_in_depth --example debugging_macros`. It needs a nightly toolchain installed next to stable (it uses the compiler's unstable `-Zunpretty=expanded` for you); this course doesn't install it. |
| rust-analyzer: "Expand macro recursively" | the same for one macro call, in your editor |
| `stringify!(…)` | the tokens a metavariable captured, as a string: print them |
| `compile_error!("…")` in a last catch-all rule | your own message instead of "no rules expected …" |
| `trace_macros!`, `log_syntax!` | nightly only: on stable they're `E0658: use of unstable library feature` |

The example's `percent!` shows the difference a catch-all rule makes. Without one, `two_args!(1)` says `unexpected end of macro invocation` (note: `while trying to match ,`) and `two_args!(1, 2, 3)` says ``no rules expected `,` ``. With one, `percent!(fifteen)` says:

```text
error: percent! expects a number followed by %, like `percent!(15 %)`, not `fifteen`
```

> 💡 **Analogy: a proof sheet.** Before printing a thousand copies, the print shop prints **one proof** to check (`cargo expand`, `stringify!`). And a good stamp has a last slot that catches every page that doesn't fit and returns it with a note saying *why*, instead of jamming the machine.

▶ `cargo run -p macros_in_depth --example debugging_macros`

## Part 2: procedural macros with `syn` and `quote`

### 7. The pieces

A procedural macro is a Rust function that runs **inside the compiler**: tokens in, tokens out. It must live in its own crate of type `proc-macro`. This lesson's is [`macros/`](macros/Cargo.toml), package `macros_in_depth_derive`:

```toml
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0.107"
quote = "1.0.47"
syn = { version = "3.0.10", features = ["full"] }   # "full": function bodies too, for #[memoize]
```

| Crate | Job |
|---|---|
| `proc_macro` (built in) | the compiler's `TokenStream`, only usable inside a proc macro |
| `proc-macro2` | the same types, usable anywhere, so the logic can be **unit-tested** |
| `syn` | parse tokens into a syntax tree: `DeriveInput`, `ItemFn`, `Type`, or your own grammar |
| `quote` | `quote! { … }` writes code: `#var` pastes one value, `#(#items),*` repeats like `$(…),*` |

Every macro in this lesson has the same three steps: **parse** with `syn::parse_macro_input!`, **expand** in a plain function from the tree to a `proc_macro2::TokenStream` returning `syn::Result`, and **report** (an `Err` becomes a compile error, below). The library crate [`src/lib.rs`](src/lib.rs) re-exports the macros next to the trait and types their output uses, so users depend on one crate.

The `syn_and_quote` example runs all three outside the compiler: it prints a struct's token trees, parses them into a `DeriveInput`, generates getters with `quote!` and `format_ident!`, and shows an error as `:: core :: compile_error ! { "expected a struct" }`.

> 💡 **Analogy: the robot typesetter's tools.** `proc-macro2` is the box of letters, `syn` is the robot's **eyes** that read a page as sentences and words instead of loose letters, and `quote!` is its **printing press**: you lay out the page with blanks, and it fills them in.

▶ `cargo run -p macros_in_depth --example syn_and_quote`

### 8. A derive macro: `#[derive(Settings)]`

[`macros/src/settings.rs`](macros/src/settings.rs) reads a struct and writes `impl Settings`, so it can be loaded from `key = value` text:

```rust
#[derive(Settings)]
struct EditorSettings {
    /// Spaces per tab.                         <- becomes the help text
    #[setting(default = "4")]                   <- a helper attribute
    tab_width: u8,                              <- read with FromStr
    #[setting(rename = "font-size")]
    font_size: Option<f32>,                     <- Option: may be missing
}
```

- `#[proc_macro_derive(Settings, attributes(setting))]` declares the **helper attribute**. Without that, `#[setting(…)]` would be an unknown attribute.
- `syn::DeriveInput` gives the name, generics and `Data::Struct` / `Enum` / `Union`. `attr.parse_nested_meta(|meta| …)` walks `#[setting(default = "4", rename = "…")]` one `name = value` at a time.
- A `/// comment` reaches the macro as `#[doc = " comment"]`, so a derive can read documentation.
- A derive only **adds** code next to the struct; it can't change it.
- The output names everything by absolute path (`::core::result::Result`, `::macros_in_depth::FieldInfo`): procedural macros have **no `$crate`**. Inside `macros_in_depth` itself that path wouldn't resolve, so `src/lib.rs` says `extern crate self as macros_in_depth;`, the usual work-around.
- A derive macro and a trait may share a name (`Settings`), because macros and types live in different namespaces. `use macros_in_depth::Settings;` imports both, as with serde's `Serialize`.

> 💡 **Analogy: a robot that reads the blueprint and bolts on a part.** It reads every field's label (name, type, sticky notes like `default`), then builds a matching **loader** and fixes it to the side of your struct. Your struct stays exactly as you drew it.

▶ `cargo run -p macros_in_depth --example derive_settings`

### 9. Good error messages: `syn::Error`, spans and `quote_spanned!`

A proc macro should never panic: the user would see "proc macro panicked" and a message with no location. Return a **`syn::Error`** with a **span** (a location in the user's code) instead:

| Make one with | Points at |
|---|---|
| `Error::new(span, "message")` | any span, e.g. `data.enum_token.span` |
| `Error::new_spanned(&tokens, "message")` | all of `tokens`: a type, the fields, a literal |
| `meta.error("message")` | the name inside `#[setting(…)]` being parsed |
| `error.combine(other)` | report several problems **in one build** |

`error.to_compile_error()` turns the error into `compile_error!` calls carrying those spans (`into_compile_error()` is the same, by value). The derive uses the long form:

```rust
match settings::expand(&input) {
    Ok(tokens) => tokens.into(),
    Err(error) => error.to_compile_error().into(),
}
```

The result is an error exactly where the problem is. These come from the lesson's `trybuild` snapshots (`tests/ui/*.stderr`):

```text
error: unknown setting option: expected `default` or `rename`
 --> tests/ui/settings_bad_options.rs:5:15
  |
5 |     #[setting(defualt = "8080")]
  |               ^^^^^^^

error: write the default as it would appear in the settings file, in quotes: `default = "8080"`
 --> tests/ui/settings_bad_options.rs:7:25
  |
7 |     #[setting(default = 4)]
  |                         ^
```

**Errors the compiler finds in your generated code** need care too. The derive calls `__private::required::<T>()`, which needs `T: FromStr`. A first draft built that call with `quote!`, so it carried the macro's own span, and a field type without `FromStr` was blamed on the derive:

```text
error[E0277]: the trait bound `Colour: FromStr` is not satisfied
   |
65 | #[derive(Settings)] struct T3 { colour: Colour }
   |          ^^^^^^^^ unsatisfied trait bound
```

`quote_spanned! {ty.span()=> … }` gives the generated tokens the **field type's** span instead (the function's name especially, because that's what the compiler blames). Now exercise 3's error points at the field:

```text
error[E0277]: the trait bound `LogLevel: FromStr` is not satisfied
  --> 05_mastery\02_macros_in_depth\exercises\ex03_fix_from_str.rs:30:12
   |
30 |     level: LogLevel,
   |            ^^^^^^^^ unsatisfied trait bound
   |
help: the trait `FromStr` is not implemented for `LogLevel`
  --> 05_mastery\02_macros_in_depth\exercises\ex03_fix_from_str.rs:20:1
   |
20 | enum LogLevel {
   | ^^^^^^^^^^^^^
...
note: required by a bound in `macros_in_depth::__private::required`
```

> 💡 **Analogy: a sticky note on the exact word.** A teacher who writes "something's wrong" at the top of your essay is no help. A good one puts a sticky note **on the misspelt word**. A span is that sticky note, and `combine` is putting all the notes on in one go instead of handing the essay back once per mistake.

### 10. An attribute macro: `#[memoize]`

An attribute macro gets **two** token streams, the attribute's arguments and the item it's on, and its output **replaces** the item. [`macros/src/memoize.rs`](macros/src/memoize.rs) wraps a function's body in a per-thread cache:

```rust
#[memoize]
fn fibonacci(n: u64) -> u128 {
    if n < 2 { return n.into(); }
    fibonacci(n - 1) + fibonacci(n - 2)     // each n is computed once: fibonacci(150) in under a millisecond here
}

#[memoize(capacity = 2)]                    // arguments, parsed with syn::meta::parser
fn repeat(word: String, times: usize) -> String { word.repeat(times) }
```

- It parses the function as a `syn::ItemFn`, keeps the attributes and signature, and replaces only `block`: a `thread_local!` `HashMap` keyed by a tuple of the arguments, then the original body inside `(move || -> T { … })()`, so its `return`s and `?`s still work.
- It refuses what it can't cache, at the right token: an `async fn` (at `async`), a generic function, a method (`self`), a function returning nothing, `impl Trait` arguments, and borrowed arguments:

```text
error: #[memoize] keeps arguments in its cache, so they must be owned: use `String` instead of `&str`, `Vec<T>` instead of `&[T]`
 --> tests/ui/memoize_borrowed_argument.rs:4:21
  |
4 | fn word_count(text: &str) -> usize {
  |                     ^^^^
```

- **Proc macros are not hygienic** by default: every name they generate has the call site's span (`Span::call_site()`), so it could clash with the user's names. `Span::mixed_site()` gives a generated local variable `macro_rules!`-style hygiene; `#[memoize]` uses it for `key`, `hit`, `result` and `cache`.
- One check per argument type, spanned to that type, makes "the trait bound `Shape: Hash` is not satisfied" point at `Shape` in the signature (with two follow-on E0599 errors from inside the generated code, which disappear with the first).

> 💡 **Analogy: a robot that rebuilds the machine.** A derive robot only bolts a part on the side. The attribute robot takes your machine, keeps its outside exactly the same, and fits a **memory box** inside: "if I've made this before, hand over the copy". If you give it a machine it can't fit the box into, it hands it back with a note on the part that's in the way.

▶ `cargo run -p macros_in_depth --example attribute_memoize`

### 11. A function-like macro with its own syntax: `palette!`

A function-like macro accepts **any** tokens, so it defines its own grammar by implementing `syn::parse::Parse` ([`macros/src/palette.rs`](macros/src/palette.rs)):

```rust
impl Parse for Palette {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;              // `pub`, or nothing
        input.parse::<Token![mod]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);                          // the `{ … }`
        let entries = Punctuated::parse_terminated(&content)?;   // `a = "#…", b = "#…",`
        Ok(Palette { vis, name, entries })
    }
}
```

Each `input.parse()?` either takes the next piece or fails with an error on the token it didn't expect, so `palette! { mod bad { sky: "#fff" } }` gets ``error: expected `=` `` under the `:` for free. Then the macro does what `macro_rules!` can't: it **looks inside the string literals**, checks every colour and refuses bad ones, all in one build:

```text
error: expected a colour like "#ff8800" or "#f80"
 --> tests/ui/palette_bad_colours.rs:5:15
  |
5 |         sky = "light blue",
  |               ^^^^^^^^^^^^

error: expected a colour like "#ff8800" or "#f80"
 --> tests/ui/palette_bad_colours.rs:6:15
  |
6 |         sea = "#12",
  |               ^^^^^

error: `sky` is in this palette twice
 --> tests/ui/palette_bad_colours.rs:7:9
  |
7 |         sky = "#000",
  |         ^^^
```

The output is a module of `const`s (`sunset::SKY: Rgb`), so the colours are worked out at compile time.

> 💡 **Analogy: an order form in your own words.** A `macro_rules!` stamp only knows shapes. The function-like robot **reads your whole order**, written in a little language you invented ("sky = this colour"), checks every line, and circles every line it can't make before it starts.

▶ `cargo run -p macros_in_depth --example function_like_palette`

### 12. Testing macros

| Level | What | Here |
|---|---|---|
| **Unit tests of the expansion** | call `expand(…)` with `syn::parse_quote!` input, check the tokens or the error message | 12 tests in `macros/src/*.rs`: `cargo test -p macros_in_depth_derive` |
| **Tests that use the macro** | real code with the macros, as a user's crate would write it | `tests/as_a_user.rs`, and the library's unit tests |
| **Compile-fail tests** | `trybuild` compiles each `tests/ui/*.rs` and compares the errors with its `.stderr` file | `tests/compile_fail.rs`: 4 cases |
| **Doc tests** | ` ```compile_fail ` blocks must fail to compile | `settings_map!` with a non-literal key |

Unit tests work because the expansion takes `proc_macro2` types, which exist outside the compiler. `proc_macro::TokenStream` itself can only be used inside a running macro.

`trybuild` builds each case as a small crate of its own (under `target/tests/trybuild/`, about 60 MB here), so the first run takes a while. After changing a message on purpose, run once with the environment variable `TRYBUILD=overwrite` to rewrite the `.stderr` files, then **read the diff** before committing. Test your **own** messages this way; the compiler's own wording (E0277 and friends) can change between Rust versions.

> 💡 **Analogy: testing the robot.** Watch it work on single pages at its desk (unit tests), let it print a real book (tests that use it), and feed it **deliberately broken pages** to check it writes the right sticky note on the right word (compile-fail tests).

## The mini project: settings from macros

`src/` is a small library for typed settings where macros write most of the code, and `src/main.rs` uses it to check a pretend web server's settings:

| Piece | Kind | File |
|---|---|---|
| `#[derive(Settings)]` | derive | [`macros/src/settings.rs`](macros/src/settings.rs) → trait in [`src/settings.rs`](src/settings.rs) |
| `#[memoize]` | attribute | [`macros/src/memoize.rs`](macros/src/memoize.rs) |
| `palette!` | function-like | [`macros/src/palette.rs`](macros/src/palette.rs) → `Rgb` in [`src/color.rs`](src/color.rs) |
| `settings_map!`, `ensure!` | `macro_rules!`, exported | [`src/macros.rs`](src/macros.rs) |
| `__private` | helpers for generated code | [`src/lib.rs`](src/lib.rs) |

```bash
cargo run -p macros_in_depth -- --help
cargo run -p macros_in_depth -- name=demo port=9000
cargo run -p macros_in_depth -- --file 05_mastery/02_macros_in_depth/server.conf
cargo run -p macros_in_depth -- --file 05_mastery/02_macros_in_depth/server.conf port=80
cargo test -p macros_in_depth -p macros_in_depth_derive   # 23 unit, 3 integration, 4 compile-fail cases, 4 doc tests
```

```text
Server settings (key = value):
  name             Name shown in every response. [required]
                   (String)
  host             Host name or IP address to listen on. [default: 127.0.0.1]
                   (String)
  port             Port to listen on. [default: 8080]
                   (u16)
  max-connections  Most connections served at the same time. [default: 100]
                   (usize)
  log-file         Where to write the log; the terminal if not set.
                   (Option<String>)
palette: heading #5fafff, good #5fd75f, bad #ff5f5f, dim #8a8a8a
```

Every line of that help screen comes from the struct's doc comments and attributes. Mistakes get one clear line each (`error: port 80 needs administrator rights: use 1024 or above`, ``unknown key `log_file` (did you mean `log-file`?)``), coloured with the `palette!` colours when the output is a terminal.

> 💡 **Analogy: a form that fills itself in.** You draw the form once (the struct) and write a note on each box (doc comments, defaults). The robots build everything else: the reader that fills the form from a settings file, the help page that explains every box, the colours on the page, and the polite note when someone writes in the wrong box.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| A macro used above its `macro_rules!` | ``cannot find macro `x` in this scope`` (no code) | Define it first, or `#[macro_use]` / `pub(crate) use` |
| A caller's variable inside the macro body | E0425 cannot find value (… not accessible due to macro hygiene) | Pass it in as `$x:ident` or `$x:expr` |
| `module::helper()` in a macro used elsewhere | E0433 cannot find module or crate | `$crate::module::helper()` |
| `$e:expr foo` in a pattern | `` `$e:expr` is followed by `foo` `` (no code) | Only `=>`, `,`, `;` may follow `expr` |
| `$a` outside its repetition | ``variable `a` is still repeating at this depth`` | Wrap it in `$( … )*` |
| `$x` pasted twice | compiles, but runs twice | `let x = $x;` once |
| A fragment rule above a rule for a specific token | `unexpected token … while parsing argument for this … fragment` | Specific tokens first |
| A catch-all `$($t:tt)*` rule first | every call matches it | Put it last |
| `trace_macros!` on stable | E0658 use of unstable library feature | `cargo expand`, `stringify!` |
| `panic!` in a proc macro | "proc macro panicked", no useful location | Return a `syn::Error` with a span |
| Output built with `quote!` only | type errors point at the derive, not the field | `quote_spanned!(ty.span()=> …)` |
| `$crate` in a proc macro | doesn't exist there | An absolute path, plus `extern crate self as name;` |
| `#[setting]` without `attributes(setting)` in the derive | the attribute is unknown | Declare helper attributes in `#[proc_macro_derive]` |
| Generated `let` names in a proc macro | can clash with the user's names | `Span::mixed_site()`, or unusual names |
| `x.clone()` in generated code | may call a `clone` method from one of the user's own traits | `::core::clone::Clone::clone(&x)` |
| `TRYBUILD=overwrite` left on | wrong snapshots get accepted | Run it once, then read the diff |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p macros_in_depth --example fragment_specifiers
cargo run -p macros_in_depth --example repetitions
cargo run -p macros_in_depth --example recursion_and_internal_rules
cargo run -p macros_in_depth --example hygiene_and_crate_paths
cargo run -p macros_in_depth --example debugging_macros
cargo run -p macros_in_depth --example syn_and_quote
cargo run -p macros_in_depth --example derive_settings
cargo run -p macros_in_depth --example attribute_memoize
cargo run -p macros_in_depth --example function_like_palette
```

The comments in each example show what goes wrong, and with which error. Try a few: break a rule on purpose and read the message.

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_hygiene` | Make it compile (E0425): pass the names in | `cargo run -p macros_in_depth --features exercises --bin ex01_fix_hygiene` |
| 2 | `ex02_fix_crate_path` | Make it compile (E0433) with `$crate` | `cargo run -p macros_in_depth --features exercises --bin ex02_fix_crate_path` |
| 3 | `ex03_fix_from_str` | Make it compile (E0277): `FromStr` for a derived field | `cargo run -p macros_in_depth --features exercises --bin ex03_fix_from_str` |
| 4 | `ex04_recursive_macros` | `count!`, `max!`, `hash_map!`, pass 3 tests | `cargo test -p macros_in_depth --features exercises --bin ex04_recursive_macros` |
| 5 | `ex05_derive_logic` | A derive's logic with `syn` and `quote`, pass 4 tests | `cargo test -p macros_in_depth --features exercises --bin ex05_derive_logic` |
| 6 | `ex06_parse_custom_syntax` | `impl Parse` for a routing DSL, pass 3 tests | `cargo test -p macros_in_depth --features exercises --bin ex06_parse_custom_syntax` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p macros_in_depth --features solutions --bins`).

**4. The mini project:** run the commands above, then extend it:
- Add a `#[setting(env = "PORT")]` option to the derive: read the environment variable when the key is missing. Add a `tests/ui` case for a misspelt option.
- Teach `palette!` a third colour form, `rgb(255, 136, 0)`, by extending `Entry`'s `Parse` impl.
- Make `#[memoize]` refuse functions that take `&mut` arguments with a message of your own, and add a unit test in `macros/src/memoize.rs`.

## Checklist

- [ ] `macro_rules!`: rules tried in order, every fragment specifier, what may follow a fragment, `expr_2021`
- [ ] Repetitions: `*`, `+`, `?`, separators, metavariables in step, nesting, the empty case
- [ ] Recursion, evaluating arguments once, `@internal` rules, tt munchers, the recursion limit
- [ ] Hygiene: locals and `$crate` are hygienic, items are not; names resolve at the definition site
- [ ] Scope and exporting: textual scope, `#[macro_use]`, `pub(crate) use`, `#[macro_export]`, `$crate`, `__private`
- [ ] Debugging: `cargo expand` (what it needs), `stringify!`, catch-all `compile_error!` rules
- [ ] Proc macro crates: `proc-macro = true`, `proc_macro` vs `proc_macro2`, `syn`, `quote`
- [ ] A derive macro with a helper attribute, doc comments, absolute paths and `extern crate self`
- [ ] Errors: `syn::Error`, `new_spanned`, `meta.error`, `combine`, `to_compile_error`, `quote_spanned!`
- [ ] An attribute macro: arguments, `ItemFn`, replacing the body, `Span::mixed_site()`
- [ ] A function-like macro with a `Parse` impl: `braced!`, `Punctuated`, `Token![…]`
- [ ] Testing macros: unit tests on `proc_macro2`, integration tests, `trybuild`, `compile_fail` doc tests
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `macro_rules!` matches tokens against rules in order; fragments say what fits each blank, repetitions stamp code once per match, and recursion with internal rules can parse little languages.
- Hygiene keeps a macro's local variables apart from yours and looks names up where the macro was defined; use `$crate` for paths, and `#[macro_export]` plus a hidden `__private` module for macros other crates use.
- A procedural macro is a program in its own crate: `syn` parses the input, a plain `expand` function builds the output with `quote!`, and problems come back as `syn::Error`s with spans.
- Derive macros add an impl, attribute macros replace the item they're on, function-like macros parse any syntax you design.
- Test the expansion function directly, test the macro in use, and pin your error messages with `trybuild`.

**In one sentence for a child:** a macro is a helper that writes code for you before the program is built, either a stamp that fills in blanks or a robot that reads your code and builds new parts, and a good one puts a sticky note on the exact word you got wrong.

**Next:** [03 Performance](../03_performance/README.md): profiling, allocation and benchmarks.
