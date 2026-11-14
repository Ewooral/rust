<!-- hand-written lesson: gen_course must not overwrite -->
# Trait Objects, and What Rust Offers Instead of OOP

**The Rust Book, chapter 18:** [Object-Oriented Programming Features](https://doc.rust-lang.org/book/ch18-00-oop.html)
· Rust by Example: [Returning traits with `dyn`](https://doc.rust-lang.org/rust-by-example/trait/dyn.html), [Supertraits](https://doc.rust-lang.org/rust-by-example/trait/supertraits.html)
· Reference: [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html), [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
· std: [`std::any`](https://doc.rust-lang.org/std/any/index.html)

Is Rust object-oriented? It depends on what you mean. Rust has no `class` keyword and **no inheritance**. It does have everything else people use OOP for, built from pieces you already know:

| You want… | Classic OOP gives you | Rust gives you |
|---|---|---|
| data and behaviour together | a class | a `struct` or `enum` plus `impl` |
| hidden details | `private` members | privacy by default, `pub` to opt in |
| shared code | a base class | default trait methods, and composition |
| one interface, many types | subclasses, virtual methods | traits: generics (static) or `dyn Trait` (dynamic) |
| an interface or abstract class | `interface`, `abstract` | a trait with required and default methods |

This lesson covers the Book's three sections, then goes further: which traits can be `dyn`, supertraits, and getting the concrete type back with `Any`.

> 💡 **The big picture: LEGO, not a family tree.** Classic OOP builds types as a **family tree**: a child class inherits everything from its parent, whether it wants it or not, and a change high up ripples down to every descendant. Rust builds with **LEGO**: every type is its own brick, and **badges** (traits) say what each brick can do. To reuse a part, you snap one brick inside another (composition). To keep different bricks in one box, you label the box with a badge: "anything that can draw itself goes in here". That labelled box is a **trait object**.

## 1. Characteristics of object-oriented languages

The Book checks Rust against three features people expect from an OO language.

### Objects contain data and behaviour

The "Gang of Four" design-patterns book defines an object as a package of data plus the procedures that operate on it. By that definition, a Rust struct or enum with an `impl` block **is** an object, even though Rust doesn't call it one.

> 💡 **Analogy:** a toy robot comes with its batteries (data) and its buttons (behaviour) in one box. A struct with an `impl` is that box. It doesn't matter that the label on the box says "struct" instead of "object".

### Encapsulation hides implementation details

Callers use an object's **public API** and can't reach its insides, so the author can change those insides later without breaking anyone. In Rust, everything is **private by default**, and `pub` opts in, one item at a time:

```rust
pub struct AveragedCollection {
    list: Vec<i32>,   // private
    average: f64,     // private: a cache that must always match `list`
}

impl AveragedCollection {
    pub fn add(&mut self, value: i32) { self.list.push(value); self.update_average(); }
    pub fn average(&self) -> f64 { self.average }   // read-only access
    fn update_average(&mut self) { /* … */ }          // private helper
}
```

Because `list` and `average` are private, the only way to change the list is `add` or `remove`, and both update the average. Nobody can make the cache lie. From outside the module, the compiler refuses:

```text
error[E0616]: field `average` of struct `AveragedCollection` is private
error[E0451]: fields `list` and `average` of struct `AveragedCollection` are private
```

The first is assigning `scores.average = 99.0`, the second is building the struct directly with `AveragedCollection { list, average }`. The author could now swap the `Vec` for a running total and a count, and no caller would notice.

> 💡 **Analogy: a vending machine.** You press buttons (the public methods) and get a snack. You can't open the back and rearrange the snacks yourself, so the machine's count of what's left is always right. And the owner can redesign the inside tomorrow: your buttons still work.

▶ `cargo run -p trait_objects_and_oop --example encapsulation`

### Inheritance as a type system and as code sharing

**Inheritance** lets an object take its parent's data and behaviour without defining them again. Rust has none: you can't define a struct that inherits another struct's fields or methods. People use inheritance for two reasons, and Rust has a tool for each.

1. **To reuse code.** Write the shared behaviour once as a **default trait method**. Each type gets it for free and may override it (the Book's `Summary::summarize` in lesson 03/01). For shared *data*, use **composition**: a `Car` *has* an `Engine` field and delegates to it, instead of *being* an engine.
2. **For polymorphism**, meaning code that works with several types. Rust uses generics bounded by traits (*bounded parametric polymorphism*), or trait objects (section 2).

```rust
trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> String;
    fn describe(&self) -> String { format!("{} says {}", self.name(), self.sound()) } // shared
}

struct Car { name: String, engine: Engine }          // composition: a part, not a parent
impl Car {
    fn start(&self) -> String { format!("{}: {}", self.name, self.engine.start()) } // delegation
}
```

Inheritance has fallen out of favour in many languages. A subclass often inherits more than it should, and some languages allow only one parent, which limits design. Rust's way is to opt in, one trait at a time.

> 💡 **Analogy: a family recipe book vs a toolbelt.** Inheritance is being handed your grandmother's entire recipe book: you get the 300 recipes you wanted *and* the ones you didn't, and if she changes one, yours changes too. Traits are a **toolbelt**: you clip on just the tools you need (a default method is a tool that comes ready to use), and composition is carrying a friend's tool **in your bag** instead of becoming your friend.

▶ `cargo run -p trait_objects_and_oop --example instead_of_inheritance`

## 2. Using trait objects to abstract over shared behaviour

The Book's example is a GUI library. It ships a `Button`, and its users must be able to add their own components, such as a `SelectBox`, that the library has never heard of. The screen must hold a list of components of **different types** and call `draw` on each.

```rust
pub trait Draw {
    fn draw(&self);
}

pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,   // a list of trait objects
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}

// In the user's crate: a type the library has never seen.
struct SelectBox { width: u32, height: u32, options: Vec<String> }
impl Draw for SelectBox {
    fn draw(&self) { /* … */ }
}
```

- **`dyn Draw`** means "some value of *some* type that implements `Draw`". Its size isn't known at compile time, so it always sits behind a pointer: `Box<dyn Draw>`, `&dyn Draw` or `Rc<dyn Draw>`. Leave out `dyn` (`Box<Draw>`) and you get E0782 "expected a type, found a trait". Leave out the box (`Vec<dyn Draw>`) and you get E0277: "the size for values of type `(dyn Draw + 'static)` cannot be known at compilation time".
- A trait object is more like an "object" than a plain struct: it combines data (the value) with behaviour (the methods). Unlike an object, you can't add *data* to a trait, only behaviour.
- **Duck typing, checked by the compiler.** `run` doesn't care what each component is, only that it can `draw`. If it doesn't implement `Draw`, it never gets in:

```text
error[E0277]: the trait bound `String: Draw` is not satisfied
  = note: required for the cast from `Box<String>` to `Box<dyn Draw>`
```

> 💡 **Analogy: a toy box labelled "things that can draw".** You can put a robot, a crayon or a stamp in it. The box doesn't know or care which. When it's playtime, you take each one out and say "draw!", and each does it its own way. Try to put in a sock, and the box's guard (the compiler) says no: socks can't draw.

### Generics or trait objects?

A **generic** screen looks similar, but means something different:

```rust
pub struct Screen<T: Draw> {
    pub components: Vec<T>,   // ONE type T for the whole list
}
```

That's fine for a screen of only buttons, but a screen of a Label *and* a Button is rejected. The real error from exercise 2:

```text
error[E0308]: mismatched types
   |
50 |             Button { label: "OK".into() },
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Label`, found `Button`
```

| | `Screen<T: Draw>` (generic) | `Screen { Vec<Box<dyn Draw>> }` (trait object) |
|---|---|---|
| Types in one list | one | any mix |
| Dispatch | **static**: a copy per type, direct calls, can be inlined | **dynamic**: a vtable lookup per call |
| Extra cost | none | a `Box` per item, and one indirect call |

> 💡 **Analogy:** a generic screen is an **egg box**: twelve slots, all for eggs. A trait-object screen is a **toy box**: anything with the right badge fits.

### Trait objects perform dynamic dispatch

With generics, the compiler writes a separate copy of the code for each concrete type (monomorphization), so it knows exactly which `draw` to call. With `dyn`, it can't know, so each trait object carries a second pointer, to a **vtable**: a table of that type's method addresses. That's why `&dyn Draw` is 16 bytes where `&Button` is 8. Lesson 03/01 covers this in depth: [static vs dynamic dispatch](../01_generics_traits_lifetimes/README.md#5-static-vs-dynamic-dispatch-and-impl-trait).

> 💡 **Analogy:** each toy in the box has a **little instruction card** taped to it (the vtable), saying where to find its "draw" button. Reading the card first is a tiny bit slower than already knowing, but it's what lets one box hold every kind of toy.

▶ `cargo run -p trait_objects_and_oop --example gui_trait_objects`

### Enum or trait object?

Both let one `Vec` hold different kinds of things. They make opposite changes easy:

| | `enum Shape { Circle, Square }` | `Vec<Box<dyn Shape>>` |
|---|---|---|
| The set of kinds is | **closed**: you own the list | **open**: anyone can add one |
| Adding a new kind | edit the enum and every `match` | just `impl Shape for Triangle` |
| Adding a new operation | write one new function with a `match` | edit the trait and every impl |
| Cost | no `Box`, no vtable | a `Box` and a vtable call |

If **you** know every kind (the states of a parcel, the tokens of a parser), use an enum. If **other people's code** must add kinds (plugins, GUI widgets), use a trait object.

> 💡 **Analogy:** an enum is a **set menu**: the kitchen knows every dish, so it can promise to cook each one. A trait object is a **potluck dinner**: anyone may bring a dish, as long as it's food, but nobody knows the full menu in advance.

▶ `cargo run -p trait_objects_and_oop --example enum_or_trait_object`

## 3. Implementing an object-oriented design pattern

The **state pattern** gives a value a set of internal states, each represented by a **state object**, and the value's behaviour changes with its state. The Book's example is a blog post:

1. A post starts as an empty **draft**.
2. When the draft is done, a **review** is requested.
3. When the post is **approved**, it's published.
4. Only published posts show their content, so an unapproved post can't be published by accident.

### Version 1: state objects behind `Box<dyn State>`

```rust
pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
}

impl Post {
    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)   // ask the state
    }
    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {          // move the old state out…
            self.state = Some(s.request_review())     // …and put the new one in
        }
    }
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, _post: &'a Post) -> &'a str { "" }  // default: hidden
}

struct Draft {}
impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> { Box::new(PendingReview {}) }
    fn approve(self: Box<Self>) -> Box<dyn State> { self }   // can't approve a draft
}
// PendingReview: approve -> Published.  Published: overrides `content`.
```

Things to notice:
- **`Post` never checks which state it is in.** It forwards each request, and each state decides. Adding behaviour per state means writing it in that state's `impl`.
- **`self: Box<Self>`** means the method takes ownership of the boxed state, so the old state is used up. A plain `self` won't do, because you can't move an unsized `dyn State` out of its box: that's error E0161, "cannot move a value of type `dyn State`".
- **`Option` and `take()`.** Rust doesn't allow a struct field to be empty, even for a moment. `take()` moves the state out and leaves `None` behind, so the old state can be consumed and the new one put back.
- **`content` has a default** that returns `""`. Only `Published` overrides it, so the "drafts are hidden" rule lives in one place.

The Book is honest about the downsides. The states are **coupled**: `Draft` creates `PendingReview`, so adding a state between them means editing both. The `request_review`/`approve` boilerplate is repeated in every state. And asking a draft for its content isn't an error: it silently returns `""`.

> 💡 **Analogy: rooms in a school.** The post walks from room to room: the Draft room, the Review room, the Published room. Each room has its own rules on the wall. When you ask the post "show me your text", it doesn't decide; it reads the rule on the wall of whichever room it's in, and only the Published room's rule says yes. `take()` is stepping **out** of one room before stepping **into** the next: you can't be in two rooms at once.

▶ `cargo run -p trait_objects_and_oop --example blog_state_pattern`

### Version 2: encoding states and behaviour as types

Rust can do better than the classic pattern. Make each state its own **type**, and give each type only the methods that make sense for it:

```rust
pub struct DraftPost { content: String }
pub struct PendingReviewPost { content: String }
pub struct Post { content: String }

impl Post {
    pub fn new() -> DraftPost { DraftPost { content: String::new() } }
    pub fn content(&self) -> &str { &self.content }        // only Post has this
}
impl DraftPost {
    pub fn add_text(&mut self, text: &str) { self.content.push_str(text); }
    pub fn request_review(self) -> PendingReviewPost { PendingReviewPost { content: self.content } }
}
impl PendingReviewPost {
    pub fn approve(self) -> Post { Post { content: self.content } }
}
```

Each step takes `self` **by value**, using up the old value, and returns a different type. The caller re-binds with `let`:

```rust
let mut post = Post::new();          // DraftPost
post.add_text("I ate a salad for lunch today");
let post = post.request_review();    // PendingReviewPost
let post = post.approve();           // Post
assert_eq!("I ate a salad for lunch today", post.content());
```

Now invalid steps don't exist at all. Forget to re-bind (exercise 3), and the compiler shows you that `post` is still a draft:

```text
error[E0599]: no method named `approve` found for struct `DraftPost` in the current scope
   |
17 |     pub struct DraftPost {
   |     -------------------- method `approve` not found for this struct
...
64 |     post.approve();
   |          ^^^^^^^ method not found in `DraftPost`
```

| | `Box<dyn State>` | states as types |
|---|---|---|
| Invalid step (approve a draft) | silently ignored at run time | **compile error** |
| Caller code | `post.approve();` | `let post = post.approve();` |
| Posts in different states in one `Vec` | easy | needs an enum or a trait object again |
| Cost | a `Box` and a vtable call per step | plain structs, no allocation |

The Book's conclusion: OO patterns aren't always the best fit in Rust. Ownership and the type system let you push mistakes from run time to compile time.

> 💡 **Analogy: tickets that only fit one slot.** A *draft ticket* only fits the "send for review" machine, which gives you a *pending ticket*. That only fits the "approve" machine, which gives you a *published ticket*, the only one that opens the reading room. You can't even *try* the wrong door: the ticket doesn't fit the slot. And each machine **keeps** the ticket you put in (`self` by value), so you can't use an old one twice.

▶ `cargo run -p trait_objects_and_oop --example blog_state_as_types`

## Beyond the Book: dyn compatibility (object safety)

A trait can be used as `dyn Trait` only if a **vtable** can be built for it: one fixed table of function pointers per concrete type. The rules used to be called "object safety". Each method must either fit in the vtable, or be marked `where Self: Sized` ("only for concrete types, not for trait objects"). A method fits when:

| Rule | Breaks it | Why |
|---|---|---|
| has a receiver: `&self`, `&mut self`, `self: Box<Self>` (or `Rc`, `Arc`, `Pin`) | `fn new() -> Self` | no value to look the vtable up from |
| no generic **type** parameters | `fn log<T: Debug>(&self, v: T)` | one copy per `T`, so no single slot |
| `Self` only in the receiver | `fn duplicate(&self) -> Self` | behind `dyn`, nobody knows `Self`'s size |
| no `-> impl Trait`, no `async fn` | `fn iter(&self) -> impl Iterator` | a hidden type per impl |

The trait as a whole must also have no associated `const`s, must not require `Self: Sized` (which is why `Box<dyn Clone>` is E0038), and every supertrait must be dyn compatible too. The real error from exercise 1:

```text
error[E0038]: the trait `Shape` is not dyn compatible
   |
49 |     let shapes: Vec<Box<dyn Shape>> = vec![
   |                             ^^^^^ `Shape` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
   |
13 | trait Shape {
   |       ----- this trait is not dyn compatible...
14 |     fn area(&self) -> f64;
15 |     fn scaled(&self, factor: f64) -> Self;
   |                                      ^^^^ ...because method `scaled` references the `Self` type in its return type
   = help: consider moving `scaled` to another trait
   = help: the following types implement `Shape`:
             Circle
             Square
           consider defining an enum where each variant holds one of these types,
           implementing `Shape` for this new enum and using it instead
```

Notice the last help: the compiler itself suggests the enum alternative from section 2. The usual fixes are:
- **`where Self: Sized`** on the offending method. It still works on concrete types, but calling it on a trait object is an error: "the `doubled` method cannot be invoked on a trait object".
- **Take `&dyn Debug` instead of a generic `T: Debug`**: one function that handles every type.
- **The `clone_box` pattern** for cloning trait objects: `fn clone_box(&self) -> Box<dyn Shape>`, plus `impl Clone for Box<dyn Shape>` that calls it.

> 💡 **Analogy: a phone switchboard with a fixed list.** The vtable is the switchboard's list: one phone line per job. "Do the job for *any* type T you can think of" needs infinitely many lines, so it can't be on the list. "Send me a copy of yourself" fails because nobody on the phone knows how big a box to post. `where Self: Sized` is a sign on that job saying **"in person only"**: you can still ask for it face to face (on a concrete type), just not over the phone.

▶ `cargo run -p trait_objects_and_oop --example dyn_compatibility`

## Beyond the Book: supertraits

`trait Widget: fmt::Display` means: to implement `Widget`, a type must **also** implement `Display`. It's not inheritance: no fields and no code are passed down. It's a requirement that lets `Widget`'s default methods, and anyone holding a `dyn Widget`, use `Display` too.

```rust
trait Widget: Named + fmt::Display {
    fn width(&self) -> usize;
    fn framed(&self) -> String {
        let text = self.to_string();            // Display's method, usable here
        /* … */
    }
}

fn show(widget: &dyn Widget) {
    println!("{} ({} wide): {widget}", widget.name(), widget.width()); // all in one vtable
}
```

- Forget the supertrait's `impl` and the subtrait's `impl` fails with E0277: "`Button` doesn't implement `std::fmt::Display`", required by a bound in `Widget`.
- **Trait upcasting** (stable since Rust 1.86): a `&dyn Widget` converts to a `&dyn Display` (or `Box<dyn Widget>` to `Box<dyn Display>`), because every `Widget` is a `Display`.
- The Book meets supertraits again in chapter 20 (`OutlinePrint: Display`, lesson 04/03).

> 💡 **Analogy:** to earn the **lifeguard** badge, you must already hold the **swimming** badge. The lifeguard badge doesn't teach you to swim; it just checks you can. And anyone who needs "a swimmer" can always be sent a lifeguard: that's upcasting.

▶ `cargo run -p trait_objects_and_oop --example supertraits`

## Beyond the Book: downcasting with `Any`

A trait object forgets its concrete type on purpose. Very occasionally you need it back ("if this widget is a `Slider`, set its value"). [`std::any::Any`](https://doc.rust-lang.org/std/any/trait.Any.html) is implemented for every `'static` type and carries a hidden `TypeId`:

| Call | Gives |
|---|---|
| `any.is::<T>()` | `bool` |
| `any.downcast_ref::<T>()` / `downcast_mut::<T>()` | `Option<&T>` / `Option<&mut T>` |
| `boxed.downcast::<T>()` | `Result<Box<T>, Box<dyn Any>>`: ownership back out |

For your **own** trait objects, make `Any` a supertrait. Then trait upcasting turns a `&dyn Widget` into a `&dyn Any` you can downcast:

```rust
trait Widget: Any + Debug { fn name(&self) -> &str; }

for widget in widgets.iter_mut() {
    let any: &mut dyn Any = widget.as_mut();          // upcast
    if let Some(slider) = any.downcast_mut::<Slider>() {
        slider.value = 99;
    }
}
```

(Before 1.86 you'd write an `as_any(&self) -> &dyn Any` method in every impl. You'll still see that in older code.)

- **Only `'static` types:** a struct holding a short borrow like `&'a str` can't be `dyn Any`.
- **The `Box` trap:** on a `Box<dyn Any>`, `b.type_id()` gives the id of the *Box*. Write `(*b).type_id()`. Clippy warns with `type_id_on_box`.
- **A last resort.** A chain of downcasts is usually a missing trait method, or a set of kinds that should have been an enum.

> 💡 **Analogy: a mystery gift with a hidden barcode.** From the outside, every gift box looks the same ("something that can be a widget"). `Any` lets you scan the barcode and ask "are you a teddy bear?" If the scanner says yes, you may take the teddy out and play with it as a teddy. If not, you get `None` and the box stays shut.

▶ `cargo run -p trait_objects_and_oop --example downcasting_any`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `Vec<Box<Draw>>` | E0782 expected a type, found a trait | `Box<dyn Draw>`: `dyn` is required since the 2021 edition. |
| `Vec<dyn Draw>` | E0277 size cannot be known at compilation time | Put it behind a pointer: `Box<dyn Draw>`. |
| A generic `Screen<T>` holding a Label and a Button | E0308 mismatched types | `Vec<Box<dyn Draw>>` |
| `Box::new(value)` into `Vec<Box<dyn Draw>>` | E0277 trait bound not satisfied | `impl Draw for` that type. |
| `dyn` of a trait with `-> Self`, a generic method or `fn new()` | E0038 not dyn compatible | `where Self: Sized` on that method, or take `&dyn` instead of `T`. |
| `Box<dyn Clone>` | E0038 | The `clone_box` pattern. |
| `fn next(self)` called on a `Box<dyn State>` | E0161 cannot move a value of type `dyn State` | `fn next(self: Box<Self>) -> Box<dyn State>` |
| `post.request_review();` with typed states | E0599 no method `approve` on `DraftPost` | `let post = post.request_review();` |
| Setting or building a private field from outside its module | E0616 / E0451 | Use the public methods: that's the point. |
| `impl Widget` without the supertrait's impl | E0277 doesn't implement `Display` | Implement the supertrait too. |
| `b.type_id()` on a `Box<dyn Any>` | wrong answer (clippy: `type_id_on_box`) | `(*b).type_id()` |
| A tower of traits copying a class hierarchy | compiles, hard to change | Small traits, default methods, composition, or an enum. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p trait_objects_and_oop --example encapsulation
cargo run -p trait_objects_and_oop --example instead_of_inheritance
cargo run -p trait_objects_and_oop --example gui_trait_objects
cargo run -p trait_objects_and_oop --example enum_or_trait_object
cargo run -p trait_objects_and_oop --example blog_state_pattern
cargo run -p trait_objects_and_oop --example blog_state_as_types
cargo run -p trait_objects_and_oop --example dyn_compatibility
cargo run -p trait_objects_and_oop --example supertraits
cargo run -p trait_objects_and_oop --example downcasting_any
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_not_dyn_compatible` | Make it compile (E0038), trait only | `cargo run -p trait_objects_and_oop --features exercises --bin ex01_fix_not_dyn_compatible` |
| 2 | `ex02_fix_mixed_screen` | Make it compile (E0308), generic to `dyn` | `cargo run -p trait_objects_and_oop --features exercises --bin ex02_fix_mixed_screen` |
| 3 | `ex03_fix_typed_post` | Make it compile (E0599), `main` only | `cargo run -p trait_objects_and_oop --features exercises --bin ex03_fix_typed_post` |
| 4 | `ex04_encapsulated_stats` | Encapsulation: keep a cache correct, pass 3 tests | `cargo test -p trait_objects_and_oop --features exercises --bin ex04_encapsulated_stats` |
| 5 | `ex05_blog_extensions` | State pattern: `reject`, two approvals, draft-only edits, pass 3 tests | `cargo test -p trait_objects_and_oop --features exercises --bin ex05_blog_extensions` |
| 6 | `ex06_widgets_supertrait_any` | Supertraits and downcasting, pass 3 tests | `cargo test -p trait_objects_and_oop --features exercises --bin ex06_widgets_supertrait_any` |

Exercise 5 is the Book's own list of suggested changes. Try it, then ask yourself how much of it the typed version could check at compile time.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p trait_objects_and_oop --features solutions --bins`).

**4. The mini project** is a **blog dashboard** that puts the whole chapter together:

```bash
cargo run -p trait_objects_and_oop
cargo test -p trait_objects_and_oop    # 7 library tests, 3 for main.rs, 1 compile_fail doc test
```

```text
=== Blog dashboard ===
* Lunch [published]
    I ate a salad for lunch ...
* Why Rust [pending review]
    (hidden until published)
* Holiday plans [draft]
    (hidden until published)
Published:
[####........] 1/3
( ) Request review
(*) Approve
( ) Leave
+----------+
|  Apply   |
+----------+
```

- **`src/gui.rs`** is the "library": the `Draw` trait (with a default `height` method), a `Screen` holding `Box<dyn Draw>`, and four components: `Label`, `Button`, `SelectBox`, `ProgressBar`.
- **`src/blog/dynamic.rs`** is the state pattern with `Box<dyn State>`, plus a title and a `status()` the dashboard can show.
- **`src/blog/typed.rs`** is the same workflow with states as types. A `compile_fail` doc test proves a draft has no `content()`. (Stable `rustdoc` checks that it fails to compile; the `E0599` tag on it is only enforced on nightly.)
- **`src/main.rs`** is the "user's crate". It invents a `PostCard` component that the library has never heard of, and puts it on the screen next to the built-in ones: eight components of five types in one `Vec`.
- **Encapsulation:** a post's content is a private field. The only way to read it is `content()`, which asks the state, so the dashboard can't show a draft by mistake.

> 💡 **Analogy: a fridge door.** The library made the fridge, and it only has one rule: anything with a **magnet** on the back (`Draw`) can stick to it. It came with a few magnets of its own (labels, buttons). You made your own magnet at home (`PostCard`) and stuck it right next to them. The fridge never needed to know what you'd draw.

## Checklist

- [ ] [18.1 Characteristics of Object-Oriented Languages](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html)
- [ ] [18.2 Using Trait Objects to Abstract over Shared Behavior](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
- [ ] [18.3 Implementing an Object-Oriented Design Pattern](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html)
- [ ] Beyond the Book: dyn compatibility (object safety) rules, `where Self: Sized`, `clone_box`
- [ ] Beyond the Book: supertraits and trait upcasting
- [ ] Beyond the Book: downcasting with `Any`
- [ ] Exercises 1–6 and the blog dashboard done

## Recap

- Rust has objects (structs and enums with `impl`) and encapsulation (private by default), but **no inheritance**.
- Instead of inheritance: **default methods** and **composition** to reuse code, and **traits** for polymorphism, through generics (static, one type) or **trait objects** (dynamic, any mix).
- `dyn Trait` lives behind a pointer (`Box`, `&`, `Rc`) and calls methods through a vtable. Use it when *other* code must be able to add types; use an enum when you know every kind.
- The **state pattern** puts each state's rules in its own type behind `Box<dyn State>`. Rust can go further and make each state a **separate type**, so invalid steps become compile errors.
- Only **dyn compatible** traits can be `dyn`. **Supertraits** require other traits, and trait objects can be upcast to them. **`Any`** lets you downcast, as a last resort.

**In one sentence for a child:** Rust doesn't build family trees of types; it gives each type badges for what it can do, and a trait object is a toy box labelled with a badge, where anything wearing that badge may go in.

**Next:** [08 Patterns and Matching](../08_patterns_and_matching/README.md) (Book chapter 19)
