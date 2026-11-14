<!-- hand-written lesson: gen_course must not overwrite -->
# Tokio

**Docs:** [Tokio tutorial](https://tokio.rs/tokio/tutorial) · [tokio on docs.rs](https://docs.rs/tokio/1) · [Tokio topics: shutdown, tracing, bridging](https://tokio.rs/tokio/topics)
· Companion crates: [`tokio-util`](https://docs.rs/tokio-util) (`CancellationToken`, `TaskTracker`, codecs), [`tokio-stream`](https://docs.rs/tokio-stream) (stream adapters), [`futures`](https://docs.rs/futures) (`SinkExt`)
· Background: [The Async Book](https://rust-lang.github.io/async-book/)

Run it: `cargo run -p tokio_runtime` (a chat server on 127.0.0.1:6142), then `cargo run -p tokio_runtime -- client` in two other terminals  ·  Test it: `cargo test -p tokio_runtime`

[04/02 Async and Await](../../04_systems_level/02_async_await/README.md) introduced futures, `.await`, `tokio::spawn`, `join!`, `select!` and a first look at Tokio. [01 Axum](../01_axum/README.md) then ran on Tokio without you having to think about it. This lesson opens the box. After it you can:

- choose and configure a runtime (multi-thread or current-thread, `#[tokio::main]` options, `Builder`), and keep blocking work off it;
- manage tasks: spawn, join, abort, group them in a `JoinSet`, move CPU work to `spawn_blocking`;
- pick the right sync primitive (`Mutex`, `RwLock`, `Semaphore`, `Notify`, `OnceCell`) and the right channel (`mpsc`, `oneshot`, `broadcast`, `watch`);
- use `select!`, timeouts and intervals without losing data to cancellation;
- read and write sockets with `AsyncRead`/`AsyncWrite`, frame messages with a codec, and work with streams;
- shut down gracefully with `CancellationToken` and `TaskTracker`, and test timing code instantly with a paused clock.

This lesson uses Tokio 1 (1.54.1 in `Cargo.lock`), tokio-util 0.7 and tokio-stream 0.1.

> 💡 **The big picture: a restaurant kitchen.** In 04/02 one clever chef juggled many pots. Tokio is the **whole restaurant**. The **runtime** is the kitchen: a team of **chefs** (worker threads), an **order board** (the queue of tasks), a **kitchen timer** (the time driver) and a **bell** that rings when a delivery arrives at the back door (the I/O driver). Every **task** is an order on the board; any free chef picks up the next order that's ready. Slow, heavy jobs like kneading dough go to a **back room** (`spawn_blocking`) so the chefs at the stoves never stop. The kitchen talks through an **order rail** (`mpsc`), **sealed reply notes** (`oneshot`), a **loudspeaker** (`broadcast`) and a **specials board** (`watch`). At closing time a **bell** tells everyone to finish what's on the stove (`CancellationToken`), and the manager counts every chef out of the door (`TaskTracker`).

## 1. The runtime

Rust's standard library defines `Future` but has no runtime. Tokio's runtime has three parts: a **scheduler** that polls tasks, a **time driver** for `sleep`/`timeout`/`interval`, and an **I/O driver** that asks the operating system which sockets are ready (epoll on Linux, IOCP on Windows).

| | Multi-thread (default) | Current-thread |
|---|---|---|
| Threads | a pool of workers, one per CPU core by default | just the thread that calls `block_on` |
| Scheduling | work stealing: an idle worker takes tasks from a busy one | one queue, one thread |
| Spawned futures must be | `Send + 'static` | `Send + 'static` with `tokio::spawn` (`!Send` futures: `LocalSet`) |
| Good for | servers, most programs | tests, small tools, embedding in a sync program |

`#[tokio::main]` is a macro that builds a runtime and `block_on`s your `async fn main`:

```rust
#[tokio::main]                              // Builder::new_multi_thread().enable_all()
#[tokio::main(worker_threads = 2)]          // ...with .worker_threads(2)
#[tokio::main(flavor = "current_thread")]   // Builder::new_current_thread().enable_all()

let rt = tokio::runtime::Builder::new_multi_thread()   // the same thing by hand
    .worker_threads(4)
    .thread_name_fn(|| format!("worker-{}", NEXT.fetch_add(1, Ordering::Relaxed)))
    .enable_all()            // the time and I/O drivers: without them sleep/TcpStream panic
    .build()?;
let answer = rt.block_on(async { 42 });
```

Real output of the example:

```text
current_thread: 8 tasks ran on {"main"}
multi_thread: 4 workers; 8 tasks ran on 4 different threads
blocked runtime: a 10 ms timer finished after about 100 ms
with spawn_blocking: the timer finished after under 50 ms: true
```

The last two lines are the most important rule in async Rust: **never block a runtime thread.** A task that calls `std::thread::sleep`, does a long computation or a blocking file read stops every other task on that thread, because Tokio can only switch tasks at an `.await`. Building a runtime inside another (calling `block_on` from async code) panics for the same reason.

> 💡 **Analogy: the kitchen and its chefs.** A current-thread runtime is a food truck with **one cook**: fine for a small queue. The multi-thread runtime is a big kitchen with a **team of chefs** who help each other: a chef with nothing to do takes an order from a busy colleague's pile (work stealing). The golden rule in both: a chef who stands still waiting for dough to rise, instead of setting a timer and cooking something else, holds up every order on their pile.

▶ `cargo run -p tokio_runtime --example runtime`

## 2. Tasks

```rust
let handle: JoinHandle<u32> = tokio::spawn(async { 6 * 7 });
let answer = handle.await?;                   // Result<u32, JoinError>

let mut set = JoinSet::new();                 // a group of tasks
set.spawn(async { "fast" });
while let Some(result) = set.join_next().await { … }   // in FINISHING order

let primes = tokio::task::spawn_blocking(|| count_primes(200_000)).await?;  // CPU work
```

Real output:

```text
1. the task returned Ok(42)
2. panicked task -> is_panic = true
3. aborted task -> is_cancelled = true
4. detached task still finished: true
5. JoinSet finishing order: ["fast", "medium", "slow"]
6. spawn_blocking counted 17984 primes; the ticker kept running: true
```

- A **task** is a future the runtime owns and polls on its own. It is cheap: one small allocation holding its future, not a thread with its own stack.
- A **panic** stays inside its task; awaiting the handle gives `JoinError` with `is_panic()`.
- **`abort()`** cancels a task at its next `.await`. **Dropping** a `JoinHandle` does *not* stop the task; it carries on, detached. Dropping a `JoinSet` *does* abort every task in it.
- **`spawn_blocking`** runs a closure on a separate pool of threads made for blocking work (up to 512 threads by default, far more than the workers). Use it for CPU-heavy work, blocking libraries and `std::fs`.
- `tokio::spawn` needs the future to be **`'static`** (it may outlive the function that spawned it, so move owned data in) and **`Send`** (it may resume on another worker). Moving one `Sender` into tasks in a loop fails with the error from exercise 1:

```text
error[E0382]: use of moved value: `tx`
  --> 06_framework_mastery\02_tokio\exercises\ex01_fix_moved_sender.rs:26:22
   |
23 |     let (tx, mut rx) = mpsc::channel::<String>(16);
   |          -- move occurs because `tx` has type `tokio::sync::mpsc::Sender<String>`, which does not implement the `Copy` trait
24 |
25 |     for id in 1..=3 {
   |     --------------- inside of this loop
26 |         tokio::spawn(async move {
   |                      ^^^^^^^^^^ value moved here, in previous iteration of loop
```

and an `Rc` (not thread-safe) inside a spawned task fails with exercise 2's error, which has **no error code**:

```text
error: future cannot be sent between threads safely
   --> 06_framework_mastery\02_tokio\exercises\ex02_fix_rc_across_await.rs:26:22
    |
 26 |           handles.push(tokio::spawn(async move {
    |  ______________________^
 27 | |             sleep(Duration::from_millis(10)).await; // pretend to look something up
 28 | |             format!("task {i} greets {}", names[i])
 29 | |         }));
    | |__________^ future created by async block is not `Send`
    |
    = help: within `{async block@ex02_fix_rc_across_await.rs:26:35}`, the trait `Send` is not implemented for `Rc<Vec<String>>`
note: captured value is not `Send`
   --> 06_framework_mastery\02_tokio\exercises\ex02_fix_rc_across_await.rs:28:43
```

> 💡 **Analogy: orders on the board.** `tokio::spawn` pins a new order on the board; the `JoinHandle` is your **ticket stub** to collect the dish later. Throwing the stub away doesn't cancel the order (detached); `abort()` does. A `JoinSet` is a **tray of stubs** that hands you each dish as it comes out of the kitchen. `spawn_blocking` sends the dough to the **back room**, where other staff knead it without blocking the stoves. And because any chef may pick up your order halfway through, everything in it must be safe to pass between chefs (`Send`): an `Rc` is a note only one chef may read.

▶ `cargo run -p tokio_runtime --example tasks`

## 3. Sync primitives: `tokio::sync`

| Primitive | Does | Reach for it when |
|---|---|---|
| `std::sync::Mutex` | a lock; the guard must be dropped before any `.await` | short critical sections with no `.await` (most of the time!) |
| `tokio::sync::Mutex` | a lock whose guard may be held across `.await`; FIFO-fair | you really must wait for something while holding the lock |
| `tokio::sync::RwLock` | many readers or one writer, async | read-mostly data that is held across `.await` |
| `Semaphore` | N permits; `acquire().await` waits for a free one | limiting concurrency: at most 10 downloads, 5 DB connections |
| `Notify` | wake one waiter (`notify_one`) or all (`notify_waiters`); no data | "something changed, go and look" |
| `OnceCell` | a value initialised once, by an async function | lazy global config, a client created on first use |
| `Barrier` | wait until N tasks reach the same point | tests and phased work |

Real output (on the paused clock, so times are exact):

```text
1. Mutex: every task wrote in turn: [1, 2, 3]
2. RwLock: 3 readers saw ["v1", "v1", "v1"] in 100 ms total; now v2
3. Semaphore(3): 10 jobs of 100 ms, at most 3 at once, 400 ms total
4. Notify: the waiter says "woken up"
5. OnceCell: 5 tasks got "dark mode", loaded 1 time(s)
```

The common surprise: **the std `Mutex` is usually the right one** in async code, as long as you never hold its guard across an `.await` (lesson 04/02, exercise 3). Tokio's own docs recommend it for that case, because it is faster. The mini project's name list uses a std `Mutex` for exactly this reason. Another option is to avoid sharing at all: give the data to **one task** and talk to it through a channel (the actor pattern, exercise 5).

> 💡 **Analogy: kitchen tools.** A **mutex** is the single sharp knife: one cook at a time. The std knife is quick to pass around, but you must put it down before you wait for the oven; the Tokio knife may be held while you wait, and the queue for it is fair. An **RwLock** is the recipe book: everyone can read at once, but writing in it needs it to yourself. A **semaphore** is the **three stove rings**: a fourth pan waits for a free ring. **Notify** is the bell on the pass: "ding, something's ready". A **OnceCell** is lighting the oven: the first cook to need it lights it, and everyone else just uses it.

▶ `cargo run -p tokio_runtime --example sync_primitives`

## 4. Channels, and when to use each

| Channel | Shape | Use it for | Back-pressure |
|---|---|---|---|
| `mpsc::channel(n)` | many senders → one receiver, a queue | work queues, the actor pattern | `send().await` waits when full |
| `mpsc::unbounded_channel()` | the same, no limit | only when the sender can't wait (sync code) | none: memory can grow forever |
| `oneshot::channel()` | one value, once | the reply to one request | n/a |
| `broadcast::channel(n)` | many senders → **every** receiver gets every message | chat rooms, events, shutdown notices | a slow receiver gets `Lagged(missed)` and skips ahead |
| `watch::channel(v)` | one value; receivers see the **latest** | configuration, status, "current state" | n/a: old values are overwritten |

Real output (paused clock):

```text
1. mpsc::channel(2)
   [  0 ms] sent 1
   [  0 ms] sent 2
   [  0 ms] sent 3
   [100 ms] handled 1
   [100 ms] sent 4
   [200 ms] handled 2
   ...
   recv() returned None: every sender is gone
2. oneshot
   reply: Ok("42 is the answer")
   sender dropped: Err(RecvError(()))
3. broadcast::channel(2)
   fast got Ok(1)  ...  fast got Ok(4)
   slow lagged: missed 2
   slow got 3
   slow got 4
4. watch
   the observer saw ["ready", "stopping"]
```

- **Back-pressure** in part 1: the consumer took 1 at once, 2 and 3 filled the channel, and "sent 4" waited until there was room. A bounded channel stops a fast producer from burying a slow consumer.
- `recv()` returns `None` only when **every** `Sender` is dropped, including the one `main` still holds (exercise 1's second step).
- The **actor pattern**: one task owns the data; others send it commands over `mpsc`, each with a `oneshot::Sender` for the reply. No locks at all.

> 💡 **Analogy: how the kitchen talks.** `mpsc` is the **order rail**: waiters clip tickets on, one cook takes them off in order, and when the rail is full the waiters wait. `oneshot` is a **sealed note** with one answer on it. `broadcast` is the **loudspeaker**: everyone hears every announcement, and someone who wandered off for too long is told "you missed 2 announcements". `watch` is the **specials board**: nobody cares what yesterday's special was, only what it says now.

▶ `cargo run -p tokio_runtime --example channels`

## 5. `select!` and cancellation safety

```rust
tokio::select! {
    biased;                                        // optional: check branches in order
    _ = token.cancelled() => return,               // first to finish wins...
    Some(msg) = rx.recv() => handle(msg),          // ...a non-matching pattern disables a branch
    line = lines.next(), if reading => …,          // `, if cond`: a precondition
    else => break,                                 // when every branch is disabled
}
```

- `select!` polls every branch, runs the **first** one to complete, and **drops** the others. When several are ready at once it picks **randomly** (for fairness); `biased;` makes it go top to bottom.
- **Dropping a future is cancelling it.** So the question for every branch is: if this future loses the race, is anything lost? That is **cancellation safety**. Tokio's `select!` docs list them:

| Cancel safe (nothing lost) | Not cancel safe |
|---|---|
| `mpsc::Receiver::recv`, `broadcast::Receiver::recv`, `watch::Receiver::changed` | `AsyncReadExt::read_exact`, `read_to_end`, `read_to_string` (data already read is lost) |
| `TcpListener::accept`, `AsyncReadExt::read`, `AsyncWriteExt::write` | `AsyncWriteExt::write_all` (you don't know how much was written) |
| `StreamExt::next` (so `Framed::next`), `Lines::next_line` | `Mutex::lock`, `RwLock::read`/`write`, `Semaphore::acquire`, `Notify::notified` (you lose your place in the queue) |

The example shows it: a `biased` select picked another branch, yet the message was still in the channel afterwards (`3. still in the channel: "message"`).

- To keep **one** future across loop iterations (a deadline, a half-finished job), create it **before** the loop and poll it by `&mut`. That needs it **pinned**. The real error from exercise 3:

```text
error[E0277]: `PhantomPinned` cannot be unpinned
  --> 06_framework_mastery\02_tokio\exercises\ex03_fix_unpinned_sleep.rs:34:9
   |
34 | /         tokio::select! {
35 | |             _ = &mut deadline => break,
36 | |             Some(n) = rx.recv() => got.push(n),
37 | |         }
   | |         ^
   | |_________within `(PhantomData<&()>, PhantomPinned)`, the trait `Unpin` is not implemented for `PhantomPinned`
   |
   = note: consider using the `pin!` macro
   = note: required for `Sleep` to implement `Unpin`
   = note: required for `&mut Sleep` to implement `Future`
```

  The fix is `tokio::pin!(deadline);` (or `std::pin::pin!`): pinning promises the timer won't move in memory once polled.

> 💡 **Analogy: waiting for whichever comes first.** `select!` is standing at the pass with three things you're waiting for: the timer, a new order, the closing bell. You deal with whichever happens first and **stop waiting** for the others. **Cancellation safety** asks whether stopping waiting loses anything. Walking away from the order rail is fine: the ticket stays on the rail (`recv`). Walking away halfway through counting out 12 eggs is not: the eggs you'd already counted go on the floor (`read_exact`). **Pinning** the deadline is sticking the kitchen timer to the wall, so you look at the same timer every time instead of starting a new one.

▶ `cargo run -p tokio_runtime --example select_and_time`

## 6. Time: sleep, timeouts, deadlines and intervals

| Tool | Does |
|---|---|
| `sleep(dur)`, `sleep_until(instant)` | wait without blocking the thread |
| `timeout(dur, fut)` | `Ok(output)`, or `Err(Elapsed)` and the future is dropped |
| `interval(period)` | `tick().await` every period; the **first tick is immediate** |
| `MissedTickBehavior` | what an interval does after a busy spell: `Burst` (default), `Delay`, `Skip` |
| `tokio::time::Instant` | like `std::time::Instant`, but follows the paused test clock |

Real output (paused clock):

```text
1. [ 20 ms] select! picked the 20 ms sleep
4. [ 50 ms] timeout: slow -> Err(Elapsed(())), quick -> Ok(7)
5. [200 ms] collected [0, 1, 2] before the deadline
6. Burst: ticks at [0, 100, 350, 350, 400] ms
6. Delay: ticks at [0, 100, 350, 450, 550] ms
6. Skip: ticks at [0, 100, 350, 400, 500] ms
```

In part 6 the loop was busy for 250 ms after the second tick, missing the ticks due at 200 and 300 ms. **Burst** fires the missed ticks at once and then keeps the original schedule; **Delay** restarts the schedule from the late tick; **Skip** drops the missed ticks and waits for the next slot on the original grid.

> 💡 **Analogy: kitchen timers.** `sleep` is setting a timer and cooking something else until it rings. `timeout` is "if the delivery isn't here in 30 minutes, cancel it". An **interval** is the clock that chimes every hour; if you were in the cellar and missed two chimes, **Burst** chimes them all as soon as you're back, **Delay** starts counting hours from now, and **Skip** just waits for the next o'clock.

## 7. Async I/O: sockets, `AsyncRead`/`AsyncWrite`, codecs

```rust
let listener = TcpListener::bind("127.0.0.1:0").await?;
let (mut socket, peer) = listener.accept().await?;
let (mut reader, mut writer) = socket.split();          // borrowed halves (same task)
tokio::io::copy(&mut reader, &mut writer).await?;       // an echo server

let (reader, writer) = stream.into_split();             // owned halves (separate tasks)
let mut lines = BufReader::new(reader).lines();
while let Some(line) = lines.next_line().await? { … }
```

- `AsyncRead` and `AsyncWrite` are the async versions of `Read` and `Write`. Their useful methods live in the **extension traits** `AsyncReadExt` (`read`, `read_exact`, `read_to_end`, `read_to_string`) and `AsyncWriteExt` (`write_all`, `flush`, `shutdown`), which you must import, plus `AsyncBufReadExt` for `lines()`.
- `shutdown()` closes **your** writing direction: the other side reads end-of-file, while you can still read its reply.

Real output:

```text
1. echoed back: "hello, tokio"
2. three writes came back as: "onetwothree"
3. read line by line: ["one", "two", "three"]
4. codec gave the message "framed hello"
4. codec gave the message "framed bye"
```

Line 2 is why **framing** exists: TCP delivers a stream of **bytes**, not messages. Three writes can arrive as one read, and one write as several. Every protocol marks where a message ends: a newline, a length prefix, a closing tag.

**Codecs.** `tokio_util::codec` separates "how to cut bytes into messages" from "what to do with a message". A `Decoder` turns bytes into frames, an `Encoder` turns frames into bytes, and `Framed::new(socket, codec)` wraps a socket into a `Stream` of frames (read with `next()`) and a `Sink` of frames (write with `SinkExt::send` from `futures`). Ready-made codecs: `LinesCodec` (one `String` per line, with an optional maximum length), `LengthDelimitedCodec` (a length prefix, then the bytes), `BytesCodec`. After a decode error (such as a line over the maximum length), a `Framed` stream **ends**: its next `next()` is `None`, so the chat server says why and hangs up.

> 💡 **Analogy: the delivery hatch.** A TCP socket is a **conveyor belt of loose beads**: you get all the beads, in order, but nothing tells you where one necklace ends and the next begins. **Framing** is agreeing on a rule, like "a red bead ends each necklace" (a newline) or "the first bead says how many follow" (a length prefix). A **codec** is the helper at the hatch who strings the beads into necklaces for you, and unstrings yours on the way out.

▶ `cargo run -p tokio_runtime --example async_io`

## 8. Streams with `tokio-stream`

A `Stream` is an async iterator (04/02 section 4). `tokio-stream` adds adapters through its `StreamExt` and **wrappers** that turn Tokio types into streams:

| | |
|---|---|
| `tokio_stream::iter(..)` | a stream from an iterator |
| `wrappers::ReceiverStream`, `UnboundedReceiverStream` | a channel receiver as a stream |
| `wrappers::IntervalStream`, `TcpListenerStream`, `LinesStream`, `BroadcastStream`* | ticks, connections, lines, broadcast messages (*`sync` feature) |
| `.map`, `.filter`, `.take`, `.timeout(dur)`, `.throttle(dur)`, `.chunks_timeout(n, dur)`, `.merge(other)` | adapters |
| `StreamMap` | many streams merged, each item tagged with its key |

Real output (paused clock, times from the start of each part):

```text
1. even squares: [4, 16, 36]
3. ticks at [0, 100, 200] ms
4. [ 10 ms] Ok(0)
4. [ 60 ms] Err(Elapsed(()))
4. [ 90 ms] Ok(1)
5. [ 10 ms] fast
5. [ 25 ms] slow
5. [ 40 ms] fast
6. [ 20 ms] batch [0, 1, 2]
6. [ 80 ms] batch [3]
6. [140 ms] batch [4, 5]
```

`chunks_timeout(3, 50 ms)` is the everyday "write to the database in batches of up to 3, but never make an item wait more than 50 ms". Adapters that hold a timer (`timeout`, `chunks_timeout`) aren't `Unpin`, so `tokio::pin!` them before calling `next()`, the same E0277 as exercise 3.

> 💡 **Analogy: conveyor belts again.** A stream is a **sushi belt**: plates arrive one by one, whenever the kitchen finishes them. `timeout` is a sign saying "shout if no plate comes for 50 ms". `StreamMap` is **several belts joining into one**, with a label on each plate saying which belt it came from. `chunks_timeout` is packing plates into **boxes of three**, but sending a half-full box if nothing else arrives soon.

▶ `cargo run -p tokio_runtime --example streams`

## 9. Graceful shutdown: `CancellationToken` and `TaskTracker`

```rust
let token = CancellationToken::new();
let tracker = TaskTracker::new();
for id in 1..=3 {
    let token = token.clone();
    tracker.spawn(async move {
        loop {
            tokio::select! {
                _ = token.cancelled() => break,      // closing time
                _ = every.tick() => do_one_job(),
            }
        }
        tidy_up().await;                             // flush, save, say goodbye
    });
}
tracker.close();                                     // no more tasks will be added

tokio::signal::ctrl_c().await?;                      // wait for Ctrl+C
token.cancel();                                      // tell everyone
tracker.wait().await;                                // wait until they've all finished
```

Real output (paused clock; a timer plays Ctrl+C at 200 ms):

```text
1. [200 ms] "Ctrl+C": cancel the token
   [210 ms] worker 1 stopped after 5 jobs
   [220 ms] worker 2 stopped after 3 jobs
   [230 ms] worker 3 stopped after 2 jobs
   [230 ms] tracker.wait() returned: no tasks left: true
2. after child_a.cancel(): parent false, child_b false
   after parent.cancel(): child_b true
3. run_until_cancelled: quick Some("done"), slow None
4. after the DropGuard was dropped: cancelled true
```

- **`CancellationToken`** (tokio-util): clone it into every task; `cancelled().await` completes once anyone calls `cancel()`. Tokens form a tree: `child_token()` is cancelled with its parent, but cancelling a child leaves the parent alone.
- **`run_until_cancelled(fut)`** gives `Some(output)` or `None` if the token won. **`drop_guard()`** cancels when the guard is dropped, even on an early return or a panic.
- **`TaskTracker`** (tokio-util, `rt` feature): `spawn` tasks through it, `close()` it, and `wait()` returns once it's closed and every task has finished. Unlike a `JoinSet`, it doesn't keep the results, so a long-running server doesn't pile up finished tasks' return values; and dropping it doesn't abort anything.
- `tokio::signal::ctrl_c()` completes when the user presses Ctrl+C (on Unix, `tokio::signal::unix` also handles `SIGTERM`, which is what `docker stop` and systemd send).

> 💡 **Analogy: closing time.** The `CancellationToken` is the **closing bell**: every chef hears it, finishes the dish on the stove, wipes the counter and stops. A **child token** is the bell for one section: ringing it closes the pastry corner, not the whole kitchen, but the main bell closes everything. The `TaskTracker` is the **manager by the door with a clipboard**, ticking off each chef on the way out, and only locking up when the last name is ticked.

▶ `cargo run -p tokio_runtime --example graceful_shutdown`

## 10. Testing async code

- **`#[tokio::test]`** builds a runtime for each test. The default is a **current-thread** runtime; ask for `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` when the test needs real parallelism.
- **A paused clock** (`test-util` feature, in `[dev-dependencies]` here): `#[tokio::test(start_paused = true)]` freezes time, and whenever every task is waiting, Tokio jumps straight to the next timer. A test of a 10-second timeout takes microseconds, and `elapsed()` is exact, so you can assert `start.elapsed() == Duration::from_millis(200)` to prove that work ran concurrently (exercise 4). `tokio::time::advance(dur)` moves the clock by hand. Pausing needs the current-thread runtime. The same trick works in examples: `#[tokio::main(flavor = "current_thread", start_paused = true)]`, which is why this lesson's timing examples print the same numbers every run.
- **Real sockets:** bind `127.0.0.1:0`, spawn the server, connect real clients, and put a timeout on **every** read (`ChatClient::recv` gives up after 5 s), so a bug fails the test instead of hanging it. Don't combine a paused clock with real network I/O: the clock may jump ahead while a socket is still waiting.

> 💡 **Analogy: a practice kitchen with a magic clock.** In the practice kitchen, whenever everyone is just waiting for timers, the magic clock **jumps straight to the next ring**. A cake that bakes for an hour is "done" instantly, yet the clock still reads exactly one hour later, so you can check the recipe's timings without waiting for them.

## The mini project: a TCP chat server

| File | What's in it |
|---|---|
| [`src/protocol.rs`](src/protocol.rs) | `parse` a line into `Say`, `Me`, `Who`, `Quit`…; `validate_name` (no I/O, unit-tested) |
| [`src/server.rs`](src/server.rs) | `run`: accept loop in `select!` with the token; one task per client in a `TaskTracker`; `Framed<TcpStream, LinesCodec>`; a `broadcast` channel for messages; a std `Mutex<BTreeSet>` of names freed by an RAII `NameGuard`; a `timeout` on choosing a name; `Lagged` handling |
| [`src/client.rs`](src/client.rs) | `ChatClient` for the tests and the demo, with a 5 s limit on every read |
| [`src/main.rs`](src/main.rs) | `serve` on 127.0.0.1:6142 with Ctrl+C → `cancel()`; a terminal `client` that `select!`s between stdin and the socket |
| [`tests/chat.rs`](tests/chat.rs) | 5 end-to-end tests on `127.0.0.1:0` |

Each connection runs this loop (abridged from `server.rs`):

```rust
loop {
    tokio::select! {
        _ = shutdown.cancelled() => { lines.send(SHUTDOWN_NOTICE).await?; return Ok(()); }
        line = lines.next() => match line { /* parse, then announce or answer */ },
        event = events.recv() => match event {
            Ok(event) if event.from == id => {}                 // our own message
            Ok(event) => lines.send(&*event.text).await?,
            Err(RecvError::Lagged(missed)) => { /* "you missed N messages" */ }
            Err(RecvError::Closed) => return Ok(()),
        },
    }
}
```

All three futures are cancel safe, so whichever branch wins, nothing is lost. A scripted session (`cargo run -p tokio_runtime --example chat_demo`, real output, abridged):

```text
ada   <- * hello ada! online: ada
grace <- * hello grace! online: ada, grace
ada   <- * grace joined
linus -> ada
linus <- * ada is taken, try another
linus -> linus
linus <- * hello linus! online: ada, grace, linus
grace -> hello everyone
ada   <- grace: hello everyone
ada   -> /me waves
grace <- * ada waves
linus -> /quit
linus <- * bye
ada   <- * linus left
(server: Ctrl+C)
ada   <- * server shutting down, bye
grace <- * server shutting down, bye
(server: run() returned, every connection closed)
```

> 💡 **Analogy:** you've built a **walkie-talkie room**. Everyone who walks in gives their name at the door (and must be quick about it), then everything said into a walkie-talkie is heard by everyone else's. Someone too slow to keep up is told how much they missed. At closing time, a single bell tells every walkie-talkie "we're closing", and the caretaker waits until the last person has left before turning off the lights.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| Moved one `Sender` into tasks in a loop | E0382 use of moved value | `let tx = tx.clone();` before each `spawn` |
| `while let Some(m) = rx.recv().await` never ends | hangs (or a timeout) | Drop **every** sender, including `main`'s: `drop(tx)` |
| `Rc`/`RefCell` (or a std `MutexGuard`) in a spawned task across `.await` | future cannot be sent between threads safely (no code) | `Arc`; drop guards before `.await`; or `LocalSet` + `spawn_local` |
| `&mut sleep` (or a `timeout` stream) polled in a loop | E0277 `PhantomPinned` cannot be unpinned | `tokio::pin!(sleep)` |
| `std::thread::sleep` or heavy CPU work in a task | every task on that thread freezes | `tokio::time::sleep`; `spawn_blocking` |
| `rt.block_on(..)` inside async code | panic: cannot start a runtime from within a runtime | `.await` it; or `spawn_blocking` around sync code |
| `tokio::sync::Mutex` everywhere | slower, and `lock()` isn't cancel safe | std `Mutex` when no `.await` happens under the lock |
| `read_exact`/`write_all` as a `select!` branch in a loop | data silently lost | Cancel-safe operations (`read`, `recv`, `Framed::next`) in `select!` |
| `mpsc::unbounded_channel` for a fast producer | memory grows without limit | A bounded `mpsc::channel(n)`: `send().await` waits |
| Expected a dropped `JoinHandle` to stop the task | the task keeps running | `handle.abort()`, or a `JoinSet` (aborts on drop) |
| Ignored `broadcast` `RecvError::Lagged` | the receiver loop exits or panics | Handle `Lagged(n)`: tell the user, carry on |
| Expected a `Framed` stream to continue after a decode error | `next()` returns `None` | Treat a codec error as the end of the connection |
| `interval` tick expected after one period | the first tick is immediate | Call `tick().await` once before the loop, or `interval_at` |
| `start_paused`/`pause()` on a multi-thread runtime | panic | `flavor = "current_thread"` |
| Paused clock + real sockets in one test | timeouts fire too early | Real time for network tests, paused time for logic |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p tokio_runtime --example runtime
cargo run -p tokio_runtime --example tasks
cargo run -p tokio_runtime --example sync_primitives
cargo run -p tokio_runtime --example channels
cargo run -p tokio_runtime --example select_and_time
cargo run -p tokio_runtime --example async_io
cargo run -p tokio_runtime --example streams
cargo run -p tokio_runtime --example graceful_shutdown
cargo run -p tokio_runtime --example chat_demo
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_moved_sender` | Make it compile (E0382), then make it finish | `cargo run -p tokio_runtime --features exercises --bin ex01_fix_moved_sender` |
| 2 | `ex02_fix_rc_across_await` | Make it compile ("future cannot be sent between threads safely", no code) | `cargo run -p tokio_runtime --features exercises --bin ex02_fix_rc_across_await` |
| 3 | `ex03_fix_unpinned_sleep` | Make it compile (E0277 `PhantomPinned` cannot be unpinned) | `cargo run -p tokio_runtime --features exercises --bin ex03_fix_unpinned_sleep` |
| 4 | `ex04_limited_fetch` | `Semaphore` + `JoinSet`, results in order, pass 3 tests | `cargo test -p tokio_runtime --features exercises --bin ex04_limited_fetch` |
| 5 | `ex05_kv_actor` | The actor pattern: `mpsc` commands with `oneshot` replies, pass 3 tests | `cargo test -p tokio_runtime --features exercises --bin ex05_kv_actor` |
| 6 | `ex06_cancellable_worker` | `select!` + `biased` + `CancellationToken`, an interval, pass 3 tests | `cargo test -p tokio_runtime --features exercises --bin ex06_cancellable_worker` |

The tests of 4 and 6 run on the paused clock: instant, yet they check exact timings.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p tokio_runtime --features solutions --bins`, or `cargo run -p tokio_runtime --features solutions --bin sol01_fix_moved_sender` and so on).

**4. Run the mini project** and chat:

```bash
cargo run -p tokio_runtime                  # terminal 1: the server on 127.0.0.1:6142
cargo run -p tokio_runtime -- client        # terminals 2 and 3: type a name, then chat
cargo test -p tokio_runtime                 # 2 unit tests + 5 end-to-end tests
```

In a client: `/who`, `/me waves`, `/quit`. Press Ctrl+C on the server and watch every client get the shutdown notice. Then try: add a `/nick newname` command (free the old name, claim the new one, announce the change), with a test in `tests/chat.rs`.

## Checklist

- [ ] The runtime: scheduler, time driver, I/O driver; multi-thread vs current-thread; `#[tokio::main]` options and `Builder`
- [ ] Never block a runtime thread; `spawn_blocking` for CPU and blocking work
- [ ] Tasks: `spawn`, `JoinHandle`, `JoinError` (panic, cancelled), `abort`, detaching, `JoinSet`; `Send + 'static`
- [ ] Sync primitives: std vs Tokio `Mutex`, `RwLock`, `Semaphore`, `Notify`, `OnceCell`
- [ ] Channels: `mpsc` (bounded, back-pressure), `oneshot`, `broadcast` (`Lagged`), `watch`; the actor pattern
- [ ] `select!`: `biased`, patterns, preconditions, `else`; cancellation safety; pinning a future reused across iterations
- [ ] Time: `sleep`, `timeout`, `sleep_until`, `interval` and `MissedTickBehavior`
- [ ] Async I/O: `TcpListener`/`TcpStream`, `AsyncReadExt`/`AsyncWriteExt`, `split`/`into_split`, `shutdown`, lines
- [ ] Framing and codecs: `Framed`, `LinesCodec`, `LengthDelimitedCodec`
- [ ] Streams: `tokio-stream` adapters, wrappers, `StreamMap`, `chunks_timeout`
- [ ] Graceful shutdown: `CancellationToken` (children, `run_until_cancelled`, `DropGuard`), `TaskTracker`, `ctrl_c`
- [ ] Testing: `#[tokio::test]`, `start_paused`, timeouts on every network read
- [ ] Exercises 1–6 and the chat server done without looking at the solutions

## Recap

- The runtime polls tasks on worker threads (or one thread), with drivers for timers and I/O; tasks can only switch at `.await`, so never block, and send blocking work to `spawn_blocking`.
- `tokio::spawn` gives a `JoinHandle`; `JoinSet` manages groups; spawned futures must be `Send + 'static`.
- Prefer a std `Mutex` (never across `.await`), use a `Semaphore` to limit concurrency, and channels to avoid sharing: `mpsc` for queues, `oneshot` for replies, `broadcast` for everyone, `watch` for the latest value.
- `select!` runs the first branch to finish and drops the rest, so use cancel-safe operations and pin futures you keep across iterations.
- TCP is a byte stream; codecs and `Framed` turn it into messages.
- `CancellationToken` tells every task to stop, `TaskTracker` waits for them; a paused clock makes timing tests instant and exact.

**In one sentence for a child:** Tokio is a busy kitchen where a team of chefs shares one order board, passes notes on a rail, never stands still waiting for the oven, and goes home together when the closing bell rings.

**Next:** [03 SQLx](../03_sqlx/README.md): async SQL on top of Tokio.
