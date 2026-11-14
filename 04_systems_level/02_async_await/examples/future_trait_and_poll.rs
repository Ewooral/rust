//! Book 17.5 (the `Future` and `Stream` traits) and Beyond the Book:
//! futures as compiler-generated state machines, and `poll`.
//!
//! No Tokio here: a hand-written future, a hand-written state machine, a
//! hand-written stream, and a 20-line executor that drives them, to show
//! there's no magic underneath `.await`.
//!
//! Run: cargo run -p async_await --example future_trait_and_poll

use futures::{Stream, StreamExt};
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

// The trait, as std defines it:
//
//     pub trait Future {
//         type Output;
//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
//     }
//     pub enum Poll<T> { Ready(T), Pending }
//
// `poll` means "make as much progress as you can, without waiting". It
// answers `Ready(value)`, or `Pending` after arranging for `cx.waker()` to be
// called when it's worth polling again.

/// Part 1: a hand-written future: it needs `n` more polls before it's done.
struct Countdown {
    name: &'static str,
    remaining: u32,
}

impl Future for Countdown {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            println!("    {}: Ready", self.name);
            return Poll::Ready(self.name);
        }
        println!("    {}: Pending ({} to go)", self.name, self.remaining);
        self.remaining -= 1;
        // A real future would hand the waker to a timer or the OS. We're
        // ready to try again straight away, so we wake ourselves at once.
        // Forget this, and nobody ever polls again: the program hangs.
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

// Part 2: what does the compiler make of this?
//
//     async fn two_steps() -> String {
//         let a = Countdown { name: "first", remaining: 1 }.await;
//         let b = Countdown { name: "second", remaining: 2 }.await;
//         format!("{a} + {b}")
//     }
//
// A STATE MACHINE: an enum with one state per `.await`, holding exactly
// the variables still needed. Each `poll` resumes from the current state.
// Here is the same thing written by hand.
enum TwoSteps {
    WaitingForFirst(Countdown),
    WaitingForSecond { a: &'static str, second: Countdown },
    Done,
}

impl Future for TwoSteps {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        // Everything inside is Unpin, so we may take a plain &mut.
        let this = self.get_mut();
        loop {
            match this {
                TwoSteps::WaitingForFirst(first) => match Pin::new(first).poll(cx) {
                    Poll::Pending => return Poll::Pending, // "pause at the 1st .await"
                    Poll::Ready(a) => {
                        let second = Countdown {
                            name: "second",
                            remaining: 2,
                        };
                        *this = TwoSteps::WaitingForSecond { a, second };
                    }
                },
                TwoSteps::WaitingForSecond { a, second } => match Pin::new(second).poll(cx) {
                    Poll::Pending => return Poll::Pending, // "pause at the 2nd .await"
                    Poll::Ready(b) => {
                        let result = format!("{a} + {b}");
                        *this = TwoSteps::Done;
                        return Poll::Ready(result);
                    }
                },
                TwoSteps::Done => panic!("polled after completion"),
            }
        }
    }
}

async fn two_steps_async() -> String {
    let a = Countdown {
        name: "first",
        remaining: 1,
    }
    .await;
    let b = Countdown {
        name: "second",
        remaining: 2,
    }
    .await;
    format!("{a} + {b}")
}

/// Part 3: a hand-written STREAM. The trait is like Future, with `poll_next`
/// returning `Poll<Option<Item>>`: `Ready(Some(x))` is an item,
/// `Ready(None)` the end.
struct Ticks {
    next: u32,
    last: u32,
}

impl Stream for Ticks {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u32>> {
        if self.next > self.last {
            return Poll::Ready(None);
        }
        self.next += 1;
        Poll::Ready(Some(self.next - 1))
    }
}

/// Part 4: a tiny executor. It polls the future; on `Pending` it puts the thread
/// to sleep until the waker unparks it, then polls again. A real runtime
/// adds timers, I/O and many tasks, but this loop is its heart.
fn block_on<F: Future>(future: F) -> F::Output {
    struct ThreadWaker(Thread);
    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let mut future = pin!(future);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut polls = 0;
    loop {
        polls += 1;
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            println!("    (block_on: {polls} poll(s))");
            return output;
        }
        thread::park(); // sleep until woken (returns at once if already woken)
    }
}

fn main() {
    println!("1. a hand-written future:");
    let name = block_on(Countdown {
        name: "countdown",
        remaining: 3,
    });
    println!("   -> {name}\n");

    println!("2a. the state machine written by hand:");
    let by_hand = block_on(TwoSteps::WaitingForFirst(Countdown {
        name: "first",
        remaining: 1,
    }));
    println!("   -> {by_hand}");
    println!("2b. the same with async/await (same polls, same result):");
    let generated = block_on(two_steps_async());
    println!("   -> {generated}");
    println!(
        "   the compiler's state machine is {} bytes; ours is {} bytes\n",
        std::mem::size_of_val(&two_steps_async()),
        std::mem::size_of::<TwoSteps>()
    );

    println!("3. a hand-written stream, used with StreamExt adapters:");
    // `StreamExt` is an EXTENSION trait: a blanket impl gives its methods
    // (`next`, `map`, `collect`, ...) to every type that implements Stream.
    let doubled: Vec<u32> = block_on(Ticks { next: 1, last: 5 }.map(|n| n * 2).collect());
    println!("   -> {doubled:?}");
}
