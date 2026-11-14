<!-- hand-written lesson: gen_course must not overwrite -->
# Web Server Project

**The Rust Book, chapter 21:** [Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html)
· std: [`std::net`](https://doc.rust-lang.org/std/net/index.html), [`std::thread`](https://doc.rust-lang.org/std/thread/index.html), [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html)
· HTTP: [MDN, HTTP messages](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages), [RFC 9112 (HTTP/1.1)](https://www.rfc-editor.org/rfc/rfc9112)

Run it: `cargo run -p web_server`, then open <http://127.0.0.1:7878>  ·  Test it: `cargo test -p web_server`

The Book's last chapter builds a web server from nothing but the standard library: a `TcpListener`, some text parsing, threads, a channel and a `Mutex`. It ties together ownership, closures, traits, smart pointers and fearless concurrency ([lesson 04/01](../01_fearless_concurrency/README.md)). The plan:

1. Learn a little about TCP and HTTP.
2. Listen for TCP connections on a socket.
3. Parse a small number of HTTP requests.
4. Create a proper HTTP response.
5. Improve the server's throughput with a **thread pool**, and shut it down gracefully.

It's a learning server, not a production one: for real services use a framework such as [axum](../../06_framework_mastery/README.md), built on async I/O ([lesson 04/02](../02_async_await/README.md)).

> 💡 **The big picture: an ice-cream shop.** The `TcpListener` is the **shop door**, and every customer who walks in is a connection. With **one server** behind the counter, a customer who orders a slow, fancy sundae makes the whole queue wait. Hiring a **new server for every customer** works until a coach party of 10,000 arrives. The answer is a **fixed team** of servers and a **ticket machine** (the channel): customers take a ticket, and whichever server is free calls the next number. At **closing time** you turn the sign to CLOSED, the servers finish the customers they're already serving, and only then does everyone go home.

## Before you start: TCP and HTTP in two minutes

**TCP** gives you a reliable two-way stream of bytes between two programs. **HTTP** is the text conversation that browsers and servers have over it. A request is a request line, headers, a blank line and an optional body; every line ends with `\r\n` (CRLF):

```text
GET / HTTP/1.1                ← method, request target, version
Host: 127.0.0.1:7878          ← headers, one per line
User-Agent: curl/8.5.0
                              ← a blank line ends the head
```

A response has the same shape, starting with a **status line**:

```text
HTTP/1.1 200 OK
Content-Length: 176

<!DOCTYPE html> …
```

> 💡 **Analogy: a phone call with a script.** TCP is the phone line: it makes sure every word arrives, in order. HTTP is the polite script both sides follow on the call: "Hello, I'd like the page called `/`, please" and "Certainly, here it is: 176 bytes."

## 1. Building a single-threaded web server

### Listening to the TCP connection

```rust
let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

for stream in listener.incoming() {
    let stream = stream.unwrap();
    println!("Connection established!");
}
```

- `bind` returns a `Result`: it fails if the port is taken (`AddrInUse: Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)` on Windows) or, for ports below 1024, if you lack permission.
- `incoming()` yields one `TcpStream` per connection **attempt**, forever. A browser may connect more than once (for `favicon.ico`, or to retry).
- When `stream` goes out of scope it is dropped, and the connection closes.
- Port **0** asks the operating system for any free port; `listener.local_addr()` says which. The examples and tests use it so they never clash with a running server.

> 💡 **Analogy: the shop door with a bell.** `bind` hangs the door on a particular street number (the port). Each ring of the bell is a new customer (`incoming`). If another shop already has that number, you can't open there.

▶ `cargo run -p web_server --example step1_listen_and_read`

### Reading the request

```rust
let buf_reader = BufReader::new(&stream);
let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap())
    .take_while(|line| !line.is_empty())   // the blank line ends the head
    .collect();
```

`BufReader` adds buffering and gives us `lines()`. The client doesn't close the connection after its request, so we can't read "until the end": we stop at the blank line instead.

> 💡 **Analogy: an order slip.** The customer writes their order line by line and leaves one empty line at the bottom to say "that's everything". Read until the empty line, not until they leave the shop, because they're waiting for their ice cream.

### Writing a response, returning real HTML, and validating the request

```rust
let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
    ("HTTP/1.1 200 OK", "hello.html")
} else {
    ("HTTP/1.1 404 NOT FOUND", "404.html")
};
let contents = fs::read_to_string(filename).unwrap();
let length = contents.len();
let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
stream.write_all(response.as_bytes()).unwrap();
```

- **`Content-Length`** tells the browser how many bytes of body to expect. It counts **bytes**, which is exactly what `String::len` returns (exercise 5 shows why `chars().count()` is wrong).
- `write_all` keeps writing until every byte is sent (`write` alone may send only part).
- The Book's refactor puts only what **differs** (status line, file name) in the `if`, and shares the rest. Exercise 4 goes further and splits the request line into method, path and version.

> 💡 **Analogy: the reply.** The status line is the first thing you say ("Here you go!" or "Sorry, we don't sell that"), `Content-Length` is "this tub holds 176 scoops", and the body is the ice cream itself.

▶ `cargo run -p web_server --example step2_respond`

## 2. From single-threaded to multithreaded

### Simulating a slow request

The Book adds a `/sleep` route that sleeps for 5 seconds. With one thread, a fast request that arrives during that sleep has to wait:

```text
A (/sleep) done after 1.1s
B (/)      done after 1.1s  <- had to wait for A
```

> 💡 **Analogy:** one server behind the counter, making a slow sundae. The customer who only wants one scoop waits for the whole sundae to be finished.

▶ `cargo run -p web_server --example step3_slow_single_thread` (1 second instead of 5)

### Improving throughput: why a thread pool?

The obvious fix is `thread::spawn` for every connection, and it does work: B no longer waits. But there's **no limit**. 10,000 simultaneous connections would mean 10,000 threads, each with its own stack, which is an easy denial-of-service attack. A **thread pool** starts a fixed number of threads once, and queues the work beyond that. (Other designs exist: fork/join, single-threaded async I/O and multithreaded async I/O, which is how tokio and axum work.)

> 💡 **Analogy:** hiring a new server for every customer who walks in is fine on a quiet day. A coach party of 10,000 would bankrupt the shop. A fixed team of four, plus a queue, serves everyone eventually and never collapses.

▶ `cargo run -p web_server --example step4_thread_per_request`

### Designing the interface: compiler-driven development

The Book writes the code it *wishes* existed, then lets the compiler's errors say what to build next:

```rust
let pool = ThreadPool::new(4);
for stream in listener.incoming() {
    let stream = stream.unwrap();
    pool.execute(|| handle_connection(stream));   // like thread::spawn, but reusing threads
}
```

- `execute` takes a closure with the same bounds as `thread::spawn`'s: `F: FnOnce() + Send + 'static`. `FnOnce` because each job runs once; `Send` to cross to another thread; `'static` because nobody knows how long the job will wait.
- `new` checks `size > 0` with `assert!`, and documents it under `# Panics`. The Book suggests `build(size) -> Result<ThreadPool, PoolCreationError>` as an alternative, and this lesson has both (exercise 6).

> 💡 **Analogy: writing the job advert first.** You describe the team you want ("four servers, and you hand them orders"), and the compiler is the manager who replies with the next thing missing: "you haven't hired anyone yet".

### Workers, the channel, and sharing the receiver

`thread::spawn` wants its code *immediately*, but a pool's threads must start first and get their work **later**. So each thread is wrapped in a `Worker { id, thread }` that waits on a **channel**: the pool keeps the `Sender`, and every worker reads from the `Receiver`.

But a `Receiver` can't be shared by just moving it into each worker. The real error from exercise 2:

```text
error[E0382]: use of moved value: `receiver`
  --> 04_systems_level\04_web_server_project\exercises\ex02_fix_shared_receiver.rs:26:36
   |
22 |     let (sender, receiver) = mpsc::channel::<Job>();
   |                  -------- move occurs because `receiver` has type `std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send>>`, which does not implement the `Copy` trait
...
25 |     for id in 0..4 {
   |     -------------- inside of this loop
26 |         workers.push(thread::spawn(move || {
   |                                    ^^^^^^^ value moved into closure here, in previous iteration of loop
```

The fix is `Arc<Mutex<Receiver<Job>>>`: `Arc` so all the workers **own** it together, `Mutex` so only **one** of them takes a job at a time.

The job type is a trait object behind a type alias, `type Job = Box<dyn FnOnce() + Send + 'static>;`. Leave out `Send` and the error appears at `thread::spawn`, because a `Receiver<Job>` can only cross threads if `Job` can (exercise 1):

```text
error[E0277]: `(dyn FnOnce() + 'static)` cannot be sent between threads safely
   --> 04_systems_level\04_web_server_project\exercises\ex01_fix_job_not_send.rs:24:32
    |
 24 |       let worker = thread::spawn(move || {
    |  __________________-------------_^
    | |                  |
    | |                  required by a bound introduced by this call
 25 | |         for job in receiver {
 26 | |             job();
...   |
 29 | |     });
    | |_____^ `(dyn FnOnce() + 'static)` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for `(dyn FnOnce() + 'static)`
```

Ignore the compiler's last suggestion there ("use parentheses to call this trait object"): the fix is `+ Send` on `Job`, not a call.

Each worker loops like this:

```rust
let message = receiver.lock().unwrap().recv();   // the lock is released at the `;`
match message {
    Ok(job) => job(),
    Err(_) => break,   // the channel was closed: time to stop
}
```

Writing `while let Ok(job) = receiver.lock().unwrap().recv() { job(); }` compiles too, but the lock guard then lives until the end of the loop body, so **the lock is held while the job runs** and the pool serves one request at a time.

> 💡 **Analogy: the ticket machine.** The channel is the machine at the counter: the shop (the `Sender`) prints a ticket for each order. The four servers share **one** machine (`Arc`), and only one hand may pull a ticket at a time (`Mutex`). The rule that matters: pull your ticket, **let go of the machine**, and then make the ice cream. Holding the machine while you work stops everyone else.

▶ `cargo run -p web_server --example step5_thread_pool`

## 3. Graceful shutdown and cleanup

When the pool is dropped, it should wait for its workers instead of cutting them off mid-request. `Drop` gets `&mut self`, but `JoinHandle::join` takes `self`, so it needs to **own** the handle. The real error from exercise 3:

```text
error[E0507]: cannot move out of `worker.thread` which is behind a mutable reference
   --> 04_systems_level\04_web_server_project\exercises\ex03_fix_join_in_drop.rs:69:13
    |
 69 |             worker.thread.join().unwrap();
    |             ^^^^^^^^^^^^^ ------ `worker.thread` moved due to this method call
    |             |
    |             move occurs because `worker.thread` has type `JoinHandle<()>`, which does not implement the `Copy` trait
    |
note: `std::thread::JoinHandle::<T>::join` takes ownership of the receiver `self`, which moves `worker.thread`
```

The Book's final version:

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());                    // 1. close the channel
        for worker in self.workers.drain(..) {       // 2. move each Worker out, then join it
            println!("Shutting down worker {}", worker.id);
            worker.thread.join().unwrap();
        }
    }
}
```

- `sender` is an `Option<Sender<Job>>` so `drop` can `take()` it. Once it's gone, every idle worker's `recv` returns `Err` and its loop ends; busy workers finish their current job first.
- **Order matters.** Join first, and you wait forever for a worker that's waiting forever for a job: a deadlock.
- `drain(..)` empties the `Vec` and hands each `Worker` over by value. (Storing `Option<JoinHandle<()>>` and calling `take()` also works.)
- The Book demonstrates it with `listener.incoming().take(2)`: after two connections the loop ends, `main` returns, and the pool's `Drop` runs. Ctrl+C would end the process at once **without** running `Drop`.

> 💡 **Analogy: closing time.** First turn the sign to CLOSED (drop the sender), so no new tickets are printed. Each server finishes the customer in front of them, sees there are no more tickets, and hangs up their apron. The manager waits by the door until every server has left (`join`), and only then locks up. Waiting at the door *before* turning the sign would mean waiting all night.

▶ `cargo run -p web_server --example step6_graceful_shutdown`

## The finished server

| File | What's in it |
|---|---|
| [`src/pool.rs`](src/pool.rs) | `ThreadPool` (`new`, `build`, `execute`, `Drop`), `Worker`, `PoolCreationError` |
| [`src/http.rs`](src/http.rs) | `Router` (the Book's routes `/`, `/sleep`, 404), `Response`, `read_request_head`, `handle_connection` |
| [`src/server.rs`](src/server.rs) | `Server` (listener + pool), `ShutdownHandle` |
| [`src/client.rs`](src/client.rs) | a tiny HTTP client with timeouts, for the examples and tests |
| [`src/main.rs`](src/main.rs) | listens on `127.0.0.1:7878` with 4 workers, like the Book |
| `hello.html`, `404.html` | the Book's two pages |
| [`tests/server.rs`](tests/server.rs) | real TCP tests on a free port |

It follows the Book's design, with these changes (each explained below):
- `handle_connection` works on any `Read + Write`, and returns `io::Result` instead of unwrapping.
- It reads the **whole** request head (capped at 8 KiB), with a read timeout.
- A panicking job doesn't kill its worker, and a missing page is a `500`, not a crash.
- Responses also send `Content-Type` and `Connection: close`.
- The server can stop after N connections (`cargo run -p web_server -- 2`, the Book's `take(2)`) or when another thread calls `ShutdownHandle::shutdown`.

## Beyond the Book: testing a server, without and with a network

- **Without a network:** `handle_connection<S: Read + Write>(stream: S, …)` accepts a `TcpStream` in production and an in-memory `FakeStream` (a `Cursor` to read from, a `Vec<u8>` to write to) in the unit tests. This is the same trick as minigrep's testable I/O ([lesson 03/03](../../03_abstraction/03_minigrep_cli_project/README.md)).
- **With a network:** `tests/server.rs` binds `127.0.0.1:0`, runs the server on a background thread, sends real requests and shuts it down. It checks that a slow `/sleep` doesn't hold up `/`, and that a silent client can't block the only worker.
- **Never hang:** every wait in the tests has a limit: `recv_timeout`, `set_read_timeout`, `connect_timeout`. A broken server fails the test instead of freezing it. The pool's `jobs_run_in_parallel` test has two jobs that each wait (with a timeout) for the other, which can only succeed if two workers really run at once.

> 💡 **Analogy: a flight simulator.** Pilots practise in a simulator (the fake stream) where every switch can be tested safely, then do a short real flight (the TCP test) with an instructor who takes over if anything goes wrong (the timeouts).

## Beyond the Book: surviving misbehaving clients

Every `unwrap()` in the Book's `handle_connection` is a way for a client to crash a worker, and a crashed worker is never replaced. The finished server handles each case:

| Client does | Book's code | This server |
|---|---|---|
| connects and hangs up | `lines().next().unwrap()` panics | empty head: nothing to answer, `Ok(())` |
| connects and says nothing | blocks that worker forever | read timeout (5 s) frees the worker |
| sends endless headers | reads them all into memory | reads at most 8 KiB (`Read::take`) |
| sends nonsense | 404 | 404 |
| asks while a page file is missing | `read_to_string(…).unwrap()` panics | `500 INTERNAL SERVER ERROR` |
| triggers a bug that panics in a job | the worker thread dies | `catch_unwind` logs it and the worker carries on |

Two subtler points:
- **Read the whole head.** The Book's final version reads only the request line. If a server closes a socket while unread bytes are still waiting, the operating system may send a *reset* instead of a normal close, and the client can lose the response. Reading to the blank line avoids that.
- **`Connection: close`** tells the client we hang up after one response. HTTP/1.1 otherwise assumes the connection stays open for more requests.

> 💡 **Analogy: a shop assistant who stays calm.** A customer who walks in and straight out, one who stands silently at the counter, one who reads out a 100-page order, one who speaks nonsense: a good assistant handles each politely and moves on to the next customer, instead of fainting.

▶ `cargo run -p web_server --example misbehaving_clients`

## Beyond the Book: stopping on demand

`incoming()` blocks inside `accept()` until someone connects, so a flag alone can't stop the loop. `ShutdownHandle::shutdown` sets an `AtomicBool` and then **connects to the server itself**: `accept` returns, the loop sees the flag and breaks, and the pool's `Drop` finishes the requests in progress. Handling Ctrl+C the same way needs a signal handler (the [`ctrlc`](https://docs.rs/ctrlc) crate, or `tokio::signal` in async code); axum offers `with_graceful_shutdown` for this.

> 💡 **Analogy:** the person minding the door is asleep until the bell rings. To close early, you put up the CLOSED sign *and* ring your own doorbell, so they wake up, see the sign, and lock the door.

## Beyond the Book: where real servers go next

- **Thread count:** `std::thread::available_parallelism()` reports how many threads can truly run at once, a better default than a hard-coded 4.
- **Keep-alive:** real HTTP/1.1 clients reuse one connection for many requests; HTTP/2 multiplexes many at once.
- **Parsing:** real request lines and headers have many edge cases; crates such as `httparse` handle them.
- **Async I/O:** a thread per *active* connection still doesn't scale to 100,000 idle ones. Async runtimes (tokio) let a few threads juggle thousands of connections; hyper and axum build on them. Continue with [04/02 Async/await](../02_async_await/README.md) and [06 Framework Mastery: axum](../../06_framework_mastery/README.md).

> 💡 **Analogy:** our shop has four servers who each look after one customer at a time. A big restaurant uses **waiters who juggle**: take one table's order, then another's, and come back when the kitchen rings. That's async I/O.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `bind("127.0.0.1:7878")` while an old server still runs | `AddrInUse` (os error 10048 on Windows) | Stop the old one, or bind port `0` in tests. |
| `type Job = Box<dyn FnOnce()>` | E0277 cannot be sent between threads safely | `Box<dyn FnOnce() + Send + 'static>` |
| Moved one `Receiver` into several threads | E0382 use of moved value | `Arc<Mutex<Receiver<Job>>>` |
| `worker.thread.join()` in `drop(&mut self)` | E0507 cannot move out | `self.workers.drain(..)`, or `Option::take` |
| `while let Ok(job) = receiver.lock().unwrap().recv()` | compiles, but jobs run one at a time | `let message = …recv();` then `match`. |
| Joined the workers before dropping the sender | hangs forever | `drop(self.sender.take())` first. |
| `.unwrap()` on what a client sends | one odd client kills a worker | Return `io::Result` and log the error. |
| `Content-Length` from `chars().count()` | pages cut short for non-ASCII text | `body.len()`: it counts bytes. |
| `\n` instead of `\r\n` | some clients reject the response | HTTP lines end with `\r\n`. |
| Expected Ctrl+C to run `Drop` | workers are killed, no clean-up | Stop with a limit or a `ShutdownHandle`. |
| `.take(2)` with a browser | the server stops before the page loads | Browsers open extra connections (favicon); use `curl` or a bigger limit. |
| Ran from another folder with the Book's `read_to_string("hello.html")` | `NotFound` | Build the path from `env!("CARGO_MANIFEST_DIR")`, as this lesson does. |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p web_server --example step1_listen_and_read
cargo run -p web_server --example step2_respond
cargo run -p web_server --example step3_slow_single_thread
cargo run -p web_server --example step4_thread_per_request
cargo run -p web_server --example step5_thread_pool
cargo run -p web_server --example step6_graceful_shutdown
cargo run -p web_server --example misbehaving_clients
```

**2. Do the exercises** in `exercises/`. Numbers 1 to 3 are the Book's compiler-driven development, one error at a time:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_job_not_send` | Make it compile (E0277), fix the `Job` alias | `cargo run -p web_server --features exercises --bin ex01_fix_job_not_send` |
| 2 | `ex02_fix_shared_receiver` | Make it compile (E0382) with `Arc<Mutex<…>>` | `cargo run -p web_server --features exercises --bin ex02_fix_shared_receiver` |
| 3 | `ex03_fix_join_in_drop` | Make it compile (E0507), graceful shutdown | `cargo run -p web_server --features exercises --bin ex03_fix_join_in_drop` |
| 4 | `ex04_parse_request_line` | Method, path, query, version, errors, pass 3 tests | `cargo test -p web_server --features exercises --bin ex04_parse_request_line` |
| 5 | `ex05_build_response` | Status line, headers, byte-correct `Content-Length`, pass 3 tests | `cargo test -p web_server --features exercises --bin ex05_build_response` |
| 6 | `ex06_thread_pool_build` | `build`, `execute` and `Drop` for your own pool, pass 3 tests | `cargo test -p web_server --features exercises --bin ex06_thread_pool_build` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p web_server --features solutions --bins`).

**4. Run the finished server** (the mini project) and poke it:

```bash
cargo run -p web_server              # then open http://127.0.0.1:7878 in a browser
curl -i http://127.0.0.1:7878/       # 200 OK and the headers
curl -i http://127.0.0.1:7878/nope   # 404 NOT FOUND
cargo run -p web_server -- 2         # serve 2 connections, then shut down gracefully
cargo test -p web_server             # 13 unit tests + 4 end-to-end tests over real TCP
```

Open `/sleep` in one browser tab and `/` in another: the second loads at once, because a different worker serves it. Watch the terminal: `Worker 2 got a job; executing.` Stop the server with Ctrl+C.

> 💡 **Analogy:** you've built the whole ice-cream shop: a door, a ticket machine, a team of four, a polite way to refuse orders you don't understand, and a closing routine where nobody is sent home with half a sundae.

## Checklist

- [ ] [21.1 Building a Single-Threaded Web Server](https://doc.rust-lang.org/book/ch21-01-single-threaded.html)
- [ ] [21.2 From Single-Threaded to Multithreaded Server](https://doc.rust-lang.org/book/ch21-02-multithreaded.html)
- [ ] [21.3 Graceful Shutdown and Cleanup](https://doc.rust-lang.org/book/ch21-03-graceful-shutdown-and-cleanup.html)
- [ ] Beyond the Book: testing a server with a fake stream and on a free port, with timeouts everywhere
- [ ] Beyond the Book: surviving misbehaving clients (timeouts, size limits, no `unwrap`, `catch_unwind`)
- [ ] Beyond the Book: stopping on demand with a `ShutdownHandle`
- [ ] Beyond the Book: where real servers go next (keep-alive, async I/O, axum)
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `TcpListener::bind` + `incoming()` accept connections; HTTP is plain text: a request line, headers, a blank line; a response starts with a status line and needs a byte-accurate `Content-Length`.
- One thread serves one request at a time, so one slow request blocks everyone. A thread per request has no limit. A **thread pool** gives a fixed number of workers and a queue.
- The pool is a `Vec<Worker>` plus a channel of `Box<dyn FnOnce() + Send + 'static>` jobs, with the receiver shared as `Arc<Mutex<Receiver>>`; release the lock before running the job.
- Graceful shutdown: drop the sender first, then join every worker (`drain(..)` gives you ownership of each `JoinHandle`).
- A real server must also survive bad clients, and should be testable without a network.

**In one sentence for a child:** a web server is a shop where customers ring the bell and ask for pages, and a good shop has a team of helpers taking tickets from one machine, and closes by finishing every order before switching off the lights.

**Next:** [05 Mastery](../../05_mastery/README.md): you've finished the Book, and the course continues beyond it.
