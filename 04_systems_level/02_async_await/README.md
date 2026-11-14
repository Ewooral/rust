<!-- hand-written lesson: gen_course must not overwrite -->
# Async and Await

**The Rust Book, chapter 17:** [Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html)
· [The Async Book](https://rust-lang.github.io/async-book/)
· std: [`std::future`](https://doc.rust-lang.org/std/future/index.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html), [`std::task`](https://doc.rust-lang.org/std/task/index.html)
· Tokio: [tutorial](https://tokio.rs/tokio/tutorial), [docs](https://docs.rs/tokio)
· Crates: [`futures`](https://docs.rs/futures), [`tokio-stream`](https://docs.rs/tokio-stream), [`async-trait`](https://docs.rs/async-trait)

Threads (lesson 04/01) are the tool for **computing** in parallel. But most programs spend most of their time **waiting**: for the network, the disk, a database, a timer. A thread that waits does nothing, yet still costs a stack and kernel bookkeeping. **Async** lets one thread juggle thousands of waits: whenever one job has to wait, the thread works on another.

| | Threads | Async tasks |
|---|---|---|
| Best for | heavy computation (CPU-bound) | lots of waiting (I/O-bound) |
| Who switches between them | the operating system, at any moment | your code, only at `.await` points |
| Cost of one more | a stack (megabytes reserved) + the kernel | a small state machine (bytes) |
| How many at once | hundreds to thousands | hundreds of thousands |

> 💡 **The big picture: one chef, many pots.** A chef puts pasta on to boil. A bad chef stares at the pot for ten minutes. A good chef chops the salad and checks the oven, and comes back when the pasta timer pings. Each dish in progress is a **future**. `.await` means "this needs to wait, so I'll do something else until it pings". The **runtime** is the kitchen timer plus the board that says which dish is ready next. Threads would be hiring more chefs; async is one chef never wasting a wait.

### This lesson uses Tokio and `futures` directly

The Book uses its own helper crate, `trpl`, which only re-exports pieces of [Tokio](https://tokio.rs), the `futures` crate and `tokio-stream`. Real projects (and the framework lessons later in this course) use those crates directly, so this lesson does too. Every Book name has a direct equivalent:

| The Book (`trpl`) | This lesson |
|---|---|
| `trpl::block_on(fut)` (older editions: `trpl::run`) | `tokio::runtime::Runtime::new()?.block_on(fut)`, or `#[tokio::main]` |
| `trpl::spawn_task` | `tokio::spawn` |
| `trpl::sleep` | `tokio::time::sleep` |
| `trpl::join`, `trpl::join!`, `trpl::join_all` | `futures::join!` (or `tokio::join!`), `futures::future::join_all` |
| `trpl::select(a, b)` (older: `trpl::race`) | `futures::future::select(pin!(a), pin!(b))`, giving an `Either` (or `tokio::select!`) |
| `trpl::yield_now` | `tokio::task::yield_now` |
| `trpl::channel` | `tokio::sync::mpsc::unbounded_channel` |
| `trpl::stream_from_iter`, `trpl::StreamExt`, `trpl::ReceiverStream` | `tokio_stream::iter`, `tokio_stream::StreamExt`, `tokio_stream::wrappers::UnboundedReceiverStream` |
| `trpl::get`, `trpl::Html` | not used: the examples simulate slow web pages with `sleep`, so they run offline and fast |

## 1. Futures and the async syntax

```rust
async fn page_title(url: &str) -> Option<String> {
    let text = fetch(url).await;      // wait here without blocking the thread
    parse_title(&text)
}
```

- An **`async fn` returns a future**: `impl Future<Output = Option<String>>`. The compiler rewrites the body into an `async` block, and an `async { … }` block is a future too.
- **Futures are lazy.** Calling `page_title(url)` runs *none* of the body; it just builds the future. If you never use it, the compiler warns: "futures do nothing unless you `.await` or poll them".
- **`.await` is postfix**, so chains read left to right: `fetch(url).await.text().await`.
- **`.await` only works inside `async` code.** The real error from exercise 1:

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> 04_systems_level\02_async_await\exercises\ex01_fix_await_outside_async.rs:20:30
   |
19 | fn make_tea() -> String {
   | ----------------------- this is not `async`
20 |     let water = boil_water().await;
   |                              ^^^^^ only allowed inside `async` functions and blocks
```

  Make `make_tea` async and the next error is laziness in action: its caller now holds a future, not a `String`, so `println!` fails with E0277 "`impl Future<Output = String>` doesn't implement `std::fmt::Display`" until you `.await` it.
- **Something must drive the futures: a runtime** (also called an executor). Rust's standard library defines the `Future` trait but ships no runtime. `main` can't simply be `async` (E0752: `main` function is not allowed to be `async`), so you start a runtime and `block_on` your top-level future, or let `#[tokio::main]` write that for you.
- **Racing:** the Book downloads two pages and keeps whichever arrives first. `select` returns `Either::Left(first)` or `Either::Right(second)`.

> 💡 **Analogy: an order slip.** Calling an `async fn` writes an **order slip** for the kitchen, but writing it cooks nothing (lazy). `.await` hands the slip in and says "wake me when it's ready". The **runtime is the kitchen**: without one, slips just pile up on the counter. `block_on` is standing at the counter until your order comes out.

▶ `cargo run -p async_await --example futures_and_syntax`

## 2. Applying concurrency with async

The same ideas as threads, with futures:

| Threads (04/01) | Async (this lesson) |
|---|---|
| `thread::spawn(\|\| …)` → `JoinHandle`, `.join()` | `tokio::spawn(async { … })` → `JoinHandle`, `.await` |
| `move \|\|` | `async move { … }` |
| `std::sync::mpsc::channel`, blocking `recv()` | `tokio::sync::mpsc::unbounded_channel`, `recv().await` |
| join two threads | `join!(fut1, fut2)`: two futures on **one** task |

```rust
let (tx, mut rx) = mpsc::unbounded_channel();
let sender = async move {          // `move`: tx is dropped when this finishes
    for word in ["hi", "from", "the", "future"] {
        tx.send(word.to_string()).unwrap();
        sleep(Duration::from_millis(5)).await;
    }
};
let receiver = async {
    while let Some(word) = rx.recv().await { println!("received '{word}'"); }
};
futures::join!(sender, receiver);  // run both concurrently, wait for both
```

- `join!` polls its futures **in turn**, so their output alternates fairly. Awaiting `sender` and *then* `receiver` would instead send everything before receiving anything.
- **The loop that never ends:** `rx.recv().await` returns `None` only when every sender is dropped. Without `async move`, `tx` would live until the end of `main` and the `while let` would wait forever. Clone `tx` for several producers.
- A spawned task may outlive the current function, so borrowing a local fails just like with threads (E0373: async block may outlive the current function, but it borrows `names`). `async move` fixes it.

> 💡 **Analogy:** `join!` is one chef **stirring two pots in turn**: a bit of one, a bit of the other, until both are done. `tokio::spawn` is putting an order on the **board** for any free cook to pick up. The channel is the **order rail**: tickets clip on at one end and come off at the other, and the rail closes when every waiter has gone home (all senders dropped).

▶ `cargo run -p async_await --example concurrency_with_async`

## 3. Working with any number of futures

| Tool | Waits for | Notes |
|---|---|---|
| `join!(a, b, c)` | all of them | the number is fixed in your code; outputs may differ in type |
| `join_all(vec)` | all of them | any number, but they must all have **one** type; results keep the input order |
| `select(a, b)` / `select_all(vec)` | the **first** to finish | the others are dropped, which cancels them |
| `yield_now().await` | nothing: it just lets others run | for long work with no other `.await` in it |

Every `async` block has its own anonymous type, so different blocks can't share a `Vec`. The real error from exercise 2 (abridged):

```text
error[E0308]: mismatched types
  --> 04_systems_level\02_async_await\exercises\ex02_fix_mismatched_futures.rs:39:30
   |
21 |     let fast = async move {
   |                ---------- the expected `async` block
...
27 |     let slow = async move {
   |                ---------- the found `async` block
...
39 |     let futures = vec![fast, slow, printer];
   |                              ^^^^ expected `async` block, found a different `async` block
   |
   = note: no two async blocks, even if identical, have the same type
   = help: consider pinning your async block and casting it to a trait object
```

The fix is a list of **trait objects**: `Vec<Pin<Box<dyn Future<Output = ()>>>>`, filled with `Box::pin(fast)` and so on (`Pin` is section 5).

- **Futures only switch at `.await`.** A future that does long work without awaiting **starves** the others: they can't even start. Sprinkle `yield_now().await` into long loops, or move the work off to a thread (section 6).
- **Building your own abstractions:** the Book writes `timeout(future, duration)` from two pieces it already has: race the future against `sleep(duration)`. The example builds it the same way; Tokio ships a ready-made `tokio::time::timeout`.

> 💡 **Analogy:** `join_all` is a waiter who serves the table only when **every** plate is ready. `select` is waiting for **whichever taxi arrives first** and waving the others away. A future that never awaits is a child who **won't get off the slide**: nobody else gets a turn until they do. `yield_now` is "OK, your go".

▶ `cargo run -p async_await --example any_number_of_futures`

## 4. Streams: futures in sequence

A **stream** is the async version of an iterator: `stream.next().await` returns `Some(item)` when the next item is ready, and `None` at the end.

```rust
let mut messages = pin!(get_messages().timeout(Duration::from_millis(30)));
while let Some(result) = messages.next().await {
    match result {
        Ok(message) => println!("{message}"),
        Err(reason) => println!("problem: {reason}"), // late, but the stream carries on
    }
}
```

- The adapters (`map`, `filter`, `take`, `timeout`, `throttle`, `merge`, …) come from a `StreamExt` trait, which you must import.
- `next()` needs the stream to be `Unpin`; a stream returned as `impl Stream` may not be, so `pin!` it first.
- A channel's receiver becomes a stream with `UnboundedReceiverStream::new(rx)`; `tokio::time::interval` becomes one with `IntervalStream`.
- **Two `StreamExt` traits exist.** `tokio_stream::StreamExt` (what `trpl` re-exports, with `timeout`, `throttle`, `merge`) and `futures::StreamExt` (with `then`, `buffer_unordered`, `fold`). Their method names overlap, so importing both into one module gives E0034: multiple applicable items in scope. The example keeps them in separate modules.

> 💡 **Analogy: a sushi conveyor belt.** An iterator is a **bowl** of sushi: every piece is already there, take the next one. A stream is a **conveyor belt**: you wait, and each plate arrives when the kitchen finishes it. `timeout` is a sign saying "if no plate comes for 30 ms, shout"; `merge` is two belts joining into one; `throttle` lets at most one plate past every 20 ms.

▶ `cargo run -p async_await --example streams` (the first half is the Book's section)

## 5. A closer look at the traits for async

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
pub enum Poll<T> { Ready(T), Pending }
```

- **`poll`** means "make as much progress as you can **without waiting**". It returns `Ready(value)`, or `Pending` after arranging for **`cx.waker()`** to be called when it's worth polling again (a timer fires, data arrives). A runtime is, at its heart, a loop that polls futures whose wakers have fired. `.await` compiles to exactly that: poll, and on `Pending` pause here and give control back.
- **`Pin<&mut Self>`**: the state machine behind an `async` block can hold a reference to *its own* local variable across an `.await`. Moving the future in memory would leave that reference pointing at the old spot, so once polled, a future must stay put. `Pin` wraps a pointer and promises the value behind it won't move.
- **`Unpin`** is an auto trait meaning "safe to move even when pinned". Nearly every type has it (`i32`, `String`, `Box<T>`, `Pin<&mut T>`); `async` blocks and `async fn` futures don't. APIs that poll futures through `&mut` need `Unpin`, and the error says so:

```rust
let mut fut = sum_after_a_nap();
poll_once(&mut fut); // E0277: `{async fn body of sum_after_a_nap()}` cannot be unpinned
```

- **Pin it** on the stack with `std::pin::pin!(fut)` or on the heap with `Box::pin(fut)`. A `Box<dyn Future>` without the pin hits the same E0277 ("cannot be unpinned") when you `join_all` it.
- **`Stream`** is `Future` for many values: `fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>`. `StreamExt` is an **extension trait**: a blanket impl gives its methods to every `Stream`, which is why importing it is enough.

> 💡 **Analogy:** `poll` is the runtime asking "**is it ready yet?**", and the answer is "yes, here" or "not yet". The **waker is a buzzer** the kitchen gives you, so instead of asking every second, the runtime waits for the buzz. **Pin** is a cake in the oven: the tin must not be moved while it bakes, because the cake is stuck to it. **Unpin** is a sticker saying "fine to move any time", which almost everything has, except half-baked futures.

▶ `cargo run -p async_await --example future_trait_and_poll` and `cargo run -p async_await --example pin_and_boxed_futures`

## 6. Futures, tasks and threads

| Thing | What it is | Created by |
|---|---|---|
| **Future** | a value that can be polled until done | `async fn`, `async { }` |
| **Task** | a top-level future the runtime schedules on its own | `tokio::spawn` |
| **Thread** | the OS's unit; the runtime's worker threads run many tasks each | `thread::spawn`, the runtime |

- **Tasks are cheap.** 10,000 tasks that each sleep 50 ms finish in roughly the time of *one* sleep (well under a second even in a debug build), on a handful of threads.
- **Tokio's default runtime is multi-threaded with work stealing:** an idle worker thread takes tasks from a busy one, so a task may resume on a *different thread* after any `.await`. That's why `tokio::spawn` requires the future to be **`Send`**. Holding a non-`Send` value (an `Rc`, a `RefCell` borrow, a std `MutexGuard`) across an `.await` breaks that. The real error from exercise 3 (abridged; note it has no error code):

```text
error: future cannot be sent between threads safely
   --> 04_systems_level\02_async_await\exercises\ex03_fix_not_send_future.rs:25:22
    |
 25 |           handles.push(tokio::spawn(async move {
    |  ______________________^
 26 | |             let mut count = visits.lock().unwrap();
 27 | |             *count += 1;
 28 | |             sleep(Duration::from_millis(10)).await; // pretend to save it somewhere
 29 | |             println!("task {id} recorded visit number {}", *count);
 30 | |         }));
    | |__________^ future created by async block is not `Send`
    |
    = help: within `{async block@ex03_fix_not_send_future.rs:25:35}`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, i32>`
note: future is not `Send` as this value is used across an await
   --> 04_systems_level\02_async_await\exercises\ex03_fix_not_send_future.rs:28:46
    |
 26 |             let mut count = visits.lock().unwrap();
    |                 --------- has type `std::sync::MutexGuard<'_, i32>` which is not `Send`
 27 |             *count += 1;
 28 |             sleep(Duration::from_millis(10)).await; // pretend to save it somewhere
    |                                              ^^^^^ await occurs here, with `mut count` maybe used later
```

- **Blocking code in async freezes everyone** sharing that worker thread: `std::thread::sleep`, big computations, blocking file reads. Use the async versions (`tokio::time::sleep`, `tokio::fs`), or hand blocking work to `tokio::task::spawn_blocking`, which runs it on a separate thread pool and gives you a future for the result.
- **They mix:** the Book's last example has a plain thread sending on an async channel while async code receives. Tokio's `send` isn't async, so threads can use it directly.

**Rule of thumb:** lots of waiting → async tasks; heavy computation → threads or rayon (04/01); both → async, with `spawn_blocking` for the heavy parts.

> 💡 **Analogy:** **futures** are recipes in progress, **tasks** are orders pinned to the kitchen board, and **threads** are the chefs. Any free chef takes the next order that's ready to continue, even one another chef started (work stealing), so everything in an order must be safe to hand between chefs (`Send`). A chef who stops to knead dough for an hour (blocking code) leaves all their orders waiting; that job belongs in the back room (`spawn_blocking`).

▶ `cargo run -p async_await --example futures_tasks_threads`

## Beyond the Book: futures as compiler-generated state machines

The compiler turns every `async fn` into an **enum**: one variant per `.await` point, holding exactly the variables still needed there, plus a `poll` that resumes from the current variant. The example writes this by hand for a two-step `async fn` and runs both on a 20-line executor built from `std::task::Wake` and `thread::park`: same polls, same result, and the same size (48 bytes on a 64-bit machine). No heap allocation, no thread: that's why futures are so cheap, and why the size of a future is the size of its biggest state.

> 💡 **Analogy: a bookmark in a recipe.** The state machine is a bookmark that remembers which step you're on ("waiting for the oven") and what's already in the bowls. Each `poll` opens the recipe at the bookmark, does as much as it can, and moves the bookmark along.

▶ `cargo run -p async_await --example future_trait_and_poll`

## Beyond the Book: `Pin`, `Unpin` and boxed futures

`Pin<Box<dyn Future<Output = T> + Send + 'a>>` is so common that the `futures` crate names it `BoxFuture<'a, T>`, with `.boxed()` to make one (and `LocalBoxFuture` / `.boxed_local()` for futures that aren't `Send`). You need a boxed future when:

| Situation | Why | Example |
|---|---|---|
| A `Vec` of different futures | each async block has its own type | `join_all(vec![Box::pin(a), Box::pin(b)])` |
| Returning different futures from `if`/`else` | a function returns one type | `fn greet(..) -> BoxFuture<'_, String>` |
| A recursive `async fn` | the future would contain itself: infinite size (E0733: recursion in an async fn requires boxing) | `Box::pin(countdown(n - 1)).await` |
| A trait method returning a future, used as `dyn` | next section | `fn get(&self) -> BoxFuture<'_, Option<String>>` |

`+ Send` matters for `tokio::spawn`; `'a` says what the future borrows (a future borrowing `name` can't outlive `name`).

> 💡 **Analogy:** `Box::pin` puts the baking cake in a **travel oven**: you can carry the oven anywhere, and the cake inside never moves. Every travel oven looks the same from outside, whatever is baking in it, so a shelf of them (a `Vec`) is no problem.

▶ `cargo run -p async_await --example pin_and_boxed_futures`

## Beyond the Book: `async fn` in traits, and `dyn`

Since Rust 1.75 you can write `async fn` in a trait, and it works with generics (`impl Weather`, `T: Weather`). But such a trait isn't usable as `dyn Trait`: each implementation's future has a different hidden type and size, so there's no single vtable entry.

```rust
let service: Box<dyn Weather> = Box::new(FakeWeather);
// E0038: the trait `Weather` is not dyn compatible
//        ...because method `temperature` is `async`
```

Two fixes, both producing one concrete return type:

- **By hand:** `fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Option<String>>`, with a body of `Box::pin(async move { … })`.
- **`#[async_trait]`** (the `async-trait` crate) on the trait and every impl: you keep writing `async fn`, and the macro writes the boxing.

Native `async fn` in traits also doesn't let callers demand that the returned future be `Send`. For traits used with `tokio::spawn`, the boxed versions (which say `+ Send`) are still the common choice.

> 💡 **Analogy:** a job advert says "must be able to cook". Every chef brings their dish in a **differently shaped container**, so a delivery van with fixed slots (a vtable) can't carry them. Boxing puts every dish in the **same standard box**, and `async_trait` is a helper who does the boxing for you.

▶ `cargo run -p async_await --example async_traits_and_shared_state`

## Beyond the Book: sharing `Arc<dyn Trait + Send + Sync>` across `tokio::spawn`

Real services share one database pool, cache or client between many tasks: `Arc<dyn Store>`, where `trait Store: Send + Sync`. Each task gets its own `Arc::clone` (`tokio::spawn` needs `'static`), and the `Send + Sync` supertraits make the trait object safe to share. Leave them off and `tokio::spawn` refuses ("future cannot be sent between threads safely … has type `Arc<dyn SomeTrait>` which is not `Send`").

- A **std `Mutex`** is fine inside async code as long as the guard is dropped before any `.await` (exercise 3). It's faster than the async one.
- A **`tokio::sync::Mutex`** (`lock().await`) is for when you really must hold the lock across an `.await`.

> 💡 **Analogy:** one **recipe book on a stand** in the middle of the kitchen. Every chef has a note saying where it is (an `Arc` clone), and the book carries a sticker saying "safe for many chefs to read at once" (`Sync`). Without the sticker, the head chef won't let it leave the counter.

▶ `cargo run -p async_await --example async_traits_and_shared_state`

## Beyond the Book: Tokio in practice

| Tool | Does |
|---|---|
| `#[tokio::main]` | builds a runtime and `block_on`s your `async fn main` (`flavor = "current_thread"` for one thread) |
| `tokio::time::timeout(dur, fut)` | `Ok(output)`, or `Err(Elapsed)` and the future is dropped |
| `tokio::select! { … }` | waits on several branches, runs the first ready one, drops the rest; `biased;` checks them in order |
| `JoinSet` | a group of spawned tasks: `join_next().await`, `abort_all()` |
| `handle.abort()` | cancel one spawned task |

**Cancellation is just dropping.** A future that is dropped stops at the `.await` it was paused on; its destructors run, and the code after that `.await` never does. `timeout`, `select!` and `select` cancel the losers this way.

**Cancellation safety** asks: if this future is dropped half-way, is anything lost? `rx.recv()` is safe: a message is either returned or still in the channel. A future that has collected half a batch into a local `Vec` is not: dropping it throws the half batch away. In a `select!` loop, a future written directly inside `select!` is **recreated every round**, and dropped every time another branch wins. The example measures it: recreated each round, a `next_batch` future receives almost none of 12 messages; pinned once outside the loop and polled as `&mut batch`, it receives all 12. Tokio's docs mark each method as cancel safe or not.

> 💡 **Analogy:** `select!` is waiting at a stop served by **three buses**: you take the first that comes and stop waiting for the others. **Cancellation safety** is a bakery queue: if you give up and leave, do you lose the bread you'd already paid for? `recv()` keeps your bread behind the counter for next time. A recreated `next_batch` drops its half-full bag on the floor every time you leave the queue.

▶ `cargo run -p async_await --example tokio_in_practice`

## Beyond the Book: async streams with the `futures` crate

| `futures::StreamExt` / `futures::stream` | Does |
|---|---|
| `then(async_fn)` | an async `map`: one future per item, **one at a time** |
| `buffer_unordered(n)` / `buffered(n)` | run up to `n` futures **at once**; results as they finish / in input order |
| `stream::unfold(state, f)` | build a stream from a state, e.g. a paginated API with a "next page" cursor |
| `FuturesUnordered` | a set of futures that is a stream of their results, in finishing order |
| `fold`, `take_while`, `chunks`, `stream::select` | the iterator favourites, async (`select` merges two streams) |

`buffer_unordered` is the everyday way to say "download these 1,000 URLs, at most 10 at a time".

> 💡 **Analogy:** `then` is a bakery with **one oven**: each loaf goes in when the last comes out. `buffer_unordered(3)` has **three ovens**: up to three loaves bake at once, and each comes out when it's done, not in the order it went in.

▶ `cargo run -p async_await --example streams` (the second half)

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `.await` in a plain `fn` | E0728 | Make the function `async`, and `.await` it where it's called |
| `async fn main()` with no runtime | E0752 | `#[tokio::main]`, or `Runtime::new()?.block_on(…)` |
| Called an `async fn` and never awaited it | warning: unused implementer of `Future`; nothing happens | `.await` it, or `tokio::spawn` it |
| Printed the result of an `async fn` without `.await` | E0277 `impl Future<Output = …>` doesn't implement `Display` | Add `.await` |
| Different `async` blocks in one `Vec` | E0308 expected `async` block, found a different `async` block | `Vec<Pin<Box<dyn Future<Output = T>>>>` and `Box::pin` |
| `Box<dyn Future>` in `join_all` or `select` | E0277 cannot be unpinned | `Box::pin(…)`, or `pin!` |
| `tokio::spawn(async { uses(&local) })` | E0373 async block may outlive the current function | `async move` |
| `Rc`, `RefCell` or a std `MutexGuard` held across `.await` in a spawned task | future cannot be sent between threads safely | `Arc`; drop the guard before the `.await`; or `tokio::sync::Mutex` |
| `while let Some(m) = rx.recv().await` never ends | hangs | Drop every sender: `async move`, `drop(tx)` |
| `std::thread::sleep` or a long loop in async code | every other task freezes | `tokio::time::sleep`, `spawn_blocking`, or `yield_now().await` |
| A recursive `async fn` | E0733 | `Box::pin(recurse(…)).await` |
| `dyn Trait` for a trait with `async fn` | E0038 not dyn compatible | Return `BoxFuture`, or `#[async_trait]` |
| A stateful future written inside a `select!` loop | data silently lost | Pin it once before the loop; prefer cancel-safe operations |
| `futures::StreamExt` and `tokio_stream::StreamExt` in one module | E0034 multiple applicable items in scope | One per module |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p async_await --example futures_and_syntax
cargo run -p async_await --example concurrency_with_async
cargo run -p async_await --example any_number_of_futures
cargo run -p async_await --example streams
cargo run -p async_await --example future_trait_and_poll
cargo run -p async_await --example pin_and_boxed_futures
cargo run -p async_await --example futures_tasks_threads
cargo run -p async_await --example async_traits_and_shared_state
cargo run -p async_await --example tokio_in_practice
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_await_outside_async` | Make it compile (E0728, then E0277) | `cargo run -p async_await --features exercises --bin ex01_fix_await_outside_async` |
| 2 | `ex02_fix_mismatched_futures` | Make it compile (E0308), box and pin | `cargo run -p async_await --features exercises --bin ex02_fix_mismatched_futures` |
| 3 | `ex03_fix_not_send_future` | Make it compile ("future cannot be sent between threads safely") | `cargo run -p async_await --features exercises --bin ex03_fix_not_send_future` |
| 4 | `ex04_concurrent_downloads` | `join_all` + `timeout`, and a race, pass 3 tests | `cargo test -p async_await --features exercises --bin ex04_concurrent_downloads` |
| 5 | `ex05_stream_pipeline` | `filter`/`map`, `take_while`/`fold`, `unfold`, pass 3 tests | `cargo test -p async_await --features exercises --bin ex05_stream_pipeline` |
| 6 | `ex06_handwritten_future` | Write `poll` yourself, pass 3 tests | `cargo test -p async_await --features exercises --bin ex06_handwritten_future` |

The tests use `#[tokio::test(start_paused = true)]`: Tokio's clock is frozen and jumps straight to the next timer whenever every task is waiting, so a 5-second `sleep` takes no real time, and `elapsed()` is exact. That's how you test timeouts without slow or flaky tests. If exercise 6 hangs, you returned `Pending` without waking.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p async_await --features solutions --bins`).

**4. The mini project** is an **async price checker**: it asks six simulated shops for the price of a kettle at the same time.

```bash
cargo run -p async_await                 # look up a kettle
cargo run -p async_await -- toaster      # something else
cargo test -p async_await                # 8 unit tests, on the paused clock
```

| Part of `src/lib.rs` | Chapter idea |
|---|---|
| `trait Shop: Send + Sync` returning `BoxFuture` | async in a `dyn` trait, by hand |
| `quote_with_retry` | `tokio::time::timeout` on each attempt, retries with exponential backoff |
| `compare_prices` | one `tokio::spawn`ed task per shop in a `JoinSet`, sharing `Arc<dyn Shop>`; a `Semaphore` allows at most 4 requests at once; `tokio::select!` against `sleep_until(deadline)` cancels the stragglers with `abort_all` |
| `first_quote` | a race with `FuturesUnordered`: the first success wins, and returning drops (cancels) the rest |

The six shops show every outcome: a normal answer, a flaky shop saved by a retry, one so slow the deadline cancels it, one out of stock, and one always broken. The whole comparison takes about the deadline (250 ms), not the sum of all the waits.

> 💡 **Analogy:** a shopping helper with **six phones**. They ring every shop at once and put each on speaker. If a shop doesn't pick up quickly, they hang up and try again a little later. They only use four lines at a time, and when the kettle has to be bought by a set time, they hang up on anyone still dithering. Then they tell you the cheapest price, and which shop answered first.

## Checklist

- [ ] [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html)
- [ ] [17.2 Applying Concurrency with Async](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html)
- [ ] [17.3 Working With Any Number of Futures](https://doc.rust-lang.org/book/ch17-03-more-futures.html)
- [ ] [17.4 Streams: Futures in Sequence](https://doc.rust-lang.org/book/ch17-04-streams.html)
- [ ] [17.5 A Closer Look at the Traits for Async](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html)
- [ ] [17.6 Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html)
- [ ] Beyond the Book: futures as compiler-generated state machines; `poll`
- [ ] Beyond the Book: `Pin`/`Unpin` and boxed futures (`Pin<Box<dyn Future + Send + '_>>`)
- [ ] Beyond the Book: `async fn` in traits, and why `dyn` needs boxing or the `async_trait` crate
- [ ] Beyond the Book: sharing state with `Arc<dyn Trait + Send + Sync>` across `tokio::spawn`
- [ ] Beyond the Book: Tokio in practice: `tokio::select!`, timeouts, cancellation safety
- [ ] Beyond the Book: async streams with the `futures` crate
- [ ] Exercises 1–6 and the price checker done without looking at the solutions

## Recap

- An `async fn` returns a **future**: lazy, it does nothing until awaited, and a **runtime** (Tokio here) drives it. `.await` only works in async code.
- `join!`/`join_all` run futures concurrently and wait for all; `select`/`select_all` wait for the first and cancel the rest; `yield_now` stops one future starving the others.
- **Streams** are async iterators: `next().await`, with adapters from a `StreamExt` trait.
- Under the hood: `Future::poll` returns `Ready` or `Pending`, a **waker** says when to poll again, the compiler turns `async` code into a **state machine**, and **`Pin`** keeps self-referential futures from moving.
- **Tasks** are futures the runtime schedules across worker **threads**, so spawned futures must be `Send`; blocking work goes to `spawn_blocking`.
- Beyond the Book: box futures (`BoxFuture`) for `dyn` traits, recursion and mixed lists; share `Arc<dyn Trait + Send + Sync>`; use `timeout`, `select!` and `JoinSet`, and keep cancellation safety in mind; `buffer_unordered` limits how much runs at once.

**In one sentence for a child:** an async program is one clever chef with lots of pots, who never stands staring at a pot that needs to boil, but goes and does something else until its timer pings.

**Next:** [03 Advanced Features](../03_advanced_features/README.md) (Book chapter 20)
