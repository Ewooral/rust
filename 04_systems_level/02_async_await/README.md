# Async Await

**The Rust Book, chapter 17:** [Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html)

Run it: `cargo run -p async_await`  ·  Test it: `cargo test -p async_await`

## Concepts (from the Book)

- [ ] [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html)
- [ ] [17.2 Applying Concurrency with Async](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html)
- [ ] [17.3 Working With Any Number of Futures](https://doc.rust-lang.org/book/ch17-03-more-futures.html)
- [ ] [17.4 Streams: Futures in Sequence](https://doc.rust-lang.org/book/ch17-04-streams.html)
- [ ] [17.5 A Closer Look at the Traits for Async](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html)
- [ ] [17.6 Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html)

## Beyond the Book

- [ ] Futures as compiler-generated state machines; `poll`
- [ ] `Pin`/`Unpin` and boxed futures (`Pin<Box<dyn Future + Send + '_>>`)
- [ ] `async fn` in traits, and why `dyn` needs boxing or the `async_trait` crate
- [ ] Sharing state with `Arc<dyn Trait + Send + Sync>` across `tokio::spawn`
- [ ] Tokio in practice: `tokio::select!`, timeouts, cancellation safety
- [ ] Async streams with the `futures` crate
