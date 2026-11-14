<!-- hand-written lesson: gen_course must not overwrite -->
# Fearless Concurrency

**The Rust Book, chapter 16:** [Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
· Rust by Example: [Threads](https://doc.rust-lang.org/rust-by-example/std_misc/threads.html), [Channels](https://doc.rust-lang.org/rust-by-example/std_misc/channels.html)
· std: [`std::thread`](https://doc.rust-lang.org/std/thread/index.html), [`std::sync`](https://doc.rust-lang.org/std/sync/index.html), [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html)
· Crate: [`rayon`](https://docs.rs/rayon)
· Free book: [Rust Atomics and Locks](https://marabos.nl/atomics/) (Mara Bos)

**Concurrency** means parts of a program run independently; **parallelism** means they run at the same moment, on different CPU cores. Both make programs faster, and both bring famous bugs:

| Bug | What happens |
|---|---|
| **Data race** | two threads touch the same data at once, at least one writing: garbage results |
| **Race condition** | the result depends on which thread happens to go first |
| **Deadlock** | two threads each wait for the other, forever |

In most languages you find these at 3 a.m. in production. In Rust, ownership and the type system turn most of them into **compile errors**. That's what the Book means by *fearless* concurrency: you can change concurrent code without being afraid of breaking it silently.

> 💡 **The big picture: a busy kitchen.** Threads are **cooks** working at the same time. They can pass dishes through a **hatch** (a channel): once you've handed a plate over, it isn't yours any more. Or they can share one **salt shaker** that only one cook may hold at a time (a `Mutex`), with a **tally** of how many cooks know where it is (`Arc`). Before anyone starts cooking, the **head chef** (the compiler) checks the plan and refuses any plan where two cooks could grab the same knife at once. The mix-ups are caught before service, not during it.

## 1. Using threads to run code simultaneously

`thread::spawn` takes a closure and runs it on a new operating-system thread. It returns a `JoinHandle`; calling `join()` on it **blocks** until that thread has finished.

```rust
let handle = thread::spawn(|| {
    for i in 1..5 {
        println!("hi number {i} from the spawned thread!");
        thread::sleep(Duration::from_millis(1));
    }
});
for i in 1..3 {
    println!("hi number {i} from the main thread!");
}
handle.join().unwrap(); // wait for the spawned thread
```

- **The order is up to the operating system.** Run the example twice and the lines interleave differently.
- **When `main` returns, the whole process ends**, and every spawned thread is stopped mid-task. Keep the handles and `join` them.
- **Where you `join` matters.** Joining straight after the spawn means main waits first, so nothing runs side by side.
- **A thread can return a value:** `JoinHandle<T>` gives it back from `join()`.
- **A panic stays in its thread:** `join()` returns `Err`, and main carries on.
- `thread::Builder` names a thread; `thread::available_parallelism()` tells you how many can really run at once.

> 💡 **Analogy: hiring a helper cook.** `spawn` hires a helper and gives them a recipe card (the closure). The helper and you both cook at once, and who finishes a step first is luck. `join` is **waiting at the pass** until the helper says "done", and they can hand you the finished dish (the return value). If you lock up the restaurant (main returns) without waiting, the helper is sent home halfway through the soup.

▶ `cargo run -p fearless_concurrency --example spawn_and_join`

### `move` closures with threads

A closure borrows what it uses, if it can. But a spawned thread might outlive the function that started it, so the borrow could dangle. The real error from exercise 1 (abridged):

```text
error[E0373]: closure may outlive the current function, but it borrows `chores`, which is owned by the current function
  --> 04_systems_level\01_fearless_concurrency\exercises\ex01_fix_missing_move.rs:17:32
   |
17 |     let handle = thread::spawn(|| {
   |                                ^^ may outlive borrowed value `chores`
18 |         for chore in &chores {
   |                       ------ `chores` is borrowed here
   |
help: to force the closure to take ownership of `chores` (and any other referenced variables), use the `move` keyword
   |
17 |     let handle = thread::spawn(move || {
   |                                ++++
```

`move ||` makes the closure **take ownership** of everything it uses. After that, `main` can't use the value (E0382), so clone it first if both need it, or get it back as the thread's return value. Even a plain number needs `move`: without it the closure borrows the number.

Why so strict? Look at the signature:

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

`'static` here means "**holds no short-lived borrows**" (lesson 03/01), not "lives forever". An owned `String` qualifies, and so does a `&'static str`. `Send` is section 4.

> 💡 **Analogy: lending vs giving.** Without `move`, you **lend** the helper your notebook, but they might take it home and you might throw it away tonight, so the head chef forbids it. With `move`, you **give** it to them: it's theirs now, and you can't scribble in it any more. `'static` is the rule "no short-term loans to helpers who might stay late".

▶ `cargo run -p fearless_concurrency --example move_closures`

## 2. Transferring data between threads with message passing

The Book quotes the Go documentation: "Do not communicate by sharing memory; instead, share memory by communicating." A **channel** has two halves: a **transmitter** (`tx`) and a **receiver** (`rx`).

```rust
let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    let val = String::from("hi");
    tx.send(val).unwrap();
});
let received = rx.recv().unwrap(); // blocks until a value arrives
```

| Call | Does |
|---|---|
| `tx.send(v)` | moves `v` into the channel. `Err` if the receiver is gone. |
| `rx.recv()` | **blocks** until a value arrives. `Err` once every sender is dropped and the channel is empty. |
| `rx.try_recv()` | never blocks: `Err(Empty)` if nothing is waiting yet |
| `for v in rx` | receive until every sender is dropped |
| `tx.clone()` | another sender: **m**ultiple **p**roducer, **s**ingle **c**onsumer (`mpsc`) |
| `mpsc::sync_channel(n)` | a **bounded** channel: `send` waits while `n` messages are queued |

**Sending transfers ownership.** Once a value is sent, the sender can't touch it, so two threads can never fight over it. The real error from exercise 2:

```text
error[E0382]: borrow of moved value: `message`
  --> 04_systems_level\01_fearless_concurrency\exercises\ex02_fix_use_after_send.rs:19:33
   |
17 |             let message = format!("parcel number {n}");
   |                 ------- move occurs because `message` has type `String`, which does not implement the `Copy` trait
18 |             tx.send(message).unwrap();
   |                     ------- value moved here
19 |             println!("sending: {message}");
   |                                 ^^^^^^^ value borrowed here after move
```

**The loop that never ends.** `for msg in rx` only stops when **every** sender is dropped. With several producers, the original `tx` in `main` is still alive, so `drop(tx)` once you've handed out the clones.

> 💡 **Analogy: a post box.** `send` is posting a letter: once it's in the box, you can't read it or change it any more, because it belongs to whoever receives it. Cloning `tx` builds more post boxes that all empty into **one** sorting office (the receiver). The sorting office closes only when every post box has been taken away, which is why a forgotten `tx` keeps it open forever. A `sync_channel(2)` is a **conveyor belt with room for two plates**: when it's full, the cook waits.

▶ `cargo run -p fearless_concurrency --example message_passing`

## 3. Shared-state concurrency

Sometimes many threads really do need the same data. A **mutex** (*mutual exclusion*) lets only one thread at a time get at it.

```rust
let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];
for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    })); // `num` drops when the closure ends, which unlocks the mutex
}
```

- **`lock()`** waits for the lock and returns a `MutexGuard`: a smart pointer that derefs to the data and **unlocks when dropped** (lesson 03/06's `Deref` and `Drop`). You can't forget to unlock, and you can't reach the data without locking.
- **`Arc<T>`** is `Rc<T>` with an **atomic** reference count, so threads can clone and drop it safely. `Rc` can't cross threads at all. The real error from exercise 3 (abridged):

```text
error[E0277]: `Rc<std::sync::Mutex<i32>>` cannot be sent between threads safely
   --> 04_systems_level\01_fearless_concurrency\exercises\ex03_fix_rc_not_send.rs:21:36
    |
 21 |           let handle = thread::spawn(move || {
    |                        ------------- ^------
    |                        |             |
    |  ______________________|_____________within this `{closure@ex03_fix_rc_not_send.rs:21:36}`
    | |                      |
    | |                      required by a bound introduced by this call
 22 | |             let mut total = scoreboard.lock().unwrap();
 23 | |             *total += points;
 24 | |         });
    | |_________^ `Rc<std::sync::Mutex<i32>>` cannot be sent between threads safely
    |
    = help: within `{closure@ex03_fix_rc_not_send.rs:21:36}`, the trait `Send` is not implemented for `Rc<std::sync::Mutex<i32>>`
```

- **The pattern mirrors lesson 03/06:** `Rc<RefCell<T>>` on one thread becomes `Arc<Mutex<T>>` across threads. Like `RefCell`, a `Mutex` gives *interior mutability*: you change the data through a shared `Arc`.
- **Hold the lock briefly.** Copy what you need out (`let snapshot = data.lock().unwrap().clone();`) and let the guard drop at the `;`. A `while let Ok(job) = rx.lock().unwrap().recv()` keeps the lock for the **whole loop body**; a separate `let job = …;` releases it at once (the mini project does this).
- **Deadlock.** Thread 1 holds lock A and waits for B, while thread 2 holds B and waits for A: both wait forever. The cure is to **always lock in the same order**. A `Mutex` isn't re-entrant either: calling `lock()` twice on one thread waits for itself. `try_lock()` reports `WouldBlock` instead of waiting.
- **Poisoning.** If a thread panics while holding the lock, the mutex is marked *poisoned* and later `lock()` calls return `Err`. That's why you see `.unwrap()` everywhere; `into_inner()` on the error recovers the data if you decide it's still fine.

> 💡 **Analogy: the café toilet key.** There's one key on a big wooden spoon (`Mutex`). Whoever holds it can go in, and everyone else queues. The `MutexGuard` is **holding the spoon**: when you come out, it goes back on the hook by itself (drop), so nobody can walk off with it. `Arc` is the **notice saying where the key is**, photocopied for every customer, with a counter that can't get muddled when two people take a copy at once (`Rc`'s counter can). **Deadlock** is two children who each grab one of the two crayons they both need and wait for the other to let go; the rule "always pick up red before blue" ends it. **Poisoning** is a sign on the door: "someone was taken ill in here". You may still go in, but you should check first.

▶ `cargo run -p fearless_concurrency --example shared_state`

## 4. Extensible concurrency with `Send` and `Sync`

Rust's language knows almost nothing about threads: `Mutex`, channels and `Arc` are ordinary library code. What makes them safe are two **marker traits** (no methods, just a promise):

| Trait | Means | Not implemented by |
|---|---|---|
| `Send` | ownership can be **moved** to another thread | `Rc<T>`, raw pointers, `MutexGuard` |
| `Sync` | `&T` can be **shared** between threads (`T: Sync` exactly when `&T: Send`) | `Rc<T>`, `RefCell<T>`, `Cell<T>`, raw pointers |

- They're **auto traits**: a struct whose fields are all `Send` is `Send` automatically. One `Rc` field makes the whole struct non-`Send`.
- `thread::spawn` requires `F: Send`; sharing `&T` with other threads requires `T: Sync`. Break a rule, and you get E0277 "cannot be sent between threads safely" or "cannot be shared between threads safely".
- Implementing them by hand is **`unsafe`** (`unsafe impl Send for MyType {}`): you promise something the compiler can't check. Only do it for types built on raw pointers, with a `// SAFETY:` comment saying why. The opposite is safe: a `PhantomData<*const ()>` field opts a type **out**.
- A quick compile-time test: `fn assert_send<T: Send>() {}` and then `assert_send::<MyType>();`.

> 💡 **Analogy: two stickers on every toy.** **Send** says "you may **give** this toy to another child". **Sync** says "lots of children may **look at** this toy at the same time". A poster on the wall is Sync; a diary isn't. `Rc` is a toy with a little scoreboard of who's playing with it, and the scoreboard breaks if two children write on it at once, so it gets neither sticker. `unsafe impl` is you sticking a label on yourself: allowed, but if you're wrong it's on you.

▶ `cargo run -p fearless_concurrency --example send_and_sync`

## Beyond the Book: scoped threads

`thread::scope` (Rust 1.63) creates threads that may **borrow** local data, because the scope **joins them all** before it returns. No `move`, no `Arc`, no clones:

```rust
let numbers: Vec<u64> = (1..=1_000).collect();
let (evens, odds) = thread::scope(|s| {
    let evens = s.spawn(|| numbers.iter().filter(|n| *n % 2 == 0).sum::<u64>());
    let odds = s.spawn(|| numbers.iter().filter(|n| *n % 2 == 1).sum::<u64>());
    (evens.join().unwrap(), odds.join().unwrap())
}); // every scoped thread has finished here, so `numbers` is free again
```

Mutable borrows work as long as no two threads touch the same data (`chunks_mut` gives each thread its own slice). The borrow checker still stops data races:

```rust
let mut count = 0;
thread::scope(|s| {
    for _ in 0..4 {
        s.spawn(|| count += 1); // E0499: cannot borrow `count` as mutable more than once at a time
    }
});
```

> 💡 **Analogy: a playdate in a fenced garden.** Friends may borrow your toys, because the gate stays locked until every friend is back inside (the end of the scope). Nobody can wander off with a toy, so you don't need to give toys away (`move`) or buy copies (`clone`). Two friends still can't paint the same picture at once.

▶ `cargo run -p fearless_concurrency --example scoped_threads`

## Beyond the Book: data parallelism with `rayon`

For "do this to every item" work, [rayon](https://docs.rs/rayon) turns an iterator parallel with one word: `iter()` becomes `par_iter()` (or `into_par_iter()`, `par_iter_mut()`, `par_sort()`). It runs a pool of one thread per core and balances the load by **work stealing**.

```rust
use rayon::prelude::*;
let primes = (1..200_000).into_par_iter().filter(|&n| is_prime(n)).count();
let (a, b) = rayon::join(|| sum(left), || sum(right)); // fork-join
```

- It's safe for the same reason as everything else in this chapter: the closures must be `Send + Sync`, so a data race won't compile.
- Parallelism has overhead. For tiny jobs, or work that waits on disk or the network (that's async, the next lesson), it can be slower. Measure, in `--release`.

> 💡 **Analogy: a homework party.** The teacher splits a big pile of worksheets between helpers. Anyone who finishes early **steals** half of a busy friend's pile, so nobody sits idle. You just say "do these worksheets, in parallel please" and rayon organises the party.

▶ `cargo run -p fearless_concurrency --example rayon_parallel` (add `--release` for real timings)

## Beyond the Book: `RwLock`, `Condvar` and `Barrier`

| Tool | Use it when | Key calls |
|---|---|---|
| `RwLock<T>` | data is read often and written rarely | `read()` (many at once), `write()` (one, alone) |
| `Condvar` | a thread must **sleep until** a condition becomes true | `wait_while(guard, cond)`, `notify_one()`, `notify_all()` |
| `Barrier` | N threads must all reach a point before any continues | `Barrier::new(n)`, `wait()` |

A `Condvar` always goes with a `Mutex`: `wait` releases the lock while sleeping and takes it back on waking. Use `wait_while` (or a `while` loop around `wait`), because threads can wake up spuriously.

> 💡 **Analogy:** an `RwLock` is a **painting in a museum**: any number of visitors can look at once, but the restorer needs the room empty. A `Condvar` is a **doorbell**: instead of opening the door every second to check for the pizza, you sleep on the sofa until it rings. A `Barrier` is a **school trip**: the coach doesn't leave until every child is back on board.

▶ `cargo run -p fearless_concurrency --example rwlock_condvar_barrier`

## Beyond the Book: atomics and memory ordering

`AtomicUsize`, `AtomicBool`, `AtomicU64` and friends are numbers that threads can update **without a lock**, using special CPU instructions: `fetch_add`, `load`, `store`, `compare_exchange`, `fetch_max`. `Arc`'s counter is one. Every operation takes an `Ordering`, which says what happens to *other* memory around it:

| Ordering | Guarantees | Typical use |
|---|---|---|
| `Relaxed` | only this one variable is updated correctly | counters, statistics, a stop flag |
| `Release` (on a store) / `Acquire` (on a load) | everything written before the `Release` is visible after the matching `Acquire` | publishing data: "the data is ready" |
| `AcqRel` | both, for read-modify-write operations | `compare_exchange` in lock-free structures |
| `SeqCst` | all `SeqCst` operations happen in one order that every thread agrees on | the safe default when unsure |

```rust
data.store(42, Ordering::Relaxed);    // write the parcel...
ready.store(true, Ordering::Release); // ...then ring the doorbell
// on another thread:
while !ready.load(Ordering::Acquire) {} // hear the doorbell...
assert_eq!(data.load(Ordering::Relaxed), 42); // ...and the parcel is there
```

Compilers and CPUs reorder memory operations to go faster; orderings say which reorderings are forbidden. Get this wrong and the bug appears only on some machines, some of the time, so use a `Mutex` or `SeqCst` unless you've measured a need, and read *Rust Atomics and Locks* before going further.

> 💡 **Analogy:** an atomic is a **tally clicker** that the whole class can press, and every press always counts, even when two land together. `Relaxed` is "I only care about the number on the clicker". **Release/Acquire** is a delivery: the driver puts the **parcel** on the step and *then* rings the **doorbell** (Release); whoever hears the bell (Acquire) is sure to find the parcel. `SeqCst` is **one announcer** reading out every event in one single order that everybody hears the same way.

▶ `cargo run -p fearless_concurrency --example atomics`

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `thread::spawn(\|\| uses(&local))` | E0373 closure may outlive the current function | `move \|\|`, or `thread::scope` to borrow |
| Used a value after `tx.send(value)` | E0382 borrow of moved value | Use it before sending, or send a clone |
| `Rc<T>` or `Rc<Mutex<T>>` in a thread | E0277 `Rc<…>` cannot be sent between threads safely | `Arc<T>`, `Arc<Mutex<T>>` |
| A `RefCell` shared between threads | E0277 cannot be shared between threads safely | `Mutex` or `RwLock` |
| `count += 1` from several threads | E0499 | A `Mutex`, or an atomic |
| `for msg in rx` never finishes | hangs | `drop(tx)`: the original sender keeps the channel open |
| No `join()` | threads stopped when `main` returns | Join every handle, or use `thread::scope` |
| `while let Ok(job) = rx.lock().unwrap().recv()` | workers take turns instead of running in parallel | `let job = …;` on its own line, so the guard drops at the `;` |
| Locking A then B here, B then A there | deadlock: hangs forever | Always lock in the same order |
| `lock()` twice on the same thread | deadlock or panic | Lock once and pass the guard along |
| `par_iter()` on tiny jobs | slower than sequential | Measure in `--release`; keep it sequential |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p fearless_concurrency --example spawn_and_join
cargo run -p fearless_concurrency --example move_closures
cargo run -p fearless_concurrency --example message_passing
cargo run -p fearless_concurrency --example shared_state
cargo run -p fearless_concurrency --example send_and_sync
cargo run -p fearless_concurrency --example scoped_threads
cargo run -p fearless_concurrency --example rayon_parallel
cargo run -p fearless_concurrency --example rwlock_condvar_barrier
cargo run -p fearless_concurrency --example atomics
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_missing_move` | Make it compile (E0373), one keyword | `cargo run -p fearless_concurrency --features exercises --bin ex01_fix_missing_move` |
| 2 | `ex02_fix_use_after_send` | Make it compile (E0382), no `.clone()` | `cargo run -p fearless_concurrency --features exercises --bin ex02_fix_use_after_send` |
| 3 | `ex03_fix_rc_not_send` | Make it compile (E0277), `Rc` → ? | `cargo run -p fearless_concurrency --features exercises --bin ex03_fix_rc_not_send` |
| 4 | `ex04_parallel_sum` | Split a sum with `thread::scope`, pass 3 tests | `cargo test -p fearless_concurrency --features exercises --bin ex04_parallel_sum` |
| 5 | `ex05_fan_in_channel` | Many producers, one channel, pass 3 tests | `cargo test -p fearless_concurrency --features exercises --bin ex05_fan_in_channel` |
| 6 | `ex06_mutex_and_atomic_counters` | `Arc<Mutex<_>>` vs `AtomicUsize`, pass 3 tests | `cargo test -p fearless_concurrency --features exercises --bin ex06_mutex_and_atomic_counters` |

If exercise 5's tests hang, that's the never-ending `for msg in rx` from section 2 (press Ctrl+C).

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p fearless_concurrency --features solutions --bins`).

**4. The mini project** is a **parallel word counter**. It counts 400,000 words in 2,000 generated documents five ways, checks that every way gives exactly the same answer, and times them:

```bash
cargo run -p fearless_concurrency               # one worker per CPU core
cargo run -p fearless_concurrency -- 2          # exactly 2 workers
cargo run -p fearless_concurrency --release     # the real speed-up
cargo test -p fearless_concurrency              # 6 unit tests
```

| Strategy (`src/lib.rs`) | Chapter idea |
|---|---|
| `count_sequential` | the baseline |
| `count_with_worker_pool` | **two channels**: jobs out through one receiver shared as `Arc<Mutex<Receiver>>`, results back with many cloned senders, plus an `AtomicUsize` progress count |
| `count_with_shared_map` | `thread::spawn` + `Arc<[String]>` + `Arc<Mutex<HashMap>>`, unwrapped at the end with `Arc::try_unwrap` |
| `count_with_scoped_threads` | `thread::scope` borrowing chunks; each thread **returns** its map |
| `count_with_rayon` | `par_iter().map(…).reduce(…)` |

Every strategy counts into a private map first and merges once, so threads hardly ever wait for a lock. That's the most important performance habit in this chapter.

> 💡 **Analogy:** a class counting words in a pile of 2,000 books. In the **worker pool**, the teacher puts the books on one table and children take them one at a time (only one hand in the pile at once: the `Mutex`). Each child keeps a private tally sheet and hands it in at the end (the result channel), and the teacher adds the sheets up. The other strategies are the same party organised differently, and the test checks that everyone got the same total.

## Checklist

- [ ] [16.1 Using Threads to Run Code Simultaneously](https://doc.rust-lang.org/book/ch16-01-threads.html)
- [ ] [16.2 Transfer Data Between Threads with Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [ ] [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- [ ] [16.4 Extensible Concurrency with Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [ ] Beyond the Book: scoped threads (`std::thread::scope`)
- [ ] Beyond the Book: data parallelism with `rayon`
- [ ] Beyond the Book: `RwLock`, `Condvar`, `Barrier`
- [ ] Beyond the Book: atomics and memory ordering (`Relaxed`, `Acquire`/`Release`, `SeqCst`)
- [ ] Exercises 1–6 and the word counter done without looking at the solutions

## Recap

- `thread::spawn` runs a closure on a new thread and returns a `JoinHandle`; `join` waits for it and returns its value. `move` gives the thread ownership of what it uses.
- Channels (`mpsc`) move values between threads: sending gives up ownership, `tx.clone()` adds producers, and the receiver stops when every sender is dropped.
- `Arc<Mutex<T>>` shares mutable data: `Arc` for shared ownership, `Mutex` for one-at-a-time access. Keep locks short, lock in a fixed order, and know that a panic poisons the lock.
- `Send` and `Sync` are the marker traits behind all of it: the compiler rejects `Rc`, `RefCell` and raw pointers where threads could misuse them.
- Beyond the Book: `thread::scope` lets threads borrow, `rayon` parallelises iterators, `RwLock`/`Condvar`/`Barrier` coordinate threads, and atomics give lock-free counters and flags with explicit memory ordering.

**In one sentence for a child:** lots of cooks can work in one kitchen at the same time, because they either pass dishes through a hatch or take turns with the one salt shaker, and the head chef checks the plan so nobody grabs the same knife.

**Next:** [02 Async and Await](../02_async_await/README.md) (Book chapter 17)
