<!-- hand-written lesson: gen_course must not overwrite -->
# Generics, Traits and Lifetimes

**The Rust Book, chapter 10:** [Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
· Rust by Example: [Generics](https://doc.rust-lang.org/rust-by-example/generics.html), [Traits](https://doc.rust-lang.org/rust-by-example/trait.html), [Lifetimes](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)
· Video: [Rust's most complicated features explained](https://www.youtube.com/watch?v=9RsgFFp67eo) (Let's Get Rusty)

This chapter gives you three tools for writing code **once** that works in many situations, without losing any speed or safety:

| Tool | Answers | Example |
|---|---|---|
| **Generics** | "Which **type**?" Any, decided by the caller. | `fn largest<T>(list: &[T]) -> &T` |
| **Traits** | "What can it **do**?" Shared behaviour. | `T: PartialOrd`, `impl Summary for NewsArticle` |
| **Lifetimes** | "How **long** is this reference valid?" | `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str` |

> 💡 **The big picture: a cookie cutter, a skills badge and a library card.** A **generic** is a cookie cutter: one shape that works on any dough (any type). A **trait** is a skills badge such as "can swim" or "can be compared". A function can say "I accept anyone with this badge" (a trait bound). A **lifetime** is the due date on a borrowed library book: the compiler checks that nobody keeps reading a book after it went back on the shelf.

Lessons 01/06 (ownership), 02/01 (structs) and 02/05 (errors) all pointed here. This is the chapter that ties them together.

## 1. Removing duplication

Before generics, the Book starts with plain functions. If you have the same loop twice, extract it into a function and make what differs a parameter. Generics do exactly the same thing, except the part that differs is a **type**.

> 💡 **Analogy:** instead of baking every cake from memory, you write the recipe down once. Generics go one step further: a recipe that says "use any fruit".

▶ `cargo run -p generics_traits_lifetimes --example removing_duplication`

## 2. Generic data types

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T { /* … */ }  // generic function

struct Point<T> { x: T, y: T }               // x and y: the same type
struct MixedPoint<T, U> { x: T, y: U }       // may differ

impl<T> Point<T> { fn x(&self) -> &T { &self.x } }            // for every Point
impl Point<f32> { fn distance_from_origin(&self) -> f32 { … } } // only Point<f32>

enum Option<T> { Some(T), None }             // you already know these two
enum Result<T, E> { Ok(T), Err(E) }
```

- `<T>` declares a type parameter. `impl<T>` declares it for the impl block.
- Without the `PartialOrd` bound, the compiler won't let you use `<` or `>`. The real error from exercise 1, which also shows the fix:

```text
error[E0369]: binary operation `<` cannot be applied to type `&T`
   |
12 |         if item < smallest {
   |            ---- ^ -------- &T
help: consider restricting type parameter `T` with trait `PartialOrd`
   |
 9 | fn smallest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
```

- **Monomorphization:** at compile time Rust writes one concrete copy of the code for each type you use (`largest::<i32>`, `largest::<char>`). So generics cost **nothing at run time**.

> 💡 **Analogy:** `Point<T>` is a pair of matching boxes: whatever you put in one, the other holds the same kind. `MixedPoint<T, U>` is two boxes that may hold different things. Monomorphization is a **photocopier** at the factory: before the program ships, it prints a separate, ready-made copy of the recipe for each fruit you actually use, so nobody has to look anything up while cooking.

▶ `cargo run -p generics_traits_lifetimes --example generic_data_types`

## 3. Traits: shared behaviour

```rust
pub trait Summary {
    fn summarize_author(&self) -> String;          // required
    fn summarize(&self) -> String {                // default, which may call required ones
        format!("(Read more from {}...)", self.summarize_author())
    }
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String { format!("@{}", self.username) }
}
```

**Traits as parameters.** All of these mean "anything that implements Summary":

```rust
fn notify(item: &impl Summary)                            // short form
fn notify<T: Summary>(item: &T)                           // trait bound
fn notify<T: Summary + Display>(item: &T)                 // several bounds
fn notify<T>(item: &T) where T: Summary + Display         // where clause, for readability
```

Use the `<T: …>` form when two parameters must be the **same** type: `fn pair<T: Summary>(a: &T, b: &T)`.

- **Returning `impl Summary`:** "some type that implements `Summary`". It must be **one** concrete type (section 6 shows why).
- **Conditional methods:** `impl<T: Display + PartialOrd> Pair<T> { fn cmp_display… }` adds the method only to pairs whose `T` qualifies.
- **Blanket implementations:** `impl<T: Display> ToString for T` is in std, which is why every displayable type has `.to_string()`.
- **The orphan rule:** you may write `impl Trait for Type` only if the trait or the type is defined in **your** crate (section 9).

> 💡 **Analogy: skills badges.** A trait is a badge ("can summarise"). Each type earns it its own way (`impl`). A default method is the standard procedure that comes with the badge: use it as-is or do it your own way. A trait bound is a job advert: "applicants must hold the driving badge". A blanket impl is a rule like "everyone with the reading badge automatically gets the library badge".

▶ `cargo run -p generics_traits_lifetimes --example traits`

## 4. Lifetimes: validating references

Every reference has a **lifetime**: the stretch of code where it's valid. Usually the compiler works it out by itself. You only write lifetimes when it **can't tell** how the references in a signature relate:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

This doesn't change how long anything lives. It's a **promise the compiler checks**: the returned reference is valid for as long as *both* inputs are, which means for the shorter of the two. Leave it out, and the compiler asks you to add it (exercise 2):

```text
error[E0106]: missing lifetime specifier
  |
9 | fn longer_name(a: &str, b: &str) -> &str {
  |                   ----     ----     ^ expected named lifetime parameter
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
```

The compiler then checks every caller. The real error from exercise 3:

```text
error[E0597]: `string2` does not live long enough
   |
18 |         let string2 = String::from("xyz");
   |             ------- binding `string2` declared here
19 |         result = longest(string1.as_str(), string2.as_str());
   |                                            ^^^^^^^ borrowed value does not live long enough
20 |     }
   |     - `string2` dropped here while still borrowed
21 |     println!("The longest string is {result}");
   |                                      ------ borrow later used here
```

> 💡 **Analogy: library due dates.** Each borrowed book (reference) has a due date (lifetime). `longest` hands you back one of two books but doesn't say which, so you must treat the answer as due back on the **earlier** of the two dates. Lifetime annotations are labels saying "these loans share a due date". The compiler is the librarian who won't let you read a book after it was returned.

**Lifetimes in structs.** A struct that holds a reference needs a lifetime, and it can't outlive the data it points into:

```rust
struct ImportantExcerpt<'a> { part: &'a str }
```

> 💡 **Analogy:** a bookmark. It's only meaningful while the book exists, so the bookmark can't be kept after the book is gone.

**Elision:** three rules fill in lifetimes for you, which is why `fn first_word(s: &str) -> &str` needs none:
1. Each reference parameter gets its own lifetime.
2. If there is exactly one input lifetime, every output gets it.
3. If one input is `&self` or `&mut self`, outputs get `self`'s lifetime.

If an output lifetime is still unknown after these rules, you get E0106.

> 💡 **Analogy:** the librarian fills in the obvious due dates for you. You only write one when it's genuinely ambiguous.

**`'static`** means valid for the whole program. String literals are `&'static str`, because they live in the binary. When the compiler suggests `'static`, first ask whether the data is really meant to live forever, or whether you're returning a reference to a local variable.

▶ `cargo run -p generics_traits_lifetimes --example lifetimes`

## Beyond the Book

These are the topics from the YouTube video and the professional list. Each has a runnable example.

### 5. Static vs dynamic dispatch, and `impl Trait`

| | **Generics / `impl Trait`** (static) | **`dyn Trait`** (dynamic) |
|---|---|---|
| Decided | at compile time | at run time |
| How it works | one copy per type (monomorphization) | one copy, plus a **vtable** lookup per call |
| Speed | direct calls, inlinable | one indirect call |
| Mixed types in one `Vec` | ✗ | ✓ `Vec<Box<dyn Shape>>` |
| Code size | grows with each type used | stays small |

- `dyn Trait` must sit behind a pointer: `&dyn`, `Box<dyn>` or `Rc<dyn>`. A `&dyn Shape` is a **fat pointer**, 16 bytes: a data pointer plus a vtable pointer.
- `impl Trait` in **argument** position is shorthand for a generic. In **return** position it's one hidden concrete type, so returning a `Circle` on one branch and a `Square` on another needs `Box<dyn Shape>`. Otherwise you get E0308: `if` and `else` have incompatible types.
- **Dyn compatibility** (once called "object safety"): a trait can be `dyn` only if a vtable can describe it. That rules out generic methods and methods returning `Self`, unless you mark them `where Self: Sized`.

> 💡 **Analogy:** static dispatch is **hiring a specialist** for each job when you plan the project. Dynamic dispatch is **phoning a switchboard** at the moment you need something: "whoever is on duty, please do this". The vtable is the switchboard's phone list. It's slightly slower, but one phone line serves every kind of worker.

▶ `cargo run -p generics_traits_lifetimes --example static_vs_dynamic_dispatch`

### 6. Conversion traits and common std traits

| Trait | Gives you |
|---|---|
| `From<T>` / `Into<T>` | infallible conversion. Implement `From` and get `Into` free. |
| `TryFrom<T>` / `TryInto<T>` | fallible conversion, returning `Result` |
| `AsRef<T>` | "can be cheaply viewed as `&T`": `fn f<S: AsRef<str>>(s: S)` takes `&str`, `String`, … |
| `Borrow<T>` | lets `HashMap<String, _>` be searched with a `&str` |
| `Display`, `Default`, `PartialOrd`/`Ord`, `Hash` | printing, default values, ordering, map keys |
| `Add`, `Mul`, `Neg`, `Index`, … | operators `+ * - []` for your own types |

> 💡 **Analogy:** `From` is a **currency exchange** that always gives you the new currency. `TryFrom` is an exchange that may **refuse** a damaged note. `AsRef` is a **window** you look through without taking anything out.

▶ `cargo run -p generics_traits_lifetimes --example std_traits`

### 7. Advanced lifetimes

- **`'a: 'b`** reads "'a outlives 'b". You need it when a value borrowed for `'b` comes from data that lives for `'a`.
- **`T: 'a`** reads "T holds no references shorter than 'a".
- **`'_`** is the anonymous lifetime: `fn words(text: &str) -> Words<'_>` shows that a borrow is there without naming it.
- **`T: 'static`** as a *bound* doesn't mean "lives forever". It means "holds no short borrows", which an owned `String` satisfies. Threads require it (lesson 04/01).
- **Higher-ranked trait bounds:** `F: for<'a> Fn(&'a str) -> &'a str` means "works for *every* lifetime the caller picks". It's needed when each call passes a different short-lived borrow.

> 💡 **Analogy:** `for<'a>` is a machine certified to "work with a book of **any** due date". A machine certified only for books due next Friday couldn't take each new loan.

▶ `cargo run -p generics_traits_lifetimes --example advanced_lifetimes`

### 8. Const generics and GATs

- **Const generics** take a *value* as a generic parameter: `struct Matrix<const N: usize>`. `Matrix<2> + Matrix<3>` is a compile-time error (E0308), not a runtime check.
- **GATs** (generic associated types) are associated types that take their own parameters: `type Item<'a> where Self: 'a;`. They make a **lending iterator** possible, one that hands out `&mut` borrows into *itself*, one at a time.

> 💡 **Analogy:** const generics are **egg boxes with the size printed on the box**: a 6-box and a 12-box are different products, and you can't pour one into the other. A GAT lending iterator is a library that lends **one book at a time**: give it back, then you get the next.

▶ `cargo run -p generics_traits_lifetimes --example const_generics_and_gats`

### 9. `PhantomData`, variance and the orphan rule

- **`PhantomData<T>`** marks a type parameter that no field uses. `Length<Metres>` vs `Length<Feet>` costs zero bytes but stops you adding metres to feet.
- **Variance** decides when one lifetime may stand in for another. `&'a T` is *covariant*: a longer-lived reference fits where a shorter one is wanted, which is why `&'static str` works anywhere. `&mut T` is *invariant*: no substitution at all. Otherwise you could write a short-lived reference into a slot that promises a long one, and the compiler stops that with "lifetime may not live long enough".
- **The orphan rule** (coherence): you may implement a trait for a type only if the trait or the type is local to your crate. `impl Display for Vec<String>` is E0117. The fix is a **newtype**: `struct List(Vec<String>)`.

> 💡 **Analogy:** `PhantomData` is a **label on a jar**, nothing inside but the label, yet it stops sugar going in the salt jar. Covariance means a ticket valid all year also works for today. Invariance is a **safe-deposit box with a strict size**: you can't swap in a smaller key and pretend it's the same. The orphan rule means you can make rules for your own house or about your own guests, but not rewrite someone else's rules for someone else's guests.

▶ `cargo run -p generics_traits_lifetimes --example variance_and_orphan_rule`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Used `<`, `+`, `{}` on a plain `T` | E0369 / E0277 | Add the trait bound: `T: PartialOrd`, `T: Add<Output = T>`, `T: Display`. |
| `-> &str` with two reference inputs | E0106 | `<'a>` on the inputs the result comes from. |
| Kept a reference past the owner's scope | E0597 | Make the owner live longer, or use the reference sooner. |
| Returned a reference to a local | E0515 / E0106 | Return the owned value. |
| `-> impl Trait` returning two different types | E0308 | `Box<dyn Trait>` |
| `Vec<T>` holding mixed types | E0308 | `Vec<Box<dyn Trait>>` |
| Called a trait method, "method not found" | E0599 | `use` the trait, or `impl` it for the type. |
| `impl ForeignTrait for ForeignType` | E0117 | Wrap it in a newtype. |
| `struct S { r: &str }` | E0106 | `struct S<'a> { r: &'a str }`, or own a `String`. |
| Sprinkled `'static` to silence errors | compiles, wrong | Find what really owns the data. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p generics_traits_lifetimes --example removing_duplication
cargo run -p generics_traits_lifetimes --example generic_data_types
cargo run -p generics_traits_lifetimes --example traits
cargo run -p generics_traits_lifetimes --example lifetimes
cargo run -p generics_traits_lifetimes --example static_vs_dynamic_dispatch
cargo run -p generics_traits_lifetimes --example std_traits
cargo run -p generics_traits_lifetimes --example advanced_lifetimes
cargo run -p generics_traits_lifetimes --example const_generics_and_gats
cargo run -p generics_traits_lifetimes --example variance_and_orphan_rule
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_missing_bound` | Make it compile (E0369) | `cargo run -p generics_traits_lifetimes --features exercises --bin ex01_fix_missing_bound` |
| 2 | `ex02_fix_missing_lifetime` | Make it compile (E0106) | `cargo run -p generics_traits_lifetimes --features exercises --bin ex02_fix_missing_lifetime` |
| 3 | `ex03_fix_does_not_live_long_enough` | Make it compile (E0597), two ways | `cargo run -p generics_traits_lifetimes --features exercises --bin ex03_fix_does_not_live_long_enough` |
| 4 | `ex04_generic_stack` | Generic struct + conditional method, pass 3 tests | `cargo test -p generics_traits_lifetimes --features exercises --bin ex04_generic_stack` |
| 5 | `ex05_trait_with_defaults` | Implement a trait, use it statically and dynamically, pass 3 tests | `cargo test -p generics_traits_lifetimes --features exercises --bin ex05_trait_with_defaults` |
| 6 | `ex06_conversion_traits` | `From`, `TryFrom` and `Display` for your own type, pass 3 tests | `cargo test -p generics_traits_lifetimes --features exercises --bin ex06_conversion_traits` |

Exercises 5 and 6 don't compile until you write the `impl` blocks. The compiler's "method not found" and "trait bound not satisfied" errors are your to-do list.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p generics_traits_lifetimes --features solutions`).

**4. Read the mini project** in `src/main.rs`: a **news feed** that grows the Book's `Summary` example using every tool in the chapter.

- **Traits:** `Summary` (with a default) and `Scored`, implemented by `NewsArticle`, `SocialPost` and `Video`.
- **A supertrait plus a blanket impl:** anything that is `Summary + Scored` is automatically a `FeedItem`.
- **Blanket impls for references:** `impl<T: Summary> Summary for &T`, so borrowed items work too.
- **Generics:** `top_n<T: Scored>` ranks any list and returns references into it. `Ranked<T>` is displayable only when `T` qualifies.
- **`dyn`:** `Feed` holds `Box<dyn FeedItem>`, a mix of all three types in one list.
- **Lifetimes:** `Highlight<'_>` borrows the first sentence straight out of an article, with no copying.

```bash
cargo run -p generics_traits_lifetimes
cargo test -p generics_traits_lifetimes
```

> 💡 **Analogy:** the feed is a **notice board**. Anyone holding the "can summarise" and "has a score" badges may pin something up, whether it's an article, a post or a video. The board sorts them by score, and highlights are sticky notes pointing at a line in an article, valid only while that article is still on the board.

## Checklist

- [ ] [10.1 Generic Data Types](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [ ] [10.2 Defining Shared Behavior with Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [ ] [10.3 Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [ ] Beyond the Book: static dispatch (generics) vs dynamic dispatch (`dyn Trait`, vtables)
- [ ] Beyond the Book: `impl Trait` in argument and return position
- [ ] Beyond the Book: lifetime bounds (`where 'a: 'b`), the anonymous lifetime `'_`, `'static`
- [ ] Beyond the Book: conversion traits: `From`/`Into`, `TryFrom`/`TryInto`, `AsRef`/`Borrow`
- [ ] Beyond the Book: common std traits: `Display`, `Default`, `PartialOrd`/`Ord`, `Hash`, operator traits
- [ ] Beyond the Book: const generics (`struct Matrix<const N: usize>`)
- [ ] Beyond the Book: generic associated types (GATs)
- [ ] Beyond the Book: higher-ranked trait bounds (`for<'a> Fn(&'a T)`)
- [ ] Beyond the Book: variance, subtyping and `PhantomData`
- [ ] Beyond the Book: coherence and the orphan rule in practice
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- **Generics** (`<T>`) write code once for many types. Monomorphization makes them free at run time.
- **Traits** define shared behaviour. Trait **bounds** (`T: Trait`, `impl Trait`, `where`) say what a generic type must be able to do. Default methods, conditional impls and blanket impls cut repetition.
- **Lifetimes** (`'a`) describe how long references stay valid, so the compiler can reject dangling ones. Elision fills in the obvious cases.
- Choose **static** dispatch (generics) for speed and **dynamic** dispatch (`dyn`) for mixing types.
- The trait system goes deep: conversions, operators, const generics, GATs, HRTBs, variance and the orphan rule.

**In one sentence for a child:** generics are cookie cutters that work on any dough, traits are skills badges that say what something can do, and lifetimes are due dates that make sure nobody reads a book after it's been returned.

**Next:** [02 Automated Tests](../02_automated_tests/README.md) (Book chapter 11)
