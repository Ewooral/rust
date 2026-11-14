//! Book 17.5 (`Pin` and `Unpin`) and Beyond the Book: boxed futures,
//! `Pin<Box<dyn Future + Send + '_>>`.
//!
//! Why pinning exists: the state machine behind an `async` block may hold a
//! reference to ITS OWN local variable across an `.await`. If the future
//! were moved in memory after that, the reference would point at the old
//! place. `Pin` is a promise that the future won't move again.
//!
//! Run: cargo run -p async_await --example pin_and_boxed_futures

use futures::future::{BoxFuture, FutureExt, join_all};
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use tokio::time::sleep;

/// Only accepts futures that are safe to move around after polling.
fn poll_once<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    let mut cx = Context::from_waker(Waker::noop());
    Pin::new(future).poll(&mut cx)
}

/// A self-referential future: `slice` borrows `numbers`, and both are
/// stored in the state machine while it waits at the `.await`.
async fn sum_after_a_nap() -> u32 {
    let numbers = [1, 2, 3, 4];
    let slice = &numbers[1..]; // points INTO this future's own state
    sleep(Duration::from_millis(5)).await;
    slice.iter().sum()
}

/// Recursion needs a box: the future would contain itself, with infinite
/// size. Without `Box::pin`:
///     error[E0733]: recursion in an async fn requires boxing
async fn countdown(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    1 + Box::pin(countdown(n - 1)).await
}

/// A function that returns DIFFERENT futures from different branches must
/// erase their types: `BoxFuture<'a, T>` is the futures crate's name for
/// `Pin<Box<dyn Future<Output = T> + Send + 'a>>`.
fn greet<'a>(name: &'a str, polite: bool) -> BoxFuture<'a, String> {
    if polite {
        async move {
            sleep(Duration::from_millis(5)).await;
            format!("Good afternoon, {name}.")
        }
        .boxed() // the same as Box::pin(async move { ... })
    } else {
        Box::pin(async move { format!("hey {name}") })
    }
}

#[tokio::main]
async fn main() {
    // 1. Unpin is an auto trait: most types have it, async blocks don't.
    let mut number_future = std::future::ready(42); // a simple Unpin future
    println!(
        "1. poll a `ready` future directly: {:?}",
        poll_once(&mut number_future)
    );
    // let mut fut = sum_after_a_nap();
    // poll_once(&mut fut); // E0277: `{async fn body of sum_after_a_nap()}` cannot be unpinned
    println!(
        "   an async fn's future is {} bytes (its state machine) and is !Unpin",
        std::mem::size_of_val(&sum_after_a_nap())
    );

    // 2. Pin it on the STACK with `pin!`. `Pin<&mut F>` is itself Unpin, so
    // it can be polled by hand, raced, or awaited by reference.
    let mut pinned = pin!(sum_after_a_nap());
    println!(
        "2. first poll of the pinned future: {:?}",
        poll_once(&mut pinned)
    );
    println!("   awaiting it to the end: {}", pinned.await);

    // 3. Pin it on the HEAP with `Box::pin`: the box can move, the future
    // inside never does. Needed to store futures of different types together.
    // Without the Pin, `Vec<Box<dyn Future<Output = u32>>>` fails:
    //     E0277: `dyn futures::Future<Output = u32>` cannot be unpinned
    let futures: Vec<Pin<Box<dyn Future<Output = u32> + Send>>> = vec![
        Box::pin(sum_after_a_nap()),
        Box::pin(async { 100 }),
        Box::pin(countdown(5)),
    ];
    println!(
        "3. join_all over boxed futures: {:?}",
        join_all(futures).await
    );

    // 4. Boxed futures returned from branches, with a lifetime: the future
    // borrows `name`, so it's `BoxFuture<'a, _>` and not `'static`.
    let name = String::from("Ama");
    let both = join_all([greet(&name, true), greet(&name, false)]).await;
    println!("4. {both:?}");

    // 5. `+ Send` in the type matters for `tokio::spawn`, which may move the
    // task to another thread. A BoxFuture is Send, so it can be spawned.
    let task: BoxFuture<'static, u32> = countdown(3).boxed();
    println!(
        "5. a spawned BoxFuture returned {}",
        tokio::spawn(task).await.unwrap()
    );
    // `.boxed_local()` gives a non-Send `LocalBoxFuture` for futures that hold
    // things like `Rc`: fine to await, but `tokio::spawn` refuses it.
}
