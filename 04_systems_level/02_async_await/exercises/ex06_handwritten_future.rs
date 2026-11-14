//! Exercise 6: write `poll` yourself (Book 17.5).
//!
//! `Countdown::new(n)` is a future that needs `n` polls that answer
//! `Pending` before it is `Ready`. Its output is how many times it was
//! polled in total. Implement `poll`:
//!
//! 1. Count this poll (`self.polls += 1`).
//! 2. If `remaining` is 0, return `Poll::Ready(self.polls)`.
//! 3. Otherwise take 1 off `remaining`, call `cx.waker().wake_by_ref()` so
//!    the executor knows to poll again, and return `Poll::Pending`.
//!
//! `Countdown` is `Unpin` (it holds only numbers), so `self` can be used
//! like a `&mut Countdown`. If a test HANGS, you returned `Pending` without
//! waking: no executor will ever poll the future again.
//!
//! Test: cargo test -p async_await --features exercises --bin ex06_handwritten_future

#![allow(unused_variables, unused_mut, dead_code)]

use std::pin::Pin;
use std::task::{Context, Poll};

struct Countdown {
    remaining: u32,
    polls: u32,
}

impl Countdown {
    fn new(remaining: u32) -> Self {
        Countdown {
            remaining,
            polls: 0,
        }
    }
}

impl Future for Countdown {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let polls = Countdown::new(3).await;
    println!("Countdown::new(3) was polled {polls} times");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::pin;
    use std::task::Waker;

    #[test]
    fn pending_until_the_count_runs_out() {
        // Poll by hand, with a waker that does nothing.
        let mut cx = Context::from_waker(Waker::noop());
        let mut future = pin!(Countdown::new(2));
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Ready(3));
    }

    #[test]
    fn a_real_executor_drives_it() {
        // `block_on` polls only when woken: this needs `wake_by_ref`.
        assert_eq!(futures::executor::block_on(Countdown::new(0)), 1);
        assert_eq!(futures::executor::block_on(Countdown::new(4)), 5);
    }

    #[tokio::test]
    async fn it_runs_alongside_other_futures() {
        let (a, b) = tokio::join!(Countdown::new(1), Countdown::new(5));
        assert_eq!((a, b), (2, 6));
    }
}
