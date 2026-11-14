<!-- hand-written lesson: gen_course must not overwrite -->
# Async Traits from Scratch

**Official docs:** [Rust blog: `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)
· Reference: [dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
· std: [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html), [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html), [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html)
· [The Async Book](https://rust-lang.github.io/async-book/)
· crates: [`async-trait`](https://docs.rs/async-trait), [`tokio::spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)
· Video: Let's Get Rusty, "Rust's most complicated features explained" (this lesson follows its plan)

Run it: `cargo run -p async_traits_from_scratch`  ·  Test it: `cargo test -p async_traits_from_scratch`

`async fn` in a trait looks like one small feature. Making it work as `dyn Trait`, inside `tokio::spawn`, with borrowed arguments, pulls in nearly every hard part of Rust at once: futures, trait objects, `Pin`, `Send`/`Sync` and lifetimes. This lesson rebuilds a `UserService` trait **by hand**, one compiler error at a time, until it is exactly what the `#[async_trait]` macro writes for you. After it you will be able to:

- read `Pin<Box<dyn Future<Output = T> + Send + 'a>>` and say what each of the five parts is for;
- choose between native `async fn` in traits, `-> impl Future + Send`, a hand-boxed future and `#[async_trait]`;
- share one `Arc<dyn Service>` between many spawned tasks, and fix the errors when you can't.

You've met the pieces before: dyn compatibility, static vs dynamic dispatch and lifetimes in [03/01 Generics, traits and lifetimes](../../03_abstraction/01_generics_traits_lifetimes/README.md#5-static-vs-dynamic-dispatch-and-impl-trait), and futures, `poll`, `Pin` and `BoxFuture` in [04/02 Async/await](../../04_systems_level/02_async_await/README.md#beyond-the-book-pin-unpin-and-boxed-futures). This lesson puts them together.

> 💡 **The big picture: a food court with buzzers.** An `async fn` doesn't hand you food, it hands you a **buzzer** (a future) that goes off when the food is ready. A trait is the sign over the counters: "every stall here gives out buzzers". Trouble starts when you want **one queue for all the stalls** (`dyn`): every stall's buzzer is a different shape and size, so the shared rack has no slot that fits them all. So we put each buzzer in a **standard box** (`Box<dyn Future>`), **glue the box to the tray** so the buzzer inside never moves (`Pin`), stamp it **"any waiter may carry this"** (`Send`), and write **"valid while this stall is open"** on it (the lifetime). `#[async_trait]` is a packing machine that does all four for you.

## The plan: seven steps, one error each

Why want `dyn UserService` at all? Because real programs choose **at run time**: a database in production and a fake in tests, a cache in front of whatever is configured, a list of services to try in turn. That's a `Box<dyn UserService>` or an `Arc<dyn UserService>`.

| Step | Example | `get_user` looks like | Fixes | Breaks next |
|---|---|---|---|---|
| 1 | `step1_native_async_fn` | `async fn get_user(&self, id: u32) -> Option<User>` | | `dyn`: E0038 · spawn: not `Send` |
| 2 | `step2_impl_future_send` | `fn get_user(…) -> impl Future<Output = Option<User>> + Send` | spawning | `dyn`: still E0038 |
| 3 | `step3_box_the_future` | `fn get_user(…) -> Box<dyn Future<Output = …> + Send + '_>` | `dyn` | `.await`: E0277 cannot be unpinned |
| 4 | `step4_pin_the_box` | `fn get_user(…) -> Pin<Box<dyn Future<Output = …> + Send + '_>>` | `.await` | |
| 5 | `step5_send_sync_spawn` | `trait UserService: Send + Sync` | spawning an `Arc<dyn UserService>` | |
| 6 | `step6_future_lifetimes` | `'_`, `<'a>`, `'static` on the future | borrowed arguments | |
| 7 | `step7_async_trait_macro` | `#[async_trait]` + `async fn` again | writes steps 3-6 for you | |

Every example uses the mini project's `User { id, name }` (from `src/service.rs`) and defines its own `UserService`, because the trait is what changes from step to step.

## 1. Native `async fn` in a trait

Since Rust 1.75 you can write this, and it works:

```rust
trait UserService {
    async fn get_user(&self, id: u32) -> Option<User>;
}

impl UserService for Database {
    async fn get_user(&self, id: u32) -> Option<User> {
        tokio::time::sleep(Duration::from_millis(5)).await; // pretend: a round trip
        self.users.get(&id).cloned()
    }
}

async fn greet(service: &impl UserService, id: u32) -> String { /* … */ }
```

With generics (`impl UserService`, `S: UserService`) this is **static dispatch**: the compiler writes a copy of `greet` for each service type, and in each copy it knows the exact future `get_user` returns. No boxing, no heap allocation.

Each impl's `async fn` returns its **own hidden future type**, a state machine holding whatever that body keeps across its `.await`s. The example prints their sizes:

```text
Database's future is 144 bytes, Guests' future is 224 bytes
```

A vtable has **one slot** per method, and a slot needs one return type of one size. So the trait can't be a trait object. Uncomment the last line of the example:

```text
error[E0038]: the trait `UserService` is not dyn compatible
  --> 05_mastery\01_async_traits_from_scratch\examples\step1_native_async_fn.rs:68:31
   |
68 |     let services: Vec<Box<dyn UserService>> = vec![Box::new(database), Box::new(Guests)];
   |                               ^^^^^^^^^^^ `UserService` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> 05_mastery\01_async_traits_from_scratch\examples\step1_native_async_fn.rs:12:14
   |
11 | trait UserService {
   |       ----------- this trait is not dyn compatible...
12 |     async fn get_user(&self, id: u32) -> Option<User>;
   |              ^^^^^^^^ ...because method `get_user` is `async`
```

Make the trait `pub` and the compiler also warns you about a second problem (real output):

```text
warning: use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
  |
9 |     async fn get_user(&self, id: u32) -> Option<User>;
  |     ^^^^^
  |
  = note: you can suppress this lint if you plan to use the trait only in your own code, or do not care about auto traits like `Send` on the `Future`
  = note: `#[warn(async_fn_in_trait)]` on by default
```

That second problem is step 2.

> 💡 **Analogy: every stall's own buzzer.** With generics you go **straight to one stall**: you know its buzzer, and it fits your hand. A shared rack for the whole food court (a vtable) needs every buzzer to be the same size, and the burger stall's buzzer is twice as big as the noodle stall's.

▶ `cargo run -p async_traits_from_scratch --example step1_native_async_fn`

## 2. What `async fn` desugars to: `-> impl Future + Send`

An `async fn` is shorthand for a plain `fn` that **returns a future**:

```rust
async fn get_user(&self, id: u32) -> Option<User>;
// means roughly
fn get_user(&self, id: u32) -> impl Future<Output = Option<User>>;
```

The trait says nothing about whether that future is `Send`, so generic code can't spawn it. `tokio::spawn` may move a task to another worker thread at any `.await`, so it demands `F: Future + Send + 'static`. With step 1's trait, `spawn_lookup` in the step 2 example fails:

```text
error: future cannot be sent between threads safely
   --> 05_mastery\01_async_traits_from_scratch\examples\step2_impl_future_send.rs:59:5
    |
 59 |     tokio::spawn(async move { service.get_user(id).await })
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future created by async block is not `Send`
    |
    = help: within `{async block@…}`, the trait `Send` is not implemented for `impl Future<Output = Option<User>>`
…
help: `Send` can be made part of the associated future's guarantees for all implementations of `UserService::get_user`
    |
 26 -     async fn get_user(&self, id: u32) -> Option<User>;
 26 +     fn get_user(&self, id: u32) -> impl std::future::Future<Output = Option<User>> + Send;
```

This error has **no error code**. Even `Database`, whose future really is `Send`, is refused: generic code only knows what the trait promises. The compiler's help is the fix, so step 2 writes the return type out:

```rust
trait UserService {
    fn get_user(&self, id: u32) -> impl Future<Output = Option<User>> + Send;
}

impl UserService for Database {
    async fn get_user(&self, id: u32) -> Option<User> { … }   // an impl may still use async fn
}
```

- An impl may keep writing `async fn`; the compiler checks that its future really is `Send`.
- The promise binds every implementor, forever: removing `+ Send` later breaks every caller that spawns, which is why the lint calls it part of a public API.
- Asking for `Send` at the **call site** instead (`S: UserService<get_user(..): Send>`, "return type notation") isn't stable yet. On this course's stable compiler it's `error[E0658]: return type notation is experimental`.

Still no `dyn`: `impl Future` is still a different hidden type in every impl, and the E0038 note now says `...because method `get_user` references an `impl Trait` type in its return type`.

> 💡 **Analogy: a promise on the sign.** Step 1's sign said "every stall gives out buzzers". Step 2's sign says "every stall gives out buzzers **that any waiter may carry**". Now the manager can send a waiter off with one. The buzzers are still all different shapes, though.

▶ `cargo run -p async_traits_from_scratch --example step2_impl_future_send`

## 3. Box the future

To give every impl the **same** return type, put the future on the heap behind a trait object:

```rust
trait UserService {
    fn get_user(&self, id: u32) -> Box<dyn Future<Output = Option<User>> + Send + '_>;
}

impl UserService for Database {
    fn get_user(&self, id: u32) -> Box<dyn Future<Output = Option<User>> + Send + '_> {
        Box::new(async move {
            tokio::task::yield_now().await;
            self.users.get(&id).cloned()
        })
    }
}
```

`Box<dyn Future<…>>` is always two pointers (the heap address and a vtable), whatever future is inside. The trait is dyn compatible now, and `Vec<Box<dyn UserService>>` compiles. (`+ Send` is step 2's promise; `+ '_` is explained in step 6.)

But `.await` fails (the real error from exercise 2):

```text
error[E0277]: `dyn Future<Output = Option<User>> + Send` cannot be unpinned
  --> 05_mastery\01_async_traits_from_scratch\exercises\ex02_fix_unpin.rs:54:40
   |
54 |         let user = service.get_user(1).await;
   |                                        ^^^^^ the trait `Unpin` is not implemented for `dyn Future<Output = Option<User>> + Send`
   |
   = note: consider using the `pin!` macro
           consider using `Box::pin` if you need to access the pinned value outside of the current scope
   = note: required for `Box<dyn Future<Output = Option<User>> + Send>` to implement `Future`
   = note: required for `Box<dyn Future<Output = Option<User>> + Send>` to implement `IntoFuture`
```

The notes explain it: `Box<F>` is only a `Future` when `F: Unpin`, and a `dyn Future` can't promise that. `Box::into_pin(service.get_user(1)).await` works, but every caller would have to remember it. Step 4 puts the pin in the signature instead.

> 💡 **Analogy: the standard box.** Every stall now puts its buzzer in the **same plain box**, so the shared rack takes them all. But the food court has a rule: you may only wait on a buzzer that is **glued down**, and a plain box isn't.

▶ `cargo run -p async_traits_from_scratch --example step3_box_the_future`

## 4. `Pin` and `Unpin`: why futures must not move

```rust
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait UserService {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}
// impls: Box::pin(async move { … }) instead of Box::new(…)
```

`Box::pin` puts the future on the heap **and** promises it will never move again. `Pin<Box<dyn Future>>` is a future you can `.await`. The alias is so common that the `futures` crate ships it as `futures::future::BoxFuture`.

**Why pin at all?** A future can **borrow from itself**. In this block, `first_word` points into `name`, and both are kept across the `.await`, so both live inside the future's state machine:

```rust
let self_referential = async {
    let name = String::from("Ada Lovelace");
    let first_word = name.split(' ').next().unwrap(); // a borrow of `name`
    tokio::task::yield_now().await;                   // ...kept across this await
    first_word.len()
};
```

If that future were moved to a new address after its first `poll`, `first_word` would still point at the **old** address. So:

- `Future::poll` takes `self: Pin<&mut Self>`: "I promise this won't move any more".
- **`Unpin`** is an auto trait meaning "moving me is always fine, pinned or not". `i32`, `String`, `Ready<i32>` and any `Box` are `Unpin`, so `Pin` costs them nothing. Async blocks and `async fn` futures are not: `require_unpin(&self_referential)` is `E0277: `{async block@…}` cannot be unpinned`.
- Pin on the **heap** with `Box::pin` (the result can be stored and returned), or on the **stack** with `std::pin::pin!` (only inside the current scope).

[04/02](../../04_systems_level/02_async_await/README.md#5-a-closer-look-at-the-traits-for-async) polls futures by hand if you want to see `Pin<&mut Self>` at work.

> 💡 **Analogy: a treasure map that says "three steps left of this map".** The future carries a note that points to **another spot inside itself**. Move the whole thing and the note now points at empty floor. **Pin** is gluing it to the table before anyone reads the note. **Unpin** is a sticker on things with no such notes, like a plain box of crayons: move them wherever you like.

▶ `cargo run -p async_traits_from_scratch --example step4_pin_the_box`

## 5. `Send` and `Sync`: one service shared by many tasks

A real program keeps **one** service (one connection pool, one cache) and shares it between many spawned tasks: `Arc<dyn UserService>`, with one `Arc::clone` per task. That needs the trait object itself to be thread-safe, and a `dyn UserService` could be *any* type, so the trait must promise it:

```rust
trait UserService: Send + Sync {
    fn get_user(&self, id: u32) -> BoxFuture<'_, Option<User>>;
}

let service: Arc<dyn UserService> = Arc::new(database);
for id in [1, 2, 3] {
    let service = Arc::clone(&service);   // each task OWNS its Arc: 'static
    tasks.push(tokio::spawn(async move { service.get_user(id).await }));
}
```

Without the supertraits, the spawn fails twice, once for `Sync` and once for `Send` (no error code):

```text
error: future cannot be sent between threads safely
   --> 05_mastery\01_async_traits_from_scratch\examples\step5_send_sync_spawn.rs:72:20
    |
 72 |           tasks.push(tokio::spawn(async move {
    |  ____________________^
 73 | |             let user = service.get_user(id).await;
 74 | |             (id, user)
 75 | |         }));
    | |__________^ future created by async block is not `Send`
    |
    = help: the trait `Sync` is not implemented for `dyn UserService`
note: captured value is not `Send`
   --> 05_mastery\01_async_traits_from_scratch\examples\step5_send_sync_spawn.rs:73:24
    |
 73 |             let user = service.get_user(id).await;
    |                        ^^^^^^^ has type `Arc<dyn UserService>` which is not `Send`
```

| Trait | Means | Needed here because |
|---|---|---|
| `Send` | a value may **move** to another thread | each task moves its `Arc` to whichever worker runs it |
| `Sync` | several threads may use it through `&` **at once** | `Arc<T>` is only `Send` if `T: Send + Sync`, and the futures hold `&self` |

The rule underneath: **`&T` is `Send` exactly when `T` is `Sync`**. So the boxed future, which holds `&self`, is `Send` only if the service is `Sync`. That's why a `Cell<u32>` counter can't go in a service (the example's comments show both errors, E0277 `Cell<u32>` cannot be shared between threads safely, and the future one), and the example counts with an `AtomicU32` instead. The same goes for a std `MutexGuard` held across an `.await` (the mini project's cache drops it first).

> 💡 **Analogy: one recipe book on a stand.** All the cooks share one book. Each gets a little card saying where the book is (an `Arc` clone). The book needs a sticker saying **"many cooks may read me at once"** (`Sync`), and each card needs one saying **"you may carry me to another kitchen"** (`Send`). Without the stickers, the head chef won't let anything leave the counter.

▶ `cargo run -p async_traits_from_scratch --example step5_send_sync_spawn`

## 6. Lifetime bounds on the returned future

A future that uses `&self` **borrows** `self`, so it must not outlive it. That's the `'a` in `BoxFuture<'a, T>`.

**`'_`: borrows `self`.** A `Box<dyn Trait>` with no lifetime means `Box<dyn Trait + 'static>`. Leave the `'_` out and the impl fails (no error code):

```text
error: lifetime may not live long enough
  --> 05_mastery\01_async_traits_from_scratch\examples\step6_future_lifetimes.rs:47:9
   |
46 |     fn get_user(&self, id: u32) -> Pin<Box<dyn Future<Output = Option<User>> + Send>> {
   |                 - let's call the lifetime of this reference `'1`
47 |         Box::pin(async move { self.users.get(&id).cloned() })
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'static`
   |
help: to declare that the trait object captures data from argument `self`, you can add an explicit `'_` lifetime bound
```

**`<'a>`: borrows two things.** With `&self` and `name: &str`, the elision rules give the output `self`'s lifetime only, but the future also holds `name`:

```text
error: lifetime may not live long enough
  --> 05_mastery\01_async_traits_from_scratch\examples\step6_future_lifetimes.rs:51:9
   |
50 |     fn find_by_name(&self, name: &str) -> BoxFuture<'_, Option<User>> {
   |                     -            - let's call the lifetime of this reference `'1`
   |                     |
   |                     let's call the lifetime of this reference `'2`
51 |         Box::pin(async move { self.users.values().find(|u| u.name == name).cloned() })
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`
   |
help: consider introducing a named lifetime parameter and update trait if needed
   |
50 -     fn find_by_name(&self, name: &str) -> BoxFuture<'_, Option<User>> {
50 +     fn find_by_name<'a>(&self, name: &'a str) -> BoxFuture<'a, Option<User>> {
```

Don't take that suggestion as it is: it ties the future to `name` and **not** to `self`, so the error just moves. Tie both: `fn find_by_name<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Option<User>>`.

**`'static`: borrows nothing.** `tokio::spawn` needs `'static`, because the task may run after the current function returns. A future that borrows the service can't be spawned (the real error from exercise 3):

```text
error[E0521]: borrowed data escapes outside of function
   --> 05_mastery\01_async_traits_from_scratch\exercises\ex03_fix_spawn_borrowed.rs:45:5
    |
 44 | fn spawn_lookup(service: &dyn UserService, id: u32) -> JoinHandle<Option<User>> {
    |                 -------  - let's call the lifetime of this reference `'1`
    |                 |
    |                 `service` is only valid in the function body
 45 |     tokio::spawn(service.get_user(id))
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |     |
    |     `service` escapes the function body here
    |     argument requires that `'1` must outlive `'static`
```

(Spawning `database.get_user(1)` for a local `database` in `main` is the same mistake, reported as E0597: `database` does not live long enough.) Two fixes:

- **Move an `Arc` into the task** (step 5): `tokio::spawn(async move { service.get_user(id).await })`. The future borrows the task's *own* `Arc`, which lives as long as the task.
- **Return a `BoxFuture<'static, T>`**: clone what the future needs (an `Arc` of the data) *before* the `async move`, so it borrows nothing. The example's `get_user_owned` does this.

> 💡 **Analogy: "valid while this stall is open".** A buzzer from a stall only works while that stall exists. `'_` is the stamp "valid while **this** stall is open". A buzzer that needs two stalls (`self` and `name`) must say "valid while **both** are open". A `'static` buzzer is a **takeaway**: it carries everything with it, so a waiter can take it anywhere, even after the stall closes.

▶ `cargo run -p async_traits_from_scratch --example step6_future_lifetimes`

## 7. `#[async_trait]`: let the macro write it

The [`async-trait`](https://docs.rs/async-trait) crate turns all of steps 3-6 back into `async fn`:

```rust
#[async_trait]
trait UserService: Send + Sync {
    async fn get_user(&self, id: u32) -> Option<User>;
    async fn find_by_name(&self, name: &str) -> Option<User>;
}

#[async_trait]
impl UserService for Database {
    async fn get_user(&self, id: u32) -> Option<User> { /* … */ }
    async fn find_by_name(&self, name: &str) -> Option<User> { /* … */ }
}
```

It rewrites each `async fn` into (roughly):

```rust
fn find_by_name<'life0, 'life1, 'async_trait>(&'life0 self, name: &'life1 str)
    -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'async_trait>>
where
    'life0: 'async_trait,   // the future may borrow self...
    'life1: 'async_trait,   // ...and name
    Self: 'async_trait;
```

That's every step of this lesson: boxed (3), pinned (4), `Send` (2, 5), and a lifetime that covers every borrowed argument (6). The mini project proves it: `InMemoryUsers` in `src/backends.rs` implements the macro-written trait **by hand**, with exactly that signature, next to impls written with the macro.

- The **impl needs the attribute too**. Forget it and every method fails with E0195:

```text
error[E0195]: lifetime parameters or bounds on method `get_user` do not match the trait declaration
  --> 05_mastery\01_async_traits_from_scratch\examples\step7_async_trait_macro.rs:47:22
   |
31 | #[async_trait]
   | -------------- this bound might be missing in the impl
32 | trait UserService: Send + Sync {
33 |     async fn get_user(&self, id: u32) -> Option<User>;
   |              ------------------------
   |              |        |
   |              |        this bound might be missing in the impl
   |              lifetimes in impl do not match this method in trait
...
47 |     async fn get_user(&self, id: u32) -> Option<User> {
   |                      ^ lifetimes do not match method in trait
```

- **`#[async_trait(?Send)]`** (on the trait and the impl) leaves out the `+ Send`, for single-threaded code that keeps an `Rc` across an `.await`. Those futures then can't be spawned ("future cannot be sent between threads safely").
- Plain (non-async) methods in the trait are left alone.
- The cost is one heap allocation per call, as in step 4.

> 💡 **Analogy: the packing machine.** Doing steps 3 to 6 by hand is boxing, gluing, stamping and labelling every buzzer yourself. `#[async_trait]` is a **machine at the end of the counter** that does all four to every buzzer that passes. Both the stall's side (the impl) and the sign (the trait) need the machine, or the boxes don't match the rack.

▶ `cargo run -p async_traits_from_scratch --example step7_async_trait_macro`

## Which one should I use?

| | native `async fn` | `-> impl Future + Send` | hand-written `BoxFuture` | `#[async_trait]` |
|---|---|---|---|---|
| `dyn Trait` | ✗ | ✗ | ✓ | ✓ |
| heap allocation per call | none | none | one | one |
| callers may `tokio::spawn` it | ✗ (not promised) | ✓ | ✓ (with `+ Send`) | ✓ (unless `?Send`) |
| extra dependency | none | none | none | `async-trait` |
| use it for | generic code you control | public generic traits used with spawn | `dyn` without a macro, full control | `dyn` with readable code |

What does "one allocation" cost? The `cost_of_boxing` example swaps in a counting global allocator and calls each kind 1,000 times:

```text
1,000 calls, heap allocations:
  native async fn (static dispatch):   0
  #[async_trait] (static dispatch):    1000
  #[async_trait] through &dyn:         1000

size of the native future: 24 bytes; of the boxed one: 16 bytes (+ its heap block)
```

One small allocation per call is nothing next to a network round trip, which is why `#[async_trait]` is everywhere in web services. In a hot loop, prefer generics and native `async fn`. (Crates such as `trait-variant` can also generate a `Send` version of a native async trait.)

> 💡 **Analogy:** going straight to one stall (generics) needs no box at all. The shared queue (`dyn`) needs a box for every order: cheap, but not free, so you only pay it when you need the shared queue.

▶ `cargo run -p async_traits_from_scratch --example cost_of_boxing`

## The mini project: a layered user directory

`src/` is the finished trait (`#[async_trait]`, `Send + Sync`) and a set of services that **wrap each other**, put together at run time from a preset name:

```text
cache ──▶ fallback ──┬──▶ timeout ──▶ remote   (slow, can go down)
                     └──▶ memory               (fast, a local copy)
```

| File | What's in it |
|---|---|
| [`src/service.rs`](src/service.rs) | `UserService`, `User`, `LookupError`, the `BoxFuture` alias |
| [`src/backends.rs`](src/backends.rs) | `InMemoryUsers` (implemented **by hand**, no macro), `RemoteUsers` (latency, can be switched off, counts calls) |
| [`src/layers.rs`](src/layers.rs) | `CachedUsers`, `TimeoutUsers` (`tokio::time::timeout`), `FallbackUsers`: each wraps an `Arc<dyn UserService>` |
| [`src/stack.rs`](src/stack.rs) | `build_stack(preset, …) -> Option<Arc<dyn UserService>>`, `lookup_many` (one `tokio::spawn` per id, answers in order) |
| [`src/main.rs`](src/main.rs) | builds a stack, looks the ids up twice, prints timings |

```bash
cargo run -p async_traits_from_scratch                                   # resilient stack, remote up
cargo run -p async_traits_from_scratch -- cached 1 2 3
cargo run -p async_traits_from_scratch -- resilient --remote-slow        # remote takes 200 ms, limit is 100 ms
cargo run -p async_traits_from_scratch -- resilient --remote-down 1 2 3 42
cargo test -p async_traits_from_scratch                                  # 10 unit tests + 1 doc test
```

```text
stack: cache(fallback(timeout(remote(200ms), 100ms) or memory))

round 1: 3 lookups at once took 132 ms
    1 -> Ada Lovelace
    2 -> Grace Hopper
    3 -> Alan Turing

round 2: 3 lookups at once took 0 ms
    1 -> Ada Lovelace
    2 -> Grace Hopper
    3 -> Alan Turing

the remote service was asked 3 times
```

Round 1 runs three lookups at once, so it takes about one time limit (100 ms) plus the fallback, not three. Remote is too slow, the timeout drops its futures, and the in-memory copy answers. Round 2 comes from the cache. With `--remote-down 1 2 3 42`, id 42 goes to the remote **every** round: the cache never stores errors, and "not found" is a real answer, so the fallback doesn't ask memory.

> 💡 **Analogy: a library help desk.** The front desk (cache) remembers recent questions. Anything new goes to the head office (remote), but if the head office doesn't answer within a minute (timeout) or is closed (down), the desk checks the local branch's copy (fallback). Every desk worker answers the same question, "who is user 7?", so you can line them up in any order behind one sign that reads `UserService`.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `Box<dyn Trait>` for a trait with `async fn` | E0038 not dyn compatible: method is `async` | Return a `BoxFuture`, or `#[async_trait]` |
| `-> impl Future<…>` in a trait, used as `dyn` | E0038: references an `impl Trait` type in its return type | Same: box the future |
| `tokio::spawn` of a native `async fn` trait method in generic code | future cannot be sent between threads safely (no code) | `fn …(…) -> impl Future<Output = T> + Send` in the trait |
| `.await` on a `Box<dyn Future>` | E0277 cannot be unpinned | `Pin<Box<…>>` and `Box::pin` (or `Box::into_pin`) |
| `Box::new(async move { … })` where a `Pin<Box<…>>` is expected | E0308 mismatched types | `Box::pin(async move { … })` |
| `Box<dyn Future …>` without `+ '_` in a method using `self` | lifetime may not live long enough (no code) | `+ '_`, or `BoxFuture<'_, T>` |
| `'_` with two borrowed arguments | lifetime may not live long enough | `<'a>` on `&'a self`, `&'a str` and `BoxFuture<'a, T>` |
| `tokio::spawn(service.get_user(id))` with `service: &dyn …` | E0521 borrowed data escapes | Move an `Arc` clone in: `async move { service.get_user(id).await }` |
| `Arc<dyn Trait>` into `tokio::spawn`, trait without `Send + Sync` | future cannot be sent between threads safely (no code) | `trait Trait: Send + Sync` |
| A `Cell`/`RefCell`/`Rc` field in a `Send + Sync` service | E0277 cannot be shared between threads safely | Atomics, `Mutex`, `Arc` |
| `#[async_trait]` on the trait but not the impl | E0195 lifetime parameters or bounds do not match | Put `#[async_trait]` on the impl too |
| A std `MutexGuard` alive across `.await` | future cannot be sent between threads safely | Copy the value out so the guard drops first, or `tokio::sync::Mutex` |
| Starting each spawned lookup and awaiting it before the next | compiles, but runs one at a time | Spawn them all, `collect`, then await the handles |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p async_traits_from_scratch --example step1_native_async_fn
cargo run -p async_traits_from_scratch --example step2_impl_future_send
cargo run -p async_traits_from_scratch --example step3_box_the_future
cargo run -p async_traits_from_scratch --example step4_pin_the_box
cargo run -p async_traits_from_scratch --example step5_send_sync_spawn
cargo run -p async_traits_from_scratch --example step6_future_lifetimes
cargo run -p async_traits_from_scratch --example step7_async_trait_macro
cargo run -p async_traits_from_scratch --example cost_of_boxing
```

Each step has its failing lines in comments, with the error they give. Uncomment one, read the error, and put it back.

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_dyn_compatible` | Make it compile (E0038): box the futures | `cargo run -p async_traits_from_scratch --features exercises --bin ex01_fix_dyn_compatible` |
| 2 | `ex02_fix_unpin` | Make it compile (E0277): pin the box | `cargo run -p async_traits_from_scratch --features exercises --bin ex02_fix_unpin` |
| 3 | `ex03_fix_spawn_borrowed` | Make it compile (E0521): spawn with an `Arc` | `cargo run -p async_traits_from_scratch --features exercises --bin ex03_fix_spawn_borrowed` |
| 4 | `ex04_cached_service` | A hand-written caching layer, pass 3 tests | `cargo test -p async_traits_from_scratch --features exercises --bin ex04_cached_service` |
| 5 | `ex05_fetch_all` | Concurrent lookups with `tokio::spawn` and a timeout, pass 3 tests | `cargo test -p async_traits_from_scratch --features exercises --bin ex05_fetch_all` |
| 6 | `ex06_async_trait_fallback` | A fallback layer with `#[async_trait]`, pass 3 tests | `cargo test -p async_traits_from_scratch --features exercises --bin ex06_async_trait_fallback` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p async_traits_from_scratch --features solutions --bins`, or `cargo run … --features solutions --bin sol01_fix_dyn_compatible`).

**4. The mini project:** run the commands above, then extend it:
- Add a `RetryUsers` layer that asks its inner service up to 3 times when the error `is_retryable()`.
- Make `CachedUsers` forget entries after a time limit (store an `Instant` with each user).
- Rewrite `InMemoryUsers`'s hand-written impl with `#[async_trait]`, and check that nothing else changes.

## Checklist

- [ ] Native `async fn` in traits: works with generics, why it isn't dyn compatible (E0038)
- [ ] Desugaring to `fn …(&self) -> impl Future<Output = T> + Send`, and why spawning needs the `+ Send`
- [ ] Boxing futures: `Box<dyn Future<Output = T> + Send + '_>` makes the trait dyn compatible
- [ ] `Pin` and `Unpin`: self-referential futures, `Box::pin`, `pin!`, the `BoxFuture` alias
- [ ] `Send + Sync` supertraits for `Arc<dyn Trait>` in `tokio::spawn`; `&T: Send` needs `T: Sync`
- [ ] Lifetime bounds on the returned future: `'_`, `<'a>` for several borrows, `'static` futures
- [ ] `#[async_trait]`: what it expands to, `?Send`, the attribute on the impl
- [ ] The cost of boxing, and which form to choose
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A native `async fn` in a trait returns a different hidden future type in every impl: great with generics, impossible as `dyn`, and not promised to be `Send`.
- Writing `-> impl Future<Output = T> + Send` makes the `Send` promise; boxing it as `dyn Future` gives every impl the same return type, so the trait becomes dyn compatible.
- `.await` needs a pinned future, because futures can point into themselves: return `Pin<Box<dyn Future<Output = T> + Send + 'a>>`, built with `Box::pin`.
- `trait X: Send + Sync` lets one `Arc<dyn X>` be shared by many spawned tasks; the future's lifetime says what it borrows, and `tokio::spawn` needs one that borrows nothing it doesn't own.
- `#[async_trait]` writes all of that for you, at the price of one small heap allocation per call.

**In one sentence for a child:** every food stall gives you a buzzer of its own shape, so to keep them all in one rack you put each in the same box, glue it down, stamp it "any waiter may carry this" and "valid while the stall is open", and `#[async_trait]` is the machine that does the packing for you.

**Next:** [02 Macros in depth](../02_macros_in_depth/README.md): `macro_rules!` and procedural macros, including how a macro like `#[async_trait]` is built.
